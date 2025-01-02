use std::io;

use hnefatafl::{/* board::Board, */ game::Game, message::Message};

fn main() -> anyhow::Result<()> {
    // iced::run("Hnefatafl", Board::update, Board::view)

    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut game = Game::default();
    loop {
        print!("{game}");

        stdin.read_line(&mut buffer)?;
        let message = Message::from(buffer.as_str());
        game.update(message)?;
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
