use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    str::FromStr,
    time::Duration,
};

use anyhow::Error;
use hnefatafl::{
    board::state::BitfieldBoardState,
    pieces::Side,
    play::Play,
    preset::{boards, rules},
};
use hnefatafl_copenhagen::{
    VERSION_ID,
    color::Color,
    game::Game,
    play::{Plae, Vertex},
    status::Status,
};
use hnefatafl_egui::ai::{Ai, BasicAi};

const ADDRESS: &str = "localhost:49152";

fn main() -> anyhow::Result<()> {
    let mut buf = String::new();
    let mut tcp = TcpStream::connect(ADDRESS)?;
    let mut reader = BufReader::new(tcp.try_clone()?);

    tcp.write_all(format!("{VERSION_ID} login ai-00\n").as_bytes())?;
    reader.read_line(&mut buf)?;
    assert_eq!(buf, "= login\n");
    buf.clear();

    loop {
        tcp.write_all(b"new_game attacker rated fischer 900000 10\n")?;

        loop {
            // "= new_game game GAME_ID ai-00 _ rated fischer 900000 10 _ false {}\n"
            reader.read_line(&mut buf)?;

            if buf.trim().is_empty() {
                return Err(Error::msg("the TCP stream has closed"));
            }

            let message: Vec<_> = buf.split_ascii_whitespace().collect();
            if message[1] == "new_game" {
                break;
            }

            buf.clear();
        }

        let buf_clone = buf.clone();
        let message: Vec<_> = buf_clone.split_ascii_whitespace().collect();
        let game_id = message[3];
        buf.clear();

        // Wait for a challenger...
        loop {
            reader.read_line(&mut buf)?;

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
        let game = Game::default();

        let game_: hnefatafl::game::Game<BitfieldBoardState<u128>> =
            hnefatafl::game::Game::new(rules::COPENHAGEN, boards::COPENHAGEN).unwrap();

        println!("{}", game_.state.board);

        let ai =
            hnefatafl_egui::ai::BasicAi::new(game_.logic, Side::Attacker, Duration::from_secs(10));

        handle_messages(ai, game, game_, game_id, &mut reader, &mut tcp)?;
    }
}

fn handle_messages(
    mut ai: BasicAi,
    mut game: Game,
    mut game_: hnefatafl::game::Game<BitfieldBoardState<u128>>,
    game_id: &str,
    reader: &mut BufReader<TcpStream>,
    tcp: &mut TcpStream,
) -> anyhow::Result<()> {
    let mut buf = String::new();
    loop {
        reader.read_line(&mut buf)?;

        if buf.trim().is_empty() {
            return Err(Error::msg("the TCP stream has closed"));
        }

        let message: Vec<_> = buf.split_ascii_whitespace().collect();

        if Some("generate_move") == message.get(2).copied() {
            let Ok((play, _lines)) = ai.next_play(&game_.state) else {
                panic!("we got an error from ai.next_play");
            };

            if let Err(invalid_play) = game_.do_play(play) {
                println!("invalid_play: {invalid_play:?}");
                tcp.write_all(format!("game {game_id} play black resigns _\n").as_bytes())?;
                return Ok(());
            }

            println!("{play}");
            let play = Plae::try_from_(Color::Black, &play.to_string())?;
            game.read_line(&play.to_string())?;
            tcp.write_all(format!("game {game_id} {play}").as_bytes())?;

            println!("{play}");
            println!("{}", game_.state.board);

            if game.status != Status::Ongoing {
                return Ok(());
            }
        } else if Some("play") == message.get(2).copied() {
            let Some(color) = message.get(3).copied() else {
                panic!("expected color");
            };
            let Ok(color) = Color::try_from(color) else {
                panic!("expected color to be a color");
            };

            let Some(from) = message.get(4).copied() else {
                panic!("expected from");
            };
            if from == "resigns" {
                return Ok(());
            }
            let Ok(from) = Vertex::try_from(from) else {
                panic!("expected from to be a vertex");
            };

            let Some(to) = message.get(5).copied() else {
                panic!("expected to");
            };
            let Ok(to) = Vertex::try_from(to) else {
                panic!("expected to to be a vertex");
            };

            let play = format!("play {color} {from} {to}\n");
            print!("{play}");
            game.read_line(&play)?;

            if game.status != Status::Ongoing {
                return Ok(());
            }

            let play = format!("{}-{}", from.fmt_other(), to.fmt_other());
            let play = Play::from_str(&play).unwrap();
            println!("{play}");
            println!();

            if let Err(invalid_play) = game_.do_play(play) {
                println!("invalid_play: {invalid_play:?}");
                tcp.write_all(format!("game {game_id} play black resigns _\n").as_bytes())?;
                return Ok(());
            }

            println!("{}", game_.state.board);
        }

        buf.clear();
    }
}
