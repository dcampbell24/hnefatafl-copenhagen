use std::{collections::HashSet, path::Path};

use hnefatafl_copenhagen::{
    game::Game,
    game_record::{game_records_from_path, Captures},
    message::Message,
    play::{Plae, Vertex},
    status::Status,
};

#[test]
#[allow(clippy::cast_precision_loss)]
fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("tests/copenhagen.csv");

    let mut count = 0;
    let mut errors_already = 0;

    let records = game_records_from_path(copenhagen_csv)?;

    let results: Vec<_> = records
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, record)| {
            let mut game = Game::default();

            for (play, captures_1) in record.plays {
                let mut captures_2_set = HashSet::new();
                let mut captures_2 = Vec::new();
                let message = Message::Play(Plae::Play(play));

                match game.update(message) {
                    Ok(Some(message)) => {
                        for vertex in message.split_ascii_whitespace() {
                            let capture = Vertex::try_from(vertex)?;
                            captures_2_set.insert(capture);
                        }
                        if let Some(king) = game.board.find_the_king()? {
                            captures_2_set.remove(&king);
                        }
                        for vertex in captures_2_set {
                            captures_2.push(vertex);
                        }
                        captures_2.sort();
                        let captures_2 = Captures(captures_2);

                        if let Some(captures_1) = captures_1 {
                            assert_eq!(captures_1, captures_2);
                        } else if !captures_2.0.is_empty() {
                            return Err(anyhow::Error::msg(
                                "The engine reports captures, but the record says there are none.",
                            ));
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(error);
                    }
                }
            }

            Ok((i, game))
        })
        .collect();

    for result in results {
        match result {
            Ok((i, game)) => {
                if game.status != Status::Ongoing {
                    assert_eq!(game.status, records[i].status);
                }
                if i > count {
                    count = i;
                }
            }
            Err(error) => {
                if error.to_string()
                    == anyhow::Error::msg("play: you already reached that position").to_string()
                {
                    errors_already += 1;
                } else {
                    return Err(error);
                }
            }
        }
    }

    println!(
        "\"play: you already reached that position\": {:.4}",
        f64::from(errors_already) / count as f64
    );

    Ok(())
}
