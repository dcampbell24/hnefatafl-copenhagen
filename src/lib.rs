use std::{
    collections::HashSet,
    fmt,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    path::Path,
    process::exit,
    str::FromStr,
};

use game::Game;
use game_record::{Captures, game_records_from_path};
use message::Message;
use play::{Plae, Vertex};
use status::Status;

pub mod accounts;
pub mod ai;
pub mod board;
pub mod color;
pub mod draw;
pub mod game;
pub mod game_record;
pub mod glicko;
pub mod message;
pub mod play;
pub mod rating;
pub mod role;
pub mod server_game;
pub mod space;
pub mod status;
pub mod time;

pub static HOME: &str = "hnefatafl-copenhagen";
pub static VERSION_ID: &str = "3b20c67e";

pub fn handle_error<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("error: {error}");
            exit(1)
        }
    }
}

/// # Errors
///
/// If the captures or game status don't match for an engine game and a record
/// game.
#[allow(clippy::missing_panics_doc)]
pub fn hnefatafl_rs() -> anyhow::Result<()> {
    let copenhagen_csv = Path::new("tests/copenhagen.csv");
    let mut count = 0;
    let records = game_records_from_path(copenhagen_csv)?;

    let results: Vec<_> = records
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, record)| {
            let mut game = Game::default();

            for (play, captures_1) in record.plays {
                let mut captures_2_set = HashSet::new();
                let mut captures_2 = Vec::new();
                let message = Message::Play(Plae::Play(play));

                match game.update(message) {
                    Ok(Some(message)) => {
                        for vertex in message.split_ascii_whitespace() {
                            let capture = Vertex::from_str(vertex)?;
                            captures_2_set.insert(capture);
                        }
                        if let Some(king) = game.board.find_the_king()? {
                            captures_2_set.remove(&king);
                        }
                        for vertex in captures_2_set {
                            captures_2.push(vertex);
                        }
                        captures_2.sort();
                        let captures_2 = Captures(captures_2);

                        if let Some(captures_1) = captures_1 {
                            assert_eq!(captures_1, captures_2);
                        } else if !captures_2.0.is_empty() {
                            return Err(anyhow::Error::msg(
                                "The engine reports captures, but the record says there are none.",
                            ));
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return Err(error);
                    }
                }
            }

            Ok((i, game))
        })
        .collect();

    for result in results {
        match result {
            Ok((i, game)) => {
                if game.status != Status::Ongoing {
                    assert_eq!(game.status, records[i].status);
                }
                if i > count {
                    count = i;
                }
            }
            Err(error) => {
                if error.to_string()
                    == anyhow::Error::msg("play: you already reached that position").to_string()
                {
                } else {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}

/// # Errors
///
/// If read fails.
pub fn read_response(reader: &mut BufReader<TcpStream>) -> anyhow::Result<String> {
    let mut reply = String::new();
    reader.read_line(&mut reply)?;
    print!("<- {reply}");
    Ok(reply)
}

/// # Errors
///
/// If write fails.
pub fn write_command(command: &str, stream: &mut TcpStream) -> anyhow::Result<()> {
    print!("-> {command}");
    stream.write_all(command.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fmt, str::FromStr};

    use crate::ai::{AI, AiBanal};

    use super::*;
    use board::{Board, STARTING_POSITION};
    use color::Color;
    use game::Game;
    use play::Vertex;
    use status::Status;

    fn assert_error_str<T: fmt::Debug>(result: anyhow::Result<T>, string: &str) {
        if let Err(error) = result {
            assert_eq!(error.to_string(), string);
        }
    }

    #[test]
    fn flood_fill_1() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            ".........X.",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....O.....",
            "....O.O....",
            "....O.O....",
            "....OKO....",
        ];

        let game = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        let vertex = Vertex::from_str("f1")?;
        assert!(game.board.flood_fill_white_wins(&vertex)?);

        Ok(())
    }

    #[test]
    fn flood_fill_2() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            ".........X.",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "....OO.....",
            "....O......",
            "....O.O....",
            "....OKO....",
        ];

        let game = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        let vertex = Vertex::from_str("f1")?;
        assert!(!game.board.flood_fill_white_wins(&vertex)?);

        Ok(())
    }

    // One

    #[test]
    fn starting_position() -> anyhow::Result<()> {
        let game = Game::default();
        assert_eq!(game.board, STARTING_POSITION.try_into()?);

        Ok(())
    }

    // Two

    #[test]
    fn first_turn() {
        let game = Game::default();
        assert_eq!(game.turn, Color::Black);
    }

    // Three

    #[test]
    fn move_orthogonally_1() -> anyhow::Result<()> {
        let board = [
            "...X.......",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "X..O......X",
            "...........",
            "...........",
            "...X.......",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        let mut result = game.read_line("play white d4 d1");
        assert!(result.is_err());
        assert_error_str(result, "play: you have to play through empty locations");

        result = game.read_line("play white d4 d11");
        assert!(result.is_err());
        assert_error_str(result, "play: you have to play through empty locations");

        result = game.read_line("play white d4 a4");
        assert!(result.is_err());
        assert_error_str(result, "play: you have to play through empty locations");

        result = game.read_line("play white d4 k4");
        assert!(result.is_err());
        assert_error_str(result, "play: you have to play through empty locations");

        Ok(())
    }

    #[test]
    fn move_orthogonally_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...O.......",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        // Play a junk move:
        let mut result = game.read_line("play white junk d1");
        assert!(result.is_err());
        assert_error_str(result, "invalid digit found in string");

        result = game.read_line("play white d4 junk");
        assert!(result.is_err());
        assert_error_str(result, "invalid digit found in string");

        // Diagonal play:
        result = game.read_line("play white d4 a3");
        assert!(result.is_err());
        assert_error_str(result, "play: you can only play in a straight line");

        // Play out of bounds:
        result = game.read_line("play white d4 m4");
        assert!(result.is_err());
        assert_error_str(result, "play: the first letter is not a legal char");

        result = game.read_line("play white d4 d12");
        assert!(result.is_err());
        assert_error_str(result, "play: invalid coordinate");

        result = game.read_line("play white d4 d0");
        assert!(result.is_err());
        assert_error_str(result, "play: invalid coordinate");

        // Don't move:
        result = game.read_line("play white d4 d4");
        assert!(result.is_err());
        assert_error_str(result, "play: you have to change location");

        // Move all the way to the right:
        let mut game_1 = game.clone();
        game_1.read_line("play white d4 a4")?;
        // Move all the way to the left:
        let mut game_2 = game.clone();
        game_2.read_line("play white d4 k4")?;
        // Move all the way up:
        let mut game_3 = game.clone();
        game_3.read_line("play white d4 d11")?;
        // Move all the way down:
        let mut game_4 = game.clone();
        game_4.read_line("play white d4 d1")?;

        Ok(())
    }

    // Four

    #[test]
    fn sandwich_capture_1() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...X.......",
            "...O.......",
            ".XO.OX.....",
            "...........",
            "...X.......",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...X.......",
            "...........",
            ".X.X.X.....",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black d2 d4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_2() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...X.......",
            "...K.......",
            ".XO.OX.....",
            "...........",
            "...X.......",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...X.......",
            "...K.......",
            ".X.X.X.....",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black d2 d4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_3() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....O.....",
            ".X.........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....X.....",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black b4 f4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_4() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..K........",
            "...........",
            "...........",
            "..X........",
            "..O........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..K........",
            "...........",
            "..O........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white c6 c4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_5() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            ".....X.....",
            ".O.........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            "...........",
            ".....O.....",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white b4 f4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_6() -> anyhow::Result<()> {
        let board_1 = [
            ".O.........",
            "...........",
            "..X........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "..X........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black c9 c11")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_7() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            ".....O.....",
            ".X.........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            ".....O.....",
            ".....X.....",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black b4 f4")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn sandwich_capture_8() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".O.O.......",
            "...........",
            "..X........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".OXO.......",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black c3 c5")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_1() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..O........",
            "...OOO.....",
            "...XXXO....",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...OOO.....",
            "..O...O....",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game_1.read_line("play white c3 c1")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            "...XXXO....",
            "...OOO.....",
            "..O........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_4 = [
            "..O...O....",
            "...OOO.....",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game_2.read_line("play white c9 c11")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_2() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "..O........",
            "XO.........",
            "XO.........",
            "XO.........",
            "O..........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "O..........",
            ".O.........",
            ".O.........",
            ".O.........",
            "O..........",
            "...........",
            "...........",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game_1.read_line("play white c7 a7")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "........O..",
            ".........OX",
            ".........OX",
            ".........OX",
            "..........O",
            "...........",
            "...........",
        ];

        let board_4 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "..........O",
            ".........O.",
            ".........O.",
            ".........O.",
            "..........O",
            "...........",
            "...........",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game_2.read_line("play white i7 k7")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_3() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "........XX.",
            ".....X..OK.",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "........XX.",
            ".......X.K.",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game_1.read_line("play black f1 h1")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".XX........",
            ".KO..X.....",
        ];

        let board_4 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".XX........",
            ".K.X.......",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            ..Default::default()
        };

        game_2.read_line("play black f1 d1")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_4() -> anyhow::Result<()> {
        let board_1 = [
            ".....X..OK.",
            "........XX.",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            ".......X.K.",
            "........XX.",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game_1.read_line("play black f11 h11")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            ".KO..X.....",
            ".XX........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_4 = [
            ".K.X.......",
            ".XX........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            ..Default::default()
        };

        game_2.read_line("play black f11 d11")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_5() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "X..........",
            "...........",
            "...........",
            "OX.........",
            "KX.........",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "X..........",
            ".X.........",
            "KX.........",
            "...........",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game_1.read_line("play black a6 a4")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            "...........",
            "KX.........",
            "OX.........",
            "...........",
            "...........",
            "X..........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_4 = [
            "...........",
            "KX.........",
            ".X.........",
            "X..........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            ..Default::default()
        };

        game_2.read_line("play black a6 a8")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_6() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..........X",
            "...........",
            "...........",
            ".........XO",
            ".........XK",
            "...........",
        ];

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..........X",
            ".........X.",
            ".........XK",
            "...........",
        ];

        let mut game_1 = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game_1.read_line("play black k6 k4")?;
        assert_eq!(game_1.board, board_2.try_into()?);

        let board_3 = [
            "...........",
            ".........XK",
            ".........XO",
            "...........",
            "...........",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_4 = [
            "...........",
            ".........XK",
            ".........X.",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_2 = game::Game {
            board: board_3.try_into()?,
            ..Default::default()
        };

        game_2.read_line("play black k6 k8")?;
        assert_eq!(game_2.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_7() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "........X.O",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            ".........XO",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black i10 j10")?;
        assert_eq!(game.board, board_2.try_into()?);

        let board_3 = [
            "...........",
            "..........X",
            "........X.O",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_4 = [
            "...........",
            "..........X",
            ".........XO",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_3.try_into()?,
            ..Default::default()
        };

        game.read_line("play black i9 j9")?;
        assert_eq!(game.board, board_4.try_into()?);

        Ok(())
    }

    #[test]
    fn shield_wall_nope() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "........X.O",
            ".........XO",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let board_2 = [
            "...........",
            ".........XO",
            ".........XO",
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board_1.try_into()?,
            ..Default::default()
        };

        game.read_line("play black i10 j10")?;
        assert_eq!(game.board, board_2.try_into()?);

        Ok(())
    }

    // Five

    #[test]
    fn kings_1() {
        let board = [
            "KK.........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let result: anyhow::Result<Board> = board.try_into();
        assert!(result.is_err());
        assert_error_str(result, "You can only have one king!");
    }

    #[test]
    fn kings_2() -> anyhow::Result<()> {
        let board = [
            ".X.........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        let result = game.read_line("play black b11 a11");
        assert!(result.is_err());
        assert_error_str(
            result,
            "play: only the king may move to a restricted square",
        );

        Ok(())
    }

    #[test]
    fn kings_3() -> anyhow::Result<()> {
        let board_1 = [
            "K..........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];
        let _board: Board = board_1.try_into()?;

        let board_2 = [
            "X..........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let result: anyhow::Result<Board> = board_2.try_into();
        assert!(result.is_err());
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        Ok(())
    }

    #[test]
    fn kings_4() -> anyhow::Result<()> {
        let board_1 = [
            "..........K",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];
        let _board: Board = board_1.try_into()?;

        let board_2 = [
            "..........X",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let result: anyhow::Result<Board> = board_2.try_into();
        assert!(result.is_err());
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        Ok(())
    }

    #[test]
    fn kings_5() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];
        let _board: Board = board_1.try_into()?;

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....X.....",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let result: anyhow::Result<Board> = board_2.try_into();
        assert!(result.is_err());
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        Ok(())
    }

    #[test]
    fn kings_6() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "K..........",
        ];
        let _board: Board = board_1.try_into()?;

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "X..........",
        ];

        let result: anyhow::Result<Board> = board_2.try_into();
        assert!(result.is_err());
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        Ok(())
    }

    #[test]
    fn kings_7() -> anyhow::Result<()> {
        let board_1 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..........K",
        ];
        let _board: Board = board_1.try_into()?;

        let board_2 = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "..........X",
        ];

        let result: anyhow::Result<Board> = board_2.try_into();
        assert!(result.is_err());
        assert_error_str(result, "Only the king is allowed on restricted squares!");

        Ok(())
    }

    // Six

    #[test]
    fn white_wins_exit() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
        ];

        let mut game_1 = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };
        let mut game_2 = game_1.clone();

        game_1.read_line("play white f1 k1")?;
        assert_eq!(game_1.status, Status::WhiteWins);
        game_2.read_line("play white f1 a1")?;
        assert_eq!(game_2.status, Status::WhiteWins);

        let board = [
            ".....K.....",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game_1 = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };
        let mut game_2 = game_1.clone();

        game_1.read_line("play white f11 k11")?;
        assert_eq!(game_1.status, Status::WhiteWins);
        game_2.read_line("play white f11 a11")?;
        assert_eq!(game_2.status, Status::WhiteWins);

        Ok(())
    }

    #[test]
    fn white_wins_escape_fort_1() -> anyhow::Result<()> {
        let board = [
            "....O.O....",
            "....OKO....",
            "....OO.....",
            "....XX.....",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white f10 f11")?;
        assert_eq!(game.status, Status::WhiteWins);

        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "....XX.....",
            "....OO.....",
            "....OKO....",
            "....O.O....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };
        game.read_line("play white f2 f1")?;
        assert_eq!(game.status, Status::WhiteWins);

        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "OOOX.......",
            ".KOX.......",
            "OO.........",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };
        game.read_line("play white b6 a6")?;
        assert_eq!(game.status, Status::WhiteWins);

        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            ".......XOOO",
            ".......XOK.",
            ".........OO",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white j6 k6")?;
        assert_eq!(game.status, Status::WhiteWins);

        Ok(())
    }

    #[test]
    fn white_wins_escape_fort_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "........XOO",
            "........OK.",
            ".......X.OO",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white j6 k6")?;
        assert_eq!(game.status, Status::Ongoing);

        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "........XOO",
            "........OK.",
            ".........OO",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white j6 k6")?;
        assert_eq!(game.status, Status::WhiteWins);

        Ok(())
    }

    #[test]
    fn white_wins_escape_fort_3() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "........X.O",
            ".........O.",
            ".......X.OK",
            "..........O",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white k5 k6")?;
        assert_eq!(game.status, Status::WhiteWins);

        Ok(())
    }

    // Seven

    #[test]
    fn kings_captured_1() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            ".....X.....",
            "...........",
            "....XKX....",
            ".....X.....",
            "...........",
            "...........",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black f8 f7")?;
        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn kings_captured_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "....XKX....",
            "...........",
            ".....X.....",
            "...........",
            "...........",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black f3 f4")?;
        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn kings_captured_3() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "....X......",
            "...XKX.....",
            "...........",
            "....X......",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black e1 e2")?;
        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn kings_captured_surround_1() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            ".....XXX...",
            "....X...XX.",
            ".X...O....X",
            "..X.......X",
            ".X.O...O..X",
            ".X..OK...X.",
            "..X...O.X..",
            "...XXX.X...",
            "......X....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black b7 c7")?;
        assert_eq!(game.status, Status::Ongoing);
        game.read_line("play white f7 g7")?;
        assert_eq!(game.status, Status::Ongoing);
        game.read_line("play black c7 d7")?;
        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn kings_captured_surround_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...O.......",
            ".....XXX...",
            "....X...XX.",
            "..X..O....X",
            "..X.......X",
            ".X.O...O..X",
            ".X..OK...X.",
            "..X...O.X..",
            "...XXX.X...",
            "......X....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black c7 d7")?;
        assert_eq!(game.status, Status::Ongoing);

        Ok(())
    }

    // Eight

    #[test]
    fn can_not_repeat_moves() -> anyhow::Result<()> {
        let mut game = Game::default();

        game.read_line("play black f2 f3")?;
        game.read_line("play white f4 g4")?;
        game.read_line("play black f3 f2")?;

        let result = game.read_line("play white g4 f4");
        assert!(result.is_err());
        assert_error_str(result, "play: you already reached that position");

        Ok(())
    }

    // Nine

    #[test]
    fn black_automatically_loses() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            ".....X.....",
            "...........",
            ".....O.....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white f1 f2")?;
        assert_eq!(game.status, Status::WhiteWins);

        Ok(())
    }

    #[test]
    fn white_automatically_loses_1() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....X.....",
            ".X..XKX....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        game.read_line("play black b1 b2")?;
        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn white_automatically_loses_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....X.....",
            "....X.X....",
            ".X..XKX....",
        ];

        let mut game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        game.read_line("play white f1 f2")?;
        game.read_line("play black b1 b2")?;
        game.read_line("play white f2 f1")?;
        game.read_line("play black b2 b1")?;

        assert_eq!(game.status, Status::BlackWins);

        Ok(())
    }

    #[test]
    fn exit_one_1() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".X...K...X.",
        ];

        let game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        assert!(!game.exit_one());

        Ok(())
    }

    #[test]
    fn exit_one_2() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
        ];

        let game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        assert!(game.exit_one());

        Ok(())
    }

    #[test]
    fn exit_one_3() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
            "...........",
        ];

        let game = game::Game {
            board: board.try_into()?,
            turn: Color::White,
            ..Default::default()
        };

        assert!(!game.exit_one());

        Ok(())
    }

    #[test]
    fn exit_one_4() -> anyhow::Result<()> {
        let board = [
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            "...........",
            ".....K.....",
        ];

        let game = game::Game {
            board: board.try_into()?,
            ..Default::default()
        };

        assert!(!game.exit_one());

        Ok(())
    }

    #[test]
    fn someone_wins() -> anyhow::Result<()> {
        let mut game = Game::default();
        let mut ai: Box<dyn AI> = Box::new(AiBanal);

        while let Some(play) = game.generate_move(&mut ai) {
            game.play(&play)?;
        }

        Ok(())
    }
}
