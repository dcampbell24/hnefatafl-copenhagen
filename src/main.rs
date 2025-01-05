use std::io;

use hnefatafl_copenhagen::{game::Game, message::Message};

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut game = Game::default();
    loop {
        if let Err(error) = stdin.read_line(&mut buffer) {
            print!("? {error}\n\n");
            buffer.clear();
            continue;
        }

        if let Some(comment_offset) = buffer.find('#') {
            buffer.replace_range(comment_offset.., "");
        }

        match Message::try_from(buffer.as_str()) {
            Err(error) => {
                print!("? {error}\n\n");
            }
            Ok(message) => match game.update(message) {
                Ok(message) => {
                    if let Some(message) = message {
                        print!("= {message}\n\n");
                    }
                }
                Err(error) => {
                    print!("? {error}\n\n");
                }
            },
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
