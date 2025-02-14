use std::{
    collections::VecDeque,
    env, f64, fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::exit,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use clap::{command, Parser};
use env_logger::Builder;
use futures::executor;
use hnefatafl_copenhagen::{
    color::Color,
    game::Game,
    handle_error,
    play::Vertex,
    rating::Rated,
    role::Role,
    server_game::{ServerGameLight, ServerGamesLight},
    space::Space,
    status::Status,
    time::{Time, TimeSettings},
    HOME, VERSION_ID,
};
use iced::{
    font::Font,
    futures::{SinkExt, Stream},
    stream,
    widget::{
        button, checkbox, column, container, radio, row, scrollable, text, text_input, Column,
        Container, Row, Scrollable,
    },
    Element, Subscription, Theme,
};
use log::{debug, error, info, LevelFilter};

const PORT: &str = ":49152";
const PADDING: u16 = 10;
const SPACING: u16 = 10;
const SPACING_B: u16 = 20;

/// A Hnefatafl Copenhagen Client
///
/// This is a TCP client that connects to a server.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Connect to the HTP server at host
    #[arg(default_value = "hnefatafl.org", long)]
    host: String,
}

fn main() -> anyhow::Result<()> {
    init_logger();

    iced::application("Hnefatafl Copenhagen", Client::update, Client::view)
        .default_font(Font::MONOSPACE)
        .subscription(Client::subscriptions)
        .theme(Client::theme)
        .run()?;

    Ok(())
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
struct Client {
    attacker: String,
    defender: String,
    challenger: bool,
    error: Option<String>,
    game: Option<Game>,
    game_id: usize,
    games_light: ServerGamesLight,
    my_turn: bool,
    password: String,
    password_real: String,
    password_show: bool,
    play_from: Option<Vertex>,
    rated: Rated,
    role_selected: Option<Role>,
    screen: Screen,
    screen_size: Size,
    sound_muted: bool,
    status: Status,
    texts: VecDeque<String>,
    texts_game: VecDeque<String>,
    text_input: String,
    timed: TimeSettings,
    time_minutes: String,
    time_add_seconds: String,
    time_attacker: TimeSettings,
    time_defender: TimeSettings,
    tx: Option<mpsc::Sender<String>>,
    username: String,
    users: Vec<User>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Screen {
    #[default]
    Login,
    Game,
    GameNew,
    GameNewFrozen,
    Games,
    Users,
}

impl Client {
    #[must_use]
    fn board(&self) -> Column<Message> {
        let (board_size, spacing) = match self.screen_size {
            Size::Large => (50, 23),
            Size::Small => (35, 22),
        };

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
        game_display = game_display.push(row_letters_1.spacing(spacing));

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
                    } else if legal_moves.moves.contains_key(&vertex) {
                        button = button.on_press(Message::PlayMoveFrom(vertex));
                    }
                }

                row = row.push(button);
            }

            row = row.push(text(format!("{y_label:2}")).size(board_size).center());

            game_display = game_display.push(row);
        }

        game_display = game_display.push(row_letters_2.spacing(spacing));
        game_display
    }

    fn subscriptions(&self) -> Subscription<Message> {
        let subscription_1 = if let Some(game) = &self.game {
            if game.timer.is_some() {
                iced::time::every(iced::time::Duration::from_millis(100))
                    .map(|_instant| Message::Tick)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        let subscription_2 = Subscription::run(pass_messages);

        Subscription::batch(vec![subscription_1, subscription_2])
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

        let text_input = column![text("texts"), text("-----"), text_input];

        column![text_input, scrollable(texting),]
    }

    pub fn theme(_self: &Self) -> Theme {
        Theme::SolarizedLight
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) {
        self.error = None;

        match message {
            Message::GameAccept(id) => {
                self.send(format!("join_game {id}\n"));
                self.screen = Screen::Game;
                self.status = Status::Ongoing;
            }
            Message::GameDecline(id) => {
                self.send(format!("decline_game {id}\n"));
            }
            Message::GameJoin(id) => {
                self.send(format!("join_game_pending {id}\n"));

                let Some(game) = self.games_light.0.get(&id) else {
                    panic!("the game must exist");
                };

                self.role_selected = if game.attacker.is_some() {
                    Some(Role::Defender)
                } else {
                    Some(Role::Attacker)
                };

                self.screen = Screen::GameNewFrozen;
            }
            Message::GameWatch(id) => {
                self.game_id = id;
                self.send(format!("watch_game {id}\n"));
            }
            Message::Leave => match self.screen {
                Screen::Game | Screen::Users => {
                    self.screen = Screen::Games;
                }
                Screen::GameNewFrozen => {
                    self.send(format!("leave_game {}\n", self.game_id));
                    self.screen = Screen::Games;
                }
                Screen::GameNew | Screen::Games | Screen::Login => {}
            },
            Message::OpenWebsite => open_url("https://hnefatafl.org"),
            Message::GameNew => self.screen = Screen::GameNew,
            Message::GameSubmit => {
                if let Some(role) = self.role_selected {
                    if self.timed.0.is_some() {
                        match (
                            self.time_minutes.parse::<u128>(),
                            self.time_add_seconds.parse::<u128>(),
                        ) {
                            (Ok(minutes), Ok(add_seconds)) => {
                                self.timed.0 = Some(Time {
                                    add_seconds,
                                    milliseconds_left: minutes * 60_000,
                                });
                            }
                            (Ok(minutes), Err(_)) => {
                                self.timed.0 = Some(Time {
                                    add_seconds: 10,
                                    milliseconds_left: minutes * 60_000,
                                });
                            }
                            (Err(_), Ok(add_seconds)) => {
                                self.timed.0 = Some(Time {
                                    add_seconds,
                                    milliseconds_left: 15 * 60_000,
                                });
                            }
                            (Err(_), Err(_)) => {
                                self.timed.0 = Some(Time {
                                    add_seconds: 10,
                                    milliseconds_left: 15 * 60_000,
                                });
                            }
                        }
                    }

                    self.screen = Screen::GameNewFrozen;

                    // new_game (attacker | defender) (rated | unrated) [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
                    self.send(format!("new_game {role} {} {:?}\n", self.rated, self.timed));
                }
            }
            Message::PasswordChanged(mut password) => {
                let password_len: Vec<_> = password.chars().collect();
                let password_len = password_len.len();
                let self_password_len: Vec<_> = self.password.chars().collect();
                let self_password_len = self_password_len.len();

                if password_len == self_password_len + 1 {
                    if let Some(ch) = password.pop() {
                        self.password_real.push(ch);
                        self.password.push('●');
                    }
                } else if password_len + 1 == self_password_len {
                    self.password.pop();
                    self.password_real.pop();
                }
            }
            Message::PasswordShow(show_password) => {
                self.password_show = show_password;
            }
            Message::PlayMoveFrom(vertex) => self.play_from = Some(vertex),
            Message::PlayMoveTo(to) => {
                let Some(from) = self.play_from.clone() else {
                    panic!("you have to have a from to get to to");
                };

                self.handle_play(None, &from.to_string(), &to.to_string());
                let Some(game) = &mut self.game else {
                    panic!("you have to be in a game to play a move")
                };

                let Some(tx) = self.tx.as_mut() else {
                    panic!("you have to have a sender at this point")
                };

                handle_error(tx.send(format!(
                    "game {} play {} {from} {to}\n",
                    self.game_id,
                    game.turn.opposite()
                )));

                if game.status == Status::Ongoing {
                    match game.turn {
                        Color::Black => {
                            if let Some(time) = &mut self.time_attacker.0 {
                                time.milliseconds_left += time.add_seconds * 1_000;
                            }
                        }
                        Color::Colorless => {}
                        Color::White => {
                            if let Some(time) = &mut self.time_defender.0 {
                                time.milliseconds_left += time.add_seconds * 1_000;
                            }
                        }
                    }
                }

                self.play_from = None;
                self.my_turn = false;
            }
            Message::PlayResign => {
                let Some(game) = &mut self.game else {
                    panic!("you have to be in a game to make a move");
                };
                let Some(tx) = self.tx.as_mut() else {
                    panic!("you have to have a sender at this point")
                };

                handle_error(tx.send(format!(
                    "game {} play {} resigns _\n",
                    self.game_id, game.turn
                )));
            }
            Message::ScreenSize(size) => self.screen_size = size,
            Message::SoundMuted(muted) => self.sound_muted = muted,
            Message::TcpConnected(tx) => {
                info!("TCP connected...");
                self.tx = Some(tx);
            }
            Message::RatedSelected(rated) => {
                self.rated = if rated { Rated::Yes } else { Rated::No };
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
                        Some("display_games") => {
                            self.games_light.0.clear();
                            let games: Vec<&str> = text.collect();
                            for chunks in games.chunks_exact(10) {
                                let id = chunks[1];
                                let attacker = chunks[2];
                                let defender = chunks[3];
                                let rated = chunks[4];
                                let timed = chunks[5];
                                let minutes = chunks[6];
                                let add_seconds = chunks[7];
                                let challenger = chunks[8];
                                let challenge_accepted = chunks[9];

                                let game = ServerGameLight::try_from((
                                    id,
                                    attacker,
                                    defender,
                                    rated,
                                    timed,
                                    minutes,
                                    add_seconds,
                                    challenger,
                                    challenge_accepted,
                                ))
                                .expect("the value should be a valid ServerGameLight");

                                self.games_light.0.insert(game.id, game);
                            }
                        }
                        Some("display_users") => {
                            let users: Vec<&str> = text.collect();
                            self.users.clear();
                            for user_wins_losses_rating in users.chunks_exact(5) {
                                let rating = user_wins_losses_rating[3];
                                let Some((rating, deviation)) = rating.split_once("±") else {
                                    panic!("the ratings has this form");
                                };
                                let (Ok(rating), Ok(deviation)) =
                                    (rating.parse::<f64>(), deviation.parse::<f64>())
                                else {
                                    panic!("the ratings has this form");
                                };

                                let logged_in = "logged_in" == user_wins_losses_rating[4];

                                self.users.push(User {
                                    name: user_wins_losses_rating[0].to_string(),
                                    wins: user_wins_losses_rating[1].to_string(),
                                    losses: user_wins_losses_rating[2].to_string(),
                                    rating: (rating, deviation),
                                    logged_in,
                                });
                            }
                            self.users
                                .sort_by(|a, b| b.rating.0.partial_cmp(&a.rating.0).unwrap());
                        }
                        Some("game_over") => {
                            self.my_turn = false;

                            text.next();
                            match text.next() {
                                Some("attacker_wins") => self.status = Status::BlackWins,
                                Some("defender_wins") => self.status = Status::WhiteWins,
                                _ => {}
                            }
                        }
                        // = join_game david abby rated fischer 900_000 10
                        Some("join_game" | "watch_game") => {
                            self.texts_game = VecDeque::new();
                            self.screen = Screen::Game;
                            self.status = Status::Ongoing;

                            let Some(attacker) = text.next() else {
                                panic!("the attacker should be supplied");
                            };
                            let Some(defender) = text.next() else {
                                panic!("the defender should be supplied");
                            };
                            self.attacker = attacker.to_string();
                            self.defender = defender.to_string();

                            let Some(_rated) = text.next() else {
                                panic!("there should be rated or unrated supplied");
                            };

                            let Some(timed) = text.next() else {
                                panic!("there should be a time setting supplied");
                            };
                            let Some(minutes) = text.next() else {
                                panic!("there should be a minutes supplied");
                            };
                            let Some(add_seconds) = text.next() else {
                                panic!("there should be a add_seconds supplied");
                            };
                            let Ok(timed) = TimeSettings::try_from(vec![
                                "time_settings",
                                timed,
                                minutes,
                                add_seconds,
                            ]) else {
                                panic!("there should be a valid time settings");
                            };

                            let mut game = Game {
                                black_time: timed.clone(),
                                white_time: timed.clone(),
                                timer: Some(Instant::now()),
                                ..Game::default()
                            };

                            self.time_attacker = timed.clone();
                            self.time_defender = timed;

                            if let Some(board) = text.next() {
                                let board = ron::from_str(board)
                                    .expect("we should be able to deserialize the board");

                                game.board = board;
                            }

                            let texts: Vec<&str> = text.collect();
                            let texts = texts.join(" ");
                            if !texts.is_empty() {
                                let texts = ron::from_str(&texts)
                                    .expect("we should be able to deserialize the text");
                                println!("{texts:?}");
                                self.texts_game = texts;
                            }

                            self.game = Some(game);
                        }
                        Some("join_game_pending") => {
                            self.challenger = true;
                            let Some(id) = text.next() else {
                                panic!("there should be an id supplied");
                            };
                            let Ok(id) = id.parse() else {
                                panic!("id should be a valid usize");
                            };
                            self.game_id = id;
                        }
                        Some("leave_game") => self.game_id = 0,
                        Some("login") => self.screen = Screen::Games,
                        Some("new_game") => {
                            // = new_game game 15 none david rated fischer 900_000 10
                            if Some("game") == text.next() {
                                self.challenger = false;
                                let Some(game_id) = text.next() else {
                                    panic!("the game id should be next");
                                };
                                let Ok(game_id) = game_id.parse() else {
                                    panic!("the game_id should be a usize")
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
                        let Ok(id) = index.parse::<usize>() else {
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

                                self.send(format!("game {index} play {result} _\n"));
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

                            self.handle_play(Some(&color.to_string()), from, to);
                            let Some(game) = &mut self.game else {
                                panic!("you have to be in a game to play a move")
                            };

                            if game.status == Status::Ongoing {
                                match game.turn {
                                    Color::Black => {
                                        if let Some(time) = &mut self.time_attacker.0 {
                                            time.milliseconds_left += time.add_seconds * 1_000;
                                        }
                                    }
                                    Color::Colorless => {}
                                    Color::White => {
                                        if let Some(time) = &mut self.time_defender.0 {
                                            time.milliseconds_left += time.add_seconds * 1_000;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::TextSend => {
                if self.text_input.trim().is_empty() {
                    return;
                }

                if self.screen == Screen::Login {
                    if let Some(username) = self.text_input.split_ascii_whitespace().next() {
                        let username = username.to_ascii_lowercase();
                        self.send(format!("{VERSION_ID} {username} {}\n", self.password_real));
                        self.username = username;
                    }
                } else if self.screen == Screen::Game {
                    self.text_input.push('\n');
                    self.send(format!("text_game {} {}", self.game_id, self.text_input));
                } else {
                    self.text_input.push('\n');
                    self.send(format!("text {}", self.text_input));
                }
                self.text_input.clear();
            }
            Message::Tick => {
                if let Some(game) = &mut self.game {
                    match game.turn {
                        Color::Black => {
                            if let Some(time) = &mut self.time_attacker.0 {
                                time.milliseconds_left = time.milliseconds_left.saturating_sub(100);

                                if self.my_turn && time.milliseconds_left == 0 {
                                    if let Some(tx) = &mut self.tx {
                                        handle_error(tx.send(format!(
                                            "game {} play black resigns _\n",
                                            self.game_id
                                        )));
                                    }

                                    handle_error(game.read_line("play black resigns"));
                                    self.time_attacker = TimeSettings(None);
                                }
                            }
                        }
                        Color::Colorless => {}
                        Color::White => {
                            if let Some(time) = &mut self.time_defender.0 {
                                time.milliseconds_left = time.milliseconds_left.saturating_sub(100);

                                if self.my_turn && time.milliseconds_left == 0 {
                                    if let Some(tx) = &mut self.tx {
                                        handle_error(tx.send(format!(
                                            "game {} play white resigns _\n",
                                            self.game_id
                                        )));
                                    }

                                    handle_error(game.read_line("play white resigns"));
                                    self.time_defender = TimeSettings(None);
                                }
                            }
                        }
                    }
                }
            }
            Message::TimeAddSeconds(string) => {
                if string.parse::<u64>().is_ok() {
                    self.time_add_seconds = string;
                }
            }
            Message::TimeCheckbox(time_selected) => {
                if time_selected {
                    self.timed.0 = Some(Time {
                        add_seconds: 10,
                        milliseconds_left: 15 * 60_000,
                    });
                } else {
                    self.timed.0 = None;
                }
            }
            Message::TimeMinutes(string) => {
                if string.parse::<u64>().is_ok() {
                    self.time_minutes = string;
                }
            }
            Message::Users => self.screen = Screen::Users,
        }
    }

    #[must_use]
    fn games(&self) -> Scrollable<Message> {
        let mut game_ids = Column::new().spacing(SPACING_B);
        let mut attackers = Column::new().spacing(SPACING_B);
        let mut defenders = Column::new().spacing(SPACING_B);
        let mut ratings = Column::new().spacing(SPACING_B);
        let mut timings = Column::new().spacing(SPACING_B);
        let mut buttons = Column::new().spacing(SPACING);

        for game in self.games_light.0.values() {
            let id = game.id;
            game_ids = game_ids.push(text(id));

            attackers = if let Some(attacker) = &game.attacker {
                attackers.push(text(attacker))
            } else {
                attackers.push(text("none"))
            };
            defenders = if let Some(defender) = &game.defender {
                defenders.push(text(defender))
            } else {
                defenders.push(text("none"))
            };

            ratings = ratings.push(text(game.rated.to_string()));
            timings = timings.push(text(game.timed.to_string()));

            if game.challenge_accepted {
                buttons = buttons.push(button("watch").on_press(Message::GameWatch(id)));
            } else if game.attacker.is_some() && game.defender.is_some() {
                buttons = buttons.push(button("join"));
            } else {
                buttons = buttons.push(button("join").on_press(Message::GameJoin(id)));
            }
        }

        let game_ids = column![text("game_id"), text("-------"), game_ids].padding(PADDING);
        let attackers = column![text("attacker"), text("--------"), attackers].padding(PADDING);
        let defenders = column![text("defender"), text("--------"), defenders].padding(PADDING);
        let ratings = column![text("rated"), text("-----"), ratings].padding(PADDING);
        let timings = column![text("timed"), text("-----"), timings].padding(PADDING);
        let buttons = column![text(""), text(""), buttons].padding(PADDING);

        scrollable(row![
            game_ids, attackers, defenders, ratings, timings, buttons
        ])
    }

    fn handle_play(&mut self, color: Option<&str>, from: &str, to: &str) {
        let mut capture = false;

        let Some(game) = &mut self.game else {
            panic!("you have to be in a game to play a move")
        };

        match color {
            Some(color) => match game.read_line(&format!("play {color} {from} {to}\n")) {
                Ok(value) => {
                    if let Some(value) = value {
                        if !value.trim().is_empty() {
                            capture = true;
                        }
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    exit(1)
                }
            },
            None => match game.read_line(&format!("play {} {from} {to}\n", game.turn)) {
                Ok(value) => {
                    if let Some(value) = value {
                        if !value.trim().is_empty() {
                            capture = true;
                        }
                    }
                }
                Err(error) => {
                    eprintln!("error: {error}");
                    exit(1)
                }
            },
        }

        if self.sound_muted {
            return;
        }

        thread::spawn(move || {
            if let Some(mut path) = dirs::data_dir() {
                path = path.join(HOME);

                let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                let file = if capture {
                    fs::File::open(path.join("capture.ogg")).unwrap()
                } else {
                    fs::File::open(path.join("move.ogg")).unwrap()
                };
                let sound = stream_handle.play_once(file).unwrap();
                sound.set_volume(1.0);
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    #[must_use]
    fn users(&self, logged_in: bool) -> Scrollable<Message> {
        let mut ratings = Column::new();
        let mut usernames = Column::new();
        let mut wins = Column::new();
        let mut losses = Column::new();

        for user in &self.users {
            if logged_in == user.logged_in {
                ratings = ratings.push(text(format!("{} ± {}", user.rating.0, user.rating.1)));
                usernames = usernames.push(text(&user.name));
                wins = wins.push(text(&user.wins));
                losses = losses.push(text(&user.losses));
            }
        }

        let ratings = column![text("rating"), text("------"), ratings].padding(PADDING);
        let usernames = column![text("username"), text("--------"), usernames].padding(PADDING);
        let wins = column![text("wins"), text("----"), wins].padding(PADDING);
        let losses = column![text("losses"), text("------"), losses].padding(PADDING);

        scrollable(row![ratings, usernames, wins, losses])
    }

    #[must_use]
    fn user_area(&self, in_game: bool) -> Container<Message> {
        let games = self.games();
        let texting = self.texting(in_game).padding(PADDING);
        let users = self.users(true);

        let user_area = column![games, users];
        let user_area = row![texting, user_area];
        container(user_area)
            .padding(PADDING)
            .style(container::bordered_box)
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Game => {
                let user_area_ = row![text(format!(
                    "username: {}, attacker: {}, defender: {}",
                    &self.username, &self.attacker, &self.defender
                )),]
                .spacing(SPACING);
                let user_area_ = container(user_area_)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let texting = self.texting(true);

                let time = row![
                    text(format!("black time: {}", self.time_attacker)),
                    text(format!("white time: {}", self.time_defender)),
                ]
                .spacing(SPACING);

                let mut user_area = Column::new().spacing(SPACING);
                user_area = user_area.push(user_area_);

                if self.my_turn {
                    user_area = user_area.push(
                        row![
                            button("Resign").on_press(Message::PlayResign),
                            button("Leave").on_press(Message::Leave),
                            button("Small Screen").on_press(Message::ScreenSize(Size::Small)),
                            button("Large Screen").on_press(Message::ScreenSize(Size::Large)),
                        ]
                        .spacing(SPACING),
                    );
                } else {
                    user_area = user_area.push(
                        row![
                            button("Resign"),
                            button("Leave").on_press(Message::Leave),
                            button("Small Screen").on_press(Message::ScreenSize(Size::Small)),
                            button("Large Screen").on_press(Message::ScreenSize(Size::Large)),
                        ]
                        .spacing(SPACING),
                    );
                }

                user_area = user_area
                    .push(checkbox("sound muted", self.sound_muted).on_toggle(Message::SoundMuted));

                if self.status == Status::WhiteWins {
                    user_area = user_area.push(text("Defender Wins!"));
                }
                if self.status == Status::BlackWins {
                    user_area = user_area.push(text("Attacker Wins!"));
                }
                user_area = user_area.push(time);
                user_area = user_area.push(texting);
                let user_area = container(user_area)
                    .padding(PADDING)
                    .style(container::bordered_box);

                row![self.board(), user_area].spacing(SPACING).into()
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

                let rated = checkbox("rated", self.rated.into()).on_toggle(Message::RatedSelected);

                let mut new_game = button("New Game");
                if self.role_selected.is_some() {
                    new_game = new_game.on_press(Message::GameSubmit);
                }

                let mut time =
                    row![checkbox("timed ", self.timed.clone().into())
                        .on_toggle(Message::TimeCheckbox)];

                if self.timed.0.is_some() {
                    time = time.push(text("minutes"));
                    time = time
                        .push(text_input("15", &self.time_minutes).on_input(Message::TimeMinutes));
                    time = time.push(text("add seconds"));
                    time = time.push(
                        text_input("10", &self.time_add_seconds).on_input(Message::TimeAddSeconds),
                    );
                }
                time = time.spacing(SPACING);

                row![new_game, text("role: "), attacker, defender, rated, time]
                    .padding(PADDING)
                    .spacing(SPACING)
                    .into()
            }
            Screen::GameNewFrozen => {
                let Some(role) = self.role_selected else {
                    panic!("You can't get to GameNewFrozen unless you have selected a role!");
                };

                let mut buttons_live = false;
                let mut game_display = column![text(format!("role: {role}"))].padding(PADDING);
                if let Some(game) = self.games_light.0.get(&self.game_id) {
                    game_display = game_display.push(text(game.to_string()));

                    if game.attacker.is_some() && game.defender.is_some() {
                        buttons_live = true;
                    }
                };

                let mut buttons = if self.challenger {
                    row![button("Leave").on_press(Message::Leave)]
                } else if buttons_live {
                    row![
                        button("Accept").on_press(Message::GameAccept(self.game_id)),
                        button("Decline").on_press(Message::GameDecline(self.game_id)),
                        button("Leave").on_press(Message::Leave),
                    ]
                } else {
                    row![
                        button("Accept"),
                        button("Decline"),
                        button("Leave").on_press(Message::Leave),
                    ]
                };
                buttons = buttons.padding(PADDING).spacing(SPACING);

                game_display.push(buttons).into()
            }
            Screen::Games => {
                let username = row![text("username: "), text(&self.username)];
                let create_game = button("Create Game").on_press(Message::GameNew);
                let users = button("Users").on_press(Message::Users);
                let website = button("Website").on_press(Message::OpenWebsite);

                let top = row![username, create_game, users, website].spacing(SPACING);
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

                let password = if self.password_show {
                    row![
                        text("password:"),
                        text_input("", &self.password_real)
                            .on_input(Message::PasswordChanged)
                            .on_submit(Message::TextSend),
                    ]
                } else {
                    row![
                        text("password:"),
                        text_input("", &self.password)
                            .on_input(Message::PasswordChanged)
                            .on_submit(Message::TextSend),
                    ]
                };

                let show_password =
                    checkbox("show password", self.password_show).on_toggle(Message::PasswordShow);

                column![username, password, show_password]
                    .padding(PADDING)
                    .spacing(SPACING)
                    .into()
            }
            Screen::Users => column![
                row![button("Leave").on_press(Message::Leave)].padding(PADDING),
                text("logged in"),
                self.users(true),
                text("logged out"),
                self.users(false),
            ]
            .into(),
        }
    }

    fn send(&mut self, string: String) {
        self.tx
            .as_mut()
            .expect("you should have a tx available by now")
            .send(string)
            .unwrap();
    }
}

#[derive(Clone, Debug)]
enum Message {
    GameAccept(usize),
    GameDecline(usize),
    GameJoin(usize),
    GameNew,
    GameSubmit,
    GameWatch(usize),
    Leave,
    OpenWebsite,
    PasswordChanged(String),
    PasswordShow(bool),
    PlayMoveFrom(Vertex),
    PlayMoveTo(Vertex),
    PlayResign,
    ScreenSize(Size),
    SoundMuted(bool),
    RatedSelected(bool),
    RoleSelected(Role),
    TcpConnected(mpsc::Sender<String>),
    TextChanged(String),
    TextReceived(String),
    TextSend,
    Tick,
    TimeAddSeconds(String),
    TimeCheckbox(bool),
    TimeMinutes(String),
    Users,
}

fn pass_messages() -> impl Stream<Item = Message> {
    stream::channel(100, move |mut sender| async move {
        let mut args = Args::parse();
        args.host.push_str(PORT);
        let address = args.host;

        let mut tcp_stream = handle_error(TcpStream::connect(&address));

        let reader = handle_error(tcp_stream.try_clone());
        let mut reader = BufReader::new(reader);
        info!("connected to {address} ...");

        let (tx, rx) = mpsc::channel();
        let _ = sender.send(Message::TcpConnected(tx)).await;
        thread::spawn(move || loop {
            let message = handle_error(rx.recv());
            let message_trim = message.trim();
            debug!("<- {message_trim}");

            handle_error(tcp_stream.write_all(message.as_bytes()));
        });

        thread::spawn(move || {
            let mut buffer = String::new();
            loop {
                let bytes = handle_error(reader.read_line(&mut buffer));
                if bytes > 0 {
                    let buffer_trim = buffer.trim();
                    debug!("-> {buffer_trim}");
                    handle_error(executor::block_on(
                        sender.send(Message::TextReceived(buffer.clone())),
                    ));
                    buffer.clear();
                } else {
                    error!("error: the TCP stream has closed");
                    exit(1);
                }
            }
        });
    })
}

fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S %z"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
    }

    builder.init();
}

fn open_url(url: &str) {
    if let Err(error) = webbrowser::open(url) {
        error!("{error}");
    }
}

#[derive(Clone, Debug, Default)]
enum Size {
    #[default]
    Small,
    Large,
}

#[derive(Clone, Debug)]
struct User {
    name: String,
    wins: String,
    losses: String,
    rating: (f64, f64),
    logged_in: bool,
}
