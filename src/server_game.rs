use std::fmt;

use crate::game::Game;

#[derive(Clone, Debug)]
pub struct ServerGame {
    pub id: usize,
    pub attacker: Option<String>,
    pub defender: Option<String>,
    pub game: Game,
}

impl fmt::Display for ServerGame {
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

        write!(f, "game {} {attacker} {defender}", self.id)
    }
}

#[derive(Clone, Debug)]
pub struct ServerGameLight {
    pub id: usize,
    pub attacker: Option<String>,
    pub defender: Option<String>,
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

        write!(f, "game: {}, {attacker}, {defender}", self.id)
    }
}

impl TryFrom<(&str, &str, &str)> for ServerGameLight {
    type Error = anyhow::Error;

    fn try_from(id_attacker_defender: (&str, &str, &str)) -> anyhow::Result<Self> {
        let (id, attacker, defender) = id_attacker_defender;
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

        Ok(Self {
            id,
            attacker,
            defender,
        })
    }
}
