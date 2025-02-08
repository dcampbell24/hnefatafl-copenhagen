use std::{
    collections::HashMap,
    env, fmt,
    fs::{exists, read_to_string, File},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::mpsc::{self, Receiver},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use clap::{command, Parser};
use env_logger::Builder;
use hnefatafl_copenhagen::{
    accounts::{Account, Accounts},
    color::Color,
    game::Game,
    glicko::Outcome,
    handle_error,
    rating::Rated,
    role::Role,
    server_game::{ArchivedGame, ServerGame, ServerGameLight},
    status::Status,
    time::TimeSettings,
};
use log::{debug, info, LevelFilter};
use password_hash::SaltString;
use rand::rngs::OsRng;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

const PORT: &str = ":49152";

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens client connections.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host
    #[arg(default_value = "0.0.0.0", long)]
    host: String,
}

// 1. Handles the leave issue... if you periodically check the time.
// 1. Send a message when the game is over.
// 2. Run update_rd on every account once every two months and calculate the
//    average rating and rd, we assume rd is 50.
// 3. watch_game 1
// 3. Display in game users.
// 4. Figure out some way of testing.
// 4. Get SSL working.
fn main() -> anyhow::Result<()> {
    init_logger();
    let mut args = Args::parse();

    let data_file = data_file();
    let mut server = if exists(&data_file)? {
        ron::from_str(&read_to_string(&data_file)?)?
    } else {
        Server::default()
    };

    args.host.push_str(PORT);
    let address = args.host;
    let listener = TcpListener::bind(&address)?;
    info!("listening on {address} ...");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || server.handle_messages(&rx));

    let tx_messages = tx.clone();
    thread::spawn(move || loop {
        handle_error(tx_messages.send(("0 server display_server".to_string(), None)));
        sleep(Duration::from_secs(4));
    });

    for (index, stream) in (1..).zip(listener.incoming()) {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, stream, &tx));
    }

    Ok(())
}

fn data_file() -> PathBuf {
    let mut data_file = if let Some(data_file) = dirs::data_dir() {
        data_file
    } else {
        PathBuf::new()
    };
    data_file.push("hnefatafl-copenhagen.ron");
    data_file
}

fn login(
    index: u64,
    mut stream: TcpStream,
    tx: &mpsc::Sender<(String, Option<mpsc::Sender<String>>)>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buf = String::new();

    reader.read_line(&mut buf)?;
    for ch in buf.trim().chars() {
        if ch.is_control() || ch == '\0' {
            return Err(anyhow::Error::msg(
                "there are control characters in the username or password",
            ));
        }
    }

    if buf.trim().is_empty() {
        return Err(anyhow::Error::msg(
            "the user tried to login with white space alone",
        ));
    }

    buf.make_ascii_lowercase();
    let mut username_password = buf.split_ascii_whitespace();

    if let (Some("login"), Some(username)) = (username_password.next(), username_password.next()) {
        let password: Vec<&str> = username_password.collect();
        let password = password.join(" ");

        if username.len() > 32 {
            return Err(anyhow::Error::msg("username is greater than 32 characters"));
        }
        if password.len() > 32 {
            return Err(anyhow::Error::msg("password is greater than 32 characters"));
        }

        let (client_tx, client_rx) = mpsc::channel();
        tx.send((
            format!("{index} {username} login {password}"),
            Some(client_tx),
        ))?;

        let message = client_rx.recv()?;
        if "= login" != message.as_str() {
            stream.write_all(b"? login\n")?;
            return Err(anyhow::Error::msg("failed to login"));
        }
        stream.write_all(b"= login\n")?;

        thread::spawn(move || receiving_and_writing(stream, &client_rx));

        let mut buf = String::new();
        for _ in 0..1_000_000 {
            reader.read_line(&mut buf)?;

            let buf_str = buf.trim();
            for char in buf_str.chars() {
                if char.is_control() || char == '\0' {
                    return Err(anyhow::Error::msg("a control character was passed"));
                }
            }

            tx.send((format!("{index} {username} {buf_str}"), None))?;
            buf.clear();
        }

        tx.send((format!("{index} {username} logout"), None))?;
    }

    Err(anyhow::Error::msg("the user didn't pass 'login username'"))
}

fn receiving_and_writing(
    mut stream: TcpStream,
    client_rx: &Receiver<String>,
) -> anyhow::Result<()> {
    loop {
        let message = client_rx.recv()?;
        stream.write_all(format!("{message}\n").as_bytes())?;
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
struct Server {
    game_id: u64,
    accounts: Accounts,
    #[serde(skip)]
    clients: HashMap<u64, mpsc::Sender<String>>,
    #[serde(skip)]
    games: ServerGames,
    passwords: HashMap<String, String>,
    #[serde(skip)]
    pending_games: ServerGamesLight,
    archived_games: Vec<ArchivedGame>,
}

#[derive(Clone, Debug, Default)]
struct ServerGames(HashMap<u64, ServerGame>);

impl fmt::Display for ServerGames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut games = Vec::new();
        for game in self.0.values() {
            games.push(game.protocol());
        }
        let games = games.join(" ");

        write!(f, "{games}")
    }
}

#[derive(Clone, Debug, Default)]
struct ServerGamesLight(HashMap<u64, ServerGameLight>);

impl fmt::Display for ServerGamesLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut games = Vec::new();
        for game in self.0.values() {
            games.push(game.protocol());
        }
        let games = games.join(" ");

        write!(f, "{games}")
    }
}

impl Server {
    fn handle_messages(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> anyhow::Result<()> {
        loop {
            if let Some((tx, ok, command)) = self.handle_messages_internal(rx) {
                if ok {
                    tx.send(format!("= {command}"))?;
                } else {
                    tx.send(format!("? {command}"))?;
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn handle_messages_internal(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let (message, option_tx) = rx.recv().ok()?;
        let index_username_command: Vec<_> = message.split_ascii_whitespace().collect();

        if let (Some(index_supplied), Some(username), command_option) = (
            index_username_command.first(),
            index_username_command.get(1),
            index_username_command.get(2),
        ) {
            let command = command_option?;

            let index_supplied = index_supplied
                .parse::<u64>()
                .expect("the index should be a valid u64");

            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
            let the_rest = the_rest.join(" ");
            match *command {
                "display_server" => {
                    debug!("0 {username} display_server");
                    for tx in &mut self.clients.values() {
                        tx.send(format!("= display_pending_games {}", &self.pending_games))
                            .ok()?;
                        tx.send(format!("= display_games {}", &self.games)).ok()?;
                        tx.send(format!("= display_users {}", &self.accounts))
                            .ok()?;
                    }
                    None
                }
                "join_game" => {
                    let Some(id) = the_rest.split_ascii_whitespace().next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(id) = id.parse::<u64>() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };

                    info!("{index_supplied} {username} join_game {id}");
                    let Some(game) = self.pending_games.0.remove(&id) else {
                        panic!("the id must refer to a valid pending game");
                    };

                    let Some(channel) = game.channel else {
                        panic!("a pending game has to have a waiting channel");
                    };

                    let (attacker_tx, defender_tx) = if game.attacker.is_some() {
                        (
                            self.clients[&channel].clone(),
                            self.clients[&index_supplied].clone(),
                        )
                    } else {
                        (
                            self.clients[&index_supplied].clone(),
                            self.clients[&channel].clone(),
                        )
                    };

                    let new_game = if let Some(attacker) = &game.attacker {
                        self.clients[&channel]
                            .send(format!(
                                "= new_game ready {attacker} {username} {}",
                                game.rated
                            ))
                            .ok()?;
                        handle_error(self.clients[&index_supplied].send(format!(
                            "= join_game {attacker} {username} {} {:?}",
                            game.rated, game.timed
                        )));

                        ServerGame {
                            id: game.id,
                            attacker: attacker.clone(),
                            attacker_tx: attacker_tx.clone(),
                            defender: (*username).to_string(),
                            defender_tx,
                            rated: game.rated,
                            game: Game {
                                black_time: game.timed.clone(),
                                white_time: game.timed,
                                timer: Some(Instant::now()),
                                ..Game::default()
                            },
                            text: String::new(),
                        }
                    } else if let Some(defender) = &game.defender {
                        self.clients[&channel]
                            .send(format!(
                                "= new_game ready {username} {defender} {}",
                                game.rated
                            ))
                            .ok()?;
                        self.clients[&index_supplied]
                            .send(format!(
                                "= join_game {username} {defender} {} {:?}",
                                game.rated, game.timed
                            ))
                            .ok()?;

                        ServerGame {
                            id: game.id,
                            attacker: (*username).to_string(),
                            attacker_tx: attacker_tx.clone(),
                            defender: defender.clone(),
                            defender_tx,
                            rated: game.rated,
                            game: Game {
                                black_time: game.timed.clone(),
                                white_time: game.timed,
                                timer: Some(Instant::now()),
                                ..Game::default()
                            },
                            text: String::new(),
                        }
                    } else {
                        panic!("there has to be an attacker or defender")
                    };

                    self.games.0.insert(id, new_game);
                    attacker_tx
                        .send(format!("game {id} generate_move black"))
                        .ok()?;

                    None
                }
                "login" => {
                    if let Some(tx) = option_tx {
                        if let Some(index_database) = self.accounts.0.get_mut(*username) {
                            // The username is in the database and already logged in.
                            if let Some(index_database) = index_database.logged_in {
                                info!(
                                        "{index_supplied} {username} login failed, {index_database} is logged in"
                                    );

                                Some(((tx), false, (*command).to_string()))
                            // The username is in the database, but not logged in yet.
                            } else {
                                let password_1 = the_rest;
                                let Some(password_2) = self.passwords.get(*username) else {
                                    panic!("we already know the username is in the database.");
                                };

                                let hash_2 = PasswordHash::try_from(password_2.as_str()).unwrap();
                                if let Err(_error) = Argon2::default()
                                    .verify_password(password_1.as_bytes(), &hash_2)
                                {
                                    info!(
                                        "{index_supplied} {username} provided the wrong password"
                                    );
                                    return Some((tx, false, (*command).to_string()));
                                }
                                info!("{index_supplied} {username} logged in");

                                self.clients.insert(index_supplied, tx);
                                index_database.logged_in = Some(index_supplied);

                                Some((
                                    self.clients[&index_supplied].clone(),
                                    true,
                                    (*command).to_string(),
                                ))
                            }
                        // The username is not in the database.
                        } else {
                            info!("{index_supplied} {username} created user account");

                            let password = the_rest;
                            let ctx = Argon2::default();
                            let salt = SaltString::generate(&mut OsRng);
                            let hash = ctx
                                .hash_password(password.as_bytes(), &salt)
                                .unwrap()
                                .to_string();

                            self.passwords.insert((*username).to_string(), hash);

                            self.clients.insert(index_supplied, tx);
                            self.accounts.0.insert(
                                (*username).to_string(),
                                Account {
                                    logged_in: Some(index_supplied),
                                    ..Default::default()
                                },
                            );

                            self.save_server();

                            Some((
                                self.clients[&index_supplied].clone(),
                                true,
                                (*command).to_string(),
                            ))
                        }
                    } else {
                        panic!("there is no channel to send on")
                    }
                }
                "logout" => {
                    // The username is in the database and already logged in.
                    if let Some(account) = self.accounts.0.get_mut(*username) {
                        if let Some(index_database) = account.logged_in {
                            if index_database == index_supplied {
                                info!("{index_supplied} {username} logged out");
                                account.logged_in = None;
                                self.clients.remove(&index_database);
                                // Remove the pending game if there is one.
                                let mut index_option = None;
                                for (index, game) in self.pending_games.0.clone() {
                                    if let Some(attacker) = &game.attacker {
                                        if attacker == username {
                                            index_option = Some(index);
                                        }
                                    }
                                    if let Some(defender) = &game.defender {
                                        if defender == username {
                                            index_option = Some(index);
                                        }
                                    }
                                }

                                if let Some(index) = index_option {
                                    self.pending_games.0.remove(&index);
                                }

                                None
                            } else {
                                Some((
                                    self.clients[&index_supplied].clone(),
                                    false,
                                    (*command).to_string(),
                                ))
                            }
                        } else {
                            Some((
                                self.clients[&index_supplied].clone(),
                                false,
                                (*command).to_string(),
                            ))
                        }
                    } else {
                        Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ))
                    }
                }
                // new_game attacker rated fischer 900000 10
                "new_game" => {
                    let mut the_rest = the_rest.split_ascii_whitespace();
                    let Some(role) = the_rest.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(role) = Role::try_from(role) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };

                    let Some(rated) = the_rest.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(rated) = Rated::try_from(rated) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };

                    let Some(timed) = the_rest.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Some(minutes) = the_rest.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Some(add_seconds) = the_rest.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(timed) =
                        TimeSettings::try_from(vec!["time-settings", timed, minutes, add_seconds])
                    else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };

                    info!(
                        "{index_supplied} {username} new_game {} {role} {rated} {timed:?}",
                        self.game_id
                    );
                    let game = if role == Role::Attacker {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: Some((*username).to_string()),
                            defender: None,
                            rated,
                            timed,
                            channel: Some(index_supplied),
                        }
                    } else {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: None,
                            defender: Some((*username).to_string()),
                            rated,
                            timed,
                            channel: Some(index_supplied),
                        }
                    };
                    let command = format!("{command} {}", game.protocol());
                    self.pending_games.0.insert(self.game_id, game);
                    self.game_id += 1;

                    Some((self.clients[&index_supplied].clone(), true, command))
                }
                // game 0 play black a4 a2
                "game" => {
                    let words: Vec<&str> = the_rest.split_ascii_whitespace().collect();
                    let Some(index) = words.first() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(index) = index.parse() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Some(color) = words.get(2) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(color) = Color::try_from(*color) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Some(from) = words.get(3) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Some(to) = words.get(4) else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let mut to = (*to).to_string();
                    if to == "_" {
                        to = String::new();
                    }

                    let Some(game) = self.games.0.get_mut(&index) else {
                        panic!("the index should be valid")
                    };

                    let mut blacks_turn_next = true;
                    if color == Color::Black {
                        if *username == game.attacker {
                            game.game
                                .read_line(&format!("play black {from} {to}"))
                                .map_err(|error| {
                                    debug!("Error: {error}");
                                    error
                                })
                                .ok()?;
                            blacks_turn_next = false;
                            // Todo: if a player quits he loses.
                            let _ok = game
                                .defender_tx
                                .send(format!("game {index} play black {from} {to}"));
                        } else {
                            return Some((
                                self.clients[&index_supplied].clone(),
                                false,
                                (*command).to_string(),
                            ));
                        }
                    } else if *username == game.defender {
                        game.game
                            .read_line(&format!("play white {from} {to}"))
                            .map_err(|error| {
                                debug!("Error: {error}");
                                error
                            })
                            .ok()?;
                        // Todo: if a player quits he loses.
                        let _ok = game
                            .attacker_tx
                            .send(format!("game {index} play white {from} {to}"));
                    } else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    }

                    // Fixme!
                    match game.game.status {
                        Status::BlackWins => {
                            let accounts = &mut self.accounts.0;
                            let (attacker_rating, defender_rating) =
                                if let (Some(attacker), Some(defender)) =
                                    (accounts.get(&game.attacker), accounts.get(&game.defender))
                                {
                                    (attacker.rating.rating, defender.rating.rating)
                                } else {
                                    panic!("the attacker and defender accounts should exist");
                                };

                            if let Some(attacker) = accounts.get_mut(&game.attacker) {
                                attacker.wins += 1;

                                if game.rated.into() {
                                    attacker
                                        .rating
                                        .update_rating(defender_rating, &Outcome::Win);
                                }
                            }
                            if let Some(defender) = accounts.get_mut(&game.defender) {
                                defender.losses += 1;

                                if game.rated.into() {
                                    defender
                                        .rating
                                        .update_rating(attacker_rating, &Outcome::Loss);
                                }
                            }

                            self.archived_games.push(ArchivedGame {
                                id: game.id,
                                attacker: game.attacker.to_string(),
                                defender: game.defender.to_string(),
                                rated: game.rated,
                                plays: game.game.plays.clone(),
                                status: game.game.status.clone(),
                                text: game.text.clone(),
                            });

                            self.save_server();
                        }
                        Status::Draw => {}
                        Status::Ongoing => {
                            if blacks_turn_next {
                                // Todo: if a player quits he loses.
                                let _ok = game
                                    .attacker_tx
                                    .send(format!("game {index} generate_move black"));
                            } else {
                                // Todo: if a player quits he loses.
                                let _ok = game
                                    .defender_tx
                                    .send(format!("game {index} generate_move white"));
                            }
                        }
                        Status::WhiteWins => {
                            let accounts = &mut self.accounts.0;
                            let (attacker_rating, defender_rating) =
                                if let (Some(attacker), Some(defender)) =
                                    (accounts.get(&game.attacker), accounts.get(&game.defender))
                                {
                                    (attacker.rating.rating, defender.rating.rating)
                                } else {
                                    panic!("the attacker and defender accounts should exist");
                                };

                            if let Some(attacker) = accounts.get_mut(&game.attacker) {
                                attacker.losses += 1;

                                if game.rated.into() {
                                    attacker
                                        .rating
                                        .update_rating(defender_rating, &Outcome::Loss);
                                }
                            }
                            if let Some(defender) = accounts.get_mut(&game.defender) {
                                defender.wins += 1;

                                if game.rated.into() {
                                    defender
                                        .rating
                                        .update_rating(attacker_rating, &Outcome::Win);
                                }
                            }

                            self.archived_games.push(ArchivedGame {
                                id: game.id,
                                attacker: game.attacker.to_string(),
                                defender: game.defender.to_string(),
                                rated: game.rated,
                                plays: game.game.plays.clone(),
                                status: game.game.status.clone(),
                                text: game.text.clone(),
                            });

                            self.save_server();
                        }
                    }

                    Some((
                        self.clients[&index_supplied].clone(),
                        true,
                        (*command).to_string(),
                    ))
                }
                "text" => {
                    info!("{index_supplied} {username} text {the_rest}");
                    for tx in &mut self.clients.values() {
                        let _ok = tx.send(format!("= text {username}: {the_rest}"));
                    }
                    None
                }
                "text_game" => {
                    let mut text = the_rest.split_ascii_whitespace();
                    let Some(id) = text.next() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };
                    let Ok(id) = id.parse::<u64>() else {
                        return Some((
                            self.clients[&index_supplied].clone(),
                            false,
                            (*command).to_string(),
                        ));
                    };

                    let text: Vec<&str> = text.collect();
                    let mut text = text.join(" ");
                    text = format!("{username}: {text}");
                    info!("{index_supplied} {username} text_game {id} {text}");

                    if let Some(game) = self.games.0.get_mut(&id) {
                        game.text.push_str(&text);
                        text = format!("= text_game {text}");
                        let _ok = game.attacker_tx.send(text.clone());
                        let _ok = game.defender_tx.send(text);
                    }

                    None
                }
                "=" => None,
                _ => Some((
                    self.clients[&index_supplied].clone(),
                    false,
                    (*command).to_string(),
                )),
            }
        } else {
            panic!("we pass the arguments in that form");
        }
    }

    fn save_server(&self) {
        let mut server = self.clone();
        for account in server.accounts.0.values_mut() {
            account.logged_in = None;
        }

        let data_file = data_file();
        if let Ok(mut file) = File::create(&data_file) {
            if let Ok(string) = to_string_pretty(&server, PrettyConfig::default()) {
                if let Err(error) = file.write_all(string.as_bytes()) {
                    log::error!("{error}");
                }
            }
        }
    }
}

pub struct LoggedIn {
    _username: String,
    _game_open: Option<Game>,
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
