use std::fmt;

use crate::{
    board::Board,
    message::{Message, Move},
};

#[derive(Debug, Default, Clone)]
pub struct Game {
    board: Board,
    pub moves: Vec<Move>,
    pub turn: Color,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)?;
        writeln!(f, "moves: {:?}", self.moves)?;
        writeln!(f, "turn: {:?}", self.turn)
    }
}

impl Game {
    pub fn update(&mut self, message: Message) {
        self.board.update(&message);
        if let Message::Move(move_) = message {
            self.moves.push(move_);
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum Color {
    #[default]
    Black,
    White,
}

impl Color {
    #[must_use]
    pub fn opposite(&self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}
