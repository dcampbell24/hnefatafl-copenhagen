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
    let child = std::process::Command::new("./target/debug/hnefatafl-server-full").spawn()?;

    thread::sleep(Duration::from_millis(10));
    // new_game user 1
    // join game user 2
    // accept user_1
    // black plays
    // white plays...

    let mut tcp_1 = TcpStream::connect(ADDRESS)?;
    let mut reader_1 = BufReader::new(tcp_1.try_clone()?);
    let mut buf_1 = String::new();

    tcp_1.write_all(format!("{VERSION_ID} david\n").as_bytes())?;
    reader_1.read_line(&mut buf_1)?;
    println!("{buf_1}");

    thread::sleep(Duration::from_millis(10));

    let mut tcp_2 = TcpStream::connect(ADDRESS)?;
    let mut reader_2 = BufReader::new(tcp_2.try_clone()?);
    let mut buf_2 = String::new();

    tcp_2.write_all(format!("{VERSION_ID} abby\n").as_bytes())?;
    reader_2.read_line(&mut buf_2)?;
    println!("{buf_2}");

    thread::sleep(Duration::from_millis(10));

    signal::kill(Pid::from_raw(i32::try_from(child.id())?), Signal::SIGTERM)?;
    Ok(())
}
