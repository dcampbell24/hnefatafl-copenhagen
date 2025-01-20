use std::{io::Write, net::TcpStream, thread::sleep, time::Duration};

fn main() -> anyhow::Result<()> {
    let address = "localhost:8000".to_string();
    let mut stream = TcpStream::connect(&address)?;
    println!("connected to {address} ...");

    let username = "foobar";
    stream.write_all(username.as_bytes())?;
    stream.write_all("\n".as_bytes())?;

    sleep(Duration::from_secs(10));
    // let mut game = Game::default();

    Ok(())
}
