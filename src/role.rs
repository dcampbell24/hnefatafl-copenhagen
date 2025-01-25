use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Role {
    #[default]
    Attacker,
    Defender,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Attacker => write!(f, "attacker"),
            Role::Defender => write!(f, "defender"),
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = anyhow::Error;

    fn try_from(string: &str) -> anyhow::Result<Self> {
        match string {
            "attacker" => Ok(Self::Attacker),
            "defender" => Ok(Self::Defender),
            _ => Err(anyhow::Error::msg(format!(
                "Error trying to convert '{string}' to a Role!"
            ))),
        }
    }
}
