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

    if let Some(username) = buf.split_ascii_whitespace().next() {
        let (client_tx, client_rx) = mpsc::channel();
        tx.send((format!("{index} {username} enter"), Some(client_tx)))?;

        let message = client_rx.recv()?;
        if "ok" != message.as_str() {
            stream.write_all(b"error\n")?;
            return Ok(());
        }
        stream.write_all(b"ok\n")?;

        let mut buf = String::new();
        for _ in 0..1_000_000 {
            reader.read_line(&mut buf)?;
            tx.send((format!("{index} {username} {}", buf.trim()), None))?;
            buf.clear();

            let message = client_rx.recv()?;
            stream.write_all(format!("{message}\n").as_bytes())?;
        }

        tx.send((format!("{index} {username} leave"), None))?;
    }

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
            let (message, option_tx) = rx.recv()?;

            let index_username_command: Vec<_> = message.split_ascii_whitespace().collect();

            println!("{message}");

            if let (Some(index_supplied), Some(username), Some(command)) = (
                index_username_command.first(),
                index_username_command.get(1),
                index_username_command.get(2),
            ) {
                let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();
                let the_rest = the_rest.join(" ");
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
                    "text" => {
                        println!("sending text... {the_rest}");

                        let index = index_supplied.parse::<usize>()?;
                        self.clients[index].send("ok".to_string())?;

                        // This isn't working right.
                        for tx in &mut self.clients {
                            tx.send("ok".to_string())?;
                        }
                    }
                    _ => {
                        let index = index_supplied.parse::<usize>()?;
                        self.clients[index].send("ok".to_string())?;
                    }
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
