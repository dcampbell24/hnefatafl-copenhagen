use std::{
    io::{stdin, BufRead, BufReader},
    net::TcpStream,
    thread,
};

use hnefatafl_copenhagen::{read_response, write_command};

fn main() -> anyhow::Result<()> {
    let address = "localhost:8000".to_string();
    let mut stream = TcpStream::connect(&address)?;
    let reader = BufReader::new(stream.try_clone()?);
    println!("connected to {address} ...");

    let mut stdin = stdin().lock();

    thread::spawn(move || reading_and_printing(reader));

    loop {
        let mut buf = String::new();
        stdin.read_line(&mut buf)?;

        write_command(buf.as_str(), &mut stream)?;
    }
}

fn reading_and_printing(mut reader: BufReader<TcpStream>) -> anyhow::Result<()> {
    loop {
        let _message = read_response(&mut reader)?;
    }
}
