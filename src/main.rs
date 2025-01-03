use std::io;

use hnefatafl_copenhagen::{game::Game, message::Message};

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut game = Game::default();
    loop {
        // println!("{game}");

        if let Err(error) = stdin.read_line(&mut buffer) {
            print!("? {error}\n\n");
            buffer.clear();
            continue;
        }

        match Message::try_from(buffer.as_str()) {
            Err(error) => {
                print!("? {error}\n\n");
                buffer.clear();
                continue;
            }
            Ok(message) => {
                if let Err(error) = game.update(message) {
                    print!("? {error}\n\n");
                    buffer.clear();
                    continue;
                }
            }
        }

        buffer.clear();
    }
}

/*
/// Hnefatafl Text Protocol
struct Htp {
    id: Option<u64>,
    command: String,
    args: Vec<String>,
}
*/
