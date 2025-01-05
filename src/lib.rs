use std::io;

use game::Game;
use message::Message;

pub mod board;
pub mod color;
pub mod game;
pub mod message;
pub mod play;
pub mod space;
pub mod status;
pub mod time;

pub fn hnefatafl_text_protocol() {
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
