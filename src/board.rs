use std::fmt;

use anyhow::Context;

use crate::{
    color::Color,
    play::{Play, Vertex},
    status::Status,
};

use super::space::Space;

pub const STARTING_POSITION: [&str; 11] = [
    "...XXXXX...",
    ".....X.....",
    "...........",
    "X....O....X",
    "X...OOO...X",
    "XX.OOKOO.XX",
    "X...OOO...X",
    "X....O....X",
    "...........",
    ".....X.....",
    "...XXXXX...",
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
    spaces: [[Space; 11]; 11],
}

impl Default for Board {
    fn default() -> Self {
        STARTING_POSITION.try_into().unwrap()
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for line in self.spaces {
            write!(f, r#"""#)?;
            for space in line {
                match space {
                    Space::Black => write!(f, "X")?,
                    Space::Empty => write!(f, ".")?,
                    Space::King => write!(f, "K")?,
                    Space::White => write!(f, "O")?,
                }
            }
            writeln!(f, r#"""#)?;
        }

        Ok(())
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters = "   ABCDEFGHJKL";
        let bar = "─".repeat(11);

        writeln!(f, "\n{letters}\n  ┌{bar}┐")?;
        for (mut i, line) in self.spaces.iter().enumerate() {
            i = 11 - i;

            write!(f, "{i:2}│")?;
            for (j, space) in line.iter().enumerate() {
                if ((i, j) == (1, 0)
                    || (i, j) == (11, 0)
                    || (i, j) == (1, 10)
                    || (i, j) == (11, 10)
                    || (i, j) == (6, 5))
                    && *space == Space::Empty
                {
                    write!(f, "■")?;
                } else {
                    write!(f, "{space}")?;
                }
            }
            writeln!(f, "│{i:2}")?;
        }
        write!(f, "  └{bar}┘\n{letters}")
    }
}

impl TryFrom<[&str; 11]> for Board {
    type Error = anyhow::Error;

    fn try_from(value: [&str; 11]) -> anyhow::Result<Self> {
        let mut spaces = [[Space::Empty; 11]; 11];
        let mut kings = 0;

        for (y, row) in value.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let space = ch.try_into()?;
                match space {
                    Space::Black | Space::White => {
                        let vertex = Vertex { x, y };
                        if RESTRICTED_SQUARES.contains(&vertex) {
                            return Err(anyhow::Error::msg(
                                "Only the king is allowed on restricted squares!",
                            ));
                        }
                    }
                    Space::Empty => {}
                    Space::King => {
                        kings += 1;
                        if kings > 1 {
                            return Err(anyhow::Error::msg("You can only have one king!"));
                        }
                    }
                }

                spaces[y][x] = space;
            }
        }

        Ok(Self { spaces })
    }
}

impl Board {
    fn captures(&mut self, play_to: &Vertex, color_from: &Color) -> anyhow::Result<()> {
        if let Some(up_1) = play_to.up() {
            let space = self.get(&up_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(up_2) = up_1.up() {
                    if (RESTRICTED_SQUARES.contains(&up_2) && self.get(&up_2)? != Space::King)
                        || self.get(&up_2)?.color() == *color_from
                    {
                        self.set_if_not_king(&up_1, Space::Empty)?;
                    }
                }
            }
        }

        if let Some(left_1) = play_to.left() {
            let space = self.get(&left_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(left_2) = left_1.left() {
                    if (RESTRICTED_SQUARES.contains(&left_2) && self.get(&left_2)? != Space::King)
                        || self.get(&left_2)?.color() == *color_from
                    {
                        self.set_if_not_king(&left_1, Space::Empty)?;
                    }
                }
            }
        }

        if let Some(down_1) = play_to.down() {
            let space = self.get(&down_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(down_2) = down_1.down() {
                    if (RESTRICTED_SQUARES.contains(&down_2) && self.get(&down_2)? != Space::King)
                        || self.get(&down_2)?.color() == *color_from
                    {
                        self.set_if_not_king(&down_1, Space::Empty)?;
                    }
                }
            }
        }

        if let Some(right_1) = play_to.right() {
            let space = self.get(&right_1)?;
            if space != Space::King && space.color() == color_from.opposite() {
                if let Some(right_2) = right_1.right() {
                    if (RESTRICTED_SQUARES.contains(&right_2) && self.get(&right_2)? != Space::King)
                        || self.get(&right_2)?.color() == *color_from
                    {
                        self.set_if_not_king(&right_1, Space::Empty)?;
                    }
                }
            }
        }

        Ok(())
    }

    // y counts up going down.
    #[allow(clippy::too_many_lines)]
    fn captures_shield_wall(&mut self, color_from: &Color) -> anyhow::Result<()> {
        // bottom row
        for x_1 in 1..11 {
            let vertex_1 = Vertex { x: x_1, y: 10 };
            if self.get(&vertex_1)?.color() == *color_from {
                let mut count = 0;
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 10 };
                    let vertex_3 = Vertex { x: x_2, y: 9 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 10 };
                let color = self.get(&vertex)?.color();
                if count > 0 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for x_2 in start..finish {
                        self.set_if_not_king(&Vertex { x: x_2, y: 10 }, Space::Empty)?;
                    }
                }
            }
        }

        // top row
        for x_1 in 1..11 {
            let vertex_1 = Vertex { x: x_1, y: 0 };
            if self.get(&vertex_1)?.color() == *color_from {
                let mut count = 0;
                let start = x_1 + 1;

                for x_2 in start..11 {
                    let vertex_2 = Vertex { x: x_2, y: 0 };
                    let vertex_3 = Vertex { x: x_2, y: 1 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: finish, y: 0 };
                let color = self.get(&vertex)?.color();
                if count > 0 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for x_2 in start..finish {
                        self.set_if_not_king(&Vertex { x: x_2, y: 0 }, Space::Empty)?;
                    }
                }
            }
        }

        // left row
        for y_1 in 1..11 {
            let vertex_1 = Vertex { x: 0, y: y_1 };
            if self.get(&vertex_1)?.color() == *color_from {
                let mut count = 0;
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 0, y: y_2 };
                    let vertex_3 = Vertex { x: 1, y: y_2 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 0, y: finish };
                let color = self.get(&vertex)?.color();
                if count > 0 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for y_2 in start..finish {
                        self.set_if_not_king(&Vertex { x: 0, y: y_2 }, Space::Empty)?;
                    }
                }
            }
        }

        // right row
        for y_1 in 1..11 {
            let vertex_1 = Vertex { x: 10, y: y_1 };
            if self.get(&vertex_1)?.color() == *color_from {
                let mut count = 0;
                let start = y_1 + 1;

                for y_2 in start..11 {
                    let vertex_2 = Vertex { x: 10, y: y_2 };
                    let vertex_3 = Vertex { x: 9, y: y_2 };
                    let color_2 = self.get(&vertex_2)?.color();
                    let color_3 = self.get(&vertex_3)?.color();
                    if color_2 == color_from.opposite() && color_3 == *color_from {
                        count += 1;
                    } else {
                        break;
                    }
                }

                let finish = start + count;
                let vertex = Vertex { x: 10, y: finish };
                let color = self.get(&vertex)?.color();
                if count > 0 && (color == *color_from || RESTRICTED_SQUARES.contains(&vertex)) {
                    for y_2 in start..finish {
                        self.set_if_not_king(&Vertex { x: 10, y: y_2 }, Space::Empty)?;
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
            .get(vertex.y)
            .context("get: index is out of y bounds")?;

        Ok(*column
            .get(vertex.x)
            .context("get: index is out of x bounds")?)
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

        if color_from == Color::Colorless {
            return Err(anyhow::Error::msg("play: you didn't select a color"));
        } else if *turn != color_from {
            return Err(anyhow::Error::msg("play: it isn't your turn"));
        }

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
        self.captures_shield_wall(&color_from)?;

        // Todo: Check for shield wall.
        // Todo: Check for a draw or black win.

        Ok(Status::Ongoing)
    }

    fn set(&mut self, vertex: &Vertex, space: Space) {
        self.spaces[vertex.y][vertex.x] = space;
    }

    fn set_if_not_king(&mut self, vertex: &Vertex, space: Space) -> anyhow::Result<()> {
        if self.get(vertex)? != Space::King {
            self.set(vertex, space);
        }

        Ok(())
    }
}
