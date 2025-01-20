use std::{io::BufReader, net::TcpStream, thread::sleep, time::Duration};

use hnefatafl_copenhagen::{read_response, write_command};

fn main() -> anyhow::Result<()> {
    let address = "localhost:8000".to_string();
    let mut stream = TcpStream::connect(&address)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    println!("connected to {address} ...");

    let username = "foobar\n";
    write_command(username, &mut stream)?;
    let _message = read_response(&mut reader)?;

    sleep(Duration::from_secs(10));
    // let mut game = Game::default();

    Ok(())
}
