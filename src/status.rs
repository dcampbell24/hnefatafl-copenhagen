use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    BlackWins,
    Draw,
    #[default]
    Ongoing,
    WhiteWins,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlackWins => write!(f, "attacker_wins"),
            Self::Draw => write!(f, "draw"),
            Self::Ongoing => write!(f, "ongoing"),
            Self::WhiteWins => write!(f, "defender_wins"),
        }
    }
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        match value {
            "Black" => Ok(Self::BlackWins),
            "Draw" => Ok(Self::Draw),
            "Ongoing" => Ok(Self::Ongoing),
            "White" => Ok(Self::WhiteWins),
            _ => Err(anyhow::Error::msg("invalid status")),
        }
    }
}
