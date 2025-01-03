use crate::move_::Move;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    Move(Move),
    Quit,
    ShowBoard,
}

impl TryFrom<&str> for Message {
    type Error = anyhow::Error;

    fn try_from(message: &str) -> anyhow::Result<Self> {
        let args: Vec<&str> = message.split_whitespace().collect();

        if args.is_empty() {
            return Ok(Message::Empty);
        }

        match *args.first().unwrap() {
            "move" => {
                let move_ = Move::try_from(args)?;
                Ok(Message::Move(move_))
            }
            "quit" => Ok(Message::Quit),
            "show_board" => Ok(Message::ShowBoard),
            _ => Ok(Message::Empty),
        }
    }
}
