use std::{fmt, str::FromStr};

use crate::color::Color;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Role {
    #[default]
    Attacker,
    Defender,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Attacker => write!(f, "attacker"),
            Role::Defender => write!(f, "defender"),
        }
    }
}

impl TryFrom<&Color> for Role {
    type Error = anyhow::Error;

    fn try_from(color: &Color) -> anyhow::Result<Self> {
        match color {
            Color::Black => Ok(Self::Attacker),
            Color::Colorless => Err(anyhow::Error::msg("the piece must be black or white")),
            Color::White => Ok(Self::Defender),
        }
    }
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> anyhow::Result<Self> {
        match string {
            "attacker" => Ok(Self::Attacker),
            "defender" => Ok(Self::Defender),
            _ => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{string}' to a Role!"
            ))),
        }
    }
}
