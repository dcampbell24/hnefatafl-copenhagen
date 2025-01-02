use std::{fmt, process::exit};

use anyhow::Ok;

use crate::{board::Board, color::Color, message::Message, move_::Move, space::Space};

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
    /// If the move is out of bounds.
    pub fn update(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Empty => {}
            Message::Move(_) => {
                if let Message::Move(move_) = message {
                    let space = self.board.get(&move_)?;
                    let color = space.color();
                    if self.turn == color {
                        // Check if the piece has an uninterrupted line to the space it moves to.

                        self.board.set(&move_.from, Space::Empty);
                        self.board.set(&move_.to, space);

                        // Check for a win.
                        // Check for captures.

                        self.moves.push(move_);
                        self.turn = self.turn.opposite();
                    }
                }
            }
            Message::Quit => exit(0),
            Message::ShowBoard => print!("{self}"),
        }

        Ok(())
    }
}
