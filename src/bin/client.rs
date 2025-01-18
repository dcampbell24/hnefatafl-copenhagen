use std::io::BufReader;
use std::net::TcpStream;

use hnefatafl_copenhagen::game::Game;
use hnefatafl_copenhagen::{read_response, write_command};

fn main() -> anyhow::Result<()> {
    let mut stream = TcpStream::connect("localhost:8000")?;

    let mut buf = String::new();

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut game = Game::default();

    for i in 0..10 {
        println!("{i}");

        let message = read_response(&mut reader)?;

        if let Some(word) = message
            .as_str()
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .first()
        {
            match *word {
                "play" => {
                    println!("word: {word}");
                    game.read_line(&message)?;
                }
                "generate_move" => {
                    println!("word: {word}");
                    if let Some(message) = game.read_line(&message)? {
                        write_command(&format!("play {message}\n"), &mut stream)?;
                    }
                }
                _ => unreachable!("You can't get here!"),
            }
        }

        buf.clear();
    }

    Ok(())
}

/*
fn send_command(
    command: &str,
    writer: &mut TcpStream,
    // reader: &mut BufReader<TcpStream>,
) -> anyhow::Result<()> /* -> String */ {
    print!("-> {command}");
    writer.write_all(command.as_bytes())?;

    Ok(())
    // let mut reply = String::new();
    // reader.read_line(&mut reply).unwrap();
    // reader.read_line(&mut reply).unwrap();
    // print!("<- {}", &reply);
    // reply
}
*/
