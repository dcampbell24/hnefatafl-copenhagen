use std::{
    io::{self, BufReader},
    net::TcpStream,
    process::{Command, ExitStatus},
};

use clap::command;
use clap::{self, Parser};

use hnefatafl_copenhagen::{game::Game, read_response, write_command};

/// Hnefatafl Copenhagen
///
/// This plays the game locally using the Hnefatafl Text Protocol.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Displays the game
    #[arg(default_value_t = false, long)]
    display_game: bool,

    /// Listen for HTP drivers on host and port
    #[arg(long, value_name = "host:port")]
    tcp: Option<String>,
}

/// # Errors
///
/// If the command `clear_screen()` fails.
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(tcp) = args.tcp {
        let address = tcp.as_str();
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

        return Ok(());
    }

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut game = Game::default();

    if args.display_game {
        #[cfg(any(target_family = "unix", target_family = "windows"))]
        clear_screen()?;
        println!("{game}\n");
        println!("Enter 'list_commands' for a list of commands.");
    }

    loop {
        if let Err(error) = stdin.read_line(&mut buffer) {
            println!("? {error}\n");
            buffer.clear();
            continue;
        }
        let result = game.read_line(&buffer);

        if args.display_game {
            #[cfg(any(target_family = "unix", target_family = "windows"))]
            clear_screen()?;
            println!("{game}\n");
        }

        match result {
            Err(error) => println!("? {error}\n"),
            Ok(message) => {
                if let Some(message) = message {
                    println!("= {message}\n");
                }
            }
        }

        buffer.clear();
    }
}

fn clear_screen() -> anyhow::Result<ExitStatus> {
    #[cfg(target_family = "unix")]
    let exit_status = Command::new("clear").status()?;

    #[cfg(target_family = "windows")]
    let exit_status = Command::new("cls").status()?;

    Ok(exit_status)
}
