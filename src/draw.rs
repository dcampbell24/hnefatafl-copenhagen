use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Draw {
    Accept,
    Decline,
}

impl fmt::Display for Draw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accept => write!(f, "accept"),
            Self::Decline => write!(f, "decline"),
        }
    }
}

impl TryFrom<&str> for Draw {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        match value {
            "accept" => Ok(Self::Accept),
            "decline" => Ok(Self::Decline),
            s => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{s}' to a Draw!"
            ))),
        }
    }
}
