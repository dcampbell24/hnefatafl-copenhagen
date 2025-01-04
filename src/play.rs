use anyhow::Context;

pub const BOARD_LETTERS: &str = "abcdefghjkl";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Play {
    pub from: Vertex,
    pub to: Vertex,
}

impl TryFrom<Vec<&str>> for Play {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        if args.len() != 3 {
            return Err(anyhow::Error::msg("play: wrong number of arguments"));
        }

        Ok(Self {
            from: Vertex::try_from(args[1])?,
            to: Vertex::try_from(args[2])?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vertex {
    pub x: usize,
    pub y: usize,
}

impl TryFrom<&str> for Vertex {
    type Error = anyhow::Error;

    fn try_from(vertex: &str) -> anyhow::Result<Self> {
        let mut chars = vertex.chars();

        if let Some(mut ch) = chars.next() {
            ch = ch.to_ascii_lowercase();
            let y = BOARD_LETTERS
                .find(ch)
                .context("play: the first letter is not a legal char")?;

            let mut x = chars.as_str().parse()?;

            if x < 12 {
                x = 11 - x;
                return Ok(Self { x, y });
            }
        }

        Err(anyhow::Error::msg("play: invalid coordinate"))
    }
}

impl Vertex {
    #[must_use]
    pub fn up(&self) -> Option<Vertex> {
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
    pub fn left(&self) -> Option<Vertex> {
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
    pub fn down(&self) -> Option<Vertex> {
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
    pub fn right(&self) -> Option<Vertex> {
        if self.y < 10 {
            Some(Vertex {
                x: self.x,
                y: self.y + 1,
            })
        } else {
            None
        }
    }
}
