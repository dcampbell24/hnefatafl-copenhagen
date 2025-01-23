use std::{
    collections::{HashMap, HashSet},
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver},
    thread,
};

use chrono::Utc;
use clap::{command, Parser};
use env_logger::Builder;
use hnefatafl_copenhagen::game::Game;
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
    //     join_game,
    //     watch_game,
    //     list_active_games,
    //     list_archived_games,
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

    if let Some(username) = buf.split_ascii_whitespace().next() {
        let (client_tx, client_rx) = mpsc::channel();
        tx.send((format!("{index} {username} login"), Some(client_tx)))?;

        let message = client_rx.recv()?;
        if "=" != message.as_str() {
            stream.write_all(b"?\n")?;
            return Ok(());
        }
        stream.write_all(b"=\n")?;

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
    clients: Vec<mpsc::Sender<String>>,
    _games: Vec<ServerGame>,
    _game_ids: HashSet<usize>,
    usernames: HashMap<String, Option<usize>>,
}

impl Server {
    fn handle_messages(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> anyhow::Result<()> {
        loop {
            let (tx, ok) = self.handle_messages_internal(rx);
            if let Some(tx) = tx {
                if ok {
                    tx.send('='.to_string())?;
                } else {
                    tx.send('?'.to_string())?;
                }
            }
        }
    }

    fn handle_messages_internal(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> (Option<mpsc::Sender<String>>, bool) {
        let (message, option_tx) = rx.recv().expect("error receiving message");
        let index_username_command: Vec<_> = message.split_ascii_whitespace().collect();

        if let (Some(index_supplied), Some(username), command_option) = (
            index_username_command.first(),
            index_username_command.get(1),
            index_username_command.get(2),
        ) {
            let Some(command) = command_option else {
                return (None, false);
            };

            let index_supplied = index_supplied
                .parse::<usize>()
                .expect("the index should be a valid usize");

            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
            let the_rest = the_rest.join(" ");
            match *command {
                "login" => {
                    if let Some(tx) = option_tx {
                        self.clients.push(tx);

                        if let Some(index_database) = self.usernames.get_mut(*username) {
                            // The username is in the database and already logged in.
                            if let Some(index_database) = index_database {
                                info!(
                                        "{index_supplied} {username} login failed, {index_database} is logged in"
                                    );

                                (None, false)
                            // The username is in the database, but not logged in yet.
                            } else {
                                info!("{index_supplied} {username} logged in");
                                *index_database = Some(index_supplied);

                                (Some(self.clients[index_supplied].clone()), true)
                            }
                        // The username is not in the database.
                        } else {
                            info!("{index_supplied} {username} created user account");
                            self.usernames
                                .insert((*username).to_string(), Some(index_supplied));

                            (Some(self.clients[index_supplied].clone()), true)
                        }
                    } else {
                        panic!("there is no channel to send on")
                    }
                }
                "logout" => {
                    // The username is in the database and already logged in.
                    if let Some(index_database_option) = self.usernames.get_mut(*username) {
                        if let Some(index_database) = index_database_option {
                            if *index_database == index_supplied {
                                info!("{index_supplied} {username} logged out");
                                *index_database_option = None;
                                (None, true)
                            } else {
                                (Some(self.clients[index_supplied].clone()), false)
                            }
                        } else {
                            (Some(self.clients[index_supplied].clone()), false)
                        }
                    } else {
                        (Some(self.clients[index_supplied].clone()), false)
                    }
                }
                "text" => {
                    info!("{index_supplied} {username} text {the_rest}");
                    for tx in &mut self.clients {
                        // fixme
                        tx.send(format!("text {the_rest}")).expect("sending failed");
                    }
                    (Some(self.clients[index_supplied].clone()), true)
                }
                _ => (Some(self.clients[index_supplied].clone()), false),
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

struct ServerGame {
    _id: usize,
    _attacker: String,
    _defender: String,
    _game: Game,
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
