use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Smtp {
    pub username: String,
    pub password: String,
    pub service: String,
}
