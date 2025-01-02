use std::fmt;

use anyhow::Context;
use iced::{
    widget::{button, text, Column, Row},
    Element,
};

use crate::{
    color::Color,
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

    /// # Errors
    ///
    /// If the move is out of bounds.
    pub fn get(&self, vertex: &Vertex) -> anyhow::Result<Space> {
        let column = self
            .spaces
            .get(vertex.x)
            .context("Index is out of bounds.")?;

        Ok(column
            .get(vertex.y)
            .context("Index is out of bounds.")?
            .clone())
    }

    /// # Errors
    ///
    /// If the move is illegal.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    pub fn move_(&mut self, move_: &Move, turn: &Color) -> anyhow::Result<()> {
        let space = self.get(&move_.from)?;
        let color = space.color();

        if *turn == color {
            let x_diff = move_.from.x as i32 - move_.to.x as i32;
            let y_diff = move_.from.y as i32 - move_.to.y as i32;

            if x_diff != 0 && y_diff != 0 {
                return Err(anyhow::Error::msg("you can only move in a straight line"));
            }

            if x_diff == 0 && y_diff == 0 {
                return Err(anyhow::Error::msg("you have to change location"));
            }

            if x_diff != 0 {
                let x_diff_sign = x_diff.signum();
                for x_diff in 1..=x_diff.abs() {
                    let vertex = Vertex {
                        x: (move_.from.x as i32 - (x_diff * x_diff_sign)) as usize,
                        y: move_.from.y,
                    };

                    let space = self.get(&vertex)?;
                    if space != Space::Empty && space != Space::Exit {
                        return Err(anyhow::Error::msg(
                            "you have to move through empty locations",
                        ));
                    }
                }
            } else {
                let y_diff_sign = y_diff.signum();
                for y_diff in 1..=y_diff.abs() {
                    let vertex = Vertex {
                        x: move_.from.x,
                        y: (move_.from.y as i32 - (y_diff * y_diff_sign)) as usize,
                    };
                    let space = self.get(&vertex)?;
                    if space != Space::Empty && space != Space::Exit {
                        return Err(anyhow::Error::msg(
                            "you have to move through empty locations",
                        ));
                    }
                }
            }

            self.set(&move_.from, Space::Empty);
            self.set(&move_.to, space);
            // Check for a win.
            // Check for captures.
        } else {
            return Err(anyhow::Error::msg("it isn't your turn"));
        }

        Ok(())
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

    fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.x][vertex.y] = space;
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

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        self.draw()
    }
}
