use anyhow::Context;

use crate::move_::Move;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    FinalStatus,
    KnownCommand(String),
    Move(Move),
    Name,
    ProtocolVersion,
    Quit,
    ShowBoard,
    Version,
}

pub static COMMANDS: [&str; 8] = [
    "final_status",
    "known_command",
    "move",
    "name",
    "protocol_version",
    "quit",
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
                args.get(1).context("known_command: needs an argument")?.to_string(),
            )),
            "move" => {
                let move_ = Move::try_from(args)?;
                Ok(Message::Move(move_))
            }
            "name" => Ok(Message::Name),
            "protocol_version" => Ok(Message::ProtocolVersion),
            "quit" => Ok(Message::Quit),
            "show_board" => Ok(Message::ShowBoard),
            "version" => Ok(Message::Version),
            text => {
                if text.trim().len() == 0 {
                    Ok(Message::Empty)
                } else {
                    Err(anyhow::Error::msg(format!("unrecognized command: {text}")))
                }
            }
        }
    }
}
