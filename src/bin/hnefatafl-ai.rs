use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use hnefatafl_copenhagen::{VERSION_ID, color::Color, game::Game, play::Vertex, status::Status};

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

            let message: Vec<_> = buf.split_ascii_whitespace().collect();
            if Some("challenge_requested") == message.get(1).copied() {
                println!("{message:?}");
                buf.clear();

                break;
            }

            buf.clear();
        }

        tcp.write_all(format!("join_game {game_id}\n").as_bytes())?;
        let mut game = Game::default();

        loop {
            reader.read_line(&mut buf)?;
            let message: Vec<_> = buf.split_ascii_whitespace().collect();

            if Some("generate_move") == message.get(2).copied() {
                let Some(play) = game.generate_move() else {
                    panic!("we are not passing the empty string")
                };

                tcp.write_all(format!("game {game_id} play {play} _\n").as_bytes())?;
                println!("{play:?}");
            }

            if Some("play") == message.get(2).copied() {
                let Some(color) = message.get(3).copied() else {
                    panic!("expected color");
                };
                let Ok(color) = Color::try_from(color) else {
                    panic!("expected color to be a color");
                };

                let Some(from) = message.get(4).copied() else {
                    panic!("expected from");
                };
                let Ok(from) = Vertex::try_from(from) else {
                    panic!("expected from to be a vertex");
                };

                let Some(to) = message.get(5).copied() else {
                    panic!("expected to");
                };
                let Ok(to) = Vertex::try_from(to) else {
                    panic!("expected to to be a vertex");
                };

                game.read_line(&format!("play {color} {from} {to}\n"))?;

                if game.status != Status::Ongoing {
                    break;
                }
            }

            buf.clear();
        }
    }
}
