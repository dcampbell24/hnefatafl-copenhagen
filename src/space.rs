use std::fmt;

use serde::{Deserialize, Serialize};

use crate::role::Role;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Space {
    Empty,
    Attacker,
    King,
    Defender,
}

impl TryFrom<char> for Space {
    type Error = anyhow::Error;

    fn try_from(value: char) -> anyhow::Result<Self> {
        match value {
            'X' => Ok(Self::Attacker),
            'O' => Ok(Self::Defender),
            '.' => Ok(Self::Empty),
            'K' => Ok(Self::King),
            ch => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{ch}' to a Space!"
            ))),
        }
    }
}

impl fmt::Display for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attacker => write!(f, "♟"),
            Self::Empty => write!(f, "."),
            Self::King => write!(f, "♔"),
            Self::Defender => write!(f, "♙"),
        }
    }
}

impl Space {
    #[must_use]
    pub fn role(&self) -> Role {
        match self {
            Self::Attacker => Role::Attacker,
            Self::Defender | Self::King => Role::Defender,
            Self::Empty => Role::Roleless,
        }
    }

    /// # Panics
    ///
    /// If you take the index of an empty space.
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Self::Attacker => 0,
            Self::Defender => 1,
            Self::King => 2,
            Self::Empty => panic!("we should not take the index of an empty space"),
        }
    }
}
