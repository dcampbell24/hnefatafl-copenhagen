// You can connect to this server
// for now the server will hand out a user number
//
// You can create or join a game
// list all the available games
// watch a game in progress
// review already played games
//
// a server has a database of finished games
use std::io::BufReader;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use clap::command;
use clap::{self, Parser};

use hnefatafl_copenhagen::{read_response, write_command};

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
    let args = Args::parse();
    let setup_commands = vec!["# There arn't any.".to_string()];

    start(&args.host_port, &setup_commands)
}

struct Htp {
    black_connection: TcpStream,
    white_connection: TcpStream,
}

impl Htp {
    fn start(&mut self, _setup_commands: Vec<String>) -> anyhow::Result<()> {
        let mut black_reader = BufReader::new(self.black_connection.try_clone()?);
        let mut white_reader = BufReader::new(self.white_connection.try_clone()?);

        /*
        for command in setup_commands {
            send_command(&command, &mut self.black_connection, &mut black_reader)?;
            send_command(&command, &mut self.white_connection, &mut white_reader)?;
        }
        */

        for i in 1..1_000 {
            println!("\n*** turn {} ***", 2 * i - 1);
            write_command("generate_move black\n", &mut self.black_connection)?;
            let black_move = read_response(&mut black_reader)?;

            write_command(&black_move, &mut self.white_connection)?;

            println!("\n*** turn {} ***", 2 * i);
            write_command("generate_move white\n", &mut self.white_connection)?;
            let white_move = read_response(&mut white_reader)?;

            write_command(&white_move, &mut self.black_connection)?;

            if black_move == "= pass\n" && white_move == "= pass\n" {
                break;
            }
        }

        self.black_connection.shutdown(Shutdown::Both)?;
        self.white_connection.shutdown(Shutdown::Both)?;

        Ok(())
    }
}

fn start(address: &str, setup_commands: &[String]) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;
    println!("listening on {address} ...");

    let mut players = Vec::new();

    for stream in listener.incoming() {
        let stream = stream?;

        if players.is_empty() {
            players.push(stream);
        } else {
            let mut game = Htp {
                black_connection: players.pop().unwrap(),
                white_connection: stream,
            };
            let setup_commands = setup_commands.to_owned();

            thread::spawn(move || game.start(setup_commands));
        }
    }

    Ok(())
}
