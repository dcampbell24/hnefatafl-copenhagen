use std::fmt;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
            Self::BlackWins => write!(f, "black_wins"),
            Self::Draw => write!(f, "draw"),
            Self::Ongoing => write!(f, "ongoing"),
            Self::WhiteWins => write!(f, "white_wins"),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        match value {
            "Black" => Ok(Self::BlackWins),
            "Draw" => Ok(Self::Draw),
            "Ongoing" => Ok(Self::Ongoing),
            "White" => Ok(Self::WhiteWins),
            _ => Err(anyhow::Error::msg("invalid status")),
        }
    }
}
