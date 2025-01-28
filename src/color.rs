use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Color {
    // attacker
    #[default]
    Black,
    Colorless,
    // defender
    White,
}

impl Color {
    #[must_use]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
            Self::Colorless => Self::Colorless,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Black => write!(f, "black"),
            Self::Colorless => write!(f, "colorless"),
            Self::White => write!(f, "white"),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;

    fn try_from(color: &str) -> Result<Self, Self::Error> {
        match color {
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            _ => Err(anyhow::Error::msg("a color is expected")),
        }
    }
}
