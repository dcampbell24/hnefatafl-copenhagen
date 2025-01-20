// You can connect to this server
// for now the server will hand out a user number
//
// You can create or join a game
// list all the available games
// watch a game in progress
// review already played games
//
// a server has a database of finished games

use std::{
    collections::{HashMap, HashSet},
    io::{BufRead, BufReader},
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
    // Accept TCP connections.
    // Get a login.
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

    for (index, stream) in listener.incoming().enumerate() {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, &stream, &tx));
    }

    Ok(())
}

fn login(index: usize, stream: &TcpStream, tx: &mpsc::Sender<String>) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    let username = buf.to_string();

    tx.send(format!("{index} enter {username}"))?;

    for _ in 0..1_000_000 {
        reader.read_line(&mut buf)?;
    }

    tx.send(format!("{index} leave {username}"))?;

    Ok(())
}

#[derive(Default)]
struct Server {
    usernames: HashMap<String, Option<u64>>,
    _game_ids: HashSet<u64>,
    _games: Vec<ServerGame>,
}

impl Server {
    fn handle_login(&mut self, rx: &mpsc::Receiver<String>) -> anyhow::Result<()> {
        loop {
            let index_command_username = rx.recv()?;
            let index_command_username: Vec<_> =
                index_command_username.split_ascii_whitespace().collect();
            if let (Some(index_supplied), Some(command), Some(username)) = (
                index_command_username.first(),
                index_command_username.get(1),
                index_command_username.get(2),
            ) {
                match *command {
                    "enter" => {
                        if let Some(index_database) = self.usernames.get_mut(*username) {
                            // The username is in the database and already logged in.
                            if let Some(index_database) = index_database {
                                println!("{username} failed to login, {index_database} is active");
                            // The username is in the database, but not logged in yet.
                            } else if let Ok(index_supplied) = index_supplied.parse::<u64>() {
                                println!("{index_supplied} {username} logged in");
                                *index_database = Some(index_supplied);
                            }
                        // The username is not in the database.
                        } else if let Ok(index_supplied) = index_supplied.parse::<u64>() {
                            println!("{index_supplied} {username} created new user account");
                            self.usernames
                                .insert((*username).to_string(), Some(index_supplied));
                        }
                    }
                    "leave" => {
                        // The username is in the database and already logged in.
                        if let Some(index_database_option) = self.usernames.get_mut(*username) {
                            if let Some(index_database) = index_database_option {
                                if let Ok(index_supplied) = index_supplied.parse::<u64>() {
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
    _id: u64,
    _attacker: String,
    _defender: String,
    _game: Game,
}
