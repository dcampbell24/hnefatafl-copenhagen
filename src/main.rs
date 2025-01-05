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

        if let Some(string) = game.read_line(&buffer) {
            print!("{string}");
        }

        buffer.clear();
    }
}
