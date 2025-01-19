use std::{io::Write, net::TcpStream, thread::sleep, time::Duration};

fn main() -> anyhow::Result<()> {
    let address = "localhost:8000".to_string();
    let mut stream = TcpStream::connect(&address)?;
    println!("connected to {address} ...");

    stream.write_all(b"foobar\n")?;
    sleep(Duration::from_secs(10));
    // let mut game = Game::default();

    Ok(())
}
