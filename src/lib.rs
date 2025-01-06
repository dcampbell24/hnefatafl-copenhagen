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

    use board::{Board, STARTING_POSITION};
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
    fn starting_position() -> anyhow::Result<()> {
        let game = Game::default();
        assert_eq!(game.board, STARTING_POSITION.try_into()?);

        Ok(())
    }

    #[test]
    fn first_turn() {
        let game = Game::default();
        assert_eq!(game.turn, Color::Black);
    }

    #[test]
    fn move_orthogonally() -> anyhow::Result<()> {
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
            board: board.try_into()?,
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
        assert_error_str(result, "get: index is out of x bounds");

        // Don't move:
        result = game.read_line("play d4 d4");
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "play: you have to change location");

        // Move all the way to the right:
        let mut game_1 = game.clone();
        game_1.read_line("play d4 a4")?;
        // Move all the way to the left:
        let mut game_2 = game.clone();
        game_2.read_line("play d4 l4")?;
        // Move all the way up:
        let mut game_3 = game.clone();
        game_3.read_line("play d4 d11")?;
        // Move all the way down:
        let mut game_4 = game.clone();
        game_4.read_line("play d4 d1")?;

        Ok(())
    }

    #[test]
    fn sandwich_capture() -> anyhow::Result<()> {
        let board_1a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   X       ",
            "   O       ",
            " XO OX     ",
            "           ",
            "   X       ",
            "           ",
        ];

        let board_1b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   X       ",
            "           ",
            " X X X     ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_1 = Game {
            board: board_1a.try_into()?,
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::Black,
        };

        game_1.read_line("play d2 d4")?;
        assert_eq!(game_1.board, board_1b.try_into()?);

        let board_2a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     O     ",
            " X         ",
            "           ",
            "           ",
            "           ",
        ];

        let board_2b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     X     ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_2 = Game {
            board: board_2a.try_into()?,
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::Black,
        };

        game_2.read_line("play b4 f4")?;
        assert_eq!(game_2.board, board_2b.try_into()?);

        let board_3a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "  K        ",
            "           ",
            "           ",
            "  X        ",
            "  O        ",
            "           ",
        ];

        let board_3b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "  K        ",
            "           ",
            "  O        ",
            "           ",
        ];

        let mut game_3 = Game {
            board: board_3a.try_into()?,
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::White,
        };

        game_3.read_line("play c6 c4")?;
        assert_eq!(game_3.board, board_3b.try_into()?);

        let board_4a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     K     ",
            "     X     ",
            " O         ",
            "           ",
            "           ",
            "           ",
        ];

        let board_4b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     K     ",
            "           ",
            "     O     ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_4 = Game {
            board: board_4a.try_into()?,
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::White,
        };

        game_4.read_line("play b4 f4")?;
        assert_eq!(game_4.board, board_4b.try_into()?);

        Ok(())
    }

    #[test]
    fn kings() -> anyhow::Result<()> {
        let mut board = [
            "KK         ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let result: anyhow::Result<Board> = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "You can only have one king!");

        board = [
            "X          ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let result: anyhow::Result<Board> = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        board = [
            " X         ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game = Game {
            board: board.try_into()?,
            plays: Vec::new(),
            status: Status::default(),
            timer: None,
            black_time: None,
            white_time: None,
            turn: Color::Black,
        };
        let result = game.read_line("play b11 a11");
        assert_eq!(result.is_err(), true);
        assert_error_str(
            result,
            "play: only the king may move to a restricted square",
        );

        board = [
            "K          ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];
        let _board: Board = board.try_into()?;

        board = [
            "          K",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];
        let _board: Board = board.try_into()?;

        board = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     K     ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];
        let _board: Board = board.try_into()?;

        board = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "K          ",
        ];
        let _board: Board = board.try_into()?;

        board = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "          K",
        ];
        let _board: Board = board.try_into()?;

        Ok(())
    }
}
