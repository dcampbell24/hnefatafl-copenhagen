use std::{borrow::Cow, fmt, process::exit, time::Instant};

use crate::{
    board::Board,
    color::Color,
    message::{Message, COMMANDS},
    play::{Play, BOARD_LETTERS},
    status::Status,
    time::Time,
};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Game {
    pub board: Board,
    pub plays: Vec<Play>,
    pub status: Status,
    pub timer: Option<Instant>,
    pub black_time: Option<Time>,
    pub white_time: Option<Time>,
    pub turn: Color,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}\n", self.board)?;

        write!(f, "Plays: ")?;
        for play in &self.plays {
            write!(f, "{play}, ")?;
        }
        writeln!(f)?;

        writeln!(f, "status: {:?}", self.status)?;

        if let Some(time) = &self.black_time {
            writeln!(f, "black_time: {time}")?;
        } else {
            writeln!(f, "black_time: infinite")?;
        }

        if let Some(time) = &self.white_time {
            writeln!(f, "white_time: {time}")?;
        } else {
            writeln!(f, "white_time: infinite")?;
        }

        write!(f, "turn: {:?}", self.turn)
    }
}

impl Game {
    fn generate_move(&mut self) -> anyhow::Result<Option<String>> {
        for letter_from in BOARD_LETTERS.chars() {
            for i_from in 1..12 {
                let mut vertex_from = letter_from.to_string();
                vertex_from.push_str(&i_from.to_string());

                for letter_to in BOARD_LETTERS.chars() {
                    for i_to in 1..12 {
                        let mut vertex_to = letter_to.to_string();
                        vertex_to.push_str(&i_to.to_string());

                        let message = format!("play {vertex_from} {vertex_to}");
                        let message = Message::try_from(message.as_str())?;

                        let turn = self.turn.clone();
                        if let Ok(_message) = self.update(message) {
                            return Ok(Some(format!("{turn:?} {vertex_from} {vertex_to}")));
                        }
                    }
                }
            }
        }

        Err(anyhow::Error::msg("unable to generate move"))
    }

    /// # Errors
    ///
    /// If the play is illegal.
    pub fn read_line(&mut self, buffer: &str) -> anyhow::Result<Option<String>> {
        let mut buffer = Cow::from(buffer);
        if let Some(comment_offset) = buffer.find('#') {
            buffer.to_mut().replace_range(comment_offset.., "");
        }

        self.update(Message::try_from(buffer.as_ref())?)
    }

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
                Status::Ongoing => Ok(Some("The game is ongoing.".to_string())),
            },
            Message::GenerateMove => self.generate_move(),
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
            Message::Name => Ok(Some("hnefatafl-copenhagen".to_string())),
            Message::Play(play) => {
                if self.status == Status::Ongoing {
                    if let (status, Some(time), Some(timer)) = match self.turn {
                        Color::Black => (
                            Status::WhiteWins,
                            self.black_time.as_mut(),
                            self.timer.as_mut(),
                        ),
                        Color::Colorless => {
                            unreachable!("It can't be no one's turn when the game is ongoing!")
                        }
                        Color::White => (
                            Status::BlackWins,
                            self.white_time.as_mut(),
                            self.timer.as_mut(),
                        ),
                    } {
                        time.time_left = time.time_left.saturating_sub(timer.elapsed());
                        *timer = Instant::now();

                        if time.time_left.as_secs() == 0 {
                            self.status = status;
                            return Ok(Some(String::new()));
                        }

                        time.time_left += time.add_time;
                    }

                    if self.board.get(&play.from)?.color() != play.color {
                        return Err(anyhow::Error::msg(
                            "play: you are trying to move the wrong color",
                        ));
                    }

                    self.status = self.board.play(&play, &self.status, &self.turn)?;
                    self.plays.push(play);

                    if self.status == Status::Ongoing {
                        self.turn = self.turn.opposite();
                    }

                    Ok(Some(String::new()))
                } else {
                    Err(anyhow::Error::msg("play: the game is already over"))
                }
            }
            Message::ProtocolVersion => Ok(Some("1-beta".to_string())),
            Message::Quit => exit(0),
            Message::ResetBoard => {
                *self = Game::default();
                Ok(Some(String::new()))
            }
            Message::ShowBoard => Ok(Some(self.board.to_string())),
            Message::TimeSettings(mut time_settings) => {
                if let Some(time) = time_settings.time_settings.take() {
                    self.black_time = Some(time.clone());
                    self.white_time = Some(time);
                    self.timer = Some(Instant::now());
                } else {
                    self.black_time = None;
                    self.white_time = None;
                    self.timer = None;
                }

                Ok(Some(String::new()))
            }
            Message::Version => Ok(Some("0.1.0-beta".to_string())),
        }
    }
}
