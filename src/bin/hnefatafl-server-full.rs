use std::{
    collections::HashMap,
    env, fmt,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver},
    thread::{self, sleep},
    time::Duration,
};

use chrono::Utc;
use clap::{command, Parser};
use env_logger::Builder;
use hnefatafl_copenhagen::{game::Game, handle_error, role::Role, server_game::ServerGameLight};
use log::{info, LevelFilter};

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens client connections.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host and port
    #[arg(default_value = "localhost:8000", index = 1, value_name = "host:port")]
    host_port: String,
}

fn main() -> anyhow::Result<()> {
    // X connected to localhost:8000 ...
    // X <- david
    // X -> = login
    // X <- new_game attacker [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
    // X -> = new_game game 1 david none
    // X -> = display_pending_games game 1 david none
    // X -> = display_users david
    // X connected to localhost:8000 ...
    // X <- abby
    // X -> = login
    // join_game 1
    // display_active_games game 1 david abby
    // watch_game 1,
    // display_archived_games game 1 david abby
    // X <- text A message!
    // X -> = text A message!
    // text_game 1 A message!

    init_logger();

    let mut server = Server::default();
    let args = Args::parse();
    let address = &args.host_port;
    let listener = TcpListener::bind(address)?;
    info!("listening on {address} ...");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || server.handle_messages(&rx));

    let tx_messages = tx.clone();
    thread::spawn(move || loop {
        tx_messages
            .send(("0 server display_server".to_string(), None))
            .unwrap();

        sleep(Duration::from_secs(10));
    });

    for (index, stream) in (1..).zip(listener.incoming()) {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, stream, &tx));
    }

    Ok(())
}

fn login(
    index: u128,
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

#[derive(Default)]
struct Server {
    accounts: Accounts,
    clients: HashMap<u128, mpsc::Sender<String>>,
    pending_games: GamesLight,
    game_id: u128,
}

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Debug, Default)]
struct Account {
    logged_in: Option<u128>,
}

#[derive(Clone, Debug, Default)]
struct GamesLight(HashMap<u128, ServerGameLight>);

impl fmt::Display for GamesLight {
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
                .parse::<u128>()
                .expect("the index should be a valid u128");

            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
            let the_rest = the_rest.join(" ");
            match *command {
                "display_server" => {
                    info!("0 {username} display_server");
                    for tx in &mut self.clients.values() {
                        handle_error(
                            tx.send(format!("= display_pending_games {}", &self.pending_games)),
                        );
                        handle_error(tx.send(format!("= display_users {}", &self.accounts)));
                    }
                    (None, true, (*command).to_string())
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
                                },
                            );

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
                    if let Some(index_database_option) = self.accounts.0.get_mut(*username) {
                        if let Some(index_database) = index_database_option.logged_in {
                            if index_database == index_supplied {
                                info!("{index_supplied} {username} logged out");
                                *index_database_option = Account { logged_in: None };
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

                    info!("{index_supplied} {username} new_game {role}");
                    let game = if role == Role::Attacker {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: Some((*username).to_string()),
                            defender: None,
                        }
                    } else {
                        ServerGameLight {
                            id: self.game_id,
                            attacker: None,
                            defender: Some((*username).to_string()),
                        }
                    };
                    let command = format!("{command} {}", game.protocol());
                    self.pending_games.0.insert(self.game_id, game);
                    self.game_id += 1;

                    (Some(self.clients[&index_supplied].clone()), true, command)
                }
                "text" => {
                    info!("{index_supplied} {username} text {the_rest}");
                    for tx in &mut self.clients.values() {
                        handle_error(tx.send(format!("= text {the_rest}")));
                    }
                    (None, true, (*command).to_string())
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
