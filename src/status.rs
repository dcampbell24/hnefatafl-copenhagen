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
