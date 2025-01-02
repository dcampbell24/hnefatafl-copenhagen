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
                let move_ = Move {
                    from: Vertex { x: 0, y: 0 },
                    to: Vertex { x: 0, y: 0 },
                };
                Message::Move(move_)
            }
            "quit" => Message::Quit,
            "show_board" => Message::ShowBoard,
            _ => Message::Empty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Move {
    pub from: Vertex,
    pub to: Vertex,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub x: u8,
    pub y: u8,
}
