use std::{
    collections::VecDeque,
    fmt,
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
    widget::{button, column, container, radio, row, text, text_input, Column, Row},
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
    game: Option<Game>,
    game_new: GameNew,
    role_selected: Option<Role>,
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
    Game,
    GameNew,
    GameNewFrozen,
    Games,
}

impl Client {
    #[must_use]
    fn board(&self) -> Row<Message> {
        let Some(game) = &self.game else {
            return row![];
        };

        let mut game_display: Row<'_, Message> = Row::new();
        let board_size = 50;

        for y in 0..11 {
            let mut column = Column::new();
            for x in 0..11 {
                let button = match game.board.get(&Vertex { x, y }) {
                    Space::Empty => button(text(" ").size(board_size)),
                    Space::Black => button(text("X").size(board_size)),
                    Space::King => button(text("K").size(board_size)),
                    Space::White => button(text("O").size(board_size)),
                };
                column = column.push(button);
            }
            game_display = game_display.push(column);
        }

        game_display
    }

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
                let Some(game) = &mut self.game else {
                    return;
                };
                let _result = game.update(message);
            }
            Message::GameNew => self.screen = Screen::GameNew,
            //self.game = Some(Game::default());

            // new_game (attacker | defender) [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
            //if let Some(tx) = &self.tx {
            //    tx.send(format!("{}", self.game_new)).unwrap();
            //}
            // }
            Message::GameLeave => self.screen = Screen::Games,
            Message::GameSubmit => {
                if self.role_selected.is_some() {
                    self.screen = Screen::GameNewFrozen;
                }
            }
            Message::TcpConnected(tx) => {
                println!("TCP connected...");
                self.tx = Some(tx);
            }
            Message::RoleSelected(role) => {
                self.role_selected = Some(role);
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
    fn user_area(&self) -> Column<Message> {
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
        user_area
    }

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        let screen: Element<'_, Message> = match self.screen {
            Screen::Game => {
                let game = self.board();
                let user_area = self.user_area();
                let game = row![game, user_area];

                let leave_game = button("Leave Game").on_press(Message::GameLeave);
                let buttons = row![leave_game];

                column![game, buttons].into()
            }
            Screen::GameNew => {
                let attacker = radio(
                    "attacker ",
                    Role::Attacker,
                    self.role_selected,
                    Message::RoleSelected,
                );

                let defender = radio(
                    "defender ",
                    Role::Defender,
                    self.role_selected,
                    Message::RoleSelected,
                );

                let role = row![text("role: "), attacker, defender];
                column![role, button("New Game").on_press(Message::GameSubmit)].into()
            }
            Screen::GameNewFrozen => {
                let Some(role) = self.role_selected else {
                    panic!("You can't get to GameNewFrozen unless you have selected a role!");
                };

                text(format!("role: {role}")).into()
            }
            Screen::Games => {
                let user_area = self.user_area();

                let create_game = button("Create Game").on_press(Message::GameNew);
                let buttons = row![create_game];

                column![user_area, buttons].into()
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct GameNew {
    role: Role,
}

impl fmt::Display for GameNew {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "new_game {}", self.role)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Role {
    #[default]
    Attacker,
    Defender,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Attacker => write!(f, "attacker"),
            Role::Defender => write!(f, "defender"),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    _Game(message::Message),
    GameLeave,
    GameNew,
    GameSubmit,
    RoleSelected(Role),
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
