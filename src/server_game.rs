use std::{fmt, sync::mpsc::Sender};

use serde::{Deserialize, Serialize};

use crate::{
    game::Game,
    play::Play,
    rating::Rated,
    status::Status,
    time::{Time, TimeSettings},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchivedGame {
    pub id: u64,
    pub attacker: String,
    pub defender: String,
    pub rated: Rated,
    pub plays: Vec<Play>,
    pub status: Status,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct ServerGame {
    pub id: u64,
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

#[derive(Clone, Debug)]
pub struct ServerGameLight {
    pub id: u64,
    pub attacker: Option<String>,
    pub defender: Option<String>,
    pub rated: Rated,
    pub timed: TimeSettings,
    pub channel: Option<u64>,
}

impl ServerGameLight {
    #[must_use]
    pub fn protocol(&self) -> String {
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

        format!(
            "game {} {attacker} {defender} {} {:?}",
            self.id, self.rated, self.timed
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
            "{}: {attacker}, {defender}, {}, {:?}",
            self.id, self.rated, self.timed
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
        let id = id.parse::<u64>()?;

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
            rated: Rated::try_from(rated)?,
            timed,
            channel: None,
        })
    }
}
