use std::{fmt, process::exit};

use anyhow::Ok;

use crate::{
    board::Board,
    color::Color,
    message::{Message, COMMANDS},
    play::Play,
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
    pub fn update(&mut self, message: Message) -> anyhow::Result<()> {
        match message {
            Message::Empty => {}
            Message::FinalStatus => match self.status {
                Status::BlackWins => print!("= Black wins!\n\n"),
                Status::Draw => print!("= It's a draw!\n\n"),
                Status::WhiteWins => print!("= White wins!\n\n"),
                Status::Ongoing => print!("= The game is ongoing.\n\n"),
            },
            Message::KnownCommand(command) => {
                if COMMANDS.contains(&command.as_str()) {
                    println!("= true\n");
                } else {
                    println!("= false\n");
                }
            }
            Message::ListCommands => {
                println!("=");
                for command in COMMANDS {
                    println!("{command}");
                }
                println!();
            }
            Message::Name => println!("= hnefatafl-copenhagen\n"),
            Message::Play(play) => {
                let status = self.board.play(&play, &self.status, &self.turn)?;
                if status == Status::Ongoing {
                    self.turn = self.turn.opposite();
                }
                self.status = status;
                self.plays.push(play);

                print!("=\n\n");
            }
            Message::ProtocolVersion => println!("= 1-beta\n"),
            Message::Quit => exit(0),
            Message::ResetBoard => {
                *self = Game::default();
                println!("=\n");
            }
            Message::ShowBoard => print!("=\n{}", self.board),
            Message::Version => println!("= 0.1.0-beta\n"),
        }

        Ok(())
    }
}
