use crate::move_::Move;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    Move(Move),
    Quit,
    ShowBoard,
}

impl From<&str> for Message {
    fn from(message: &str) -> Self {
        let args: Vec<&str> = message.split_whitespace().collect();

        if args.is_empty() {
            return Message::Empty;
        }

        match *args.first().unwrap() {
            "move" => {
                let move_ = Move::try_from(args).unwrap();
                Message::Move(move_)
            }
            "quit" => Message::Quit,
            "show_board" => Message::ShowBoard,
            _ => Message::Empty,
        }
    }
}
