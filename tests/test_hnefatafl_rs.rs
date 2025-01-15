use std::path::Path;

use hnefatafl_copenhagen::{
    game::Game, game_record::game_records_from_path, message::Message, status::Status,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

#[test]
#[allow(clippy::cast_precision_loss)]
fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("tests/copenhagen.csv");

    let mut count = 0;
    let mut errors_already = 0;

    let records = game_records_from_path(copenhagen_csv)?;

    let results: Vec<_> = records
        .clone()
        // .into_iter()
        .into_par_iter()
        .enumerate()
        .map(|(i, record)| {
            let mut game = Game::default();

            for play in record.plays {
                let message = Message::Play(play);

                if let Err(error) = game.update(message) {
                    return Err((i, game, error));
                }
            }

            Ok((i, game))
        })
        .collect();

    let mut errors = Vec::new();
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
            Err((i, game, error)) => {
                if error.to_string()
                    == anyhow::Error::msg("play: you already reached that position").to_string()
                {
                    errors_already += 1;
                } else {
                    errors.push((i, game, error));
                }

                if i > count {
                    count = i;
                }
            }
        }
    }

    for (i, game, error) in errors.iter().take(5) {
        println!("{i}");
        println!("{game}");
        println!("{error}");
    }

    println!("errors: {:.4}", errors.len() as f64 / count as f64);

    println!(
        "\"play: you already reached that position\": {:.4}",
        f64::from(errors_already) / count as f64
    );

    Ok(())
}
