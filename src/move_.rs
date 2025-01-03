use anyhow::Context;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Move {
    pub from: Vertex,
    pub to: Vertex,
}

impl TryFrom<Vec<&str>> for Move {
    type Error = anyhow::Error;

    fn try_from(args: Vec<&str>) -> Result<Self, Self::Error> {
        if args.len() != 3 {
            return Err(anyhow::Error::msg("move: wrong number of arguments"));
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

        if let Some(ch) = chars.next() {
            let y = "abcdefghjkl"
                .find(ch)
                .context("the first letter is not a legal char")?;

            let mut x = chars.as_str().parse()?;

            if x < 12 {
                x = 11 - x;
                return Ok(Self { x, y });
            }
        }

        Err(anyhow::Error::msg("invalid coordinate"))
    }
}
