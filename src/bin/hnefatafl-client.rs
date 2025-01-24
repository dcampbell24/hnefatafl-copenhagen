use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
};

use futures::executor;
use hnefatafl_copenhagen::{game::Game, message, play::Vertex, space::Space};
use iced::{
    font::Font,
    futures::{SinkExt, Stream},
    stream,
    widget::{button, row, text, text_input, Column, Row},
    Subscription, Theme,
};

const ADDRESS: &str = "localhost:8000";

fn main() -> anyhow::Result<()> {
    iced::application("Hnefatafl Copenhagen", Client::update, Client::view)
        .default_font(Font::MONOSPACE)
        .subscription(Client::pass_messages)
        .window_size(iced::Size::INFINITY)
        .theme(Client::theme)
        .run()?;

    Ok(())
}

#[derive(Debug, Default)]
struct Client {
    game: Game,
    screen: Screen,
    tx: Option<mpsc::Sender<String>>,
    texts: Vec<String>,
    text_input: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Screen {
    #[default]
    Login,
    Games,
}

impl Client {
    fn pass_messages(_self: &Self) -> Subscription<Message> {
        Subscription::run(pass_messages)
    }

    pub fn theme(_self: &Self) -> Theme {
        Theme::SolarizedLight
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::_Game(message) => {
                let _result = self.game.update(message);
            }
            Message::TcpConnected(tx) => {
                println!("TCP connected...");
                self.tx = Some(tx);
            }
            Message::TextChanged(string) => {
                self.text_input = string;
            }
            Message::TextReceived(string) => {
                let mut text = string.split_ascii_whitespace();
                if Some("text") == text.next() {
                    let text: Vec<&str> = text.collect();
                    let text = text.join(" ");
                    self.texts.push(text);
                }
            }
            Message::TextSend => {
                if let Some(tx) = &self.tx {
                    self.text_input.push('\n');
                    if self.screen == Screen::Login {
                        tx.send(self.text_input.clone()).unwrap();
                    } else {
                        tx.send(format!("text {}", self.text_input)).unwrap();
                    }
                }
                self.text_input.clear();

                if self.screen == Screen::Login {
                    self.screen = Screen::Games;
                }
            }
        }
    }

    #[must_use]
    pub fn view(&self) -> Row<Message> {
        match self.screen {
            Screen::Games => {
                let mut row = Row::new();

                for y in 0..11 {
                    let mut column = Column::new();
                    for x in 0..11 {
                        let button = match self.game.board.get(&Vertex { x, y }) {
                            Space::Empty => button(text(" ")),
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
            Screen::Login => {
                row![
                    text("username:"),
                    text_input("", &self.text_input)
                        .on_input(Message::TextChanged)
                        .on_submit(Message::TextSend),
                ]
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    _Game(message::Message),
    TcpConnected(mpsc::Sender<String>),
    TextChanged(String),
    TextReceived(String),
    TextSend,
}

fn pass_messages() -> impl Stream<Item = Message> {
    stream::channel(100, |mut sender| async move {
        let mut tcp_stream = TcpStream::connect(ADDRESS).unwrap();
        let mut reader = BufReader::new(tcp_stream.try_clone().unwrap());
        println!("connected to {ADDRESS} ...");

        let (tx, rx) = mpsc::channel();
        let _ = sender.send(Message::TcpConnected(tx)).await;
        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            print!("<- {message}");
            tcp_stream.write_all(message.as_bytes()).unwrap();
        });

        thread::spawn(move || {
            let mut buffer = String::new();
            loop {
                if reader.read_line(&mut buffer).unwrap() != 0 {
                    let _ = executor::block_on(sender.send(Message::TextReceived(buffer.clone())));
                    buffer.clear();
                }
            }
        });
    })
}
