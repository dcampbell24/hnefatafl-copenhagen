use std::{fmt, process::exit};

use anyhow::Ok;

use crate::{board::Board, color::Color, message::Message, move_::Move};

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
    /// # Errors
    ///
    /// If the move is illegal.
    pub fn update(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Empty => {}
            Message::Move(move_) => {
                self.board.move_(&move_, &self.turn)?;
                self.moves.push(move_);
                self.turn = self.turn.opposite();
            }
            Message::Quit => exit(0),
            Message::ShowBoard => print!("{}", self.board),
        }

        Ok(())
    }
}
