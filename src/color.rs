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
