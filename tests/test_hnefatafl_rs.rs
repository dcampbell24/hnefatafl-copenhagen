use std::{fs::exists, path::Path};

use hnefatafl_copenhagen::{
    color::Color,
    game::Game,
    message::Message,
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
struct GameRecord {
    plays: Vec<Play>,
    status: Status,
}

fn game_records_from_path(path: &Path) -> anyhow::Result<Vec<GameRecord>> {
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

#[test]
#[ignore = "it takes too long"]
fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("../hnefatafl-rs/resources/test/games/copenhagen.csv");

    if exists(copenhagen_csv)? {
        let mut count = 0;
        let mut errors = 0;
        let mut index = 0;

        let records = game_records_from_path(copenhagen_csv)?;
        for record in records {
            let mut game = Game::default();
            index += 1;
            if index >= 161 {
                break;
            }

            for play in record.plays {
                count += 1;
                let message = Message::Play(play);

                match game.update(message) {
                    Ok(Some(captures)) => print!("{captures}"),
                    Err(error) => {
                        if error.to_string()
                            == anyhow::Error::msg("play: you already reached that position")
                                .to_string()
                        {
                            errors += 1;
                            break;
                        }

                        println!("{game}");
                        println!("{index}");
                        return Err(error);
                    }
                    _ => {}
                }
            }

            if game.status != Status::Ongoing {
                assert_eq!(game.status, record.status);
            }
        }

        println!(
            "\"play: you already reached that position\": {:02}",
            f64::from(errors) / f64::from(count)
        );
    }

    Ok(())
}
