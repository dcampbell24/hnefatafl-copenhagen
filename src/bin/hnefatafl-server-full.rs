use std::{
    collections::HashMap,
    env, fmt,
    fs::{read_to_string, File},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver},
    thread::{self, sleep},
    time::Duration,
};

use chrono::Utc;
use clap::{command, Parser};
use env_logger::Builder;
use hnefatafl_copenhagen::{
    color::Color,
    game::Game,
    handle_error,
    role::Role,
    server_game::{ArchivedGame, ServerGame, ServerGameLight},
    status::Status,
};
use log::{debug, info, LevelFilter};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens client connections.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host and port
    #[arg(default_value = "localhost:8000", index = 1, value_name = "host:port")]
    host_port: String,
    /// Load the server state from a file
    #[arg(long)]
    load: Option<String>,
}

fn main() -> anyhow::Result<()> {
    // 3. new_game attacker [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
    // 3. watch_game 1
    // 3. Display in game users.
    // 1. need passwords, you can enter no password
    // 1. need offline storage of the server state
    // 2. glicko rating system
    // 4. figure out some way of testing

    init_logger();
    let args = Args::parse();

    let mut server = if let Some(file) = args.load {
        ron::from_str(&read_to_string(&file)?)?
    } else {
        Server::default()
    };

    let address = &args.host_port;
    let listener = TcpListener::bind(address)?;
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

fn login(
    index: u64,
    mut stream: TcpStream,
    tx: &mpsc::Sender<(String, Option<mpsc::Sender<String>>)>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    if buf.trim().is_empty() {
        return Ok(());
    }

    buf.make_ascii_lowercase();

    if let Some(username) = buf.split_ascii_whitespace().next() {
        let (client_tx, client_rx) = mpsc::channel();
        tx.send((format!("{index} {username} login"), Some(client_tx)))?;

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
            tx.send((format!("{index} {username} {}", buf.trim()), None))?;
            buf.clear();
        }

        tx.send((format!("{index} {username} logout"), None))?;
    }

    Ok(())
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
    accounts: Accounts,
    archived_games: Vec<ArchivedGame>,
    #[serde(skip)]
    clients: HashMap<u64, mpsc::Sender<String>>,
    #[serde(skip)]
    pending_games: ServerGamesLight,
    #[serde(skip)]
    games: ServerGames,
    game_id: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Accounts(HashMap<String, Account>);

impl fmt::Display for Accounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names = Vec::new();
        for (name, account) in &self.0 {
            if account.logged_in.is_some() {
                names.push(name.to_string());
            }
        }
        names.sort_unstable();
        let names = names.join(" ");

        write!(f, "{names}")
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Account {
    logged_in: Option<u64>,
    wins: u32,
    losses: u32,
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
            let (tx, ok, command) = self.handle_messages_internal(rx);
            if let Some(tx) = tx {
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
    ) -> (Option<mpsc::Sender<String>>, bool, String) {
        let (message, option_tx) = handle_error(rx.recv());
        let index_username_command: Vec<_> = message.split_ascii_whitespace().collect();

        if let (Some(index_supplied), Some(username), command_option) = (
            index_username_command.first(),
            index_username_command.get(1),
            index_username_command.get(2),
        ) {
            let Some(command) = command_option else {
                return (None, false, String::new());
            };

            let index_supplied = index_supplied
                .parse::<u64>()
                .expect("the index should be a valid u64");

            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
            let the_rest = the_rest.join(" ");
            match *command {
                "display_server" => {
                    debug!("0 {username} display_server");
                    for tx in &mut self.clients.values() {
                        handle_error(
                            tx.send(format!("= display_pending_games {}", &self.pending_games)),
                        );
                        handle_error(tx.send(format!("= display_games {}", &self.games)));
                        handle_error(tx.send(format!("= display_users {}", &self.accounts)));
                    }
                    (None, true, (*command).to_string())
                }
                "join_game" => {
                    let Some(id) = the_rest.split_ascii_whitespace().next() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(id) = id.parse::<u64>() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
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
                        ServerGame {
                            id: game.id,
                            attacker: attacker.clone(),
                            attacker_tx: attacker_tx.clone(),
                            defender: (*username).to_string(),
                            defender_tx,
                            game: Game::default(),
                            text: String::new(),
                        }
                    } else if let Some(defender) = &game.defender {
                        ServerGame {
                            id: game.id,
                            attacker: (*username).to_string(),
                            attacker_tx: attacker_tx.clone(),
                            defender: defender.clone(),
                            defender_tx,
                            game: Game::default(),
                            text: String::new(),
                        }
                    } else {
                        panic!("there has to be an attacker or defender")
                    };

                    self.games.0.insert(id, new_game);

                    handle_error(self.clients[&channel].send("= new_game ready".to_string()));
                    handle_error(self.clients[&index_supplied].send("= join_game".to_string()));
                    handle_error(attacker_tx.send(format!("game {id} generate_move black")));

                    (None, true, String::new())
                }
                "login" => {
                    if let Some(tx) = option_tx {
                        if let Some(index_database) = self.accounts.0.get_mut(*username) {
                            // The username is in the database and already logged in.
                            if let Some(index_database) = index_database.logged_in {
                                info!(
                                        "{index_supplied} {username} login failed, {index_database} is logged in"
                                    );

                                (Some(tx), false, (*command).to_string())
                            // The username is in the database, but not logged in yet.
                            } else {
                                info!("{index_supplied} {username} logged in");
                                self.clients.insert(index_supplied, tx);
                                index_database.logged_in = Some(index_supplied);

                                (
                                    Some(self.clients[&index_supplied].clone()),
                                    true,
                                    (*command).to_string(),
                                )
                            }
                        // The username is not in the database.
                        } else {
                            info!("{index_supplied} {username} created user account");
                            self.clients.insert(index_supplied, tx);
                            self.accounts.0.insert(
                                (*username).to_string(),
                                Account {
                                    logged_in: Some(index_supplied),
                                    ..Default::default()
                                },
                            );

                            self.save_server();

                            (
                                Some(self.clients[&index_supplied].clone()),
                                true,
                                (*command).to_string(),
                            )
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

                                (None, true, (*command).to_string())
                            } else {
                                (
                                    Some(self.clients[&index_supplied].clone()),
                                    false,
                                    (*command).to_string(),
                                )
                            }
                        } else {
                            (
                                Some(self.clients[&index_supplied].clone()),
                                false,
                                (*command).to_string(),
                            )
                        }
                    } else {
                        (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        )
                    }
                }
                "new_game" => {
                    let Some(role) = the_rest.split_ascii_whitespace().next() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(role) = Role::try_from(role) else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };

                    info!(
                        "{index_supplied} {username} new_game {} {role}",
                        self.game_id
                    );
                    let game = if role == Role::Attacker {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: Some((*username).to_string()),
                            defender: None,
                            channel: Some(index_supplied),
                        }
                    } else {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: None,
                            defender: Some((*username).to_string()),
                            channel: Some(index_supplied),
                        }
                    };
                    let command = format!("{command} {}", game.protocol());
                    self.pending_games.0.insert(self.game_id, game);
                    self.game_id += 1;

                    (Some(self.clients[&index_supplied].clone()), true, command)
                }
                // game 0 play black a4 a2
                "game" => {
                    let words: Vec<&str> = the_rest.split_ascii_whitespace().collect();
                    let Some(index) = words.first() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(index) = index.parse() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Some(color) = words.get(2) else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(color) = Color::try_from(*color) else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Some(from) = words.get(3) else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Some(to) = words.get(4) else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };

                    let Some(game) = self.games.0.get_mut(&index) else {
                        panic!("the index should be valid")
                    };

                    let mut blacks_turn_next = true;
                    if color == Color::Black {
                        if *username == game.attacker {
                            handle_error(game.game.read_line(&format!("play black {from} {to}")));
                            blacks_turn_next = false;
                            // Todo: if a player quits he loses.
                            let _ok = game
                                .defender_tx
                                .send(format!("game {index} play black {from} {to}"));
                        } else {
                            return (
                                Some(self.clients[&index_supplied].clone()),
                                false,
                                (*command).to_string(),
                            );
                        }
                    } else if *username == game.defender {
                        handle_error(game.game.read_line(&format!("play white {from} {to}")));
                        // Todo: if a player quits he loses.
                        let _ok = game
                            .attacker_tx
                            .send(format!("game {index} play white {from} {to}"));
                    } else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    }

                    match game.game.status {
                        Status::BlackWins => {
                            if let Some(attacker) = self.accounts.0.get_mut(&game.attacker) {
                                attacker.wins += 1;
                            }
                            if let Some(defender) = self.accounts.0.get_mut(&game.defender) {
                                defender.losses += 1;
                            }

                            self.archived_games.push(ArchivedGame {
                                id: game.id,
                                attacker: game.attacker.to_string(),
                                defender: game.defender.to_string(),
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
                            if let Some(attacker) = self.accounts.0.get_mut(&game.attacker) {
                                attacker.losses += 1;
                            }
                            if let Some(defender) = self.accounts.0.get_mut(&game.defender) {
                                defender.wins += 1;
                            }

                            self.archived_games.push(ArchivedGame {
                                id: game.id,
                                attacker: game.attacker.to_string(),
                                defender: game.defender.to_string(),
                                plays: game.game.plays.clone(),
                                status: game.game.status.clone(),
                                text: game.text.clone(),
                            });

                            self.save_server();
                        }
                    }

                    (
                        Some(self.clients[&index_supplied].clone()),
                        true,
                        (*command).to_string(),
                    )
                }
                "text" => {
                    info!("{index_supplied} {username} text {the_rest}");
                    for tx in &mut self.clients.values() {
                        let _ok = tx.send(format!("= text {the_rest}"));
                    }
                    (None, true, (*command).to_string())
                }
                "text_game" => {
                    let mut text = the_rest.split_ascii_whitespace();
                    let Some(id) = text.next() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(id) = id.parse::<u64>() else {
                        return (
                            Some(self.clients[&index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };

                    let text: Vec<&str> = text.collect();
                    let mut text = text.join(" ");
                    text = format!("{text}\n");
                    info!("{index_supplied} {username} text_game {id} {text}");

                    if let Some(game) = self.games.0.get_mut(&id) {
                        game.text.push_str(&text);
                        text = format!("= text_game {text}\n");
                        let _ok = game.attacker_tx.send(text.clone());
                        let _ok = game.defender_tx.send(text);
                    }

                    (None, true, (*command).to_string())
                }
                "=" => {
                    // todo
                    // = game id play color a2 a3
                    // Update the state of game id
                    // send game id play opposite_color
                    // check if the game has ended
                    // if not send game id generate_move opposite_color
                    (None, true, String::new())
                }
                _ => (
                    Some(self.clients[&index_supplied].clone()),
                    false,
                    (*command).to_string(),
                ),
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

        if let Ok(mut file) = File::create("hnefatafl-copenhagen.ron") {
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
