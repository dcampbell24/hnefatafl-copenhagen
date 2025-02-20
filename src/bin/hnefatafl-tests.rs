use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    str,
};

use std::{thread, time::Duration};

use hnefatafl_copenhagen::VERSION_ID;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};

const ADDRESS: &str = "localhost:49152";

fn main() -> anyhow::Result<()> {
    std::process::Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("hnefatafl-server-full")
        .arg("--features")
        .arg("server")
        .arg("--release")
        .output()?;

    let child = std::process::Command::new("./target/release/hnefatafl-server-full")
        .arg("--skip-loading-data-file")
        .arg("--skip-advertising-updates")
        .spawn()?;

    let id = child.id();
    std::panic::set_hook(Box::new(move |info| {
        signal::kill(Pid::from_raw(i32::try_from(id).unwrap()), Signal::SIGTERM).unwrap();
        println!("{info}");
    }));

    thread::sleep(Duration::from_millis(10));

    let mut buf = String::new();

    let mut tcp_1 = TcpStream::connect(ADDRESS)?;
    let mut reader_1 = BufReader::new(tcp_1.try_clone()?);

    tcp_1.write_all(format!("{VERSION_ID} david\n").as_bytes())?;
    reader_1.read_line(&mut buf)?;
    assert_eq!(buf, "= login\n");
    buf.clear();

    tcp_1.write_all(b"change_password\n")?;
    reader_1.read_line(&mut buf)?;
    assert_eq!(buf, "= change_password\n");
    buf.clear();

    tcp_1.write_all(b"new_game attacker rated fischer 900000 10\n")?;
    reader_1.read_line(&mut buf)?;
    assert_eq!(
        buf,
        "= new_game game 0 david _ rated fischer 900000 10 _ false {}\n"
    );
    buf.clear();

    let mut tcp_2 = TcpStream::connect(ADDRESS)?;
    let mut reader_2 = BufReader::new(tcp_2.try_clone()?);

    tcp_2.write_all(format!("{VERSION_ID} abby\n").as_bytes())?;
    reader_2.read_line(&mut buf)?;
    assert_eq!(buf, "= login\n");
    buf.clear();

    tcp_2.write_all(b"join_game_pending 0\n")?;
    reader_2.read_line(&mut buf)?;
    assert_eq!(buf, "= join_game_pending 0\n");
    buf.clear();

    // Todo: "join_game_pending 0\n" should not be allowed!
    tcp_1.write_all(b"join_game 0\n")?;
    reader_1.read_line(&mut buf)?;
    assert_eq!(buf, "= join_game david abby rated fischer 900000 10\n");
    buf.clear();

    reader_2.read_line(&mut buf)?;
    assert_eq!(buf, "= join_game david abby rated fischer 900000 10\n");
    buf.clear();

    reader_1.read_line(&mut buf)?;
    assert_eq!(buf, "game 0 generate_move black\n");
    buf.clear();

    tcp_1.write_all(b"game 0 play black resigns _\n")?;
    reader_1.read_line(&mut buf)?;
    assert_eq!(buf, "= game_over 0 defender_wins\n");
    buf.clear();

    reader_2.read_line(&mut buf)?;
    assert_eq!(buf, "game 0 play black resigns \n");
    buf.clear();

    reader_2.read_line(&mut buf)?;
    assert_eq!(buf, "= game_over 0 defender_wins\n");
    buf.clear();

    signal::kill(Pid::from_raw(i32::try_from(child.id())?), Signal::SIGTERM)?;
    Ok(())
}
