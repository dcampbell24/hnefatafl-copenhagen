use std::{collections::HashMap, fmt};

#[cfg(feature = "server")]
use lettre::message::Mailbox;
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
    #[serde(default)]
    pub email: Option<Email>,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub logged_in: Option<usize>,
    #[serde(default)]
    pub draws: u64,
    #[serde(default)]
    pub wins: u64,
    #[serde(default)]
    pub losses: u64,
    #[serde(default)]
    pub rating: Rating,
    #[serde(default)]
    pub send_emails: bool,
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

#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Eq, Serialize)]
pub struct Email {
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub code: Option<u32>,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub verified: bool,
}

#[cfg(feature = "server")]
impl Email {
    #[must_use]
    pub fn to_mailbox(&self) -> Option<Mailbox> {
        Some(Mailbox::new(
            Some(self.username.clone()),
            self.address.parse().ok()?,
        ))
    }

    #[must_use]
    pub fn tx(&self) -> String {
        // Note: We use a FIGURE SPACE to separate the username from the address so
        // .split_ascii_whitespace() does not treat it as a space.
        format!("{} <{}>", self.username, self.address)
    }
}
