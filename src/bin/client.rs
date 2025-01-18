use std::io::BufReader;
use std::net::TcpStream;

use clap::command;
use clap::{self, Parser};

use hnefatafl_copenhagen::game::Game;
use hnefatafl_copenhagen::{read_response, write_command};

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens for HTP engines
/// to connect and then plays them against each other.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host and port
    #[arg(default_value = "localhost:8000", index = 1, value_name = "host:port")]
    host_port: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let address = args.host_port.as_str();
    let mut stream = TcpStream::connect(address)?;
    println!("connected to {address} ...");

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut game = Game::default();

    for i in 1..11 {
        println!("\n*** turn {i} ***");

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
