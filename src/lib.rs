pub mod board;
pub mod color;
pub mod game;
pub mod message;
pub mod play;
pub mod space;
pub mod status;
pub mod time;

#[cfg(test)]
mod tests {
    use std::fmt;

    use board::STARTING_POSITION;
    use color::Color;
    use game::Game;
    use status::Status;

    use super::*;

    fn assert_error_str<T: fmt::Debug>(result: anyhow::Result<T>, string: &str) {
        if let Err(error) = result {
            assert_eq!(error.to_string(), string);
        }
    }

    #[test]
    fn starting_position() {
        let game = Game::default();
        assert_eq!(game.board, STARTING_POSITION.into());
    }

    #[test]
    fn first_turn() {
        let game = Game::default();
        assert_eq!(game.turn, Color::Black);
    }

    #[test]
    fn move_orthogonally() {
        let board = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   O       ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game = Game {
            board: board.into(),
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::White,
        };

        // Play a junk move:
        let mut result = game.read_line("play junk d1");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "invalid digit found in string");

        result = game.read_line("play d4 junk");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "invalid digit found in string");

        // Diagonal play:
        result = game.read_line("play d4 a3");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "play: you can only play in a straight line");

        // Play out of bounds:
        result = game.read_line("play d4 m4");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "play: the first letter is not a legal char");

        result = game.read_line("play d4 d12");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "play: invalid coordinate");

        result = game.read_line("play d4 d0");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Index is out of x bounds.");

        // Don't move:
        result = game.read_line("play d4 d4");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "play: you have to change location");

        // Move all the way to the right:
        let mut game_1 = game.clone();
        assert_eq!(game_1.read_line("play d4 a4").is_err(), false);
        // Move all the way to the left:
        let mut game_2 = game.clone();
        assert_eq!(game_2.read_line("play d4 l4").is_err(), false);
        // Move all the way up:
        let mut game_3 = game.clone();
        assert_eq!(game_3.read_line("play d4 d11").is_err(), false);
        // Move all the way down:
        let mut game_4 = game.clone();
        assert_eq!(game_4.read_line("play d4 d1").is_err(), false);
    }
}
