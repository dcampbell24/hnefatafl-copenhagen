use anyhow::Context;

use crate::play::Play;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    FinalStatus,
    KnownCommand(String),
    ListCommands,
    Name,
    Play(Play),
    ProtocolVersion,
    Quit,
    ResetBoard,
    ShowBoard,
    Version,
}

pub static COMMANDS: [&str; 10] = [
    "final_status",
    "known_command",
    "list_commands",
    "move",
    "name",
    "protocol_version",
    "quit",
    "reset_board",
    "show_board",
    "version",
];

impl TryFrom<&str> for Message {
    type Error = anyhow::Error;

    fn try_from(message: &str) -> anyhow::Result<Self> {
        // Todo: remove comments "#.*\n"

        let args: Vec<&str> = message.split_whitespace().collect();

        if args.is_empty() {
            return Ok(Message::Empty);
        }

        match *args.first().unwrap() {
            "final_status" => Ok(Message::FinalStatus),
            "known_command" => Ok(Message::KnownCommand(
                (*args.get(1).context("known_command: needs an argument")?).to_string(),
            )),
            "list_commands" => Ok(Message::ListCommands),
            "name" => Ok(Message::Name),
            "play" => {
                let play = Play::try_from(args)?;
                Ok(Message::Play(play))
            }
            "protocol_version" => Ok(Message::ProtocolVersion),
            "quit" => Ok(Message::Quit),
            "reset_board" => Ok(Message::ResetBoard),
            "show_board" => Ok(Message::ShowBoard),
            "version" => Ok(Message::Version),
            text => {
                if text.trim().is_empty() {
                    Ok(Message::Empty)
                } else {
                    Err(anyhow::Error::msg(format!("unrecognized command: {text}")))
                }
            }
        }
    }
}
