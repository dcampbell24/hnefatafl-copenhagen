use std::{
    io,
    process::{Command, ExitStatus},
};

use clap::command;
use clap::{self, Parser};

use hnefatafl_copenhagen::game::Game;

/// Hnefatafl Copenhagen
///
/// This plays the game locally using the Hnefatafl Text Protocol.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Displays the game
    #[arg(default_value_t = false, long)]
    display_game: bool,
}

/// # Errors
///
/// If the command `clear_screen()` fails.
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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
