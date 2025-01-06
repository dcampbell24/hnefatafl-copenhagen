use crate::color::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
            ' ' => Ok(Self::Empty),
            'K' => Ok(Self::King),
            ch => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{ch}' to a Space!"
            ))),
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
