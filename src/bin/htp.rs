use std::io;

use hnefatafl_copenhagen::game::Game;

pub fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut game = Game::default();

    loop {
        if let Err(error) = stdin.read_line(&mut buffer) {
            print!("? {error}\n\n");
            buffer.clear();
            continue;
        }

        match game.read_line(&buffer) {
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
