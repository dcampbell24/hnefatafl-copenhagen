use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::role::Role;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Color {
    /// attacker
    #[default]
    Black,
    Colorless,
    /// defender
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

impl From<&Role> for Color {
    fn from(role: &Role) -> Self {
        match role {
            Role::Attacker => Color::Black,
            Role::Defender => Color::White,
        }
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(color: &str) -> anyhow::Result<Self> {
        match color {
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            _ => Err(anyhow::Error::msg("a color is expected")),
        }
    }
}
