use std::{collections::HashMap, fmt, sync::mpsc::Sender, time::Instant};

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
    pub text: String,
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
            text: game.text.clone(),
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
    pub text: String,
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
        username: String,
        attacker_tx: Sender<String>,
        defender_tx: Sender<String>,
        game: ServerGameLight,
    ) -> Self {
        if let Some(attacker) = game.attacker {
            Self {
                id: game.id,
                attacker,
                attacker_tx,
                defender: username,
                defender_tx,
                rated: game.rated,
                game: Game {
                    black_time: game.timed.clone(),
                    white_time: game.timed,
                    timer: Some(Instant::now()),
                    ..Game::default()
                },
                text: String::new(),
            }
        } else if let Some(defender) = game.defender {
            ServerGame {
                id: game.id,
                attacker: username,
                attacker_tx,
                defender,
                defender_tx,
                rated: game.rated,
                game: Game {
                    black_time: game.timed.clone(),
                    white_time: game.timed,
                    timer: Some(Instant::now()),
                    ..Game::default()
                },
                text: String::new(),
            }
        } else {
            panic!("there has to be an attacker or defender")
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

impl fmt::Display for ServerGames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut games = Vec::new();
        for game in self.0.values() {
            games.push(game.protocol());
        }
        let games = games.join(" ");

        write!(f, "{games}")
    }
}

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
    pub channel: Option<usize>,
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
                channel: Some(index_supplied),
            }
        } else {
            ServerGameLight {
                id: game_id,
                attacker: None,
                defender: Some(username),
                challenger: Challenger::default(),
                rated,
                timed,
                channel: Some(index_supplied),
            }
        }
    }
}

impl fmt::Debug for ServerGameLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attacker = if let Some(name) = &self.attacker {
            name
        } else {
            "none"
        };

        let defender = if let Some(name) = &self.defender {
            name
        } else {
            "none"
        };

        write!(
            f,
            "game {} {attacker} {defender} {} {:?} {:?}",
            self.id, self.rated, self.timed, self.challenger
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
            "{}: {attacker}, {defender}, {}, time: {}, {}",
            self.id, self.rated, self.timed, self.challenger
        )
    }
}

impl TryFrom<(&str, &str, &str, &str, &str, &str, &str)> for ServerGameLight {
    type Error = anyhow::Error;

    fn try_from(
        id_attacker_defender_rated: (&str, &str, &str, &str, &str, &str, &str),
    ) -> anyhow::Result<Self> {
        let (id, attacker, defender, rated, timed, minutes, add_seconds) =
            id_attacker_defender_rated;
        let id = id.parse::<usize>()?;

        let attacker = if attacker == "none" {
            None
        } else {
            Some(attacker.to_string())
        };

        let defender = if defender == "none" {
            None
        } else {
            Some(defender.to_string())
        };

        let timed = match timed {
            "fischer" => TimeSettings(Some(Time {
                add_seconds: add_seconds.parse::<u128>()?,
                milliseconds_left: minutes.parse::<u128>()?,
            })),
            // "un-timed"
            _ => TimeSettings(None),
        };

        Ok(Self {
            id,
            attacker,
            defender,
            challenger: Challenger::default(),
            rated: Rated::try_from(rated)?,
            timed,
            channel: None,
        })
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
