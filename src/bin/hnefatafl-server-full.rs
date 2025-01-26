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
use hnefatafl_copenhagen::{game::Game, role::Role, server_game::ServerGameLight};
use log::{info, LevelFilter};

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens for HTP engines
/// to connect and then plays them against each other.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Listen for HTP drivers on host and port
    #[arg(default_value = "localhost:8000", index = 1, value_name = "host:port")]
    host_port: String,
}

fn main() -> anyhow::Result<()> {
    // X Accept TCP connections.
    // X Get a login.
    // Wait for a message such as
    //     new_game,
    //         one player creates a game, then it gets added to the pending games
    //             = new_game ID
    //             new_game (attacker | defender) [TIME_MINUTES] [ADD_SECONDS_AFTER_EACH_MOVE]
    //             ? create_game | = create_game game_id
    //         another play chooses to join a pending game
    //         then the game is added to the active games
    //     join_game game_id,
    //     watch_game game_id,
    //     list_active_games -- returns all of the game_id and a game summary, move #,
    //     list_archived_games -- return all of the game_id s of completed games,
    //     send_message
    //         X there is a general chat you join when logged on
    //         an in game chat you join when you join a game

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

    for (index, stream) in (0..).zip(listener.incoming()) {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, stream, &tx));
    }

    Ok(())
}

fn login(
    index: usize,
    mut stream: TcpStream,
    tx: &mpsc::Sender<(String, Option<mpsc::Sender<String>>)>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    while buf.trim().is_empty() {
        reader.read_line(&mut buf)?;
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
    clients: Vec<mpsc::Sender<String>>,
    pending_games: GamesLight,
    game_id: usize,
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
    logged_in: Option<usize>,
}

#[derive(Clone, Debug, Default)]
struct GamesLight(HashMap<usize, ServerGameLight>);

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
        // Todo: is it ok to ignore errors?
        let Ok((message, option_tx)) = rx.recv() else {
            return (None, false, String::new());
        };
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
                .parse::<usize>()
                .expect("the index should be a valid usize");

            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
            let the_rest = the_rest.join(" ");
            match *command {
                "display_server" => {
                    info!("{index_supplied} {username} display_server");
                    // Todo: is it ok to ignore errors?
                    for tx in &mut self.clients {
                        let _ok = tx
                            .send(format!("= display_pending_games {}", &self.pending_games))
                            .is_ok();
                        let _ok = tx
                            .send(format!("= display_users {}", &self.accounts))
                            .is_ok();
                    }
                    (None, true, (*command).to_string())
                }
                "login" => {
                    if let Some(tx) = option_tx {
                        self.clients.push(tx);

                        if let Some(index_database) = self.accounts.0.get_mut(*username) {
                            // The username is in the database and already logged in.
                            if let Some(index_database) = index_database.logged_in {
                                info!(
                                        "{index_supplied} {username} login failed, {index_database} is logged in"
                                    );

                                (
                                    Some(self.clients[index_supplied].clone()),
                                    false,
                                    (*command).to_string(),
                                )
                            // The username is in the database, but not logged in yet.
                            } else {
                                info!("{index_supplied} {username} logged in");
                                index_database.logged_in = Some(index_supplied);

                                (
                                    Some(self.clients[index_supplied].clone()),
                                    true,
                                    (*command).to_string(),
                                )
                            }
                        // The username is not in the database.
                        } else {
                            info!("{index_supplied} {username} created user account");
                            self.accounts.0.insert(
                                (*username).to_string(),
                                Account {
                                    logged_in: Some(index_supplied),
                                },
                            );

                            (
                                Some(self.clients[index_supplied].clone()),
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
                                (None, true, (*command).to_string())
                            } else {
                                (
                                    Some(self.clients[index_supplied].clone()),
                                    false,
                                    (*command).to_string(),
                                )
                            }
                        } else {
                            (
                                Some(self.clients[index_supplied].clone()),
                                false,
                                (*command).to_string(),
                            )
                        }
                    } else {
                        (
                            Some(self.clients[index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        )
                    }
                }
                "new_game" => {
                    let Some(role) = the_rest.split_ascii_whitespace().next() else {
                        return (
                            Some(self.clients[index_supplied].clone()),
                            false,
                            (*command).to_string(),
                        );
                    };
                    let Ok(role) = Role::try_from(role) else {
                        return (
                            Some(self.clients[index_supplied].clone()),
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
                    let command = format!("{command} {game}");
                    self.pending_games.0.insert(self.game_id, game);
                    self.game_id += 1;

                    (Some(self.clients[index_supplied].clone()), true, command)
                }
                "text" => {
                    info!("{index_supplied} {username} text {the_rest}");
                    for tx in &mut self.clients {
                        // Todo: is it ok to ignore errors?
                        let _ok = tx.send(format!("= text {the_rest}")).is_ok();
                    }
                    (None, true, (*command).to_string())
                }
                _ => (
                    Some(self.clients[index_supplied].clone()),
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
