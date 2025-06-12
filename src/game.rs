use std::{borrow::Cow, collections::HashMap, fmt, process::exit, str::FromStr};

use chrono::Local;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
#[cfg(feature = "js")]
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    ai::{AI, AiBanal},
    board::Board,
    color::Color,
    message::{COMMANDS, Message},
    play::{Captures, Plae, Play, Vertex},
    role::Role,
    space::Space,
    status::Status,
    time::TimeSettings,
};

#[cfg(not(feature = "js"))]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Game {
    #[serde(skip)]
    pub ai: AiBanal,
    pub board: Board,
    pub plays: Vec<Play>,
    pub previous_boards: PreviousBoards,
    pub status: Status,
    pub time: TimeUnix,
    pub black_time: TimeSettings,
    pub white_time: TimeSettings,
    pub turn: Color,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Game {
    #[serde(skip)]
    #[wasm_bindgen(skip)]
    pub ai: AiBanal,
    #[wasm_bindgen(skip)]
    pub board: Board,
    #[wasm_bindgen(skip)]
    pub plays: Vec<Play>,
    #[wasm_bindgen(skip)]
    pub previous_boards: PreviousBoards,
    #[wasm_bindgen(skip)]
    pub status: Status,
    #[wasm_bindgen(skip)]
    pub time: TimeUnix,
    #[wasm_bindgen(skip)]
    pub black_time: TimeSettings,
    #[wasm_bindgen(skip)]
    pub white_time: TimeSettings,
    #[wasm_bindgen(skip)]
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

        match &self.black_time {
            TimeSettings::Timed(time) => writeln!(f, "black_time: {time}")?,
            TimeSettings::UnTimed => writeln!(f, "black_time: infinite")?,
        }

        match &self.white_time {
            TimeSettings::Timed(time) => writeln!(f, "white_time: {time}")?,
            TimeSettings::UnTimed => writeln!(f, "white_time: infinite")?,
        }

        write!(f, "turn: {}", self.turn)
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl Game {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        Game::default()
    }

    /// # Errors
    ///
    /// If the command is illegal or invalid.
    #[wasm_bindgen]
    pub fn read_line_js(&mut self, buffer: &str) -> String {
        let mut buffer = Cow::from(buffer);
        if let Some(comment_offset) = buffer.find('#') {
            buffer.to_mut().replace_range(comment_offset.., "");
        }

        match Message::from_str(buffer.as_ref()) {
            Ok(message) => match self.update(message) {
                Ok(update) => {
                    if let Some(update) = update {
                        format!("= {update}")
                    } else {
                        String::new()
                    }
                }
                Err(err) => format!("? {err}"),
            },
            Err(err) => format!("? {err}"),
        }
    }
}

impl Game {
    #[must_use]
    pub fn all_legal_moves(&self) -> LegalMoves {
        let mut possible_vertexes = Vec::new();
        let mut legal_moves = LegalMoves {
            color: self.turn.clone(),
            moves: HashMap::new(),
        };

        for y in 0..11 {
            for x in 0..11 {
                let vertex = Vertex { x, y };
                if self.board.get(&vertex).color() == self.turn {
                    possible_vertexes.push(vertex);
                }
            }
        }

        for vertex_from in possible_vertexes {
            let mut vertexes_to = Vec::new();

            for y in 0..11 {
                for x in 0..11 {
                    let vertex_to = Vertex { x, y };
                    let play = Play {
                        color: self.turn.clone(),
                        from: vertex_from.clone(),
                        to: vertex_to.clone(),
                    };

                    if let Ok(_board_captures_status) = self.board.play_internal(
                        &Plae::Play(play),
                        &self.status,
                        &self.turn,
                        &self.previous_boards,
                    ) {
                        vertexes_to.push(vertex_to);
                    }
                }
            }

            if !vertexes_to.is_empty() {
                legal_moves.moves.insert(vertex_from, vertexes_to);
            }
        }

        legal_moves
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn all_legal_plays(&self) -> Vec<Plae> {
        let moves = self.all_legal_moves();

        if moves.moves.is_empty() {
            match Role::try_from(&moves.color).unwrap() {
                Role::Attacker => return vec![Plae::BlackResigns],
                Role::Defender => return vec![Plae::WhiteResigns],
            }
        }

        let mut plays = Vec::new();
        for (from, tos) in moves.moves {
            for to in tos {
                plays.push(Plae::Play(Play {
                    color: moves.color.clone(),
                    from: from.clone(),
                    to,
                }));
            }
        }

        plays.sort();
        plays
    }

    #[must_use]
    pub fn exit_one(&self) -> bool {
        let exit_1 = Vertex { x: 0, y: 0 };
        let exit_2 = Vertex { x: 10, y: 0 };
        let exit_3 = Vertex { x: 0, y: 10 };
        let exit_4 = Vertex { x: 10, y: 10 };

        let mut game = self.clone();
        if let Ok(Some(king)) = self.board.find_the_king() {
            for exit in [exit_1, exit_2, exit_3, exit_4] {
                if game
                    .play(&Plae::Play(Play {
                        color: self.turn.clone(),
                        from: king.clone(),
                        to: exit,
                    }))
                    .is_ok()
                {
                    return true;
                }
            }
        }

        false
    }

    #[must_use]
    pub fn generate_move(&self, ai: &mut Box<dyn AI>) -> Option<Plae> {
        ai.generate_move(self)
    }

    /// # Errors
    ///
    /// If the game is already over or the move is illegal.
    pub fn play(&mut self, play: &Plae) -> anyhow::Result<Captures> {
        if self.status == Status::Ongoing {
            if let (status, TimeSettings::Timed(timer), TimeUnix::Time(time)) = match self.turn {
                Color::Black => (Status::WhiteWins, &mut self.black_time, &mut self.time),
                Color::Colorless => {
                    unreachable!("It can't be no one's turn when the game is ongoing!")
                }
                Color::White => (Status::BlackWins, &mut self.white_time, &mut self.time),
            } {
                let now = Local::now().to_utc().timestamp_millis();
                timer.milliseconds_left -= now - *time;

                if timer.milliseconds_left <= 0 {
                    self.status = status;
                    return Ok(Captures::default());
                }

                timer.milliseconds_left += timer.add_seconds * 1_000;
                *time = Local::now().to_utc().timestamp_millis();
            }

            match play {
                Plae::BlackResigns => {
                    if self.turn == Color::Black {
                        self.status = Status::WhiteWins;
                        Ok(Captures::default())
                    } else {
                        Err(anyhow::Error::msg("You can't resign for the other player."))
                    }
                }
                Plae::WhiteResigns => {
                    if self.turn == Color::White {
                        self.status = Status::BlackWins;
                        Ok(Captures::default())
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
                    self.plays.push(play.clone());

                    if self.status == Status::Ongoing {
                        self.turn = self.turn.opposite();

                        if !self.board.a_legal_move_exists(
                            &self.status,
                            &self.turn,
                            &self.previous_boards,
                        ) {
                            match self.turn {
                                Color::Black => self.status = Status::WhiteWins,
                                Color::Colorless => {}
                                Color::White => self.status = Status::BlackWins,
                            }
                        }
                    }

                    let captures = Captures(captures);
                    Ok(captures)
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
        let mut ai: Box<dyn AI> = Box::new(AiBanal);

        match message {
            Message::Empty => Ok(None),
            Message::FinalStatus => Ok(Some(format!("{}", self.status))),
            Message::GenerateMove => Ok(self.generate_move(&mut ai).map(|play| play.to_string())),
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
            Message::Play(play) => self.play(&play).map(|captures| Some(captures.to_string())),
            Message::PlayFrom => {
                let moves = self.all_legal_moves();
                Ok(Some(format!(
                    "{} {}",
                    moves.color,
                    moves
                        .moves
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" ")
                )))
            }
            Message::PlayTo(from) => {
                let (color, vertex) = from;
                let moves = self.all_legal_moves();
                if color != moves.color {
                    return Err(anyhow::Error::msg(format!(
                        "tried play_to {color}, but it's {} turn",
                        moves.color
                    )));
                }

                if let Some(moves) = moves.moves.get(&vertex) {
                    Ok(Some(
                        moves
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" "),
                    ))
                } else {
                    Err(anyhow::Error::msg("invalid from vertex"))
                }
            }
            Message::ProtocolVersion => Ok(Some("1-beta".to_string())),
            Message::Quit => exit(0),
            Message::ResetBoard => {
                *self = Game::default();
                Ok(Some(String::new()))
            }
            Message::ShowBoard => Ok(Some(self.board.to_string())),
            Message::TimeSettings(time_settings) => {
                match time_settings {
                    TimeSettings::Timed(time) => {
                        self.black_time = TimeSettings::Timed(time.clone());
                        self.white_time = TimeSettings::Timed(time);
                        self.time = TimeUnix::default();
                    }
                    TimeSettings::UnTimed => {
                        self.black_time = TimeSettings::UnTimed;
                        self.white_time = TimeSettings::UnTimed;
                        self.time = TimeUnix::UnTimed;
                    }
                }

                Ok(Some(String::new()))
            }
            Message::Version => {
                let version = env!("CARGO_PKG_VERSION");
                Ok(Some(version.to_string()))
            }
        }
    }

    #[must_use]
    pub fn utility(&self) -> i32 {
        match self.status {
            Status::Ongoing => {}
            Status::BlackWins => return i32::MIN,
            Status::Draw => return 0,
            Status::WhiteWins => return i32::MAX,
        }

        let mut utility = 0;

        let mut white_left = 0;
        let mut black_left = 0;
        for space in self.board.spaces {
            match space {
                Space::Black => black_left += 1,
                Space::Empty | Space::King => {}
                Space::White => white_left += 1,
            }
        }

        utility += white_left * 2;
        utility -= black_left;

        if self.exit_one() {
            utility += 100;
        }

        utility
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalMoves {
    pub color: Color,
    pub moves: HashMap<Vertex, Vec<Vertex>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum TimeUnix {
    Time(i64),
    UnTimed,
}

impl Default for TimeUnix {
    fn default() -> Self {
        Self::Time(Local::now().to_utc().timestamp_millis())
    }
}
