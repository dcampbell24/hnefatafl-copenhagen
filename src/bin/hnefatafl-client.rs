use std::{io::Write, net::TcpStream, thread::sleep, time::Duration};

fn main() -> anyhow::Result<()> {
    let address = "localhost:8000".to_string();
    let mut stream = TcpStream::connect(&address)?;
    println!("connected to {address} ...");

    let username = b"foobar\n";
    stream.write_all(username)?;

    sleep(Duration::from_secs(10));
    // let mut game = Game::default();

    Ok(())
}
