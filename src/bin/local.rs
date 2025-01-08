use std::{
    io,
    process::{Command, ExitStatus},
};

use hnefatafl_copenhagen::game::Game;

pub fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut game = Game::default();

    clear_screen().unwrap();
    println!("{game}\n");

    loop {
        if let Err(error) = stdin.read_line(&mut buffer) {
            println!("? {error}\n");
            buffer.clear();
            continue;
        }
        let result = game.read_line(&buffer);

        #[cfg(any(target_family = "unix", target_family = "windows"))]
        clear_screen().unwrap();

        println!("{game}\n");
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
