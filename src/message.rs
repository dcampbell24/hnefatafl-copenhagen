use anyhow::Context;

use crate::play::Play;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    FinalStatus,
    GenerateMove,
    KnownCommand(String),
    ListCommands,
    Name,
    Play(Play),
    ProtocolVersion,
    Quit,
    ResetBoard,
    ShowBoard,
    TimeLeft,
    TimeSettings(TimeSettings),
    Version,
}

pub static COMMANDS: [&str; 13] = [
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
    "time_left",
    "time_settings",
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
            "generate_move" => Ok(Message::GenerateMove),
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
            "time_left" => Ok(Message::TimeLeft),
            "time_settings" => {
                let time_settings = TimeSettings::try_from(message)?;
                Ok(Message::TimeSettings(time_settings))
            }
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
