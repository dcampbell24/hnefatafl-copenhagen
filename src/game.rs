use std::{fmt, process::exit};

use anyhow::Ok;

use crate::{board::Board, color::Color, message::Message, move_::Move, status::Status};

#[derive(Debug, Default, Clone)]
pub struct Game {
    board: Board,
    pub moves: Vec<Move>,
    pub status: Status,
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
                let status = self.board.move_(&move_, &self.status, &self.turn)?;
                if status == Status::Ongoing {
                    self.turn = self.turn.opposite();
                }
                self.status = status;
                self.moves.push(move_);
            }
            Message::Quit => exit(0),
            Message::ShowBoard => print!("{}", self.board),
        }

        Ok(())
    }
}
