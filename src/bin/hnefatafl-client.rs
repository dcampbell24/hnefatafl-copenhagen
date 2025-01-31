use core::panic;
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
    color::Color, game::Game, handle_error, play::Vertex, role::Role, server_game::ServerGameLight,
    space::Space,
};
use iced::{
    font::Font,
    futures::{SinkExt, Stream},
    stream,
    widget::{
        button, column, container, radio, row, scrollable, text, text_input, Column, Container, Row,
    },
    Element, Subscription, Theme,
};

const PADDING: u16 = 20;
const SPACING: u16 = 10;

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
    attacker: String,
    defender: String,
    error: Option<String>,
    game: Option<Game>,
    game_id: u64,
    games: Vec<ServerGameLight>,
    my_turn: bool,
    password: String,
    play_from: Option<Vertex>,
    role_selected: Option<Role>,
    screen: Screen,
    tx: Option<mpsc::Sender<String>>,
    texts: VecDeque<String>,
    texts_game: VecDeque<String>,
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
    fn board(&self) -> Column<Message> {
        let board_size = 50;

        let letters = " ABCDEFGHJKL";
        let mut row_letters_1 = Row::new();
        for letter in letters.chars() {
            row_letters_1 = row_letters_1.push(text(letter).size(board_size).center());
        }
        let mut row_letters_2 = Row::new();
        for letter in letters.chars() {
            row_letters_2 = row_letters_2.push(text(letter).size(board_size).center());
        }

        let Some(game) = &self.game else {
            return column![];
        };

        let mut game_display = Column::new();
        game_display = game_display.push(row_letters_1.spacing(23));

        let mut possible_moves = None;
        if self.my_turn {
            if let Some(game) = self.game.as_ref() {
                possible_moves = Some(game.board.all_legal_moves(
                    &game.status,
                    &game.turn,
                    &game.previous_boards,
                ));
            }
        }

        for y in 0..11 {
            let mut row = Row::new();

            let y_label = 11 - y;
            row = row.push(text(format!("{y_label:2}")).size(board_size).center());

            for x in 0..11 {
                let vertex = Vertex { x, y };

                let mut button = match game.board.get(&vertex) {
                    Space::Empty => {
                        if (y, x) == (0, 0)
                            || (y, x) == (10, 0)
                            || (y, x) == (0, 10)
                            || (y, x) == (10, 10)
                            || (y, x) == (5, 5)
                        {
                            button(text("□").size(board_size))
                        } else {
                            button(text(" ").size(board_size))
                        }
                    }
                    Space::Black => button(text("●").size(board_size)),
                    Space::King => button(text("△").size(board_size)),
                    Space::White => button(text("○").size(board_size)),
                };

                if let Some(Ok(legal_moves)) = &possible_moves {
                    if let Some(vertex_from) = self.play_from.as_ref() {
                        if let Some(vertexes) = legal_moves.moves.get(vertex_from) {
                            if vertexes.contains(&vertex) {
                                button = button.on_press(Message::PlayMoveTo(vertex));
                            }
                        }
                    } else if let Some(_vertexes) = legal_moves.moves.get(&vertex) {
                        button = button.on_press(Message::PlayMoveFrom(vertex));
                    }
                }

                row = row.push(button);
            }

            row = row.push(text(format!("{y_label:2}")).size(board_size).center());

            game_display = game_display.push(row);
        }

        game_display = game_display.push(row_letters_2.spacing(23));
        game_display
    }

    fn pass_messages(_self: &Self) -> Subscription<Message> {
        Subscription::run(pass_messages)
    }

    fn texting(&self, in_game: bool) -> Column<Message> {
        let text_input = text_input("", &self.text_input)
            .on_input(Message::TextChanged)
            .on_paste(Message::TextChanged)
            .on_submit(Message::TextSend);

        let mut texting = Column::new();
        if in_game {
            for message in &self.texts_game {
                texting = texting.push(text(message));
            }
        } else {
            for message in &self.texts {
                texting = texting.push(text(message));
            }
        }

        let text_input = column![text("texts:"), text_input,];

        column![text_input, scrollable(texting),]
    }

    pub fn theme(_self: &Self) -> Theme {
        Theme::SolarizedLight
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) {
        self.error = None;

        match message {
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
            Message::PasswordChanged(password) => {
                self.password = password;
            }
            Message::PlayMoveFrom(vertex) => self.play_from = Some(vertex),
            Message::PlayMoveTo(to) => {
                let Some(game) = &mut self.game else {
                    panic!("you have to be in a game to make a move");
                };
                let Some(from) = self.play_from.as_ref() else {
                    panic!("you have to have a from to get to to");
                };
                let Some(tx) = self.tx.as_mut() else {
                    panic!("you have to have a sender at this point")
                };

                handle_error(tx.send(format!(
                    "game {} play {} {from} {to}\n",
                    self.game_id, game.turn
                )));
                handle_error(game.read_line(&format!("play {} {from} {to}\n", game.turn)));
                self.play_from = None;
                self.my_turn = false;
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
                match text.next() {
                    Some("=") => match text.next() {
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
                            let users: Vec<&str> = text.collect();
                            let mut users_wins_losses_rating = Vec::new();
                            for user_wins_losses_rating in users.chunks_exact(4) {
                                let user = user_wins_losses_rating[0];
                                let wins = user_wins_losses_rating[1];
                                let losses = user_wins_losses_rating[2];
                                let rating = user_wins_losses_rating[3];

                                users_wins_losses_rating.push(format!(
                                    "{user}: wins: {wins}, losses: {losses}, rating: {rating}"
                                ));
                            }
                            self.users = users_wins_losses_rating;
                        }
                        Some("join_game") => {
                            self.game = Some(Game::default());
                            self.texts_game = VecDeque::new();
                            self.screen = Screen::Game;

                            let Some(attacker) = text.next() else {
                                panic!("the attacker should be supplied");
                            };
                            let Some(defender) = text.next() else {
                                panic!("the defender should be supplied");
                            };
                            self.attacker = attacker.to_string();
                            self.defender = defender.to_string();
                        }
                        Some("login") => self.screen = Screen::Games,
                        Some("new_game") => {
                            // = new_game game 1 david none
                            let next_word = text.next();
                            if Some("ready") == next_word {
                                self.game = Some(Game::default());
                                self.texts_game = VecDeque::new();
                                self.screen = Screen::Game;

                                let Some(attacker) = text.next() else {
                                    panic!("the attacker should be supplied");
                                };
                                let Some(defender) = text.next() else {
                                    panic!("the defender should be supplied");
                                };
                                self.attacker = attacker.to_string();
                                self.defender = defender.to_string();
                            } else if Some("game") == next_word {
                                let Some(game_id) = text.next() else {
                                    panic!("the game id should be next");
                                };
                                let Ok(game_id) = game_id.parse() else {
                                    panic!("the game_id should be a u64")
                                };
                                self.game_id = game_id;
                            }
                        }
                        Some("text") => {
                            let text: Vec<&str> = text.collect();
                            let text = text.join(" ");
                            self.texts.push_front(text);
                        }
                        Some("text_game") => {
                            let text: Vec<&str> = text.collect();
                            let text = text.join(" ");
                            self.texts_game.push_front(text);
                        }
                        _ => {}
                    },
                    Some("?") => {
                        if Some("login") == text.next() {
                            exit(1);
                        }
                    }
                    Some("game") => {
                        // Plays the move then sends the result back.
                        let Some(index) = text.next() else {
                            return;
                        };
                        let Ok(id) = index.parse::<u64>() else {
                            panic!("the game_id should be a valid u64");
                        };
                        self.game_id = id;

                        // game 0 generate_move black
                        let text_word = text.next();
                        if text_word == Some("generate_move") {
                            let username_start: String = self.username.chars().take(3).collect();
                            if username_start == "ai-" {
                                let Some(color) = text.next() else {
                                    return;
                                };
                                let Ok(color) = Color::try_from(color) else {
                                    return;
                                };
                                let Some(game) = &mut self.game else {
                                    panic!("a game should exist to play in one");
                                };

                                let result = game
                                    .read_line(&format!("generate_move {color}"))
                                    .expect("generate_move should be valid")
                                    .expect("an empty string wasn't passed");

                                let Some(tx) = &self.tx else {
                                    panic!("there should be an established channel by now");
                                };

                                handle_error(tx.send(format!("game {index} play {result}\n")));
                            } else {
                                self.my_turn = true;
                            }
                        // game 0 play black a3 a4
                        } else if text_word == Some("play") {
                            let Some(color) = text.next() else {
                                return;
                            };
                            let Ok(color) = Color::try_from(color) else {
                                return;
                            };
                            let Some(from) = text.next() else {
                                return;
                            };
                            let Some(to) = text.next() else {
                                return;
                            };
                            let Some(game) = &mut self.game else {
                                panic!("a game should exist to play in one");
                            };

                            game.read_line(&format!("play {color} {from} {to}"))
                                .expect("play should be valid")
                                .expect("an empty string wasn't passed");
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

                    if self.screen == Screen::Login {
                        if let Some(username) = self.text_input.split_ascii_whitespace().next() {
                            let username = username.to_ascii_lowercase();
                            handle_error(tx.send(format!("{username} {}\n", self.password)));
                            self.username = username;
                        }
                    } else if self.screen == Screen::Game {
                        self.text_input.push('\n');
                        handle_error(
                            tx.send(format!("text_game {} {}", self.game_id, self.text_input)),
                        );
                    } else {
                        self.text_input.push('\n');
                        handle_error(tx.send(format!("text {}", self.text_input)));
                    }
                    self.text_input.clear();
                }
            }
        }
    }

    #[must_use]
    fn user_area(&self, in_game: bool) -> Container<Message> {
        let mut games = Column::new().padding(PADDING);
        games = games.push(text("games:"));
        for game in &self.games {
            let id = game.id;
            let join = button("join").on_press(Message::GameJoin(id));
            games = games.push(row![text(game.to_string()), join]);
        }

        let texting = self.texting(in_game).padding(PADDING);

        let mut users = column![text("users:")].padding(PADDING);
        for user in &self.users {
            users = users.push(text(user));
        }

        let user_area = row![games, texting, users];
        container(user_area)
            .padding(PADDING)
            .style(container::bordered_box)
    }

    #[must_use]
    pub fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Game => {
                let leave_game = button("Leave Game").on_press(Message::GameLeave);
                let user_area = row![
                    text(format!(
                        "username: {}, attacker: {}, defender: {}",
                        &self.username, &self.attacker, &self.defender
                    )),
                    leave_game,
                ]
                .spacing(SPACING);
                let user_area = container(user_area)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let texting = self.texting(true);
                let game = self.board();
                let game = row![game, texting];

                column![user_area, game].into()
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
                let username = row![text("username: "), text(&self.username)];
                let create_game = button("Create Game").on_press(Message::GameNew);
                let top = row![username, create_game].spacing(SPACING);
                let top = container(top)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let user_area = self.user_area(false);

                column![top, user_area].into()
            }
            Screen::Login => {
                let username = row![
                    text("username:"),
                    text_input("", &self.text_input)
                        .on_input(Message::TextChanged)
                        .on_submit(Message::TextSend),
                ];

                let password = row![
                    text("password:"),
                    text_input("", &self.password)
                        .on_input(Message::PasswordChanged)
                        .on_submit(Message::TextSend),
                ];

                column![username, password].spacing(SPACING).into()
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    GameJoin(u64),
    GameLeave,
    GameNew,
    GameSubmit,
    PasswordChanged(String),
    PlayMoveFrom(Vertex),
    PlayMoveTo(Vertex),
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
