use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    str::FromStr,
    thread,
};

use anyhow::Error;
use clap::{Parser, command};
use hnefatafl_copenhagen::{
    VERSION_ID,
    ai::{AI, AiBanal, AiBasic},
    color::Color,
    game::Game,
    play::Vertex,
    role::Role,
    status::Status,
};

// Move 26, defender wins, corner escape, time per move 15s 2025-03-06 (hnefatafl-equi).

const PORT: &str = ":49152";

/// A Hnefatafl Copenhagen AI
///
/// This is an AI client that connects to a server.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    username: String,

    #[arg(default_value = "", long)]
    password: String,

    /// attacker or defender
    #[arg(long)]
    role: Role,

    /// Connect to the HTP server at host
    #[arg(default_value = "hnefatafl.org", long)]
    host: String,

    /// Choose an AI to play as
    #[arg(default_value = "banal", long)]
    ai: String,

    /// Challenge the AI with AI CHALLENGER
    #[arg(long)]
    challenger: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut username = "ai-".to_string();
    username.push_str(&args.username);

    let mut address = args.host.to_string();
    address.push_str(PORT);

    let mut buf = String::new();
    let mut tcp = TcpStream::connect(address.clone())?;
    let mut reader = BufReader::new(tcp.try_clone()?);

    tcp.write_all(format!("{VERSION_ID} login {username} {}\n", args.password).as_bytes())?;
    reader.read_line(&mut buf)?;
    assert_eq!(buf, "= login\n");
    buf.clear();

    if let Some(ai_2) = args.challenger {
        new_game(&mut tcp, args.role, &mut reader, &mut buf)?;

        let message: Vec<_> = buf.split_ascii_whitespace().collect();
        let game_id = message[3].to_string();
        buf.clear();

        let game_id_2 = game_id.clone();
        let ai = args.ai;
        thread::spawn(move || accept_challenger(&ai, &mut reader, &mut buf, &mut tcp, &game_id));

        let mut buf_2 = String::new();
        let mut tcp_2 = TcpStream::connect(address)?;
        let mut reader_2 = BufReader::new(tcp_2.try_clone()?);

        tcp_2.write_all(format!("{VERSION_ID} login ai-01 PASSWORD\n").as_bytes())?;
        reader_2.read_line(&mut buf_2)?;
        assert_eq!(buf_2, "= login\n");

        tcp_2.write_all(format!("join_game_pending {game_id_2}\n").as_bytes())?;

        handle_messages(ai_2.as_str(), &game_id_2, &mut reader_2, &mut tcp_2, true)?;
    } else {
        loop {
            new_game(&mut tcp, args.role, &mut reader, &mut buf)?;

            let message: Vec<_> = buf.split_ascii_whitespace().collect();
            let game_id = message[3].to_string();
            buf.clear();

            wait_for_challenger(&mut reader, &mut buf, &mut tcp, &game_id)?;

            handle_messages(args.ai.as_str(), &game_id, &mut reader, &mut tcp, true)?;
        }
    }

    Ok(())
}

fn accept_challenger(
    ai: &str,
    reader: &mut BufReader<TcpStream>,
    buf: &mut String,
    tcp: &mut TcpStream,
    game_id: &str,
) -> anyhow::Result<()> {
    wait_for_challenger(reader, buf, tcp, game_id)?;

    handle_messages(ai, game_id, reader, tcp, false)?;
    Ok(())
}

// "= new_game game GAME_ID ai-00 _ rated fischer 900000 10 _ false {}\n"
fn new_game(
    tcp: &mut TcpStream,
    role: Role,
    reader: &mut BufReader<TcpStream>,
    buf: &mut String,
) -> anyhow::Result<()> {
    tcp.write_all(format!("new_game {role} rated fischer 900000 10\n").as_bytes())?;

    loop {
        // "= new_game game GAME_ID ai-00 _ rated fischer 900000 10 _ false {}\n"
        reader.read_line(buf)?;

        if buf.trim().is_empty() {
            return Err(Error::msg("the TCP stream has closed"));
        }

        let message: Vec<_> = buf.split_ascii_whitespace().collect();
        if message[1] == "new_game" {
            return Ok(());
        }

        buf.clear();
    }
}

fn wait_for_challenger(
    reader: &mut BufReader<TcpStream>,
    buf: &mut String,
    tcp: &mut TcpStream,
    game_id: &str,
) -> anyhow::Result<()> {
    loop {
        reader.read_line(buf)?;

        if buf.trim().is_empty() {
            return Err(Error::msg("the TCP stream has closed"));
        }

        let message: Vec<_> = buf.split_ascii_whitespace().collect();
        if Some("challenge_requested") == message.get(1).copied() {
            println!("{message:?}");
            buf.clear();

            break;
        }

        buf.clear();
    }

    tcp.write_all(format!("join_game {game_id}\n").as_bytes())?;
    Ok(())
}

fn handle_messages(
    ai: &str,
    game_id: &str,
    reader: &mut BufReader<TcpStream>,
    tcp: &mut TcpStream,
    io_on: bool,
) -> anyhow::Result<()> {
    let mut game = Game::default();
    let mut ai = choose_ai(ai)?;

    if io_on {
        println!("{game}\n");
    }

    let mut buf = String::new();
    loop {
        reader.read_line(&mut buf)?;

        if buf.trim().is_empty() {
            return Err(Error::msg("the TCP stream has closed"));
        }

        let message: Vec<_> = buf.split_ascii_whitespace().collect();

        if Some("generate_move") == message.get(2).copied() {
            let play = game
                .generate_move(&mut ai)
                .expect("the game must be in progress");

            game.play(&play)?;

            tcp.write_all(format!("game {game_id} {play}").as_bytes())?;

            if io_on {
                print!("{play}");
                println!("{}\n", game.board);
            }

            if game.status != Status::Ongoing {
                return Ok(());
            }
        } else if Some("play") == message.get(2).copied() {
            let Some(color) = message.get(3).copied() else {
                panic!("expected color");
            };
            let Ok(color) = Color::from_str(color) else {
                panic!("expected color to be a color");
            };

            let Some(from) = message.get(4).copied() else {
                panic!("expected from");
            };
            if from == "resigns" {
                return Ok(());
            }
            let Ok(from) = Vertex::from_str(from) else {
                panic!("expected from to be a vertex");
            };

            let Some(to) = message.get(5).copied() else {
                panic!("expected to");
            };
            let Ok(to) = Vertex::from_str(to) else {
                panic!("expected to to be a vertex");
            };

            let play = format!("play {color} {from} {to}\n");
            game.read_line(&play)?;

            if io_on {
                print!("{play}");
                println!("{}\n", game.board);
            }

            if game.status != Status::Ongoing {
                return Ok(());
            }
        } else if Some("game_over") == message.get(1).copied() {
            return Ok(());
        }

        buf.clear();
    }
}

fn choose_ai(ai: &str) -> anyhow::Result<Box<dyn AI>> {
    match ai {
        "banal" => Ok(Box::new(AiBanal)),
        "basic" => Ok(Box::new(AiBasic::default())),
        _ => Err(anyhow::Error::msg("you didn't choose a valid AI")),
    }
}
