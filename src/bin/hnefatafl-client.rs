use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc,
    thread,
    time::Duration,
};

use hnefatafl_copenhagen::{game::Game, message, play::Vertex, space::Space};
use iced::{
    futures::{channel::mpsc::Sender, SinkExt, Stream},
    stream,
    widget::{button, row, text, text_input, Column, Row},
    Subscription,
};

const ADDRESS: &str = "localhost:8000";

fn main() -> anyhow::Result<()> {
    iced::application("Hnefatafl Copenhagen", Client::update, Client::view)
        .subscription(Client::pass_messages)
        .run()?;

    Ok(())
}

#[derive(Debug, Default)]
struct Client {
    game: Game,
    tx: Option<mpsc::Sender<String>>,
    texts: Vec<String>,
    text_input: String,
}

impl Client {
    fn pass_messages(_self: &Self) -> Subscription<Message> {
        Subscription::run(pass_messages)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::_Game(message) => {
                let _result = self.game.update(message);
            }
            Message::TcpConnected(tx) => {
                println!("TCP connected...");
                self.tx = Some(tx);
            }
            Message::TextChanged(string) => {
                self.text_input = string;
            }
            Message::TextReceived(string) => {
                print!("-> {string}, we made it!");
            }
            Message::TextSend => {
                if let Some(tx) = &self.tx {
                    self.text_input.push('\n');
                    tx.send(self.text_input.clone()).unwrap();
                }
                self.text_input.clear();
            }
        }
    }

    #[must_use]
    pub fn view(&self) -> Row<Message> {
        let mut row = Row::new();

        for y in 0..11 {
            let mut column = Column::new();
            for x in 0..11 {
                let button = match self.game.board.get(&Vertex { x, y }) {
                    Space::Empty => button(text("  ")),
                    Space::Black => button(text("X")),
                    Space::King => button(text("K")),
                    Space::White => button(text("O")),
                };
                column = column.push(button);
            }
            row = row.push(column);
        }

        let mut column = Column::new();
        column = column.push("Texts:");
        column = column.push(
            text_input("", &self.text_input)
                .on_input(Message::TextChanged)
                .on_submit(Message::TextSend),
        );

        for message in &self.texts {
            column = column.push(text(message));
        }

        row![row, column]
    }
}

#[derive(Clone, Debug)]
enum Message {
    _Game(message::Message),
    TcpConnected(mpsc::Sender<String>),
    TextChanged(String),
    TextReceived(String),
    TextSend,
}

fn pass_messages() -> impl Stream<Item = Message> {
    stream::channel(100, |mut sender| async move {
        let mut tcp_stream = TcpStream::connect(ADDRESS).unwrap();
        let mut reader = BufReader::new(tcp_stream.try_clone().unwrap());
        println!("connected to {ADDRESS} ...");

        let (tx, rx) = mpsc::channel();
        let _ = sender.send(Message::TcpConnected(tx)).await;
        thread::spawn(move || loop {
            let message = rx.recv().unwrap();
            print!("<- {}", &message);
            tcp_stream.write_all(message.as_bytes()).unwrap();
        });

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            reader.read_line(&mut buffer).unwrap();
            print!("-> {buffer}");
            if let Err(error) = tx.send(buffer) {
                println!("{error}");
                return;
            }
        });

        thread::spawn(move || send_message(rx, sender));
    })
}

async fn send_message(rx: mpsc::Receiver<String>, mut sender: Sender<Message>) {
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(message) => {
                let _ = sender.send(Message::TextReceived(message)).await;
            }
            Err(error) => {
                println!("{error}");
            }
        }
    }
}
