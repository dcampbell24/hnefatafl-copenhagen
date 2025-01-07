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
    fn starting_position_1() -> anyhow::Result<()> {
        let game = Game::default();
        assert_eq!(game.board, STARTING_POSITION.try_into()?);

        Ok(())
    }

    #[test]
    fn first_turn_2() {
        let game = Game::default();
        assert_eq!(game.turn, Color::Black);
    }

    #[test]
    fn move_orthogonally_3() -> anyhow::Result<()> {
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

        let mut game = Game::default();
        game.board = board.try_into()?;
        game.turn = Color::White;

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
    fn sandwich_capture_4a() -> anyhow::Result<()> {
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

        let mut game_1 = Game::default();

        game_1.board = board_1a.try_into()?;
        game_1.read_line("play d2 d4")?;
        assert_eq!(game_1.board, board_1b.try_into()?);

        let board_1aa = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   X       ",
            "   K       ",
            " XO OX     ",
            "           ",
            "   X       ",
            "           ",
        ];

        let board_1bb = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   X       ",
            "   K       ",
            " X X X     ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_1a = Game::default();
        game_1a.board = board_1aa.try_into()?;
        game_1a.read_line("play d2 d4")?;
        assert_eq!(game_1a.board, board_1bb.try_into()?);

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

        let mut game_2 = Game::default();
        game_2.board = board_2a.try_into()?;
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

        let mut game_3 = Game::default();
        game_3.board = board_3a.try_into()?;
        game_3.turn = Color::White;
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

        let mut game_4 = Game::default();
        game_4.board = board_4a.try_into()?;
        game_4.turn = Color::White;
        game_4.read_line("play b4 f4")?;
        assert_eq!(game_4.board, board_4b.try_into()?);

        // Todo: finish the rest...
        let board_5a = [
            " O         ",
            "           ",
            "  X        ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let board_5b = [
            "  X        ",
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

        let mut game_5 = Game::default();
        game_5.board = board_5a.try_into()?;
        game_5.read_line("play c9 c11")?;
        assert_eq!(game_5.board, board_5b.try_into()?);

        let board_6a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     K     ",
            "     O     ",
            " X         ",
            "           ",
            "           ",
            "           ",
        ];

        let board_6b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "     K     ",
            "     O     ",
            "     X     ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_6 = Game::default();
        game_6.board = board_6a.try_into()?;
        game_6.read_line("play b4 f4")?;
        assert_eq!(game_6.board, board_6b.try_into()?);

        let board_7a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            " O O       ",
            "           ",
            "  X        ",
            "           ",
            "           ",
        ];

        let board_7b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            " OXO       ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let mut game_7 = Game::default();
        game_7.board = board_7a.try_into()?;
        game_7.read_line("play c3 c5")?;
        assert_eq!(game_7.board, board_7b.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_4b() -> anyhow::Result<()> {
        let board_1a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "  O        ",
            "   OOO     ",
            "   XXXO    ",
        ];

        let board_1b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "   OOO     ",
            "  O   O    ",
        ];

        let mut game_1 = Game::default();
        game_1.board = board_1a.try_into()?;
        game_1.turn = Color::White;
        game_1.read_line("play c3 c1")?;
        assert_eq!(game_1.board, board_1b.try_into()?);

        let board_2a = [
            "   XXXO    ",
            "   OOO     ",
            "  O        ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        let board_2b = [
            "  O   O    ",
            "   OOO     ",
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

        let mut game_2 = Game::default();
        game_2.board = board_2a.try_into()?;
        game_2.turn = Color::White;
        game_2.read_line("play c9 c11")?;
        assert_eq!(game_2.board, board_2b.try_into()?);

        let board_3a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "  O        ",
            "XO         ",
            "XO         ",
            "XO         ",
            "O          ",
            "           ",
            "           ",
        ];

        let board_3b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "O          ",
            " O         ",
            " O         ",
            " O         ",
            "O          ",
            "           ",
            "           ",
        ];

        let mut game_3 = Game::default();
        game_3.board = board_3a.try_into()?;
        game_3.turn = Color::White;
        game_3.read_line("play c7 a7")?;
        assert_eq!(game_3.board, board_3b.try_into()?);

        let board_4a = [
            "           ",
            "           ",
            "           ",
            "           ",
            "        O  ",
            "         OX",
            "         OX",
            "         OX",
            "          O",
            "           ",
            "           ",
        ];

        let board_4b = [
            "           ",
            "           ",
            "           ",
            "           ",
            "          O",
            "         O ",
            "         O ",
            "         O ",
            "          O",
            "           ",
            "           ",
        ];

        let mut game_4 = Game::default();
        game_4.board = board_4a.try_into()?;
        game_4.turn = Color::White;
        game_4.read_line("play j7 l7")?;
        assert_eq!(game_4.board, board_4b.try_into()?);

        // Handle the restricted squares,

        Ok(())
    }

    #[test]
    fn kings_5() -> anyhow::Result<()> {
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

        let mut game_8 = Game::default();
        game_8.board = board.try_into()?;
        let result = game_8.read_line("play b11 a11");
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

        let mut result: anyhow::Result<Board> = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");

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
            "          X",
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

        result = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");

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
            "     X     ",
            "           ",
            "           ",
            "           ",
            "           ",
            "           ",
        ];

        result = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");

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
            "X          ",
        ];

        result = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");

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
            "          X",
        ];

        result = board.try_into();
        assert_eq!(result.is_err(), true);
        assert_error_str(result, "Only the king is allowed on restricted squares!");
        Ok(())
    }

    #[test]
    fn white_wins_6a() -> anyhow::Result<()> {
        let board = [
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
            "     K     ",
        ];

        let mut game = Game::default();
        game.board = board.try_into()?;
        game.turn = Color::White;

        game.read_line("play f1 l1")?;
        assert_eq!(game.status, Status::WhiteWins);

        Ok(())
    }
}
