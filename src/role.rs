use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

// use crate::role::Role;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Role {
    #[default]
    Attacker,
    Defender,
    Roleless,
}

impl Role {
    #[must_use]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Attacker => Self::Defender,
            Self::Defender => Self::Attacker,
            Self::Roleless => Self::Roleless,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Attacker => write!(f, "attacker"),
            Role::Defender => write!(f, "defender"),
            Role::Roleless => write!(f, "roleless"),
        }
    }
}

impl TryFrom<&Role> for Role {
    type Error = anyhow::Error;

    fn try_from(role: &Role) -> anyhow::Result<Self> {
        match role {
            Role::Attacker => Ok(Self::Attacker),
            Role::Roleless => Err(anyhow::Error::msg("the piece must be attacker or defender")),
            Role::Defender => Ok(Self::Defender),
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
