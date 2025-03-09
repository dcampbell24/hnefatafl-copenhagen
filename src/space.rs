use std::fmt;

use serde::{Deserialize, Serialize};

use crate::color::Color;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Space {
    Empty,
    Black,
    King,
    White,
}

impl TryFrom<char> for Space {
    type Error = anyhow::Error;

    fn try_from(value: char) -> anyhow::Result<Self> {
        match value {
            'X' => Ok(Self::Black),
            'O' => Ok(Self::White),
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
            Self::Black => write!(f, "♙"),
            Self::Empty => write!(f, "."),
            Self::King => write!(f, "♚"),
            Self::White => write!(f, "♟"),
        }
    }
}

impl Space {
    #[must_use]
    pub fn color(&self) -> Color {
        match self {
            Self::Black => Color::Black,
            Self::White | Self::King => Color::White,
            Self::Empty => Color::Colorless,
        }
    }

    /// # Panics
    ///
    /// If you take the index of an empty space.
    #[must_use]
    pub fn index(&self) -> usize {
        match self {
            Self::Black => 0,
            Self::White => 1,
            Self::King => 2,
            Self::Empty => panic!("we should not take the index of an empty space"),
        }
    }
}
