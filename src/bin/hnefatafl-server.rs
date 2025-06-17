use std::io::BufReader;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use clap::command;
use clap::{self, Parser};

use hnefatafl_copenhagen::game::Game;
use hnefatafl_copenhagen::status::Status;
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
    start(&args.host_port)
}

struct Htp {
    attacker_connection: TcpStream,
    defender_connection: TcpStream,
}

impl Htp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut attacker_reader = BufReader::new(self.attacker_connection.try_clone()?);
        let mut defender_reader = BufReader::new(self.defender_connection.try_clone()?);

        let mut game = Game::default();

        for i in 1..10_000 {
            println!("\n*** turn {} ***", 2 * i - 1);
            write_command("generate_move attacker\n", &mut self.attacker_connection)?;
            let attacker_move = read_response(&mut attacker_reader)?;

            game.read_line(&attacker_move)?;
            write_command(&attacker_move, &mut self.defender_connection)?;
            if game.status != Status::Ongoing {
                break;
            }

            println!("\n*** turn {} ***", 2 * i);
            write_command("generate_move defender\n", &mut self.defender_connection)?;
            let defender_move = read_response(&mut defender_reader)?;

            game.read_line(&defender_move)?;
            write_command(&defender_move, &mut self.attacker_connection)?;
            if game.status != Status::Ongoing {
                break;
            }
        }

        self.attacker_connection.shutdown(Shutdown::Both)?;
        self.defender_connection.shutdown(Shutdown::Both)?;

        Ok(())
    }
}

fn start(address: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;
    println!("listening on {address} ...");

    let mut players = Vec::new();

    for stream in listener.incoming() {
        let stream = stream?;

        if players.is_empty() {
            players.push(stream);
        } else {
            let mut game = Htp {
                attacker_connection: players.pop().unwrap(),
                defender_connection: stream,
            };

            thread::spawn(move || game.start());
        }
    }

    Ok(())
}
