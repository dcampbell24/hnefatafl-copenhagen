use std::fmt;

use anyhow::Context;
use iced::{
    widget::{button, text, Column, Row},
    Element,
};

use crate::{
    message::Message,
    move_::{Move, Vertex},
};

use super::space::Space;

#[derive(Debug, Clone)]
pub struct Board {
    pub spaces: Vec<Vec<Space>>,
}

impl Default for Board {
    fn default() -> Self {
        Board::new()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl From<Vec<&str>> for Board {
    fn from(value: Vec<&str>) -> Self {
        let mut rows = Vec::new();

        for row in value {
            let mut columns = Vec::new();
            for ch in row.chars() {
                columns.push(ch.into());
            }
            rows.push(columns);
        }

        Self { spaces: rows }
    }
}

impl Board {
    /// # Errors
    ///
    /// If the move is out of bounds.
    pub fn get(&self, move_: &Move) -> anyhow::Result<Space> {
        let column = self
            .spaces
            .get(move_.from.x)
            .context("Index is out of bounds.")?;

        Ok(column
            .get(move_.from.y)
            .context("Index is out of bounds.")?
            .clone())
    }

    pub fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.x][vertex.y] = space;
    }

    fn new() -> Self {
        let spaces = vec![
            "E  XXXXX  E",
            "     X     ",
            "           ",
            "X    O    X",
            "X   OOO   X",
            "XX OOKOO XX",
            "X   OOO   X",
            "X    O    X",
            "           ",
            "     X     ",
            "E  XXXXX  E",
        ];

        spaces.into()
    }

    #[must_use]
    pub fn show(&self) -> String {
        let mut board = String::new();
        let letters = "   ABCDEFGHJKL \n";

        board.push_str(letters);
        board.push_str("  ┌");
        board.push_str(&"─".repeat(11));
        board.push('┐');
        board.push('\n');

        for (mut i, line) in self.spaces.iter().enumerate() {
            i = 11 - i;
            board.push_str(&format!("{i:2}"));
            board.push('│');

            for space in line {
                match space {
                    Space::Black => board.push('X'),
                    Space::Empty => board.push('.'),
                    Space::Exit => board.push('E'),
                    Space::King => board.push('K'),
                    Space::White => board.push('O'),
                }
            }

            board.push('│');
            board.push('\n');
        }

        board.push_str("  └");
        board.push_str(&"─".repeat(11));
        board.push('┘');
        board.push('\n');

        board
    }

    fn draw(&self) -> Element<Message> {
        let mut columns = Row::new();

        for (board_column, spaces_column) in Board::new().spaces.iter().zip(&self.spaces) {
            let mut row = Column::new();
            for (_board_row, spaces_row) in board_column.iter().zip(spaces_column) {
                let button = match spaces_row {
                    Space::Empty | Space::Exit => button(text("  ")),
                    Space::Black => button(text("X")),
                    Space::King => button(text("K")),
                    Space::White => button(text("o")),
                };
                row = row.push(button);

                /*
                match board_row {
                    Space::Empty => row = row.push(button),
                    Space::Exit => row = row.push(button),
                    Space::Black => row = row.push(button),
                    Space::King => row = row.push(button),
                    Space::White => row = row.push(button),
                }
                */
            }
            columns = columns.push(row);
        }

        columns.into()
    }

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        self.draw()
    }
}
