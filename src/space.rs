#[derive(Debug, Default, Clone)]
pub enum Space {
    #[default]
    Empty,
    Exit,
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
            'E' => Self::Exit,
            'K' => Self::King,
            ch => panic!("error trying to convert '{ch}' to a Space"),
        }
    }
}
