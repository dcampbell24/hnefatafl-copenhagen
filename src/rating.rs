use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Rated {
    #[default]
    No,
    Yes,
}

impl fmt::Display for Rated {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rated::No => write!(f, "unrated"),
            Rated::Yes => write!(f, "rated"),
        }
    }
}

impl From<bool> for Rated {
    fn from(boolean: bool) -> Self {
        if boolean {
            Self::Yes
        } else {
            Self::No
        }
    }
}

impl From<Rated> for bool {
    fn from(rated: Rated) -> Self {
        match rated {
            Rated::Yes => true,
            Rated::No => false,
        }
    }
}

impl FromStr for Rated {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> anyhow::Result<Self> {
        match string {
            "rated" => Ok(Self::Yes),
            "unrated" => Ok(Self::No),
            _ => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{string}' to Rated!"
            ))),
        }
    }
}
