use std::{fmt, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::color::Color;

pub const BOARD_LETTERS: &str = "abcdefghjkl";
pub const BOARD_LETTERS_: &str = "abcdefghijk";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Plae {
    Play(Play),
    BlackResigns,
    WhiteResigns,
}

impl fmt::Display for Plae {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Play(play) => writeln!(f, "play {} {} {}", play.color, play.from, play.to),
            Self::BlackResigns => writeln!(f, "play black resigns _"),
            Self::WhiteResigns => writeln!(f, "play white resigns _"),
        }
    }
}

impl Plae {
    // Todo: can the player resign?
    /// # Errors
    ///
    /// If you try to convert an illegal character or you don't get vertex-vertex.
    pub fn try_from_(color: Color, play: &str) -> anyhow::Result<Self> {
        let Some((from, to)) = play.split_once('-') else {
            return Err(anyhow::Error::msg("expected: vertex-vertex"));
        };

        Ok(Self::Play(Play {
            color,
            from: Vertex::from_str_(from)?,
            to: Vertex::from_str_(to)?,
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Play {
    pub color: Color,
    pub from: Vertex,
    pub to: Vertex,
}

impl fmt::Display for Play {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} from {} to {}", self.color, self.from, self.to)
    }
}

impl TryFrom<Vec<&str>> for Plae {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        let error_str = "expected: 'play COLOR FROM TO' or 'play COLOR resign'";

        if args.len() == 3 {
            let color = Color::from_str(args[1])?;
            if args[2] == "resigns" {
                if color == Color::White {
                    return Ok(Self::WhiteResigns);
                }

                return Ok(Self::BlackResigns);
            }
        }

        if args.len() < 4 {
            return Err(anyhow::Error::msg(error_str));
        }

        Ok(Self::Play(Play {
            color: Color::from_str(args[1])?,
            from: Vertex::from_str(args[2])?,
            to: Vertex::from_str(args[3])?,
        }))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Vertex {
    pub x: usize,
    pub y: usize,
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            BOARD_LETTERS.chars().collect::<Vec<_>>()[self.x],
            11 - self.y
        )
    }
}

pub struct Captures(pub Vec<Vertex>);

impl fmt::Display for Captures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for vertex in &self.0 {
            write!(f, "{vertex} ")?;
        }

        Ok(())
    }
}

impl FromStr for Vertex {
    type Err = anyhow::Error;

    fn from_str(vertex: &str) -> anyhow::Result<Self> {
        let mut chars = vertex.chars();

        if let Some(mut ch) = chars.next() {
            ch = ch.to_ascii_lowercase();
            let x = BOARD_LETTERS
                .find(ch)
                .context("play: the first letter is not a legal char")?;

            let mut y = chars.as_str().parse()?;
            if y > 0 && y < 12 {
                y = 11 - y;
                return Ok(Self { x, y });
            }
        }

        Err(anyhow::Error::msg("play: invalid coordinate"))
    }
}

impl Vertex {
    #[must_use]
    pub fn fmt_other(&self) -> String {
        format!(
            "{}{}",
            BOARD_LETTERS_.chars().collect::<Vec<_>>()[self.x],
            11 - self.y
        )
    }

    /// # Errors
    ///
    /// If you try to convert an illegal character.
    pub fn from_str_(vertex: &str) -> anyhow::Result<Self> {
        let vertex: Vec<_> = vertex.split_terminator('x').collect();
        let mut chars = vertex[0].chars();

        if let Some(mut ch) = chars.next() {
            ch = ch.to_ascii_lowercase();
            let x = BOARD_LETTERS_
                .find(ch)
                .context("play: the first letter is not a legal char")?;

            let mut y = chars.as_str().parse()?;
            if y > 0 && y < 12 {
                y = 11 - y;
                return Ok(Self { x, y });
            }
        }

        Err(anyhow::Error::msg("play: invalid coordinate"))
    }

    #[must_use]
    pub fn up(&self) -> Option<Vertex> {
        if self.y > 0 {
            Some(Vertex {
                x: self.x,
                y: self.y - 1,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn left(&self) -> Option<Vertex> {
        if self.x > 0 {
            Some(Vertex {
                x: self.x - 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn down(&self) -> Option<Vertex> {
        if self.y < 10 {
            Some(Vertex {
                x: self.x,
                y: self.y + 1,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn right(&self) -> Option<Vertex> {
        if self.x < 10 {
            Some(Vertex {
                x: self.x + 1,
                y: self.y,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub fn touches_wall(&self) -> bool {
        self.x == 0 || self.x == 10 || self.y == 0 || self.y == 10
    }
}
