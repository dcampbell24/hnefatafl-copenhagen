use std::path::Path;

use hnefatafl_copenhagen::{
    game::Game, game_record::game_records_from_path, message::Message, status::Status,
};

#[test]
#[ignore = "it takes too long"]
fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("tests/copenhagen.csv");

    let mut count = 0;
    let mut errors = 0;

    let records = game_records_from_path(copenhagen_csv)?;
    for record in records {
        let mut game = Game::default();
        count += 1;
        if count >= 161 {
            break;
        }

        for play in record.plays {
            let message = Message::Play(play);

            match game.update(message) {
                Ok(Some(captures)) => print!("{captures}"),
                Err(error) => {
                    if error.to_string()
                        == anyhow::Error::msg("play: you already reached that position").to_string()
                    {
                        errors += 1;
                        break;
                    }

                    println!("{game}");
                    println!("{count}");
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
        "\"play: you already reached that position\": {:.4}",
        f64::from(errors) / f64::from(count)
    );

    Ok(())
}
