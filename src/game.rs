use std::{borrow::Cow, fmt, process::exit, str::FromStr};

use chrono::Local;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::{
    board::Board,
    color::Color,
    message::{COMMANDS, Message},
    play::{BOARD_LETTERS, Captures, Plae, Play},
    status::Status,
    time::TimeSettings,
};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Game {
    pub board: Board,
    pub plays: Vec<Play>,
    pub previous_boards: PreviousBoards,
    pub status: Status,
    pub time: Option<i64>,
    pub black_time: TimeSettings,
    pub white_time: TimeSettings,
    pub turn: Color,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PreviousBoards(pub FxHashSet<Board>);

impl Default for PreviousBoards {
    fn default() -> Self {
        let mut boards = FxHashSet::default();

        boards.insert(Board::default());
        Self(boards)
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n", self.board)?;

        write!(f, "plays: ")?;
        for play in &self.plays {
            write!(f, "{play}, ")?;
        }
        writeln!(f)?;

        writeln!(f, "status: {}", self.status)?;

        if let Some(time) = &self.black_time.0 {
            writeln!(f, "black_time: {time}")?;
        } else {
            writeln!(f, "black_time: infinite")?;
        }

        if let Some(time) = &self.white_time.0 {
            writeln!(f, "white_time: {time}")?;
        } else {
            writeln!(f, "white_time: infinite")?;
        }

        write!(f, "turn: {}", self.turn)
    }
}

impl Game {
    #[allow(clippy::missing_panics_doc)]
    pub fn generate_move(&mut self) -> Option<String> {
        if self.status != Status::Ongoing {
            return None;
        }

        for letter_from in BOARD_LETTERS.chars() {
            for i_from in 1..12 {
                let mut vertex_from = letter_from.to_string();
                vertex_from.push_str(&i_from.to_string());

                for letter_to in BOARD_LETTERS.chars() {
                    for i_to in 1..12 {
                        let mut vertex_to = letter_to.to_string();
                        vertex_to.push_str(&i_to.to_string());

                        let message = format!("play {} {vertex_from} {vertex_to}", self.turn);
                        let message = Message::from_str(message.as_str())
                            .expect("we must have formed a valid play");

                        let turn = self.turn.clone();
                        if let Ok(_message) = self.update(message) {
                            return Some(format!("{turn} {vertex_from} {vertex_to}"));
                        }
                    }
                }
            }
        }

        Some(format!("play {} resigns", self.turn))
    }

    fn play(&mut self, play: Plae) -> anyhow::Result<Option<String>> {
        if self.status == Status::Ongoing {
            if let (status, Some(timer), Some(time)) = match self.turn {
                Color::Black => (
                    Status::WhiteWins,
                    self.black_time.0.as_mut(),
                    self.time.as_mut(),
                ),
                Color::Colorless => {
                    unreachable!("It can't be no one's turn when the game is ongoing!")
                }
                Color::White => (
                    Status::BlackWins,
                    self.white_time.0.as_mut(),
                    self.time.as_mut(),
                ),
            } {
                let now = Local::now().to_utc().timestamp_millis();
                timer.milliseconds_left -= now - *time;

                if timer.milliseconds_left <= 0 {
                    self.status = status;
                    return Ok(Some(String::new()));
                }

                timer.milliseconds_left += timer.add_seconds * 1_000;
                *time = Local::now().to_utc().timestamp_millis();
            }

            match play {
                Plae::BlackResigns => {
                    if self.turn == Color::Black {
                        self.status = Status::WhiteWins;
                        Ok(Some(String::new()))
                    } else {
                        Err(anyhow::Error::msg("You can't resign for the other player."))
                    }
                }
                Plae::WhiteResigns => {
                    if self.turn == Color::White {
                        self.status = Status::BlackWins;
                        Ok(Some(String::new()))
                    } else {
                        Err(anyhow::Error::msg("You can't resign for the other player."))
                    }
                }
                Plae::Play(play) => {
                    let piece_color = self.board.get(&play.from).color();
                    if piece_color != play.color {
                        return Err(anyhow::Error::msg(format!(
                            "play: you are trying to move {piece_color}, but it's {}'s turn",
                            play.color
                        )));
                    }

                    let (captures, status) = self.board.play(
                        &Plae::Play(play.clone()),
                        &self.status,
                        &self.turn,
                        &mut self.previous_boards,
                    )?;
                    self.status = status;
                    self.plays.push(play);

                    if self.status == Status::Ongoing {
                        self.turn = self.turn.opposite();
                    }

                    let captures = Captures(captures);
                    Ok(Some(format!("{captures}")))
                }
            }
        } else {
            Err(anyhow::Error::msg("play: the game is already over"))
        }
    }

    /// # Errors
    ///
    /// If the command is illegal or invalid.
    pub fn read_line(&mut self, buffer: &str) -> anyhow::Result<Option<String>> {
        let mut buffer = Cow::from(buffer);
        if let Some(comment_offset) = buffer.find('#') {
            buffer.to_mut().replace_range(comment_offset.., "");
        }

        self.update(Message::from_str(buffer.as_ref())?)
    }

    /// # Errors
    ///
    /// If the command is illegal or invalid.
    pub fn update(&mut self, message: Message) -> anyhow::Result<Option<String>> {
        match message {
            Message::Empty => Ok(None),
            Message::FinalStatus => Ok(Some(format!("{}", self.status))),
            Message::GenerateMove => Ok(self.generate_move()),
            Message::KnownCommand(command) => {
                if COMMANDS.contains(&command.as_str()) {
                    Ok(Some("true".to_string()))
                } else {
                    Ok(Some("false".to_string()))
                }
            }
            Message::ListCommands => {
                let mut commands = "\n".to_string();
                commands.push_str(&COMMANDS.join("\n"));
                Ok(Some(commands))
            }
            Message::Name => {
                let name = env!("CARGO_PKG_NAME");
                Ok(Some(name.to_string()))
            }
            Message::Play(play) => self.play(play),
            Message::ProtocolVersion => Ok(Some("1-beta".to_string())),
            Message::Quit => exit(0),
            Message::ResetBoard => {
                *self = Game::default();
                Ok(Some(String::new()))
            }
            Message::ShowBoard => Ok(Some(self.board.to_string())),
            Message::TimeSettings(mut time_settings) => {
                if let Some(time) = time_settings.0.take() {
                    self.black_time.0 = Some(time.clone());
                    self.white_time.0 = Some(time);
                    self.time = Some(Local::now().to_utc().timestamp_millis());
                } else {
                    self.black_time.0 = None;
                    self.white_time.0 = None;
                    self.time = None;
                }

                Ok(Some(String::new()))
            }
            Message::Version => {
                let version = env!("CARGO_PKG_VERSION");
                Ok(Some(version.to_string()))
            }
        }
    }
}
