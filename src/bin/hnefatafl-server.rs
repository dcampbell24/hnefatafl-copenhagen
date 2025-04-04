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
    black_connection: TcpStream,
    white_connection: TcpStream,
}

impl Htp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut black_reader = BufReader::new(self.black_connection.try_clone()?);
        let mut white_reader = BufReader::new(self.white_connection.try_clone()?);

        let mut game = Game::default();

        for i in 1..10_000 {
            println!("\n*** turn {} ***", 2 * i - 1);
            write_command("generate_move black\n", &mut self.black_connection)?;
            let black_move = read_response(&mut black_reader)?;

            game.read_line(&black_move)?;
            write_command(&black_move, &mut self.white_connection)?;
            if game.status != Status::Ongoing {
                break;
            }

            println!("\n*** turn {} ***", 2 * i);
            write_command("generate_move white\n", &mut self.white_connection)?;
            let white_move = read_response(&mut white_reader)?;

            game.read_line(&white_move)?;
            write_command(&white_move, &mut self.black_connection)?;
            if game.status != Status::Ongoing {
                break;
            }
        }

        self.black_connection.shutdown(Shutdown::Both)?;
        self.white_connection.shutdown(Shutdown::Both)?;

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
                black_connection: players.pop().unwrap(),
                white_connection: stream,
            };

            thread::spawn(move || game.start());
        }
    }

    Ok(())
}
