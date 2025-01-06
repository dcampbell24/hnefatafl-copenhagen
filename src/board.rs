use std::fmt;

use anyhow::Context;

use crate::{
    color::Color,
    play::{Play, Vertex},
    status::Status,
};

use super::space::Space;

pub const STARTING_POSITION: [&str; 11] = [
    "   XXXXX   ",
    "     X     ",
    "           ",
    "X    O    X",
    "X   OOO   X",
    "XX OOKOO XX",
    "X   OOO   X",
    "X    O    X",
    "           ",
    "     X     ",
    "   XXXXX   ",
];

const EXIT_SQUARES: [Vertex; 4] = [
    Vertex { x: 0, y: 0 },
    Vertex { x: 10, y: 0 },
    Vertex { x: 0, y: 10 },
    Vertex { x: 10, y: 10 },
];

const THRONE: Vertex = Vertex { x: 5, y: 5 };

const RESTRICTED_SQUARES: [Vertex; 5] = [
    Vertex { x: 0, y: 0 },
    Vertex { x: 10, y: 0 },
    Vertex { x: 0, y: 10 },
    Vertex { x: 10, y: 10 },
    THRONE,
];

#[derive(Clone, Eq, PartialEq)]
pub struct Board {
    pub spaces: [[Space; 11]; 11],
}

impl Default for Board {
    fn default() -> Self {
        STARTING_POSITION.into()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.show())
    }
}

impl From<[&str; 11]> for Board {
    fn from(value: [&str; 11]) -> Self {
        let mut spaces = [[Space::Empty; 11]; 11];

        for (y, row) in value.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                spaces[y][x] = ch.into();
            }
        }

        Self { spaces }
    }
}

impl Board {
    fn captures(&mut self, play_to: &Vertex, color_from: &Color) -> anyhow::Result<()> {
        if let Some(up_1) = play_to.up() {
            let space = self.get(&up_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(up_2) = up_1.up() {
                    if RESTRICTED_SQUARES.contains(&up_2) || self.get(&up_2)?.color() == *color_from
                    {
                        self.set(&up_1, Space::Empty);
                    }
                }
            }
        }

        if let Some(left_1) = play_to.left() {
            let space = self.get(&left_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(left_2) = left_1.left() {
                    if RESTRICTED_SQUARES.contains(&left_2)
                        || self.get(&left_2)?.color() == *color_from
                    {
                        self.set(&left_1, Space::Empty);
                    }
                }
            }
        }

        if let Some(down_1) = play_to.down() {
            let space = self.get(&down_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(down_2) = down_1.down() {
                    if RESTRICTED_SQUARES.contains(&down_2)
                        || self.get(&down_2)?.color() == *color_from
                    {
                        self.set(&down_1, Space::Empty);
                    }
                }
            }
        }

        if let Some(right_1) = play_to.right() {
            let space = self.get(&right_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(right_2) = right_1.down() {
                    if RESTRICTED_SQUARES.contains(&right_2)
                        || self.get(&right_2)?.color() == *color_from
                    {
                        self.set(&right_1, Space::Empty);
                    }
                }
            }
        }

        Ok(())
    }

    /// # Errors
    ///
    /// If the play is out of bounds.
    pub fn get(&self, vertex: &Vertex) -> anyhow::Result<Space> {
        let column = self
            .spaces
            .get(vertex.x)
            .context("get: index is out of x bounds")?;

        Ok(*column
            .get(vertex.y)
            .context("get: index is out of y bounds")?)
    }

    /// # Errors
    ///
    /// If the play is illegal.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss
    )]
    pub fn play(&mut self, play: &Play, status: &Status, turn: &Color) -> anyhow::Result<Status> {
        // Todo: the throne is only hostile to defenders when empty.

        if *status != Status::Ongoing {
            return Err(anyhow::Error::msg(
                "play: the game has to be ongoing to play",
            ));
        }

        let space_from = self.get(&play.from)?;
        let color_from = space_from.color();

        if *turn == color_from {
            let x_diff = play.from.x as i32 - play.to.x as i32;
            let y_diff = play.from.y as i32 - play.to.y as i32;

            if x_diff != 0 && y_diff != 0 {
                return Err(anyhow::Error::msg(
                    "play: you can only play in a straight line",
                ));
            }

            if x_diff == 0 && y_diff == 0 {
                return Err(anyhow::Error::msg("play: you have to change location"));
            }

            if x_diff != 0 {
                let x_diff_sign = x_diff.signum();
                for x_diff in 1..=x_diff.abs() {
                    let vertex = Vertex {
                        x: (play.from.x as i32 - (x_diff * x_diff_sign)) as usize,
                        y: play.from.y,
                    };

                    let space = self.get(&vertex)?;
                    if space != Space::Empty {
                        return Err(anyhow::Error::msg(
                            "play: you have to play through empty locations",
                        ));
                    }
                }
            } else {
                let y_diff_sign = y_diff.signum();
                for y_diff in 1..=y_diff.abs() {
                    let vertex = Vertex {
                        x: play.from.x,
                        y: (play.from.y as i32 - (y_diff * y_diff_sign)) as usize,
                    };
                    let space = self.get(&vertex)?;
                    if space != Space::Empty {
                        return Err(anyhow::Error::msg(
                            "play: you have to play through empty locations",
                        ));
                    }
                }
            }

            if space_from != Space::King && RESTRICTED_SQUARES.contains(&play.to) {
                return Err(anyhow::Error::msg(
                    "play: only the king may move to a restricted square",
                ));
            }

            self.set(&play.from, Space::Empty);
            self.set(&play.to, space_from);

            if EXIT_SQUARES.contains(&play.to) && *turn == Color::White {
                return Ok(Status::WhiteWins);
            }

            self.captures(&play.to, &color_from)?;

            // Todo: Check for shield wall.

            // Todo: Check for a draw or black win.
        } else {
            return Err(anyhow::Error::msg("play: it isn't your turn"));
        }

        Ok(Status::Ongoing)
    }

    fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.x][vertex.y] = space;
    }

    #[must_use]
    pub fn show(&self) -> String {
        let letters = "   ABCDEFGHJKL";

        let mut board = String::new();
        board.push('\n');
        board.push_str(letters);
        board.push_str("\n  ┌");
        board.push_str(&"─".repeat(11));
        board.push('┐');
        board.push('\n');

        for (mut i, line) in self.spaces.iter().enumerate() {
            i = 11 - i;
            board.push_str(&format!("{i:2}"));
            board.push('│');

            for (j, space) in line.iter().enumerate() {
                if ((i, j) == (1, 0)
                    || (i, j) == (11, 0)
                    || (i, j) == (1, 10)
                    || (i, j) == (11, 10)
                    || (i, j) == (6, 5))
                    && *space == Space::Empty
                {
                    board.push('■');
                } else {
                    match space {
                        Space::Black => board.push('○'),
                        Space::Empty => board.push('.'),
                        Space::King => board.push('▲'),
                        Space::White => board.push('●'),
                    }
                }
            }

            board.push('│');
            board.push_str(&format!("{i:2}"));
            board.push('\n');
        }

        board.push_str("  └");
        board.push_str(&"─".repeat(11));
        board.push('┘');
        board.push('\n');
        board.push_str(letters);

        board
    }
}
