use crate::color::Color;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Space {
    Empty,
    Black,
    King,
    White,
}

impl From<char> for Space {
    fn from(value: char) -> Space {
        match value {
            'X' => Self::Black,
            'O' => Self::White,
            ' ' => Self::Empty,
            'K' => Self::King,
            ch => panic!("error trying to convert '{ch}' to a Space"),
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
}
