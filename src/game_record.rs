use std::path::Path;

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

#[derive(Debug)]
pub struct GameRecord {
    pub plays: Vec<Play>,
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
                let vertexes: Vec<_> = play.split_terminator('-').collect();

                if let (Ok(from), Ok(to)) = (
                    Vertex::try_from_(vertexes[0]),
                    Vertex::try_from_(vertexes[1]),
                ) {
                    let play = Play {
                        color: color.clone(),
                        from,
                        to,
                    };
                    plays.push(play);
                }
            }
        }

        let game_record = GameRecord {
            plays,
            status: Status::try_from(record.status.as_str())?,
        };

        game_records.push(game_record);
    }

    Ok(game_records)
}
