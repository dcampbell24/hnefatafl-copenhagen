use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

use clap::{command, Parser};
use hnefatafl_copenhagen::game::Game;

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
    //         there is a general chat you join when logged on
    //         an in game chat you join when you join a game

    let mut server = Server::default();
    let args = Args::parse();
    let address = &args.host_port;
    let listener = TcpListener::bind(address)?;
    println!("listening on {address} ...");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || server.handle_login(&rx));

    for (index, stream) in (0..).zip(listener.incoming()) {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, &stream, &tx));
    }

    Ok(())
}

fn login(
    index: u128,
    mut stream: &TcpStream,
    tx: &mpsc::Sender<(String, Option<mpsc::Sender<String>>)>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    let username = buf.to_string();

    let (client_tx, client_rx) = mpsc::channel();
    tx.send((format!("{index} enter {username}"), Some(client_tx)))?;

    let message = client_rx.recv()?;
    if "ok" != message.as_str() {
        stream.write_all(b"error\n")?;
        return Ok(());
    }
    stream.write_all(b"ok\n")?;

    for _ in 0..1_000_000 {
        reader.read_line(&mut buf)?;
    }

    tx.send((format!("{index} leave {username}"), None))?;

    Ok(())
}

#[derive(Default)]
struct Server {
    clients: Vec<mpsc::Sender<String>>,
    _games: Vec<ServerGame>,
    _game_ids: HashSet<u128>,
    usernames: HashMap<String, Option<u128>>,
}

impl Server {
    fn handle_login(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> anyhow::Result<()> {
        loop {
            let (index_command_username, option_tx) = rx.recv()?;
            let index_command_username: Vec<_> =
                index_command_username.split_ascii_whitespace().collect();
            if let (Some(index_supplied), Some(command), Some(username)) = (
                index_command_username.first(),
                index_command_username.get(1),
                index_command_username.get(2),
            ) {
                match *command {
                    "enter" => {
                        if let Some(tx) = option_tx {
                            if let Some(index_database) = self.usernames.get_mut(*username) {
                                // The username is in the database and already logged in.
                                if let Some(index_database) = index_database {
                                    println!(
                                        "{username} failed to login, {index_database} is active"
                                    );
                                    tx.send("error".to_string())?;
                                // The username is in the database, but not logged in yet.
                                } else if let Ok(index_supplied) = index_supplied.parse::<u128>() {
                                    println!("{index_supplied} {username} logged in");
                                    *index_database = Some(index_supplied);
                                    tx.send("ok".to_string())?;
                                } else {
                                    tx.send("error".to_string())?;
                                }
                            // The username is not in the database.
                            } else if let Ok(index_supplied) = index_supplied.parse::<u128>() {
                                println!("{index_supplied} {username} created new user account");
                                self.usernames
                                    .insert((*username).to_string(), Some(index_supplied));
                                tx.send("ok".to_string())?;
                            } else {
                                tx.send("error".to_string())?;
                            }

                            self.clients.push(tx);
                        }
                    }
                    "leave" => {
                        // The username is in the database and already logged in.
                        if let Some(index_database_option) = self.usernames.get_mut(*username) {
                            if let Some(index_database) = index_database_option {
                                if let Ok(index_supplied) = index_supplied.parse::<u128>() {
                                    if *index_database == index_supplied {
                                        println!("{index_supplied} {username} logged out");
                                        *index_database_option = None;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct LoggedIn {
    _username: String,
    _game_open: Option<Game>,
}

struct ServerGame {
    _id: u128,
    _attacker: String,
    _defender: String,
    _game: Game,
}
