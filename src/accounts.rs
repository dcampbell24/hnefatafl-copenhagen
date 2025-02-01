use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::glicko::Rating;

impl fmt::Display for Accounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accounts = Vec::new();
        for (name, account) in &self.0 {
            if account.logged_in.is_some() {
                accounts.push(format!(
                    "{name} {} {} {}",
                    account.wins, account.losses, account.rating
                ));
            }
        }
        accounts.sort_unstable();
        let names = accounts.join(" ");

        write!(f, "{names}")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Account {
    pub logged_in: Option<u64>,
    pub wins: u64,
    pub losses: u64,
    pub rating: Rating,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Accounts(pub HashMap<String, Account>);
