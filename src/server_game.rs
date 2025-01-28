use std::{fmt, sync::mpsc::Sender};

use crate::game::Game;

#[derive(Clone, Debug)]
pub struct ArchivedGame {
    pub id: u128,
    pub attacker: String,
    pub defender: String,
    pub game: Game,
}

#[derive(Clone, Debug)]
pub struct ServerGame {
    pub id: u128,
    pub attacker: String,
    pub attacker_tx: Sender<String>,
    pub defender: String,
    pub defender_tx: Sender<String>,
    pub game: Game,
}

impl ServerGame {
    #[must_use]
    pub fn protocol(&self) -> String {
        format!("game {} {} {}", self.id, self.attacker, self.defender)
    }
}

impl fmt::Display for ServerGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}, {} ", self.id, self.attacker, self.defender)
    }
}

#[derive(Clone, Debug)]
pub struct ServerGameLight {
    pub id: u128,
    pub attacker: Option<String>,
    pub defender: Option<String>,
    pub channel: Option<u128>,
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

        format!("game {} {attacker} {defender}", self.id)
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

        write!(f, "{}: {attacker}, {defender} ", self.id)
    }
}

impl TryFrom<(&str, &str, &str)> for ServerGameLight {
    type Error = anyhow::Error;

    fn try_from(id_attacker_defender: (&str, &str, &str)) -> anyhow::Result<Self> {
        let (id, attacker, defender) = id_attacker_defender;
        let id = id.parse::<u128>()?;

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

        Ok(Self {
            id,
            attacker,
            defender,
            channel: None,
        })
    }
}
