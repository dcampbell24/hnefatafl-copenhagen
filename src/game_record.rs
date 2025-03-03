use std::{fmt, path::Path, str::FromStr};

use crate::{
    color::Color,
    play::{Play, Vertex},
    status::Status,
};

#[derive(Debug, serde::Deserialize)]
struct Record {
    moves: String,
    _black_captures: u64,
    _white_captures: u64,
    status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captures(pub Vec<Vertex>);

impl fmt::Display for Captures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for vertex in &self.0 {
            write!(f, "{vertex} ")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GameRecord {
    pub plays: Vec<(Play, Option<Captures>)>,
    pub status: Status,
}

/// # Errors
///
/// If the game records are invalid.
pub fn game_records_from_path(path: &Path) -> anyhow::Result<Vec<GameRecord>> {
    let mut game_records = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;

    for result in rdr.deserialize() {
        let record: Record = result?;
        let mut color = Color::White;
        let mut plays = Vec::new();

        for play in record.moves.split_ascii_whitespace() {
            color = color.opposite();

            if play.contains('-') {
                let vertexes: Vec<_> = play.split('-').collect();
                let vertex_1_captures: Vec<_> = vertexes[1].split('x').collect();

                if let (Ok(from), Ok(to)) = (
                    Vertex::from_str_(vertexes[0]),
                    Vertex::from_str_(vertex_1_captures[0]),
                ) {
                    let play = Play {
                        color: color.clone(),
                        from,
                        to,
                    };

                    if vertex_1_captures.get(1).is_some() {
                        let mut captures = Vec::new();
                        for capture in vertex_1_captures.into_iter().skip(1) {
                            let vertex = Vertex::from_str_(capture)?;
                            if !captures.contains(&vertex) {
                                captures.push(vertex);
                            }
                        }

                        captures.sort();
                        plays.push((play, Some(Captures(captures))));
                    } else {
                        plays.push((play, None));
                    }
                }
            }
        }

        let game_record = GameRecord {
            plays,
            status: Status::from_str(record.status.as_str())?,
        };

        game_records.push(game_record);
    }

    Ok(game_records)
}
