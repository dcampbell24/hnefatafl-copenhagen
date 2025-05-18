use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::glicko::Rating;

impl fmt::Display for Accounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accounts = Vec::new();
        for (name, account) in &self.0 {
            accounts.push(format!("{name} {account}"));
        }
        accounts.sort_unstable();
        let accounts = accounts.join(" ");

        write!(f, "{accounts}")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Account {
    pub email: Option<Email>,
    pub password: String,
    pub logged_in: Option<usize>,
    pub draws: u64,
    pub wins: u64,
    pub losses: u64,
    pub rating: Rating,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.logged_in.is_some() {
            write!(
                f,
                "{} {} {} {} logged_in",
                self.wins, self.losses, self.draws, self.rating
            )
        } else {
            write!(
                f,
                "{} {} {} {} logged_out",
                self.wins, self.losses, self.draws, self.rating
            )
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Accounts(pub HashMap<String, Account>);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Email {
    pub address: String,
    pub code: Option<u32>,
    pub verified: bool,
}
