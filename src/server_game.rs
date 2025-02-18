use std::{
    collections::{HashMap, VecDeque},
    fmt,
    str::FromStr,
    sync::mpsc::Sender,
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{
    game::Game,
    play::Play,
    rating::Rated,
    role::Role,
    status::Status,
    time::{Time, TimeSettings},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchivedGame {
    pub id: usize,
    pub attacker: String,
    pub defender: String,
    pub rated: Rated,
    pub plays: Vec<Play>,
    pub status: Status,
    pub texts: VecDeque<String>,
}

impl ArchivedGame {
    #[must_use]
    pub fn new(game: &ServerGame) -> Self {
        Self {
            id: game.id,
            attacker: game.attacker.to_string(),
            defender: game.defender.to_string(),
            rated: game.rated,
            plays: game.game.plays.clone(),
            status: game.game.status.clone(),
            texts: game.texts.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServerGame {
    pub id: usize,
    pub attacker: String,
    pub attacker_tx: Sender<String>,
    pub defender: String,
    pub defender_tx: Sender<String>,
    pub rated: Rated,
    pub game: Game,
    pub texts: VecDeque<String>,
}

impl ServerGame {
    #[must_use]
    pub fn protocol(&self) -> String {
        format!(
            "game {} {} {} {}",
            self.id, self.attacker, self.defender, self.rated
        )
    }

    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(
        attacker_tx: Sender<String>,
        defender_tx: Sender<String>,
        game: ServerGameLight,
    ) -> Self {
        let (Some(attacker), Some(defender)) = (game.attacker, game.defender) else {
            panic!("attacker and defender should be set");
        };

        Self {
            id: game.id,
            attacker,
            attacker_tx,
            defender,
            defender_tx,
            rated: game.rated,
            game: Game {
                black_time: game.timed.clone(),
                white_time: game.timed,
                time: Some(Local::now().to_utc().timestamp_millis()),
                ..Game::default()
            },
            texts: VecDeque::new(),
        }
    }
}

impl fmt::Display for ServerGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}, {}, {} ",
            self.id, self.attacker, self.defender, self.rated
        )
    }
}

#[derive(Clone, Debug, Default)]
pub struct ServerGames(pub HashMap<usize, ServerGame>);

#[derive(Clone, Default)]
pub struct Challenger(pub Option<String>);

impl fmt::Debug for Challenger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(challenger) = &self.0 {
            write!(f, "{challenger}")?;
        } else {
            write!(f, "_")?;
        }

        Ok(())
    }
}

impl fmt::Display for Challenger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "challenger: ")?;
        if let Some(challenger) = &self.0 {
            write!(f, "{challenger}")?;
        } else {
            write!(f, "none")?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ServerGameLight {
    pub id: usize,
    pub attacker: Option<String>,
    pub defender: Option<String>,
    pub challenger: Challenger,
    pub rated: Rated,
    pub timed: TimeSettings,
    pub attacker_channel: Option<usize>,
    pub defender_channel: Option<usize>,
    pub spectators: HashMap<String, usize>,
    pub challenge_accepted: bool,
}

impl ServerGameLight {
    #[must_use]
    pub fn new(
        game_id: usize,
        username: String,
        rated: Rated,
        timed: TimeSettings,
        index_supplied: usize,
        role: Role,
    ) -> Self {
        if role == Role::Attacker {
            ServerGameLight {
                id: game_id,
                attacker: Some(username),
                defender: None,
                challenger: Challenger::default(),
                rated,
                timed,
                attacker_channel: Some(index_supplied),
                defender_channel: None,
                spectators: HashMap::new(),
                challenge_accepted: false,
            }
        } else {
            ServerGameLight {
                id: game_id,
                attacker: None,
                defender: Some(username),
                challenger: Challenger::default(),
                rated,
                timed,
                attacker_channel: None,
                defender_channel: Some(index_supplied),
                spectators: HashMap::new(),
                challenge_accepted: false,
            }
        }
    }
}

impl fmt::Debug for ServerGameLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attacker = if let Some(name) = &self.attacker {
            name
        } else {
            "_"
        };

        let defender = if let Some(name) = &self.defender {
            name
        } else {
            "_"
        };

        let Ok(spectators) = ron::ser::to_string(&self.spectators) else {
            panic!("we should be able to serialize the spectators")
        };

        write!(
            f,
            "game {} {attacker} {defender} {} {:?} {:?} {} {spectators}",
            self.id, self.rated, self.timed, self.challenger, self.challenge_accepted
        )
    }
}

impl fmt::Display for ServerGameLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attacker = if let Some(name) = &self.attacker {
            &format!("attacker: {name}")
        } else {
            "attacker: none"
        };

        let defender = if let Some(name) = &self.defender {
            &format!("defender: {name}")
        } else {
            "defender: none"
        };

        write!(
            f,
            "#{}: {attacker}, {defender}, {}, time: {}, {}",
            self.id, self.rated, self.timed, self.challenger
        )
    }
}

impl TryFrom<&[&str]> for ServerGameLight {
    type Error = anyhow::Error;

    fn try_from(vector: &[&str]) -> anyhow::Result<Self> {
        let id = vector[1];
        let attacker = vector[2];
        let defender = vector[3];
        let rated = vector[4];
        let timed = vector[5];
        let minutes = vector[6];
        let add_seconds = vector[7];
        let challenger = vector[8];
        let challenge_accepted = vector[9];
        let spectators = vector[10];

        let id = id.parse::<usize>()?;

        let attacker = if attacker == "_" {
            None
        } else {
            Some(attacker.to_string())
        };

        let defender = if defender == "_" {
            None
        } else {
            Some(defender.to_string())
        };

        let timed = match timed {
            "fischer" => TimeSettings(Some(Time {
                add_seconds: add_seconds.parse::<i64>()?,
                milliseconds_left: minutes.parse::<i64>()?,
            })),
            // "un-timed"
            _ => TimeSettings(None),
        };

        let Ok(challenge_accepted) = <bool as FromStr>::from_str(challenge_accepted) else {
            panic!("the value should be a bool");
        };

        let spectators =
            ron::from_str(spectators).expect("we should be able to deserialize the spectators");

        let mut game = Self {
            id,
            attacker,
            defender,
            challenger: Challenger::default(),
            rated: Rated::try_from(rated)?,
            timed,
            attacker_channel: None,
            defender_channel: None,
            spectators,
            challenge_accepted,
        };

        if challenger != "_" {
            game.challenger.0 = Some(challenger.to_string());
        }

        Ok(game)
    }
}

#[derive(Clone, Default)]
pub struct ServerGamesLight(pub HashMap<usize, ServerGameLight>);

impl fmt::Debug for ServerGamesLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for game in self.0.values() {
            write!(f, "{game:?} ")?;
        }

        Ok(())
    }
}
