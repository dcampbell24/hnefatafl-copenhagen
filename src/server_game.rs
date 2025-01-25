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
