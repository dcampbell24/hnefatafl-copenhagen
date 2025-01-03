use crate::move_::Move;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    FinalStatus,
    Move(Move),
    Quit,
    ShowBoard,
}

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
            "move" => {
                let move_ = Move::try_from(args)?;
                Ok(Message::Move(move_))
            }
            "quit" => Ok(Message::Quit),
            "show_board" => Ok(Message::ShowBoard),
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
