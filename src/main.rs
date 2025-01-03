use std::io;

use hnefatafl_copenhagen::{/* board::Board, */ game::Game, message::Message, status::Status};

fn main() {
    // iced::run("Hnefatafl", Board::update, Board::view)

    let mut buffer = String::new();
    let stdin = io::stdin();

    let mut game = Game::default();
    loop {
        println!("{game}");

        if let Err(error) = stdin.read_line(&mut buffer) {
            println!("Error: {error}");
        }

        let status = game.status.clone();
        match Message::try_from(buffer.as_str()) {
            Err(error) => println!("Error: {error}"),
            Ok(message) => {
                if let Err(error) = game.update(message) {
                    println!("Error: {error}");
                }
            }
        }

        if status != game.status {
            match game.status {
                Status::BlackWins => println!("Black wins!"),
                Status::Draw => println!("It's a draw!"),
                Status::WhiteWins => println!("White wins!"),
                Status::Ongoing => unreachable!("The game can't go back to ongoing!"),
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
