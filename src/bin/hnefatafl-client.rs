use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    thread,
};

use hnefatafl_copenhagen::{game::Game, message, play::Vertex, space::Space};
use iced::widget::{button, row, text, text_input, Column, Row};

fn main() -> anyhow::Result<()> {
    iced::run("Hnefatafl Copenhagen", Client::update, Client::view)?;
    Ok(())
}

#[derive(Debug)]
struct Client {
    game: Game,
    tcp_stream: TcpStream,
    texts: Vec<String>,
    text_input: String,
}

impl Default for Client {
    fn default() -> Self {
        let address = "localhost:8000".to_string();
        let tcp_stream = TcpStream::connect(&address).unwrap();
        println!("connected to {address} ...");

        // Read a line from the server.
        let mut reader = BufReader::new(tcp_stream.try_clone().unwrap());
        let mut buffer = String::new();
        thread::spawn(move || loop {
            reader.read_line(&mut buffer).unwrap();
            print!("<- {buffer}");
            buffer.clear();
        });

        Client {
            game: Game::default(),
            tcp_stream,
            texts: Vec::new(),
            text_input: String::new(),
        }
    }
}

impl Client {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::_Game(message) => {
                let _result = self.game.update(message);
            }
            Message::TextChanged(string) => {
                self.text_input = string;
            }
            Message::TextSend => {
                self.text_input.push('\n');
                print!("-> {}", &self.text_input);
                self.tcp_stream
                    .write_all(self.text_input.as_bytes())
                    .unwrap();

                self.text_input.clear();
            }
        }
    }

    #[must_use]
    pub fn view(&self) -> Row<Message> {
        let mut row = Row::new();

        for y in 0..11 {
            let mut column = Column::new();
            for x in 0..11 {
                let button = match self.game.board.get(&Vertex { x, y }) {
                    Space::Empty => button(text("  ")),
                    Space::Black => button(text("X")),
                    Space::King => button(text("K")),
                    Space::White => button(text("O")),
                };
                column = column.push(button);
            }
            row = row.push(column);
        }

        let mut column = Column::new();
        column = column.push("Texts:");
        column = column.push(
            text_input("", &self.text_input)
                .on_input(Message::TextChanged)
                .on_submit(Message::TextSend),
        );

        for message in &self.texts {
            column = column.push(text(message));
        }

        row![row, column]
    }
}

#[derive(Clone, Debug)]
enum Message {
    _Game(message::Message),
    // TextReceive(String),
    TextChanged(String),
    TextSend,
}
