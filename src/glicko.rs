use std::{
    f64::consts::{LOG10_2, PI},
    fmt,
};

use serde::{Deserialize, Serialize};

/// ln 10 / 400
const Q: f64 = 0.005_756_5;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Rating {
    pub rating: f64,
    /// Ratings Deviation
    pub rd: f64,
}

impl Rating {
    #[must_use]
    pub fn rd_sq(&self) -> f64 {
        self.rd * self.rd
    }

    pub fn update_rd(&mut self) {
        // This assumes 30 2 month periods must pass before one's rating
        // deviation is the same as a new player and that a typical RD is 50.
        let c = 63.2;

        let rd_new = f64::sqrt(self.rd_sq() + (c * c));
        self.rd = rd_new.clamp(30.0, 350.0);
    }

    pub fn update_rating(&mut self, rating: f64, outcome: &Outcome) {
        self.rating += (Q / (1.0 / self.rd_sq()) + (1.0 / self.d_sq(rating)))
            * self.g()
            * (outcome.score() - self.e(rating));

        self.rd = f64::sqrt(1.0 / ((1.0 / self.rd_sq()) + (1.0 / self.d_sq(rating))));
    }

    #[must_use]
    fn d_sq(&self, rating: f64) -> f64 {
        1.0 / ((Q * Q) * (self.g() * self.g()) * self.e(rating) * (1.0 - self.e(rating)))
    }

    #[must_use]
    fn e(&self, rating: f64) -> f64 {
        1.0 / (1.0 + exp10(-self.g() * ((self.rating - rating) / 400.0)))
    }

    #[must_use]
    fn g(&self) -> f64 {
        1.0 / f64::sqrt(1.0 + ((3.0 * Q * Q * self.rd_sq()) / (PI * PI)))
    }
}

impl Default for Rating {
    fn default() -> Self {
        Self {
            rating: 1_500.0,
            rd: 350.0,
        }
    }
}

impl fmt::Display for Rating {
    // With a confidence interval of 95%.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}±{:.2}", self.rating.round(), 1.96 * self.rd.round())
    }
}

#[derive(Clone, Debug)]
pub enum Outcome {
    Draw,
    Loss,
    Win,
}

impl Outcome {
    #[must_use]
    pub fn score(&self) -> f64 {
        match self {
            Outcome::Draw => 0.5,
            Outcome::Loss => 0.0,
            Outcome::Win => 1.0,
        }
    }
}

#[must_use]
pub fn exp10(mut exponent: f64) -> f64 {
    exponent /= LOG10_2;
    exponent.exp2()
}

#[cfg(test)]
mod tests {
    use crate::glicko::Outcome;

    use super::{exp10, Rating};

    #[allow(clippy::float_cmp)]
    #[test]
    fn pow_10() {
        assert_eq!(exp10(2.0).round(), 100.0);
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn rd_increases() {
        let mut rating = Rating::default();
        assert_eq!(rating.rd, 350.0);

        rating.update_rd();
        assert_eq!(rating.rd, 350.0);

        rating.rd = 30.0;
        rating.update_rd();
        assert_eq!(rating.rd.round(), 70.0);

        rating.rd = 300.0;
        rating.update_rd();
        assert_eq!(rating.rd.round(), 307.0);
    }

    #[allow(clippy::float_cmp)]
    #[test]
    fn rating_and_rd_changes() {
        let rating = Rating::default();
        assert_eq!(rating.rating, 1_500.0);

        let mut rating_1 = rating.clone();
        rating_1.update_rating(1_600.0, &Outcome::Win);
        assert_eq!(rating_1.rating.round(), 1_781.0);

        let mut rating_2 = rating.clone();
        rating_2.update_rating(1_500.0, &Outcome::Win);
        assert_eq!(rating_2.rating.round(), 1_736.0);

        let mut rating_3 = rating.clone();
        rating_3.update_rating(1_500.0, &Outcome::Loss);
        assert_eq!(rating_3.rating.round(), 1_264.0);

        let mut rating_4 = rating.clone();
        rating_4.update_rating(1_500.0, &Outcome::Loss);
        assert_eq!(rating_4.rating.round(), 1_264.0);
        assert_eq!(rating_4.rd.round(), 299.0);
        rating_4.update_rd();
        assert_eq!(rating_4.rd.round(), 305.0);

        let mut rating_5 = rating.clone();
        rating_5.update_rating(1_500.0, &Outcome::Draw);
        assert_eq!(rating_5.rating.round(), 1_500.0);

        let mut rating_6 = rating.clone();
        rating_6.update_rating(1_600.0, &Outcome::Draw);
        assert_eq!(rating_6.rating.round(), 1545.0);
    }
}
