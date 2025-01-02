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
                    // x: letters, y: digits
                    from: Vertex { x: 1, y: 5 },
                    to: Vertex { x: 2, y: 5 },
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
    pub x: usize,
    pub y: usize,
}
