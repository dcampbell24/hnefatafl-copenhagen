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

#[test]
fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("../hnefatafl-rs/resources/test/games/copenhagen.csv");

    if exists(copenhagen_csv)? {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(copenhagen_csv)?;

        let mut index = 0;
        for result in rdr.deserialize() {
            let record: Record = result?;
            let mut color = Color::White;
            let mut plays = Vec::new();

            index += 1;

            if index != 100_000 {
                continue;
            }

            /*
            if index == 7  ||  index == 8 || index == 11 || index > 24 {
                continue;
            }
            */

            println!("{index}");

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

            let mut game = Game::default();
            for play in plays {
                // println!("{play}");
                let message = Message::Play(play);
                game.update(message)?;
            }

            println!("{}", game.board);
            assert_eq!(game.status, Status::try_from(record.status.as_str())?);
        }
    }

    Ok(())
}
