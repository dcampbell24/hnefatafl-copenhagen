use std::{
    io::{stdin, BufRead, BufReader, Write},
    net::TcpStream,
    thread,
};

use hnefatafl_copenhagen::{game::Game, message, play::Vertex, space::Space};
use iced::widget::{button, row, text, Column, Row};

fn main() -> anyhow::Result<()> {
    iced::run("Hnefatafl Copenhagen", Client::update, Client::view)?;
    Ok(())
}

#[derive(Debug)]
struct Client {
    game: Game,
    texts: Vec<String>,
}

impl Default for Client {
    fn default() -> Self {
        let address = "localhost:8000".to_string();
        let mut stream = TcpStream::connect(&address).unwrap();
        println!("connected to {address} ...");

        // Read a line from the server.
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut buffer = String::new();
        thread::spawn(move || loop {
            reader.read_line(&mut buffer).unwrap();
            print!("<- {buffer}");
            buffer.clear();
        });

        // Read a line from stdin and write to the server.
        let stdin = stdin();
        let mut buffer = String::new();
        thread::spawn(move || loop {
            stdin.read_line(&mut buffer).unwrap();
            print!("-> {buffer}");
            stream.write_all(buffer.as_bytes()).unwrap();
            buffer.clear();
        });

        Client {
            game: Game::default(),
            texts: Vec::new(),
        }
    }
}

impl Client {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::_Game(message) => {
                let _result = self.game.update(message);
            } /*
              Message::TextReceive(string) => {
                  self.reader.read_line(&mut self.read_buffer).unwrap();
              }
              */
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
    // TextSend(String),
}
