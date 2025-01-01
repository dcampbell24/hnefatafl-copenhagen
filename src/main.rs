fn main() -> iced::Result {
    iced::run("Hnefatafl", update, Board::view)
}

fn update(_board: &mut Board, message: Message) {
    match message {
        Message::Increment => {},
    }
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

use iced::widget::{button, text, Column, Row};
use iced::Element;

#[derive(Debug, Clone)]
struct Board {
    spaces: Vec<Vec<Space>>,
}

impl Default for Board {
    fn default() -> Self {
        Board::start()
    }
}

impl Board {
    fn board() -> Self {
        let spaces = vec![
            "E  XXXXX  E",
            "     X     ",
            "           ",
            "X    O    X",
            "X   OOO   X",
            "XX OOKOO XX",
            "X   OOO   X",
            "X    O    X",
            "           ",
            "     X     ",
            "E  XXXXX  E",
        ];

        spaces.into()
    }

    fn start() -> Self {
        let spaces = vec![
            "   XXXXX   ",
            "     X     ",
            "           ",
            "X    O    X",
            "X   OOO   X",
            "XX OOKOO XX",
            "X   OOO   X",
            "X    O    X",
            "           ",
            "     X     ",
            "   XXXXX   ",
        ];

        spaces.into()
    }

    fn draw(&self) -> Element<Message> {
        let mut columns = Row::new();

        for (board_column, spaces_column) in Board::board().spaces.iter().zip(&self.spaces) {
            let mut row = Column::new();
            for (board_row, spaces_row) in board_column.iter().zip(spaces_column) {
                let button = match spaces_row {
                    Space::Empty | Space::Exit => button(text("  ")),
                    Space::Black => button(text("X")),
                    Space::King => button(text("K")),
                    Space::White => button(text("o")),
                };

                match board_row {
                    Space::Empty => row = row.push(button),
                    Space::Exit => row = row.push(button),
                    Space::Black => row = row.push(button),
                    Space::King => row = row.push(button),
                    Space::White => row = row.push(button),
                }
            }
            columns = columns.push(row);
        }

        columns.into()
    }

    fn view(&self) -> Element<Message> {
        self.draw()
    }
}

impl From<Vec<&str>> for Board {
    fn from(value: Vec<&str>) -> Self {
        let mut rows = Vec::new();

        for row in value {
            let mut columns = Vec::new();
            for ch in row.chars() {
                columns.push(ch.into());
            }
            rows.push(columns);
        }

        Self {
            spaces: rows,
        }
    }
}

#[derive(Debug, Default, Clone)]
enum Space {
    #[default]
    Empty,
    Exit,
    Black,
    King,
    White,
}

impl From<char> for Space {
    fn from(value: char) -> Space {
        match value {
            'X' => Self::Black,
            'O' => Self::White,
            ' ' => Self::Empty,
            'E' => Self::Exit,
            'K' => Self::King,
            ch @ _ => panic!("Trying to convert '{}' to a Space.", ch),
        }
    }
}