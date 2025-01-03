use std::{fmt, process::exit};

use crate::{
    board::Board,
    color::Color,
    message::{Message, COMMANDS},
    play::{Play, BOARD_LETTERS},
    status::Status,
};

#[derive(Debug, Default, Clone)]
pub struct Game {
    board: Board,
    pub plays: Vec<Play>,
    pub status: Status,
    pub turn: Color,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.board)?;
        writeln!(f, "plays: {:?}", self.plays)?;
        writeln!(f, "turn: {:?}", self.turn)
    }
}

impl Game {
    /// # Errors
    ///
    /// If the play is illegal.
    pub fn update(&mut self, message: Message) -> anyhow::Result<Option<String>> {
        match message {
            Message::Empty => Ok(None),
            Message::FinalStatus => match self.status {
                Status::BlackWins => Ok(Some("Black wins!".to_string())),
                Status::Draw => Ok(Some("It's a draw!".to_string())),
                Status::WhiteWins => Ok(Some("White wins!".to_string())),
                Status::Ongoing => Ok(Some("The game is ongoing".to_string())),
            },
            Message::GenerateMove => {
                for letter in BOARD_LETTERS.chars() {
                    for i in 1..12 {
                        let mut vertex = letter.to_string();
                        vertex.push_str(&i.to_string());

                        let message = format!("play {vertex} a1");
                        let message = Message::try_from(message.as_str())?;
                        match self.update(message) {
                            Ok(_message) => return Ok(Some(format!("{vertex} a1"))),
                            Err(error) => println!("{error}"),
                        }
                    }
                }

                Err(anyhow::Error::msg("unable to generate move"))
            }
            Message::KnownCommand(command) => {
                if COMMANDS.contains(&command.as_str()) {
                    Ok(Some("true".to_string()))
                } else {
                    Ok(Some("false".to_string()))
                }
            }
            Message::ListCommands => Ok(Some(COMMANDS.join("\n"))),
            Message::Name => Ok(Some("hnefatafl-copenhagen".to_string())),
            Message::Play(play) => {
                let status = self.board.play(&play, &self.status, &self.turn)?;
                if status == Status::Ongoing {
                    self.turn = self.turn.opposite();
                }
                self.status = status;
                self.plays.push(play);

                Ok(Some(String::new()))
            }
            Message::ProtocolVersion => Ok(Some("1-beta".to_string())),
            Message::Quit => exit(0),
            Message::ResetBoard => {
                *self = Game::default();
                Ok(Some(String::new()))
            }
            Message::ShowBoard => Ok(Some(format!("\n{}", self.board))),
            Message::Version => Ok(Some("0.1.0-beta".to_string())),
        }
    }
}
