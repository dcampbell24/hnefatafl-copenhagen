use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::exit,
    sync::mpsc,
    thread,
};

use futures::executor;
use hnefatafl_copenhagen::{game::Game, message, play::Vertex, space::Space};
use iced::{
    font::Font,
    futures::{SinkExt, Stream},
    stream,
    widget::{button, column, container, row, text, text_input, Column, Row},
    Color, Element, Subscription, Theme,
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
    error: Option<String>,
    game: Game,
    screen: Screen,
    tx: Option<mpsc::Sender<String>>,
    texts: VecDeque<String>,
    text_input: String,
    username: String,
    users: Vec<String>,
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
        self.error = None;
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
                let one = text.next();
                let two = text.next();
                match one {
                    Some("=") => match two {
                        Some("display_users") => {
                            let users: Vec<String> = text.map(ToString::to_string).collect();
                            self.users = users;
                        }
                        Some("login") => self.screen = Screen::Games,
                        Some("text") => {
                            let text: Vec<&str> = text.collect();
                            let text = text.join(" ");
                            self.texts.push_front(text);
                        }
                        _ => {}
                    },
                    Some("?") => {
                        if Some("login") == two {
                            // Fixme:
                            exit(1);
                        }
                    }
                    _ => {}
                }
            }
            Message::TextSend => {
                if let Some(tx) = &self.tx {
                    self.text_input.push('\n');
                    if self.screen == Screen::Login {
                        tx.send(self.text_input.clone()).unwrap();
                        if let Some(username) = self.text_input.split_ascii_whitespace().next() {
                            self.username = username.to_string();
                        }
                    } else {
                        tx.send(format!("text {}", self.text_input)).unwrap();
                    }
                    self.text_input.clear();
                }
            }
        }
    }

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        let screen: Element<'_, Message> = match self.screen {
            Screen::Games => {
                let mut game: Row<'_, Message> = Row::new();
                let board_size = 64;

                for y in 0..11 {
                    let mut column = Column::new();
                    for x in 0..11 {
                        let button = match self.game.board.get(&Vertex { x, y }) {
                            Space::Empty => button(text(" ").size(board_size)),
                            Space::Black => button(text("X").size(board_size)),
                            Space::King => button(text("K").size(board_size)),
                            Space::White => button(text("O").size(board_size)),
                        };
                        column = column.push(button);
                    }
                    game = game.push(column);
                }

                let mut texting = Column::new();
                texting = texting.push("texts:");
                texting = texting.push(
                    text_input("", &self.text_input)
                        .on_input(Message::TextChanged)
                        .on_submit(Message::TextSend),
                );

                for message in &self.texts {
                    texting = texting.push(text(message));
                }
                let texting = container(texting).padding(10).style(container::rounded_box);

                let username = row![text("username: "), text(&self.username)];
                let username = container(username)
                    .padding(10)
                    .style(container::rounded_box);

                let mut users = column![text("users:")];
                for user in &self.users {
                    users = users.push(text(user));
                }
                let users = container(users).padding(10).style(container::rounded_box);

                let user_area = column![username, row![texting, users]];
                row![game, user_area].into()
            }
            Screen::Login => row![
                text("username:"),
                text_input("", &self.text_input)
                    .on_input(Message::TextChanged)
                    .on_submit(Message::TextSend),
            ]
            .into(),
        };

        if let Some(error) = &self.error {
            column![
                // Solarized Red
                text(error).color(Color::from_rgb8(220, 50, 47)),
                screen,
            ]
            .into()
        } else {
            screen
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
                    print!("-> {buffer}");
                    let _ = executor::block_on(sender.send(Message::TextReceived(buffer.clone())));
                    buffer.clear();
                }
            }
        });
    })
}
