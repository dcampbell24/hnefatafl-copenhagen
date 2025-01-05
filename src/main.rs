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

        game.read_line(&buffer);

        buffer.clear();
    }
}
