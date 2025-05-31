// Don't open the terminal on Windows.
#![windows_subsystem = "windows"]

#[macro_use]
extern crate rust_i18n;

#[cfg(feature = "sound")]
use std::{io::Cursor, time::Duration};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, f64,
    fmt::{self, Write as fmt_write},
    fs::{self, File},
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{Shutdown, TcpStream},
    path::PathBuf,
    process::exit,
    str::{FromStr, SplitAsciiWhitespace},
    sync::mpsc::{self, Sender},
    thread,
};

use chrono::{Local, Utc};
use clap::{Parser, command};
use env_logger::Builder;
use futures::{SinkExt, executor};
use hnefatafl_copenhagen::{
    VERSION_ID,
    accounts::Email,
    color::Color,
    draw::Draw,
    game::Game,
    glicko::{CONFIDENCE_INTERVAL_95, Rating},
    handle_error,
    play::{BOARD_LETTERS, Vertex},
    rating::Rated,
    role::Role,
    server_game::{ServerGameLight, ServerGamesLight},
    space::Space,
    status::Status,
    time::{Time, TimeSettings},
};
#[cfg(target_os = "linux")]
use iced::window::settings::PlatformSpecific;
use iced::{
    Element, Event, Pixels, Subscription,
    alignment::{Horizontal, Vertical},
    event,
    font::Font,
    futures::Stream,
    stream,
    widget::{
        Column, Container, Row, Scrollable, button, checkbox, column, container, pick_list, radio,
        row, scrollable, text, text_editor, text_input, tooltip,
    },
    window::{self, icon},
};
use log::{LevelFilter, debug, error, info, trace};
use rust_i18n::t;
use serde::{Deserialize, Serialize};

const PORT: &str = ":49152";
const PADDING: u16 = 10;
const SPACING: Pixels = Pixels(10.0);
const SPACING_B: Pixels = Pixels(20.0);

i18n!();

/// A Hnefatafl Copenhagen Client
///
/// This is a TCP client that connects to a server.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Connect to the HTP server at host.
    #[arg(default_value = "hnefatafl.org", long)]
    host: String,
}

fn init_client() -> Client {
    let data_file = data_file();
    let mut error = None;
    let mut client: Client = match &fs::read_to_string(&data_file) {
        Ok(string) => match ron::from_str(string.as_str()) {
            Ok(client) => client,
            Err(err) => {
                error = Some(format!(
                    "error parsing the ron file {}: {err}",
                    data_file.display()
                ));
                Client::default()
            }
        },
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                Client::default()
            } else {
                error = Some(format!(
                    "error opening the file {}: {err}",
                    data_file.display()
                ));
                Client::default()
            }
        }
    };

    if error.is_some() {
        client.error_persistent = error;
    }

    rust_i18n::set_locale(&client.locale_selected.txt());

    client.text_input = client.username.clone();

    let mut strings = HashMap::new();

    strings.insert("Login".to_string(), t!("Login").to_string());

    strings.insert(
        "Create Account".to_string(),
        t!("Create Account").to_string(),
    );

    strings.insert(
        "Reset Password".to_string(),
        t!("Reset Password").to_string(),
    );

    strings.insert("Leave".to_string(), t!("Leave").to_string());
    strings.insert("Quit".to_string(), t!("Quit").to_string());
    strings.insert("Dark".to_string(), t!("Dark").to_string());
    strings.insert("Light".to_string(), t!("Light").to_string());
    strings.insert("Create Game".to_string(), t!("Create Game").to_string());
    strings.insert("Users".to_string(), t!("Users").to_string());

    strings.insert(
        "Account Settings".to_string(),
        t!("Account Settings").to_string(),
    );

    strings.insert("Rules".to_string(), t!("Rules").to_string());
    strings.insert("Reset Email".to_string(), t!("Reset Email").to_string());

    strings.insert(
        "Change Password".to_string(),
        t!("Change Password").to_string(),
    );

    strings.insert(
        "Delete Account".to_string(),
        t!("Delete Account").to_string(),
    );

    strings.insert(
        "REALLY DELETE ACCOUNT".to_string(),
        t!("REALLY DELETE ACCOUNT").to_string(),
    );

    strings.insert("New Game".to_string(), t!("New Game").to_string());
    strings.insert("Accept".to_string(), t!("Accept").to_string());
    strings.insert("Decline".to_string(), t!("Decline").to_string());
    strings.insert("Watch".to_string(), t!("Watch").to_string());
    strings.insert("Join".to_string(), t!("Join").to_string());
    strings.insert("Resume".to_string(), t!("Resume").to_string());

    strings.insert("Resign".to_string(), t!("Resign").to_string());
    strings.insert("Request Draw".to_string(), t!("Request Draw").to_string());
    strings.insert("Accept Draw".to_string(), t!("Accept Draw").to_string());

    client.strings = strings;
    client
}

fn main() -> anyhow::Result<()> {
    init_logger();

    #[cfg(not(feature = "icon_2"))]
    let king = include_bytes!("king_1_256x256.rgba").to_vec();

    #[cfg(feature = "icon_2")]
    let king = include_bytes!("king_2_256x256.rgba").to_vec();

    iced::application(init_client, Client::update, Client::view)
        .title("Hnefatafl Copenhagen")
        .subscription(Client::subscriptions)
        .window(window::Settings {
            #[cfg(target_os = "linux")]
            platform_specific: PlatformSpecific {
                #[cfg(feature = "icon_2")]
                application_id: "org.hnefatafl.hnefatafl_client".to_string(),
                #[cfg(not(feature = "icon_2"))]
                application_id: "hnefatafl-client".to_string(),
                ..PlatformSpecific::default()
            },
            icon: Some(icon::from_rgba(king, 256, 256)?),
            ..window::Settings::default()
        })
        // For screenshots.
        /*
        .window_size(iced::Size {
            width: 870.0,
            height: 541.0,
        })
        */
        .theme(Client::theme)
        .default_font(Font::MONOSPACE)
        .run()?;

    Ok(())
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default, Deserialize, Serialize)]
struct Client {
    #[serde(skip)]
    attacker: String,
    #[serde(skip)]
    defender: String,
    #[serde(skip)]
    delete_account: bool,
    #[serde(default)]
    email_everyone: bool,
    #[serde(skip)]
    captures: HashSet<Vertex>,
    #[serde(skip)]
    challenger: bool,
    #[serde(skip)]
    connected_to: String,
    #[serde(skip)]
    content: text_editor::Content,
    #[serde(skip)]
    email: Option<Email>,
    #[serde(skip)]
    emails_bcc: Vec<String>,
    #[serde(skip)]
    error: Option<String>,
    #[serde(skip)]
    error_email: Option<String>,
    #[serde(skip)]
    error_persistent: Option<String>,
    #[serde(skip)]
    game: Option<Game>,
    #[serde(skip)]
    game_id: usize,
    #[serde(skip)]
    games_light: ServerGamesLight,
    #[serde(default)]
    locale_selected: Locale,
    #[serde(skip)]
    my_turn: bool,
    #[serde(skip)]
    password: String,
    #[serde(skip)]
    password_no_save: bool,
    #[serde(skip)]
    password_show: bool,
    #[serde(skip)]
    play_from: Option<Vertex>,
    #[serde(skip)]
    play_from_previous: Option<Vertex>,
    #[serde(skip)]
    play_to_previous: Option<Vertex>,
    #[serde(skip)]
    rated: Rated,
    #[serde(skip)]
    request_draw: bool,
    #[serde(skip)]
    role_selected: Option<Role>,
    #[serde(skip)]
    screen: Screen,
    #[serde(skip)]
    screen_size: Size,
    #[serde(skip)]
    sound_muted: bool,
    #[serde(skip)]
    spectators: Vec<String>,
    #[serde(skip)]
    status: Status,
    #[serde(skip)]
    texts: VecDeque<String>,
    #[serde(skip)]
    texts_game: VecDeque<String>,
    #[serde(skip)]
    text_input: String,
    #[serde(default)]
    theme: Theme,
    #[serde(skip)]
    timed: TimeSettings,
    #[serde(skip)]
    time_minutes: String,
    #[serde(skip)]
    time_add_seconds: String,
    #[serde(skip)]
    time_attacker: TimeSettings,
    #[serde(skip)]
    time_defender: TimeSettings,
    #[serde(skip)]
    tx: Option<mpsc::Sender<String>>,
    #[serde(default)]
    username: String,
    #[serde(skip)]
    users: HashMap<String, User>,
    #[serde(skip)]
    strings: HashMap<String, String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Screen {
    AccountSettings,
    EmailEveryone,
    #[default]
    Login,
    Game,
    GameNew,
    GameNewFrozen,
    Games,
    Users,
}

impl Client {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    fn board(&self) -> Row<Message> {
        let letters: Vec<_> = BOARD_LETTERS.chars().collect();
        let (board_size, letter_size, piece_size, spacing) = match self.screen_size {
            Size::Large => (75, 55, 60, 6),
            Size::Medium => (65, 45, 50, 8),
            Size::Small => (55, 35, 40, 11),
            Size::Tiny => (40, 20, 25, 16),
        };

        let Some(game) = &self.game else {
            return row![];
        };

        let mut game_display = Row::new().spacing(2);

        let mut possible_moves = None;
        if self.my_turn {
            if let Some(game) = self.game.as_ref() {
                possible_moves = Some(game.all_legal_moves());
            }
        }

        let mut column = column![text(" ").size(letter_size)].spacing(spacing);

        for i in 0..11 {
            let i = 11 - i;
            column = column.push(
                text(format!("{i:2}"))
                    .size(letter_size)
                    .align_y(Vertical::Center),
            );
        }
        game_display = game_display.push(column);

        for (x, letter) in letters.iter().enumerate() {
            let mut column = Column::new().spacing(2).align_x(Horizontal::Center);
            column = column.push(text(letter).size(letter_size));

            for y in 0..11 {
                let vertex = Vertex { x, y };

                let mut text_ = match game.board.get(&vertex) {
                    Space::Empty => {
                        if (y, x) == (0, 0)
                            || (y, x) == (10, 0)
                            || (y, x) == (0, 10)
                            || (y, x) == (10, 10)
                            || (y, x) == (5, 5)
                        {
                            text("⌘")
                        } else {
                            text(" ")
                        }
                    }
                    Space::Black => text("♟"),
                    Space::King => text("♔"),
                    Space::White => text("♙"),
                };

                text_ = text_
                    .size(piece_size)
                    .shaping(text::Shaping::Advanced)
                    .center();

                if let (Some(from), Some(to)) = (&self.play_from_previous, &self.play_to_previous) {
                    let x_diff = from.x as i128 - to.x as i128;
                    let y_diff = from.y as i128 - to.y as i128;
                    let mut arrow = " ";

                    if y_diff < 0 {
                        arrow = "↓";
                    } else if y_diff > 0 {
                        arrow = "↑";
                    } else if x_diff < 0 {
                        arrow = "→";
                    } else if x_diff > 0 {
                        arrow = "←";
                    }

                    if (y, x) == (from.y, from.x) {
                        text_ = text(arrow)
                            .size(piece_size)
                            .shaping(text::Shaping::Advanced)
                            .center();
                    }
                }

                if self.captures.contains(&vertex) {
                    text_ = text("X").size(piece_size).center();
                }

                let mut button_ = button(text_).width(board_size).height(board_size);

                if let Some(legal_moves) = &possible_moves {
                    if let Some(vertex_from) = self.play_from.as_ref() {
                        if let Some(vertexes) = legal_moves.moves.get(vertex_from) {
                            if vertex == *vertex_from {
                                button_ = button_.on_press(Message::PlayMoveRevert);
                            }
                            if vertexes.contains(&vertex) {
                                button_ = button_.on_press(Message::PlayMoveTo(vertex));
                            }
                        }
                    } else if legal_moves.moves.contains_key(&vertex) {
                        button_ = button_.on_press(Message::PlayMoveFrom(vertex));
                    }
                }

                column = column.push(button_);
            }

            column = column.push(
                text(letters[x])
                    .size(letter_size)
                    .align_x(Horizontal::Center),
            );
            game_display = game_display.push(column);
        }

        let mut column = column![text(" ").size(letter_size)].spacing(spacing);
        for i in 0..11 {
            let i = 11 - i;
            column = column.push(
                text(format!("{i:2}"))
                    .size(letter_size)
                    .align_y(Vertical::Center),
            );
        }

        game_display = game_display.push(column);
        game_display
    }

    fn subscriptions(&self) -> Subscription<Message> {
        #[cfg(feature = "timer")]
        let subscription_1 = if let Some(game) = &self.game {
            if game.time.is_some() {
                iced::time::every(iced::time::Duration::from_millis(100))
                    .map(|_instant| Message::Tick)
            } else {
                Subscription::none()
            }
        } else {
            Subscription::none()
        };

        #[cfg(not(feature = "timer"))]
        let subscription_1 = Subscription::none();

        let subscription_2 = Subscription::run(pass_messages);

        let subscription_3 = event::listen_with(|event, _status, _id| match event {
            Event::Window(iced::window::Event::Resized(size)) => {
                Some(Message::WindowResized((size.width, size.height)))
            }
            _ => None,
        });

        Subscription::batch(vec![subscription_1, subscription_2, subscription_3])
    }

    fn texting(&self, in_game: bool) -> Container<Message> {
        let text_input = text_input(&format!("{}…", t!("message")), &self.text_input)
            .on_input(Message::TextChanged)
            .on_paste(Message::TextChanged)
            .on_submit(Message::TextSend);

        let mut texting = Column::new();
        if in_game {
            for message in &self.texts_game {
                texting = texting.push(text(message).shaping(text::Shaping::Advanced));
            }
        } else {
            for message in &self.texts {
                texting = texting.push(text(message).shaping(text::Shaping::Advanced));
            }
        }

        container(column![text_input, scrollable(texting)].spacing(SPACING))
            .padding(PADDING)
            .style(container::bordered_box)
    }

    pub fn theme(&self) -> iced::Theme {
        match self.theme {
            Theme::Dark => iced::Theme::SolarizedDark,
            Theme::Light => iced::Theme::SolarizedLight,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) {
        self.error = None;

        match message {
            Message::AccountSettings => self.screen = Screen::AccountSettings,
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.save_client();
            }
            Message::ConnectedTo(address) => self.connected_to = address,
            Message::DeleteAccount => {
                if self.delete_account {
                    self.send("delete_account\n".to_string());
                } else {
                    self.delete_account = true;
                }
            }
            Message::EmailEveryone => {
                self.screen = Screen::EmailEveryone;
                self.send("emails_bcc\n".to_string());
            }
            Message::EmailReset => {
                self.email = None;
                self.send("email_reset\n".to_string());
            }
            Message::GameAccept(id) => {
                self.send(format!("join_game {id}\n"));
                self.game_id = id;
            }
            Message::GameDecline(id) => {
                self.send(format!("decline_game {id}\n"));
            }
            Message::GameJoin(id) => {
                self.game_id = id;
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
                Screen::AccountSettings
                | Screen::EmailEveryone
                | Screen::GameNew
                | Screen::Users => {
                    self.screen = Screen::Games;
                    self.text_input = String::new();
                }
                Screen::Game => {
                    if self.username == self.attacker || self.username == self.defender {
                        self.send(format!(
                            "text_game {} {}.\n",
                            self.game_id,
                            t!("I'm leaving")
                        ));
                    }
                    self.screen = Screen::Games;
                    self.my_turn = false;
                    self.request_draw = false;

                    if self.spectators.contains(&self.username) {
                        self.send(format!("leave_game {}\n", self.game_id));
                    }
                    self.spectators = Vec::new();
                }
                Screen::GameNewFrozen => {
                    self.send(format!("leave_game {}\n", self.game_id));
                    self.screen = Screen::Games;
                }
                Screen::Games => {
                    self.send("logout\n".to_string());
                }
                Screen::Login => self.send("quit\n".to_string()),
            },
            Message::LocaleSelected(locale) => {
                rust_i18n::set_locale(&locale.txt());

                let string_keys: Vec<_> = self.strings.keys().cloned().collect();
                for string in string_keys {
                    self.strings.insert(string.clone(), t!(string).to_string());
                }

                self.locale_selected = locale;
                self.save_client();
            }
            Message::OpenUrl(string) => open_url(&string),
            Message::GameNew => self.screen = Screen::GameNew,
            Message::GameResume(id) => {
                self.game_id = id;
                self.send(format!("resume_game {id}\n"));
                self.send(format!("text_game {id} {}.\n", t!("I rejoined")));
            }
            Message::GameSubmit => {
                if let Some(role) = self.role_selected {
                    if self.timed.0.is_some() {
                        match (
                            self.time_minutes.parse::<i64>(),
                            self.time_add_seconds.parse::<i64>(),
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
            Message::PasswordChanged(password) => {
                let (password, ends_with_whitespace) = split_whitespace(&password);
                self.password_no_save = ends_with_whitespace;
                self.password = password;
            }
            Message::PasswordShow(show_password) => {
                self.password_show = show_password;
            }
            Message::PlayDraw => {
                let game = get_game(&mut self.game);
                let tx = get_tx(&mut self.tx);

                handle_error(tx.send(format!("request_draw {} {}\n", self.game_id, game.turn,)));
            }
            Message::PlayDrawDecision(draw) => {
                let tx = get_tx(&mut self.tx);
                handle_error(tx.send(format!("draw {} {draw}\n", self.game_id)));
            }
            Message::PlayMoveFrom(vertex) => self.play_from = Some(vertex),
            Message::PlayMoveTo(to) => {
                let Some(from) = self.play_from.clone() else {
                    panic!("you have to have a from to get to to");
                };

                let mut turn = Color::Colorless;
                if let Some(game) = &self.game {
                    turn = game.turn.clone();
                }

                self.handle_play(None, &from.to_string(), &to.to_string());
                let game = get_game(&mut self.game);
                let tx = get_tx(&mut self.tx);

                handle_error(tx.send(format!("game {} play {} {from} {to}\n", self.game_id, turn)));

                if game.status == Status::Ongoing {
                    match game.turn {
                        Color::Black => {
                            if let Some(time) = &mut self.time_defender.0 {
                                time.milliseconds_left += time.add_seconds * 1_000;
                            }
                        }
                        Color::Colorless => {}
                        Color::White => {
                            if let Some(time) = &mut self.time_attacker.0 {
                                time.milliseconds_left += time.add_seconds * 1_000;
                            }
                        }
                    }
                }

                self.play_from_previous = self.play_from.clone();
                self.play_to_previous = Some(to);
                self.play_from = None;
                self.my_turn = false;
            }
            Message::PlayMoveRevert => self.play_from = None,
            Message::PlayResign => {
                let game = get_game(&mut self.game);
                let tx = get_tx(&mut self.tx);

                handle_error(tx.send(format!(
                    "game {} play {} resigns _\n",
                    self.game_id, game.turn
                )));
            }
            Message::SoundMuted(muted) => self.sound_muted = muted,
            Message::TcpConnected(tx) => self.tx = Some(tx),
            Message::RatedSelected(rated) => {
                self.rated = if rated { Rated::Yes } else { Rated::No };
            }
            Message::ResetPassword(account) => {
                let tx = get_tx(&mut self.tx);
                handle_error(tx.send(format!("{VERSION_ID} reset_password {account}\n")));
            }
            Message::RoleSelected(role) => {
                self.role_selected = Some(role);
            }
            Message::TextChanged(string) => {
                if self.screen == Screen::Login {
                    let string: Vec<_> = string.split_whitespace().collect();
                    if let Some(string) = string.first() {
                        self.text_input = string.to_ascii_lowercase();
                    } else {
                        self.text_input = String::new();
                    }
                } else {
                    self.text_input = string;
                }
            }
            Message::TextEdit(action) => {
                self.content.perform(action);
            }
            Message::TextReceived(string) => {
                let mut text = string.split_ascii_whitespace();
                match text.next() {
                    Some("=") => {
                        let text_next = text.next();
                        match text_next {
                            Some("display_games") => {
                                self.games_light.0.clear();
                                let games: Vec<&str> = text.collect();
                                for chunks in games.chunks_exact(11) {
                                    let game = ServerGameLight::try_from(chunks)
                                        .expect("the value should be a valid ServerGameLight");

                                    self.games_light.0.insert(game.id, game);
                                }

                                if let Some(game) = self.games_light.0.get(&self.game_id) {
                                    self.spectators =
                                        game.spectators.keys().map(ToString::to_string).collect();
                                    self.spectators.sort();
                                }
                            }
                            Some("display_users") => {
                                let users: Vec<&str> = text.collect();
                                self.users.clear();
                                for user_wins_losses_rating in users.chunks_exact(6) {
                                    let rating = user_wins_losses_rating[4];
                                    let Some((mut rating, mut deviation)) = rating.split_once("±")
                                    else {
                                        panic!("the ratings has this form: {rating}");
                                    };

                                    rating = rating.trim();
                                    deviation = deviation.trim();

                                    let (Ok(rating), Ok(deviation)) =
                                        (rating.parse::<f64>(), deviation.parse::<f64>())
                                    else {
                                        panic!(
                                            "the ratings has this form: ({rating}, {deviation})"
                                        );
                                    };

                                    let logged_in = "logged_in" == user_wins_losses_rating[5];

                                    self.users.insert(
                                        user_wins_losses_rating[0].to_string(),
                                        User {
                                            name: user_wins_losses_rating[0].to_string(),
                                            wins: user_wins_losses_rating[1].to_string(),
                                            losses: user_wins_losses_rating[2].to_string(),
                                            draws: user_wins_losses_rating[3].to_string(),
                                            rating: Rating {
                                                rating,
                                                rd: deviation / CONFIDENCE_INTERVAL_95,
                                            },
                                            logged_in,
                                        },
                                    );
                                }
                            }
                            Some("draw") => {
                                self.request_draw = false;
                                if let Some("accept") = text.next() {
                                    self.my_turn = false;
                                    self.status = Status::Draw;

                                    if let Some(game) = &mut self.game {
                                        game.turn = Color::Colorless;
                                    }
                                }
                            }
                            Some("email") => {
                                if let (Some(address), Some(verified)) = (text.next(), text.next())
                                {
                                    self.email = Some(Email {
                                        username: String::new(),
                                        address: address.to_string(),
                                        code: None,
                                        verified: verified.parse().unwrap(),
                                    });
                                }
                            }
                            Some("emails_bcc") => {
                                self.emails_bcc = text.map(ToString::to_string).collect();
                            }
                            Some("email_code") => {
                                if let Some(email) = &mut self.email {
                                    email.verified = true;
                                }
                                self.error_email = None;
                            }
                            Some("game_over") => {
                                self.my_turn = false;
                                if let Some(game) = &mut self.game {
                                    game.turn = Color::Colorless;
                                }

                                text.next();
                                match text.next() {
                                    Some("attacker_wins") => self.status = Status::BlackWins,
                                    Some("defender_wins") => self.status = Status::WhiteWins,
                                    _ => {}
                                }

                                #[cfg(feature = "sound")]
                                if !self.sound_muted {
                                    thread::spawn(move || {
                                        let (_stream, stream_handle) =
                                            rodio::OutputStream::try_default()?;

                                        let game_over = include_bytes!("game_over.ogg");
                                        let cursor = Cursor::new(game_over);
                                        let sound = stream_handle.play_once(cursor)?;
                                        sound.set_volume(1.0);
                                        thread::sleep(Duration::from_secs(1));

                                        Ok::<(), anyhow::Error>(())
                                    });
                                }
                            }
                            // = join_game david abby rated fischer 900_000 10
                            Some("join_game" | "resume_game" | "watch_game") => {
                                self.screen = Screen::Game;
                                self.status = Status::Ongoing;
                                self.captures = HashSet::new();
                                self.play_from_previous = None;
                                self.play_to_previous = None;
                                self.texts_game = VecDeque::new();

                                let Some(attacker) = text.next() else {
                                    panic!("the attacker should be supplied");
                                };
                                let Some(defender) = text.next() else {
                                    panic!("the defender should be supplied");
                                };
                                self.attacker = attacker.to_string();
                                self.defender = defender.to_string();

                                let Some(rated) = text.next() else {
                                    panic!("there should be rated or unrated supplied");
                                };
                                let Ok(rated) = Rated::from_str(rated) else {
                                    panic!("rated should be valid");
                                };
                                self.rated = rated;

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
                                    time: Some(Local::now().to_utc().timestamp_millis()),
                                    ..Game::default()
                                };

                                self.time_attacker = timed.clone();
                                self.time_defender = timed;

                                if let Some(game_serialized) = text.next() {
                                    let game_deserialized = ron::from_str(game_serialized)
                                        .expect("we should be able to deserialize the game");

                                    game = game_deserialized;

                                    self.time_attacker = game.black_time.clone();
                                    self.time_defender = game.white_time.clone();

                                    match game.turn {
                                        Color::Black => {
                                            if let (Some(time), Some(time_ago)) =
                                                (&mut self.time_attacker.0, game.time)
                                            {
                                                let now = Local::now().to_utc().timestamp_millis();
                                                time.milliseconds_left -= now - time_ago;
                                                if time.milliseconds_left < 0 {
                                                    time.milliseconds_left = 0;
                                                }
                                            }
                                        }
                                        Color::Colorless => {}
                                        Color::White => {
                                            if let (Some(time), Some(time_ago)) =
                                                (&mut self.time_defender.0, game.time)
                                            {
                                                let now = Local::now().to_utc().timestamp_millis();
                                                time.milliseconds_left -= now - time_ago;
                                                if time.milliseconds_left < 0 {
                                                    time.milliseconds_left = 0;
                                                }
                                            }
                                        }
                                    }
                                }

                                let texts: Vec<&str> = text.collect();
                                let texts = texts.join(" ");
                                if !texts.is_empty() {
                                    let texts = ron::from_str(&texts)
                                        .expect("we should be able to deserialize the text");
                                    println!("{texts:?}");
                                    self.texts_game = texts;
                                }

                                if (self.username == attacker && game.turn == Color::Black)
                                    || (self.username == defender && game.turn == Color::White)
                                {
                                    self.my_turn = true;
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
                            Some("text") => self.texts.push_front(text_collect(text)),
                            Some("text_game") => self.texts_game.push_front(text_collect(text)),
                            _ => {}
                        }
                    }
                    Some("?") => {
                        let text_next = text.next();
                        match text_next {
                            Some("create_account" | "login") => {
                                let text: Vec<_> = text.collect();
                                let text = text.join(" ");
                                self.error = Some(text);
                            }
                            Some("email") => {
                                let text: Vec<_> = text.collect();
                                let text = text.join(" ");
                                self.error_email = Some(text);
                            }
                            Some("email_code") => {
                                self.error_email = Some("invalid email code".to_string());
                            }
                            _ => {}
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
                            self.request_draw = false;
                            self.my_turn = true;
                        // game 0 play black a3 a4
                        } else if text_word == Some("play") {
                            let Some(color) = text.next() else {
                                return;
                            };
                            let Ok(color) = Color::from_str(color) else {
                                return;
                            };
                            let Some(from) = text.next() else {
                                return;
                            };
                            let Some(to) = text.next() else {
                                return;
                            };

                            if let (Ok(from), Ok(to)) =
                                (Vertex::from_str(from), Vertex::from_str(to))
                            {
                                self.play_from_previous = Some(from);
                                self.play_to_previous = Some(to);
                            }

                            self.handle_play(Some(&color.to_string()), from, to);
                            let game = get_game(&mut self.game);

                            if game.status == Status::Ongoing {
                                match game.turn {
                                    Color::Black => {
                                        if let Some(time) = &mut self.time_defender.0 {
                                            time.milliseconds_left += time.add_seconds * 1_000;
                                        }
                                    }
                                    Color::Colorless => {}
                                    Color::White => {
                                        if let Some(time) = &mut self.time_attacker.0 {
                                            time.milliseconds_left += time.add_seconds * 1_000;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some("request_draw") => {
                        let Some(id) = text.next() else {
                            return;
                        };
                        let Ok(id) = id.parse::<usize>() else {
                            panic!("the game_id should be a valid u64");
                        };

                        if id == self.game_id {
                            self.request_draw = true;
                        }
                    }
                    _ => {}
                }
            }
            Message::TextSend => {
                match self.screen {
                    Screen::AccountSettings => {
                        self.send(format!("change_password {}\n", self.password));
                        self.password.clear();
                    }
                    Screen::EmailEveryone => {
                        // subject == self.text_input
                        let email = self.content.text().replace('\n', "\\n");
                        self.send(format!("email_everyone {} {email}\n", self.text_input));
                    }
                    Screen::Game => {
                        if !self.text_input.trim().is_empty() {
                            self.text_input.push('\n');
                            self.send(format!("text_game {} {}", self.game_id, self.text_input));
                        }
                    }
                    Screen::Games => {
                        if !self.text_input.trim().is_empty() {
                            self.text_input.push('\n');
                            self.send(format!("text {}", self.text_input));
                        }
                    }
                    Screen::GameNew | Screen::GameNewFrozen | Screen::Login | Screen::Users => {}
                }

                self.text_input.clear();
            }
            Message::TextSendEmail => {
                self.error_email = None;

                let tx = get_tx(&mut self.tx);
                handle_error(tx.send(format!("email {}\n", self.text_input)));
                self.text_input.clear();
            }
            Message::TextSendEmailCode => {
                self.error_email = None;

                let tx = get_tx(&mut self.tx);
                handle_error(tx.send(format!("email_code {}\n", self.text_input)));
            }
            Message::TextSendCreateAccount => {
                if !self.text_input.trim().is_empty() {
                    let username = self.text_input.to_string();
                    self.send(format!(
                        "{VERSION_ID} create_account {username} {}\n",
                        self.password,
                    ));
                    self.username = username;
                    self.password.clear();
                }
                self.text_input.clear();
                self.save_client();
            }
            Message::TextSendLogin => {
                if self.text_input.trim().is_empty() {
                    let username = format!("user-{:x}", rand::random::<u16>());

                    self.send(format!(
                        "{VERSION_ID} create_account {username} {}\n",
                        self.password
                    ));
                    self.username = username;
                } else {
                    let username = self.text_input.to_string();

                    self.send(format!("{VERSION_ID} login {username} {}\n", self.password));
                    self.username = username;
                }

                self.password.clear();
                self.text_input.clear();
                self.save_client();
            }
            Message::Tick => {
                if let Some(game) = &mut self.game {
                    match game.turn {
                        Color::Black => {
                            if let Some(time) = &mut self.time_attacker.0 {
                                time.milliseconds_left -= 100;
                                if time.milliseconds_left < 0 {
                                    time.milliseconds_left = 0;
                                }
                            }
                        }
                        Color::Colorless => {}
                        Color::White => {
                            if let Some(time) = &mut self.time_defender.0 {
                                time.milliseconds_left -= 100;
                                if time.milliseconds_left < 0 {
                                    time.milliseconds_left = 0;
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
            Message::WindowResized((width, height)) => {
                if width >= 1_400.0 && height >= 1_000.0 {
                    self.screen_size = Size::Large;
                } else if width >= 1_200.0 && height >= 850.0 {
                    self.screen_size = Size::Medium;
                } else if width >= 1_000.0 && height >= 750.0 {
                    self.screen_size = Size::Small;
                } else {
                    self.screen_size = Size::Tiny;
                }
            }
        }
    }

    #[must_use]
    fn users_sorted(&self) -> Vec<User> {
        let mut users: Vec<_> = self.users.values().cloned().collect();

        users.sort_by(|a, b| b.name.cmp(&a.name));
        users.sort_by(|a, b| b.rating.rating.partial_cmp(&a.rating.rating).unwrap());

        users
    }

    #[must_use]
    fn games(&self) -> Scrollable<Message> {
        let mut game_ids = Column::new().spacing(SPACING_B);
        let mut attackers = Column::new().spacing(SPACING_B);
        let mut defenders = Column::new().spacing(SPACING_B);
        let mut ratings = Column::new().spacing(SPACING_B);
        let mut timings = Column::new().spacing(SPACING_B);
        let mut buttons = Column::new().spacing(SPACING);

        let mut server_games: Vec<&ServerGameLight> = self.games_light.0.values().collect();
        server_games.sort_by(|a, b| b.id.cmp(&a.id));

        for game in server_games {
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

            let mut buttons_row = Row::new().spacing(SPACING);

            if game.challenge_accepted
                && !(Some(&self.username) == game.attacker.as_ref()
                    || Some(&self.username) == game.defender.as_ref())
            {
                buttons_row = buttons_row
                    .push(button(self.strings["Watch"].as_str()).on_press(Message::GameWatch(id)));
            } else if game.attacker.is_none() || game.defender.is_none() {
                buttons_row = buttons_row
                    .push(button(self.strings["Join"].as_str()).on_press(Message::GameJoin(id)));
            }

            if game.attacker.as_ref() == Some(&self.username)
                || game.defender.as_ref() == Some(&self.username)
            {
                buttons_row = buttons_row.push(
                    button(self.strings["Resume"].as_str()).on_press(Message::GameResume(id)),
                );
            }

            buttons = buttons.push(buttons_row);
        }

        let game_id = t!("game id");
        let game_ids = column![
            text(game_id.to_string()),
            text("-".repeat(game_id.chars().count())),
            game_ids
        ]
        .padding(PADDING);
        let attacker = t!("attacker");
        let attackers = column![
            text(attacker.to_string()),
            text("-".repeat(attacker.chars().count())),
            attackers
        ]
        .padding(PADDING);
        let defender = t!("defender");
        let defenders = column![
            text(defender.to_string()),
            text("-".repeat(defender.chars().count())),
            defenders
        ]
        .padding(PADDING);
        let rated = t!("rated");
        let ratings = column![
            text(rated.to_string()),
            text("-".repeat(rated.chars().count())),
            ratings
        ]
        .padding(PADDING);
        let timed = t!("timed");
        let timings = column![
            text(timed.to_string()),
            text("-".repeat(timed.chars().count())),
            timings
        ]
        .padding(PADDING);
        let buttons = column![text(""), text(""), buttons].padding(PADDING);

        scrollable(row![
            game_ids, attackers, defenders, ratings, timings, buttons
        ])
    }

    fn handle_play(&mut self, color: Option<&str>, from: &str, to: &str) {
        self.captures = HashSet::new();

        let game = get_game(&mut self.game);

        match color {
            Some(color) => match game.read_line(&format!("play {color} {from} {to}\n")) {
                Ok(vertexes) => {
                    if let Some(vertexes) = vertexes {
                        for vertex in vertexes.split_ascii_whitespace() {
                            let Ok(vertex) = Vertex::from_str(vertex) else {
                                panic!("this should be a valid vertex");
                            };
                            self.captures.insert(vertex);
                        }
                    }
                }
                Err(error) => {
                    error!("{error}");
                    exit(1)
                }
            },
            None => match game.read_line(&format!("play {} {from} {to}\n", game.turn)) {
                Ok(vertexes) => {
                    if let Some(vertexes) = vertexes {
                        for vertex in vertexes.split_ascii_whitespace() {
                            let Ok(vertex) = Vertex::from_str(vertex) else {
                                panic!("this should be a valid vertex");
                            };
                            self.captures.insert(vertex);
                        }
                    }
                }
                Err(error) => {
                    error!("{error}");
                    exit(1)
                }
            },
        }

        #[cfg(feature = "sound")]
        if self.sound_muted {
            return;
        }

        #[cfg(feature = "sound")]
        let capture = !self.captures.is_empty();

        #[cfg(feature = "sound")]
        thread::spawn(move || {
            let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
            let cursor = if capture {
                let capture_ogg = include_bytes!("capture.ogg").to_vec();
                Cursor::new(capture_ogg)
            } else {
                let move_ogg = include_bytes!("move.ogg").to_vec();
                Cursor::new(move_ogg)
            };
            let sound = stream_handle.play_once(cursor)?;
            sound.set_volume(1.0);
            thread::sleep(Duration::from_secs(1));

            Ok::<(), anyhow::Error>(())
        });
    }

    #[must_use]
    fn users(&self, logged_in: bool) -> Scrollable<Message> {
        let mut ratings = Column::new();
        let mut usernames = Column::new();
        let mut wins = Column::new();
        let mut losses = Column::new();
        let mut draws = Column::new();

        for user in self.users_sorted() {
            if logged_in == user.logged_in {
                ratings = ratings.push(text(user.rating.to_string_rounded()));
                usernames = usernames.push(text(user.name));
                wins = wins.push(text(user.wins));
                losses = losses.push(text(user.losses));
                draws = draws.push(text(user.draws));
            }
        }

        let rating = t!("rating");
        let ratings = column![
            text(rating.to_string()),
            text("-".repeat(rating.chars().count())),
            ratings
        ]
        .padding(PADDING);
        let username = t!("username");
        let usernames = column![
            text(username.to_string()).shaping(text::Shaping::Advanced),
            text("-".repeat(username.chars().count())),
            usernames
        ]
        .padding(PADDING);
        let win = t!("wins");
        let wins = column![
            text(win.to_string()),
            text("-".repeat(win.chars().count())),
            wins
        ]
        .padding(PADDING);
        let loss = t!("losses");
        let losses = column![
            text(loss.to_string()),
            text("-".repeat(loss.chars().count())),
            losses
        ]
        .padding(PADDING);
        let draw = t!("draws");
        let draws = column![
            text(draw.to_string()),
            text("-".repeat(draw.chars().count())),
            draws
        ]
        .padding(PADDING);

        scrollable(row![ratings, usernames, wins, losses, draws])
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
            Screen::AccountSettings => {
                let mut rating = String::new();
                let mut wins = String::new();
                let mut draws = String::new();
                let mut losses = String::new();

                for user in self.users.values() {
                    if self.username == user.name {
                        rating = user.rating.to_string_rounded();
                        wins.clone_from(&user.wins);
                        losses.clone_from(&user.losses);
                        draws.clone_from(&user.draws);
                    }
                }

                let mut columns = column![
                    text(format!(
                        "{} {} {} TCP",
                        t!("connected to"),
                        &self.connected_to,
                        t!("via")
                    )),
                    text(format!("{}: {}", t!("username"), &self.username))
                        .shaping(text::Shaping::Advanced),
                    text(format!("{}: {rating}", t!("rating"))),
                    text(format!("{}: {wins}", t!("wins"))),
                    text(format!("{}: {losses}", t!("losses"))),
                    text(format!("{}: {draws}", t!("draws"))),
                ]
                .padding(PADDING)
                .spacing(SPACING);

                if let Some(email) = &self.email {
                    let mut row = Row::new();
                    if email.verified {
                        row = row.push(text(format!(
                            "{}: [{}] {} ",
                            t!("email address"),
                            t!("verified"),
                            email.address,
                        )));
                        columns = columns.push(row);
                    } else {
                        row = row.push(text(format!(
                            "{}: [{}] {} ",
                            t!("email address"),
                            t!("unverified"),
                            email.address,
                        )));
                        columns = columns.push(row);

                        let mut row = Row::new();
                        row = row.push(text("email code: ".to_string()));
                        row = row.push(
                            text_input("", &self.text_input)
                                .on_input(Message::TextChanged)
                                .on_paste(Message::TextChanged)
                                .on_submit(Message::TextSendEmailCode),
                        );
                        columns = columns.push(row);
                    }
                } else {
                    let mut row = Row::new();
                    row = row.push(text("email address: ".to_string()));
                    row = row.push(
                        text_input("", &self.text_input)
                            .on_input(Message::TextChanged)
                            .on_paste(Message::TextChanged)
                            .on_submit(Message::TextSendEmail),
                    );

                    columns = columns.push(row);
                    columns = columns.push(row![text("email code:")]);
                }

                columns = columns.push(row![
                    button(self.strings["Reset Email"].as_str()).on_press(Message::EmailReset)
                ]);

                if let Some(error) = &self.error_email {
                    columns = columns.push(row![text(format!("error: {error}"))]);
                }

                let mut change_password_button = button(self.strings["Change Password"].as_str());

                if !self.password_no_save {
                    change_password_button = change_password_button.on_press(Message::TextSend);
                }

                columns = columns.push(
                    row![
                        change_password_button,
                        text_input("", &self.password)
                            .secure(!self.password_show)
                            .on_input(Message::PasswordChanged)
                            .on_paste(Message::PasswordChanged),
                    ]
                    .spacing(SPACING),
                );

                columns = columns.push(
                    checkbox(t!("show password"), self.password_show)
                        .on_toggle(Message::PasswordShow),
                );

                if self.delete_account {
                    columns = columns.push(
                        button(self.strings["REALLY DELETE ACCOUNT"].as_str())
                            .on_press(Message::DeleteAccount),
                    );
                } else {
                    columns = columns.push(
                        button(self.strings["Delete Account"].as_str())
                            .on_press(Message::DeleteAccount),
                    );
                }

                columns =
                    columns.push(button(self.strings["Leave"].as_str()).on_press(Message::Leave));

                columns.into()
            }
            Screen::EmailEveryone => {
                let subject = row![
                    text("Subject: "),
                    text_input("", &self.text_input)
                        .on_input(Message::TextChanged)
                        .on_paste(Message::TextChanged)
                        .on_submit(Message::TextSend),
                ];

                let editor = text_editor(&self.content)
                    .placeholder("Dear User, …")
                    .on_action(Message::TextEdit);

                let send_emails = button("Send Emails").on_press(Message::TextSend);
                let leave = button("Leave").on_press(Message::Leave);
                let mut column = column![
                    subject,
                    text("From: Hnefatafl Org <no-reply@hnefatafl.org>"),
                    text("Content-Type: text/plain; charset=utf-8"),
                    text("Content-Transfer-Encoding: 7bit"),
                    text(format!("Date: {}", Utc::now().to_rfc2822())),
                    text("Body:"),
                    editor,
                    send_emails,
                    leave,
                    text("Bcc:")
                ]
                .spacing(SPACING)
                .padding(PADDING);

                for email in &self.emails_bcc {
                    column = column.push(text(email));
                }

                scrollable(column).into()
            }
            Screen::Game => {
                let Some(game) = &self.game else {
                    panic!("we are in a game");
                };

                let mut attacker_rating = "-".to_string();
                let mut defender_rating = "-".to_string();
                for user in self.users.values() {
                    if self.attacker == user.name {
                        attacker_rating = user.rating.to_string_rounded();
                    }
                    if self.defender == user.name {
                        defender_rating = user.rating.to_string_rounded();
                    }
                }

                let captured = game.board.captured();
                let user_area_ = column![
                    row![
                        text(self.time_attacker.fmt_shorthand()).size(40).center(),
                        text("⚔").shaping(text::Shaping::Advanced).size(40).center(),
                        column![
                            text(self.attacker.to_string()),
                            text(captured.white().to_string()).shaping(text::Shaping::Advanced),
                            text(attacker_rating.to_string()).center(),
                        ]
                    ]
                    .spacing(SPACING),
                    row![
                        text(self.time_defender.fmt_shorthand()).size(40).center(),
                        text("🛡").shaping(text::Shaping::Advanced).size(40).center(),
                        column![
                            text(self.defender.to_string()),
                            text(captured.black().to_string()).shaping(text::Shaping::Advanced),
                            text(defender_rating.to_string()).center(),
                        ]
                    ]
                    .spacing(SPACING),
                ];

                let user_area_ = container(user_area_)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let mut watching = false;
                let texting = self.texting(true);

                let mut user_area = column![text(format!("#{} {}", self.game_id, &self.username,))]
                    .spacing(SPACING);

                let is_rated = match self.rated {
                    Rated::No => t!("no"),
                    Rated::Yes => t!("yes"),
                };

                user_area = user_area.push(text(format!(
                    "{}: {} {}: {is_rated}",
                    t!("move"),
                    game.previous_boards.0.len(),
                    t!("rated"),
                )));

                user_area = user_area.push(user_area_);

                let mut spectators = Column::new();
                for spectator in &self.spectators {
                    if self.username.as_str() == spectator.as_str() {
                        watching = true;
                    }

                    let mut spectator = spectator.to_string();
                    if let Some(user) = self.users.get(&spectator) {
                        let _ok = write!(spectator, " ({})", user.rating);
                    }
                    spectators = spectators.push(text(spectator.to_string()));
                }

                if !watching {
                    if self.my_turn {
                        user_area = user_area.push(
                            column![
                                row![
                                    button(self.strings["Resign"].as_str())
                                        .on_press(Message::PlayResign),
                                ]
                                .spacing(SPACING),
                                row![
                                    button(self.strings["Request Draw"].as_str())
                                        .on_press(Message::PlayDraw),
                                ]
                                .spacing(SPACING),
                            ]
                            .spacing(SPACING),
                        );
                    } else {
                        let row = if self.request_draw {
                            column![
                                row![
                                    button(self.strings["Accept Draw"].as_str())
                                        .on_press(Message::PlayDrawDecision(Draw::Accept)),
                                ]
                                .spacing(SPACING)
                            ]
                        } else {
                            Column::new()
                        };
                        user_area = user_area.push(row.spacing(SPACING));
                    }
                }

                let muted = checkbox(t!("Muted"), self.sound_muted)
                    .on_toggle(Message::SoundMuted)
                    .size(32);

                let leave = button(self.strings["Leave"].as_str()).on_press(Message::Leave);

                user_area = user_area.push(row![muted, leave].spacing(SPACING));

                match self.status {
                    Status::BlackWins => user_area = user_area.push(text("Attacker Wins!")),
                    Status::Draw => user_area = user_area.push(text("It's a draw.")),
                    Status::Ongoing => {}
                    Status::WhiteWins => user_area = user_area.push(text("Defender Wins!")),
                }

                let spectator = column![
                    text(format!("👥 ({})", self.spectators.len()))
                        .shaping(text::Shaping::Advanced)
                ];

                if self.spectators.is_empty() {
                    user_area = user_area.push(spectator);
                } else {
                    user_area = user_area.push(tooltip(
                        spectator,
                        container(spectators)
                            .style(container::bordered_box)
                            .padding(PADDING),
                        tooltip::Position::Bottom,
                    ));
                }

                user_area = user_area.push(texting);
                let user_area = container(user_area)
                    .padding(PADDING)
                    .style(container::bordered_box);

                row![self.board(), user_area].spacing(SPACING).into()
            }
            Screen::GameNew => {
                let attacker = radio(
                    t!("attacker"),
                    Role::Attacker,
                    self.role_selected,
                    Message::RoleSelected,
                );

                let defender = radio(
                    t!("defender"),
                    Role::Defender,
                    self.role_selected,
                    Message::RoleSelected,
                );

                let rated =
                    checkbox(t!("rated"), self.rated.into()).on_toggle(Message::RatedSelected);

                let mut new_game = button(self.strings["New Game"].as_str());
                if self.role_selected.is_some() {
                    new_game = new_game.on_press(Message::GameSubmit);
                }

                let leave = button(self.strings["Leave"].as_str()).on_press(Message::Leave);

                let mut time = row![
                    checkbox(t!("timed"), self.timed.clone().into())
                        .on_toggle(Message::TimeCheckbox)
                ];

                if self.timed.0.is_some() {
                    time = time.push(text(t!("minutes")));
                    time = time.push(
                        text_input("15", &self.time_minutes)
                            .on_input(Message::TimeMinutes)
                            .on_paste(Message::TimeMinutes),
                    );
                    time = time.push(text(t!("add seconds")));
                    time = time.push(
                        text_input("10", &self.time_add_seconds)
                            .on_input(Message::TimeAddSeconds)
                            .on_paste(Message::TimeAddSeconds),
                    );
                }
                time = time.spacing(SPACING);

                let row_1 = row![
                    text(format!("{}:", t!("role"))),
                    attacker,
                    defender,
                    rated,
                    time,
                ]
                .padding(PADDING)
                .spacing(SPACING);

                let row_2 = row![new_game, leave].padding(PADDING).spacing(SPACING);
                column![row_1, row_2].into()
            }
            Screen::GameNewFrozen => {
                let Some(role) = self.role_selected else {
                    panic!("You can't get to GameNewFrozen unless you have selected a role!");
                };

                let mut buttons_live = false;
                let mut game_display =
                    column![text(format!("{}: {}", t!("role"), t!(role.to_string())))]
                        .padding(PADDING);
                if let Some(game) = self.games_light.0.get(&self.game_id) {
                    game_display = game_display.push(text(game.to_string()));

                    if game.attacker.is_some() && game.defender.is_some() {
                        buttons_live = true;
                    }
                }

                let mut buttons = if self.challenger {
                    row![button(self.strings["Leave"].as_str()).on_press(Message::Leave)]
                } else if buttons_live {
                    row![
                        button(self.strings["Accept"].as_str())
                            .on_press(Message::GameAccept(self.game_id)),
                        button(self.strings["Decline"].as_str())
                            .on_press(Message::GameDecline(self.game_id)),
                        button(self.strings["Leave"].as_str()).on_press(Message::Leave),
                    ]
                } else {
                    row![
                        button(self.strings["Accept"].as_str()),
                        button(self.strings["Decline"].as_str()),
                        button(self.strings["Leave"].as_str()).on_press(Message::Leave),
                    ]
                };
                buttons = buttons.padding(PADDING).spacing(SPACING);

                game_display.push(buttons).into()
            }
            Screen::Games => {
                let mut theme = if self.theme == Theme::Light {
                    row![
                        button(
                            text(self.strings["Dark"].as_str()).shaping(text::Shaping::Advanced)
                        )
                        .on_press(Message::ChangeTheme(Theme::Dark)),
                        button(
                            text(self.strings["Light"].as_str()).shaping(text::Shaping::Advanced)
                        ),
                    ]
                } else {
                    row![
                        button(
                            text(self.strings["Dark"].as_str()).shaping(text::Shaping::Advanced)
                        ),
                        button(
                            text(self.strings["Light"].as_str()).shaping(text::Shaping::Advanced)
                        )
                        .on_press(Message::ChangeTheme(Theme::Light)),
                    ]
                };

                if self.email_everyone {
                    let email_everyone = button("Email Everyone").on_press(Message::EmailEveryone);
                    theme = theme.push(email_everyone);
                }

                let theme = theme.padding(PADDING).spacing(SPACING);

                let username = row![
                    text(format!("{}: {}", t!("username"), &self.username))
                        .shaping(text::Shaping::Advanced)
                ];
                let username = container(username)
                    .padding(PADDING / 2)
                    .style(container::bordered_box);

                let create_game = button(
                    text(self.strings["Create Game"].as_str()).shaping(text::Shaping::Advanced),
                )
                .on_press(Message::GameNew);
                let users =
                    button(text(self.strings["Users"].as_str()).shaping(text::Shaping::Advanced))
                        .on_press(Message::Users);
                let account_setting = button(
                    text(self.strings["Account Settings"].as_str())
                        .shaping(text::Shaping::Advanced),
                )
                .on_press(Message::AccountSettings);
                let website =
                    button(text(self.strings["Rules"].as_str()).shaping(text::Shaping::Advanced))
                        .on_press(Message::OpenUrl(
                            "https://hnefatafl.org/rules.html".to_string(),
                        ));

                let quit =
                    button(text(self.strings["Quit"].as_str()).shaping(text::Shaping::Advanced))
                        .on_press(Message::Leave);

                let top = row![username, create_game, users, account_setting, website, quit]
                    .padding(PADDING)
                    .spacing(SPACING);

                let user_area = self.user_area(false);

                column![theme, top, user_area].into()
            }
            Screen::Login => {
                let username = row![
                    text(format!("{}:", t!("username")))
                        .shaping(text::Shaping::Advanced)
                        .size(20),
                    text_input("", &self.text_input)
                        .on_input(Message::TextChanged)
                        .on_paste(Message::TextChanged),
                ]
                .spacing(SPACING);

                let username = container(username)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let password = row![
                    text(format!("{}:", t!("password")))
                        .shaping(text::Shaping::Advanced)
                        .size(20),
                    text_input("", &self.password)
                        .secure(!self.password_show)
                        .on_input(Message::PasswordChanged)
                        .on_paste(Message::PasswordChanged),
                ]
                .spacing(SPACING);

                let password = container(password)
                    .padding(PADDING)
                    .style(container::bordered_box);

                let show_password = checkbox(t!("show password"), self.password_show)
                    .text_shaping(text::Shaping::Advanced)
                    .on_toggle(Message::PasswordShow);

                let mut login =
                    button(text(self.strings["Login"].as_str()).shaping(text::Shaping::Advanced));
                if !self.password_no_save {
                    login = login.on_press(Message::TextSendLogin);
                }

                let mut create_account = button(
                    text(self.strings["Create Account"].as_str()).shaping(text::Shaping::Advanced),
                );
                if !self.text_input.is_empty() && !self.password_no_save {
                    create_account = create_account.on_press(Message::TextSendCreateAccount);
                }

                let mut reset_password = button(
                    text(self.strings["Reset Password"].as_str()).shaping(text::Shaping::Advanced),
                );
                if !self.text_input.is_empty() {
                    reset_password =
                        reset_password.on_press(Message::ResetPassword(self.text_input.clone()));
                }

                let website = button("https://hnefatafl.org")
                    .on_press(Message::OpenUrl("https://hnefatafl.org".to_string()));

                let quit =
                    button(text(self.strings["Quit"].as_str()).shaping(text::Shaping::Advanced))
                        .on_press(Message::Leave);

                let buttons =
                    row![login, create_account, reset_password, website, quit].spacing(SPACING);

                let mut error = text("");
                if let Some(error_) = &self.error {
                    error = text(error_);
                }

                let mut error_persistent = text("");
                if let Some(error_) = &self.error_persistent {
                    error_persistent = text(error_);
                }

                let locale = [Locale::English, Locale::Chinese, Locale::French];

                let locale = row![
                    text(format!("{}: ", t!("locale")))
                        .shaping(text::Shaping::Advanced)
                        .size(20),
                    pick_list(locale, Some(self.locale_selected), Message::LocaleSelected)
                        .text_shaping(text::Shaping::Advanced),
                ];

                column![
                    username,
                    password,
                    show_password,
                    buttons,
                    locale,
                    error,
                    error_persistent
                ]
                .padding(PADDING)
                .spacing(SPACING)
                .into()
            }
            Screen::Users => scrollable(column![
                text(t!("logged in")),
                self.users(true),
                text(t!("logged out")),
                self.users(false),
                row![button(self.strings["Leave"].as_str()).on_press(Message::Leave)]
                    .padding(PADDING),
            ])
            .spacing(SPACING)
            .into(),
        }
    }

    fn save_client(&self) {
        if let Ok(string) = ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default()) {
            if !string.trim().is_empty() {
                let data_file = data_file();

                if let Ok(mut file) = File::create(&data_file) {
                    if let Err(error) = file.write_all(string.as_bytes()) {
                        error!("{error}");
                    }
                }
            }
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
enum Locale {
    #[default]
    English,
    Chinese,
    French,
}

impl Locale {
    fn txt(self) -> String {
        match self {
            Self::English => "en".to_string(),
            Self::Chinese => "zh-CN".to_string(),
            Self::French => "fr".to_string(),
        }
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::English => write!(f, "English"),
            Self::Chinese => write!(f, "中文"),
            Self::French => write!(f, "Français"),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    AccountSettings,
    ChangeTheme(Theme),
    ConnectedTo(String),
    DeleteAccount,
    EmailEveryone,
    EmailReset,
    GameAccept(usize),
    GameDecline(usize),
    GameJoin(usize),
    GameNew,
    GameResume(usize),
    GameSubmit,
    GameWatch(usize),
    Leave,
    LocaleSelected(Locale),
    OpenUrl(String),
    PasswordChanged(String),
    PasswordShow(bool),
    PlayDraw,
    PlayDrawDecision(Draw),
    PlayMoveFrom(Vertex),
    PlayMoveTo(Vertex),
    PlayMoveRevert,
    PlayResign,
    SoundMuted(bool),
    RatedSelected(bool),
    ResetPassword(String),
    RoleSelected(Role),
    TcpConnected(mpsc::Sender<String>),
    TextChanged(String),
    TextEdit(text_editor::Action),
    TextReceived(String),
    TextSend,
    TextSendEmail,
    TextSendEmailCode,
    TextSendCreateAccount,
    TextSendLogin,
    Tick,
    TimeAddSeconds(String),
    TimeCheckbox(bool),
    TimeMinutes(String),
    Users,
    WindowResized((f32, f32)),
}

fn data_file() -> PathBuf {
    let mut data_file = if let Some(data_file) = dirs::data_dir() {
        data_file
    } else {
        PathBuf::new()
    };

    data_file.push("hnefatafl.ron");
    data_file
}

fn get_game(game: &mut Option<Game>) -> &mut Game {
    let Some(game) = game else {
        panic!("you have to be in a game to play a move")
    };
    game
}

fn get_tx(tx: &mut Option<Sender<String>>) -> &mut Sender<String> {
    let Some(tx) = tx.as_mut() else {
        panic!("you have to have a sender at this point")
    };
    tx
}

fn pass_messages() -> impl Stream<Item = Message> {
    stream::channel(
        100,
        move |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
            let mut args = Args::parse();
            args.host.push_str(PORT);
            let address = args.host;

            let mut tcp_stream = handle_error(TcpStream::connect(&address));
            let reader = handle_error(tcp_stream.try_clone());
            let mut reader = BufReader::new(reader);
            let (tx, rx) = mpsc::channel();
            let _ = sender.send(Message::TcpConnected(tx)).await;
            info!("connected to {address} ...");

            thread::spawn(move || {
                loop {
                    let message = handle_error(rx.recv());
                    let message_trim = message.trim();
                    debug!("<- {message_trim}");

                    if message_trim != "quit" {
                        handle_error(tcp_stream.write_all(message.as_bytes()));
                    }

                    if message_trim == "delete_account"
                        || message_trim == "logout"
                        || message_trim == "quit"
                    {
                        #[cfg(not(target_os = "redox"))]
                        tcp_stream
                            .shutdown(Shutdown::Both)
                            .expect("shutdown call failed");

                        exit(0);
                    }
                }
            });

            thread::spawn(move || {
                let mut buffer = String::new();
                handle_error(executor::block_on(
                    sender.send(Message::ConnectedTo(address.to_string())),
                ));

                loop {
                    let bytes = handle_error(reader.read_line(&mut buffer));
                    if bytes > 0 {
                        let buffer_trim = buffer.trim();
                        let buffer_trim_vec: Vec<_> =
                            buffer_trim.split_ascii_whitespace().collect();

                        if buffer_trim_vec[1] == "display_users"
                            || buffer_trim_vec[1] == "display_games"
                        {
                            trace!("-> {buffer_trim}");
                        } else {
                            debug!("-> {buffer_trim}");
                        }

                        handle_error(executor::block_on(
                            sender.send(Message::TextReceived(buffer.clone())),
                        ));
                        buffer.clear();
                    } else {
                        info!("the TCP stream has closed");
                        break;
                    }
                }
            });
        },
    )
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
    #[cfg(feature = "www")]
    if let Err(error) = webbrowser::open(url) {
        error!("{error}");
    }
    #[cfg(not(feature = "www"))]
    info!("You are trying to visit: {url}");
}

fn split_whitespace(string: &str) -> (String, bool) {
    let mut ends_with_whitespace = false;

    if string.ends_with(|ch: char| ch.is_whitespace()) {
        ends_with_whitespace = true;
    }

    let mut string: String = string.split_whitespace().collect();

    if string.is_empty() {
        ends_with_whitespace = false;
    }

    if ends_with_whitespace {
        string.push(' ');
    }

    (string, ends_with_whitespace)
}

fn text_collect(text: SplitAsciiWhitespace<'_>) -> String {
    let text: Vec<&str> = text.collect();
    text.join(" ")
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum Size {
    Tiny,
    #[default]
    Small,
    Medium,
    Large,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Debug)]
struct User {
    name: String,
    wins: String,
    losses: String,
    draws: String,
    rating: Rating,
    logged_in: bool,
}
