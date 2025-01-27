use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::exit,
    sync::mpsc,
    thread,
};

use clap::{command, Parser};
use futures::executor;
use hnefatafl_copenhagen::{
    game::Game, handle_error, message, play::Vertex, role::Role, server_game::ServerGameLight,
    space::Space,
};
use iced::{
    font::Font,
    futures::{SinkExt, Stream},
    stream,
    widget::{button, column, container, radio, row, text, text_input, Column, Row},
    Color, Element, Subscription, Theme,
};

/// A Hnefatafl Copenhagen Client
///
/// This is a TCP client that connects to a server.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host and port
    #[arg(default_value = "localhost:8000", index = 1, value_name = "host:port")]
    host_port: String,
}

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
    games: Vec<ServerGameLight>,
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

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) {
        self.error = None;

        match message {
            Message::_Game(message) => {
                let Some(game) = &mut self.game else {
                    return;
                };
                let _result = game.update(message);
            }
            Message::GameJoin(id) => {
                if let Some(tx) = &self.tx {
                    handle_error(tx.send(format!("join_game {id}\n")));
                }
            }
            Message::GameLeave => self.screen = Screen::Games,
            Message::GameNew => self.screen = Screen::GameNew,
            Message::GameSubmit => {
                if let (Some(role), Some(tx)) = (self.role_selected, &self.tx) {
                    self.screen = Screen::GameNewFrozen;
                    self.game = Some(Game::default());

                    // new_game (attacker | defender) [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
                    handle_error(tx.send(format!("new_game {role}\n")));
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
                        Some("display_pending_games") => {
                            self.games.clear();
                            let games: Vec<&str> = text.collect();
                            for chunks in games.chunks_exact(4) {
                                let id = chunks[1];
                                let attacker = chunks[2];
                                let defender = chunks[3];

                                self.games.push(
                                    ServerGameLight::try_from((id, attacker, defender))
                                        .expect("the value should be a valid ServerGameLight"),
                                );
                            }
                        }
                        Some("display_users") => {
                            let users: Vec<String> = text.map(ToString::to_string).collect();
                            self.users = users;
                        }
                        Some("join_game") => {
                            self.game = Some(Game::default());
                            self.screen = Screen::Game;
                        }
                        Some("login") => self.screen = Screen::Games,
                        Some("new_game") => {
                            let three = text.next();
                            if three == Some("ready") {
                                self.game = Some(Game::default());
                                self.screen = Screen::Game;
                            }
                        }
                        Some("text") => {
                            let text: Vec<&str> = text.collect();
                            let text = text.join(" ");
                            self.texts.push_front(text);
                        }
                        _ => {}
                    },
                    Some("?") => {
                        if Some("login") == two {
                            exit(1);
                        }
                    }
                    _ => {}
                }
            }
            Message::TextSend => {
                if let Some(tx) = &self.tx {
                    if self.text_input.trim().is_empty() {
                        return;
                    }

                    self.text_input.push('\n');
                    if self.screen == Screen::Login {
                        handle_error(tx.send(self.text_input.clone()));

                        if let Some(username) = self.text_input.split_ascii_whitespace().next() {
                            let username = username.to_ascii_lowercase();
                            self.username = username;
                        }
                    } else {
                        handle_error(tx.send(format!("text {}", self.text_input)));
                    }
                    self.text_input.clear();
                }
            }
        }
    }

    #[must_use]
    fn user_area(&self) -> Column<Message> {
        let mut games = Column::new();
        games = games.push(text("games:"));
        for game in &self.games {
            let id = game.id;
            let join = button("join").on_press(Message::GameJoin(id));
            games = games.push(row![text(game.to_string()), join]);
        }
        let games = container(games).padding(10).style(container::rounded_box);

        let mut texting = Column::new();
        texting = texting.push("texts:");
        texting = texting.push(
            text_input("", &self.text_input)
                .on_input(Message::TextChanged)
                .on_paste(Message::TextChanged)
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

        let user_area = column![username, row![games, texting, users]];
        user_area
    }

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        let screen: Element<'_, Message> = match self.screen {
            Screen::Game => {
                let game = self.board();
                let game = row![game, text(&self.username)];

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

#[derive(Clone, Debug)]
enum Message {
    _Game(message::Message),
    GameJoin(u128),
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
    stream::channel(100, move |mut sender| async move {
        let args = Args::parse();
        let address = &args.host_port;
        let mut tcp_stream = handle_error(TcpStream::connect(address));

        let reader = handle_error(tcp_stream.try_clone());
        let mut reader = BufReader::new(reader);
        println!("connected to {address} ...");

        let (tx, rx) = mpsc::channel();
        let _ = sender.send(Message::TcpConnected(tx)).await;
        thread::spawn(move || loop {
            let message = handle_error(rx.recv());
            print!("<- {message}");

            handle_error(tcp_stream.write_all(message.as_bytes()));
        });

        thread::spawn(move || {
            let mut buffer = String::new();
            loop {
                let bytes = handle_error(reader.read_line(&mut buffer));
                if bytes > 0 {
                    print!("-> {buffer}");
                    handle_error(executor::block_on(
                        sender.send(Message::TextReceived(buffer.clone())),
                    ));
                    buffer.clear();
                } else {
                    eprintln!("error: the TCP stream has closed");
                    exit(1);
                }
            }
        });
    })
}
