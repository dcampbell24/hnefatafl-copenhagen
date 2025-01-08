#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Color {
    #[default]
    Black,
    Colorless,
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

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;

    fn try_from(color: &str) -> Result<Self, Self::Error> {
        match color {
            "black" => Ok(Self::Black),
            "white" => Ok(Self::White),
            "colorless" => Ok(Self::Colorless),
            _ => Err(anyhow::Error::msg("a color is expected")),
        }
    }
}
