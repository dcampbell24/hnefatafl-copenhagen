use std::str::FromStr;

use anyhow::Context;

use crate::{play::Plae, time};

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    FinalStatus,
    GenerateMove,
    KnownCommand(String),
    ListCommands,
    Name,
    Play(Plae),
    ProtocolVersion,
    Quit,
    ResetBoard,
    ShowBoard,
    TimeSettings(time::TimeSettings),
    Version,
}

pub static COMMANDS: [&str; 12] = [
    "final_status",
    "generate_move",
    "known_command",
    "list_commands",
    "name",
    "play",
    "protocol_version",
    "quit",
    "reset_board",
    "show_board",
    "time_settings",
    "version",
];

impl FromStr for Message {
    type Err = anyhow::Error;

    fn from_str(message: &str) -> anyhow::Result<Self> {
        let args: Vec<&str> = message.split_whitespace().collect();

        if args.is_empty() {
            return Ok(Self::Empty);
        }

        match *args.first().unwrap() {
            "final_status" => Ok(Self::FinalStatus),
            "generate_move" => Ok(Self::GenerateMove),
            "known_command" => Ok(Self::KnownCommand(
                (*args.get(1).context("expected: known_command COMMAND")?).to_string(),
            )),
            "list_commands" => Ok(Self::ListCommands),
            "name" => Ok(Self::Name),
            "play" => {
                let play = Plae::try_from(args)?;
                Ok(Self::Play(play))
            }
            "protocol_version" => Ok(Self::ProtocolVersion),
            "quit" => Ok(Self::Quit),
            "reset_board" => Ok(Self::ResetBoard),
            "show_board" => Ok(Self::ShowBoard),
            "time_settings" => {
                let time_settings = time::TimeSettings::try_from(args)?;
                Ok(Self::TimeSettings(time_settings))
            }
            "version" => Ok(Self::Version),
            text => {
                if text.trim().is_empty() {
                    Ok(Self::Empty)
                } else {
                    Err(anyhow::Error::msg(format!("unrecognized command: {text}")))
                }
            }
        }
    }
}
