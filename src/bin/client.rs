use std::io::BufReader;
use std::net::TcpStream;

use hnefatafl_copenhagen::game::Game;
use hnefatafl_copenhagen::{read_response, write_command};

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("localhost:8000")?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut game = Game::default();

    for i in 0..10 {
        println!("\n*** turn {} ***", i + 1);

        let message = read_response(&mut reader)?;

        if let Some(word) = message
            .as_str()
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .first()
        {
            match *word {
                "play" => {
                    game.read_line(&message)?;
                }
                "generate_move" => {
                    if let Some(message) = game.read_line(&message)? {
                        write_command(&format!("play {message}\n"), &mut stream)?;
                    }
                }
                _ => unreachable!("You can't get here!"),
            }
        }
    }

    Ok(())
}
