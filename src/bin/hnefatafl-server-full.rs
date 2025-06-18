#![deny(clippy::indexing_slicing)]

use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Local, Utc};
use clap::{Parser, command};
use env_logger::Builder;
use hnefatafl_copenhagen::{
    VERSION_ID,
    accounts::{Account, Accounts, Email},
    draw::Draw,
    game::TimeUnix,
    glicko::Outcome,
    handle_error,
    rating::Rated,
    role::Role,
    server_game::{
        ArchivedGame, Challenger, ServerGame, ServerGameLight, ServerGames, ServerGamesLight,
    },
    smtp::Smtp,
    status::Status,
    time::TimeSettings,
};
use lettre::{
    SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use log::{LevelFilter, debug, error, info};
use password_hash::SaltString;
use rand::{random, rngs::OsRng};
use serde::{Deserialize, Serialize};

const PORT: &str = ":49152";

/// A Hnefatafl Copenhagen Server
///
/// This is a TCP server that listens client connections.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Whether to skip advertising updates.
    #[arg(long)]
    skip_advertising_updates: bool,

    /// Whether to use the data file.
    #[arg(long)]
    skip_the_data_file: bool,

    /// Listen for HTP drivers on host.
    #[arg(default_value = "0.0.0.0", long)]
    host: String,

    /// Whether the application is being run by systemd.
    #[arg(long)]
    systemd: bool,
}

fn main() -> anyhow::Result<()> {
    // println!("{:x}", rand::random::<u32>());
    // return Ok(());

    let mut args = Args::parse();
    init_logger(args.systemd);

    let mut server = Server::default();

    if !args.skip_the_data_file {
        let data_file = data_file();
        match &fs::read_to_string(&data_file) {
            Ok(string) => match ron::from_str(string.as_str()) {
                Ok(server_ron) => server = server_ron,
                Err(err) => return Err(anyhow::Error::msg(err.to_string())),
            },
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {}
                _ => return Err(anyhow::Error::msg(err.to_string())),
            },
        }

        match fs::read_to_string(archived_games_file()) {
            Ok(archived_games_string) => {
                let mut archived_games = Vec::new();

                for line in archived_games_string.lines() {
                    let archived_game: ArchivedGame = ron::from_str(line)?;
                    archived_games.push(archived_game);
                }

                server.archived_games = archived_games;
            }
            Err(err) => {
                error!("archived games file not found: {err}");
            }
        }
    }

    if args.skip_the_data_file {
        server.skip_the_data_file = true;
    }

    args.host.push_str(PORT);
    let address = args.host;
    let listener = TcpListener::bind(&address)?;
    info!("listening on {address} ...");

    let (tx, rx) = mpsc::channel();
    server.tx = Some(tx.clone());

    thread::spawn(move || server.handle_messages(&rx));

    if !args.skip_advertising_updates {
        let tx_messages_1 = tx.clone();
        thread::spawn(move || {
            loop {
                handle_error(tx_messages_1.send(("0 server display_server".to_string(), None)));
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    let tx_messages_2 = tx.clone();
    thread::spawn(move || {
        loop {
            handle_error(tx_messages_2.send(("0 server check_update_rd".to_string(), None)));
            thread::sleep(Duration::from_secs(60 * 60 * 24));
        }
    });

    for (index, stream) in (1..).zip(listener.incoming()) {
        let stream = stream?;
        let tx = tx.clone();
        thread::spawn(move || login(index, stream, &tx));
    }

    Ok(())
}

fn archived_games_file() -> PathBuf {
    let mut archived_games_file = if let Some(data_file) = dirs::data_dir() {
        data_file
    } else {
        PathBuf::new()
    };

    archived_games_file.push("hnefatafl-games.ron");
    archived_games_file
}

fn data_file() -> PathBuf {
    let mut data_file = if let Some(data_file) = dirs::data_dir() {
        data_file
    } else {
        PathBuf::new()
    };
    data_file.push("hnefatafl-copenhagen.ron");
    data_file
}

#[allow(clippy::too_many_lines)]
fn login(
    index: usize,
    mut stream: TcpStream,
    tx: &mpsc::Sender<(String, Option<mpsc::Sender<String>>)>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buf = String::new();
    let (client_tx, client_rx) = mpsc::channel();
    let mut username_proper = "_".to_string();
    let mut login_successful = false;

    for _ in 0..100 {
        reader.read_line(&mut buf)?;

        for ch in buf.trim().chars() {
            if ch.is_control() || ch == '\0' {
                return Err(anyhow::Error::msg(
                    "there are control characters in the username or password",
                ));
            }
        }

        if buf.trim().is_empty() {
            return Err(anyhow::Error::msg(
                "the user tried to login with defender space alone",
            ));
        }

        let buf_clone = buf.clone();
        let mut username_password_etc = buf_clone.split_ascii_whitespace();

        let version_id = username_password_etc.next();
        let create_account_login = username_password_etc.next();
        let username_option = username_password_etc.next();

        if let (Some(version_id), Some(create_account_login), Some(username)) =
            (version_id, create_account_login, username_option)
        {
            username_proper = username.to_string();
            if version_id != VERSION_ID {
                stream.write_all(
                    b"? login wrong version, update your hnefatafl-copenhagen package\n",
                )?;
                buf.clear();
                continue;
            }

            let password: Vec<&str> = username_password_etc.collect();
            let password = password.join(" ");

            if username.len() > 16 {
                stream.write_all(b"? login username is more than 16 characters\n")?;
                buf.clear();
                continue;
            }
            if password.len() > 32 {
                stream.write_all(b"? login password is more than 32 characters\n")?;
                buf.clear();
                continue;
            }

            debug!("{index} {username} {create_account_login} {password}");

            if create_account_login == "reset_password" {
                tx.send((
                    format!("0 {username} {create_account_login}"),
                    Some(client_tx.clone()),
                ))?;

                stream.write_all(
                    b"? login sent a password reset email if a verified email exists for this account\n",
                )?;
                buf.clear();
                continue;
            }

            tx.send((
                format!("{index} {username} {create_account_login} {password}"),
                Some(client_tx.clone()),
            ))?;

            let message = client_rx.recv()?;
            buf.clear();
            if create_account_login == "login" {
                if "= login" == message.as_str() {
                    login_successful = true;
                    break;
                }

                stream.write_all(b"? login password is wrong (try lowercase), account doesn't exist, or your already logged in\n")?;
                continue;
            } else if create_account_login == "create_account" {
                if "= create_account" == message.as_str() {
                    login_successful = true;
                    break;
                }

                stream.write_all(b"? create_account account already exists\n")?;
                continue;
            }

            stream.write_all(b"? login\n")?;
        }

        buf.clear();
    }

    if !login_successful {
        return Err(anyhow::Error::msg("the user failed to login"));
    }

    stream.write_all(b"= login\n")?;
    thread::spawn(move || receiving_and_writing(stream, &client_rx));

    tx.send((format!("{index} {username_proper} email_get"), None))?;

    'outer: for _ in 0..1_000_000 {
        if let Err(err) = reader.read_line(&mut buf) {
            error!("{err}");
            break 'outer;
        }

        let buf_str = buf.trim();

        if buf_str.is_empty() {
            break 'outer;
        }

        for char in buf_str.chars() {
            if char.is_control() || char == '\0' {
                break 'outer;
            }
        }

        tx.send((format!("{index} {username_proper} {buf_str}"), None))?;
        buf.clear();
    }

    tx.send((format!("{index} {username_proper} logout"), None))?;
    Ok(())
}

fn receiving_and_writing(
    mut stream: TcpStream,
    client_rx: &Receiver<String>,
) -> anyhow::Result<()> {
    loop {
        let message = client_rx.recv()?;
        stream.write_all(format!("{message}\n").as_bytes())?;
    }
}

/// Non-leap seconds since January 1, 1970 0:00:00 UTC.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnixTimestamp(pub i64);

impl Default for UnixTimestamp {
    fn default() -> Self {
        Self(Local::now().to_utc().timestamp())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Server {
    #[serde(default)]
    game_id: usize,
    #[serde(default)]
    ran_update_rd: UnixTimestamp,
    #[serde(default)]
    smtp: Smtp,
    #[serde(default)]
    accounts: Accounts,
    #[serde(skip)]
    archived_games: Vec<ArchivedGame>,
    #[serde(skip)]
    clients: HashMap<usize, mpsc::Sender<String>>,
    #[serde(skip)]
    games: ServerGames,
    #[serde(skip)]
    games_light: ServerGamesLight,
    #[serde(skip)]
    skip_the_data_file: bool,
    #[serde(skip)]
    tx: Option<mpsc::Sender<(String, Option<mpsc::Sender<String>>)>>,
}

impl Server {
    fn append_archived_game(&mut self, game: ServerGame) -> anyhow::Result<()> {
        let Some(attacker) = self.accounts.0.get(&game.attacker) else {
            return Err(anyhow::Error::msg("failed to get rating!"));
        };
        let Some(defender) = self.accounts.0.get(&game.defender) else {
            return Err(anyhow::Error::msg("failed to get rating!"));
        };
        let game = ArchivedGame::new(game, attacker.rating.clone(), defender.rating.clone());

        let archived_games_file = archived_games_file();
        let game_string = ron::ser::to_string(&game)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(archived_games_file)?;

        file.write_all(game_string.as_bytes())?;
        file.write_all("\n".as_bytes())?;

        self.archived_games.push(game);

        Ok(())
    }

    fn bcc_mailboxes(&self, username: &str) -> Vec<Mailbox> {
        let mut emails = Vec::new();

        if let Some(account) = self.accounts.0.get(username) {
            if account.send_emails {
                for account in self.accounts.0.values() {
                    if let Some(email) = &account.email {
                        if email.verified {
                            if let Some(email) = email.to_mailbox() {
                                emails.push(email);
                            }
                        }
                    }
                }
            }
        }

        emails
    }

    fn bcc_send(&self, username: &str) -> String {
        let mut emails = Vec::new();

        if let Some(account) = self.accounts.0.get(username) {
            if account.send_emails {
                for account in self.accounts.0.values() {
                    if let Some(email) = &account.email {
                        if email.verified {
                            emails.push(email.tx());
                        }
                    }
                }
            }
        }

        emails.sort();
        emails.join(" ")
    }

    /// ```sh
    /// # PASSWORD can be the empty string.
    /// <- change_password PASSWORD
    /// -> = change_password
    /// ```
    fn change_password(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        info!("{index_supplied} {username} change_password");

        let account = self.accounts.0.get_mut(username)?;
        let password = the_rest.join(" ");

        if password.len() > 32 {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                format!("{command} password is greater than 32 characters"),
            ));
        }

        let hash = hash_password(&password)?;
        account.password = hash;
        self.save_server();

        Some((
            self.clients.get(&index_supplied)?.clone(),
            true,
            (*command).to_string(),
        ))
    }

    /// ```sh
    /// # server internal
    /// ```
    ///
    /// c = 63.2
    ///
    /// This assumes 30 2 month periods must pass before one's rating
    /// deviation is the same as a new player and that a typical RD is 50.
    #[must_use]
    fn check_update_rd(&mut self) -> bool {
        // Seconds in two months:
        // 60.0 * 60.0 * 24.0 * 30.417 * 2.0 = 5_256_057.6
        let two_months = 5_256_058;

        let now = Local::now().to_utc().timestamp();
        if now - self.ran_update_rd.0 >= two_months {
            for account in self.accounts.0.values_mut() {
                account.rating.update_rd();
            }
            self.ran_update_rd = UnixTimestamp(now);
            true
        } else {
            false
        }
    }

    /// ```sh
    /// # PASSWORD can be the empty string.
    /// <- VERSION_ID create_account player-1 PASSWORD
    /// -> = login
    /// ```
    fn create_account(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
        option_tx: Option<Sender<String>>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let password = the_rest.join(" ");
        let tx = option_tx?;
        if self.accounts.0.contains_key(username) {
            info!("{index_supplied} {username} is already in the database");
            Some((tx, false, (*command).to_string()))
        } else {
            info!("{index_supplied} {username} created user account");

            let hash = hash_password(&password)?;
            self.clients.insert(index_supplied, tx);
            self.accounts.0.insert(
                (*username).to_string(),
                Account {
                    password: hash,
                    logged_in: Some(index_supplied),
                    ..Default::default()
                },
            );

            self.save_server();

            Some((
                self.clients.get(&index_supplied)?.clone(),
                true,
                (*command).to_string(),
            ))
        }
    }

    fn decline_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        mut command: String,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let channel = self.clients.get(&index_supplied)?;

        let Some(id) = the_rest.first() else {
            return Some((channel.clone(), false, command));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((channel.clone(), false, command));
        };

        info!("{index_supplied} {username} decline_game {id}");

        if let Some(game_old) = self.games_light.0.remove(&id) {
            let mut attacker = None;
            let mut attacker_channel = None;
            let mut defender = None;
            let mut defender_channel = None;

            if Some(username.to_string()) == game_old.attacker {
                attacker = game_old.attacker;
                attacker_channel = game_old.attacker_channel;
            } else {
                defender = game_old.defender;
                defender_channel = game_old.defender_channel;
            }

            let game = ServerGameLight {
                id,
                attacker,
                defender,
                challenger: Challenger::default(),
                rated: game_old.rated,
                timed: game_old.timed,
                attacker_channel,
                defender_channel,
                spectators: game_old.spectators,
                challenge_accepted: false,
                game_over: false,
            };

            command = format!("{command} {game:?}");
            self.games_light.0.insert(id, game);
        }

        Some((channel.clone(), true, command))
    }

    fn delete_account(&mut self, username: &str, index_supplied: usize) {
        info!("{index_supplied} {username} delete_account");

        self.accounts.0.remove(username);
        self.save_server();
    }

    fn display_server(&mut self, username: &str) -> Option<(mpsc::Sender<String>, bool, String)> {
        debug!("0 {username} display_server");
        for tx in &mut self.clients.values() {
            tx.send(format!("= display_games {:?}", &self.games_light))
                .ok()?;

            tx.send(format!("= display_users {}", &self.accounts))
                .ok()?;
        }

        for game in self.games.0.values_mut() {
            match game.game.turn {
                Role::Attacker => {
                    if game.game.status == Status::Ongoing {
                        if let (TimeUnix::Time(game_time), TimeSettings::Timed(attacker_time)) =
                            (&mut game.game.time, &mut game.game.attacker_time)
                        {
                            if attacker_time.milliseconds_left > 0 {
                                let now = Local::now().to_utc().timestamp_millis();
                                attacker_time.milliseconds_left -= now - *game_time;
                                *game_time = now;
                            } else if let Some(tx) = &mut self.tx {
                                let _ok = tx.send((
                                    format!(
                                        "0 {} game {} play attacker resigns _",
                                        game.attacker, game.id
                                    ),
                                    None,
                                ));
                            }
                        }
                    }
                }
                Role::Roleless => {}
                Role::Defender => {
                    if game.game.status == Status::Ongoing {
                        if let (TimeUnix::Time(game_time), TimeSettings::Timed(defender_time)) =
                            (&mut game.game.time, &mut game.game.defender_time)
                        {
                            if defender_time.milliseconds_left > 0 {
                                let now = Local::now().to_utc().timestamp_millis();
                                defender_time.milliseconds_left -= now - *game_time;
                                *game_time = now;
                            } else if let Some(tx) = &mut self.tx {
                                let _ok = tx.send((
                                    format!(
                                        "0 {} game {} play defender resigns _",
                                        game.defender, game.id
                                    ),
                                    None,
                                ));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn draw(
        &mut self,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(draw) = the_rest.get(1) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(draw) = Draw::from_str(draw) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(mut game) = self.games.0.remove(&id) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let message = format!("= draw {draw}");
        let _ok = game.attacker_tx.send(message.clone());
        let _ok = game.defender_tx.send(message.clone());

        if draw == Draw::Accept {
            let Some(game_light) = self.games_light.0.get(&id) else {
                return Some((
                    self.clients.get(&index_supplied)?.clone(),
                    false,
                    (*command).to_string(),
                ));
            };
            for spectator in game_light.spectators.values() {
                if let Some(sender) = self.clients.get(spectator) {
                    let _ok = sender.send(message.clone());
                }
            }

            game.game.status = Status::Draw;

            let accounts = &mut self.accounts.0;
            let (attacker_rating, defender_rating) = if let (Some(attacker), Some(defender)) =
                (accounts.get(&game.attacker), accounts.get(&game.defender))
            {
                (attacker.rating.rating, defender.rating.rating)
            } else {
                panic!("the attacker and defender accounts should exist");
            };

            if let Some(attacker) = accounts.get_mut(&game.attacker) {
                attacker.draws += 1;

                if game.rated.into() {
                    attacker
                        .rating
                        .update_rating(defender_rating, &Outcome::Draw);
                }
            }
            if let Some(defender) = accounts.get_mut(&game.defender) {
                defender.draws += 1;

                if game.rated.into() {
                    defender
                        .rating
                        .update_rating(attacker_rating, &Outcome::Draw);
                }
            }

            if let Some(game) = self.games_light.0.get_mut(&id) {
                game.game_over = true;
            }

            if !self.skip_the_data_file {
                self.append_archived_game(game)
                    .map_err(|err| {
                        error!("{err}");
                    })
                    .ok()?;
            }

            self.save_server();
        }

        None
    }

    #[allow(clippy::too_many_lines)]
    fn game(
        &mut self,
        index_supplied: usize,
        username: &str,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        if the_rest.len() < 5 {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        }

        let index = the_rest.first()?;
        let Ok(index) = index.parse() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let role = the_rest.get(2)?;
        let Ok(role) = Role::from_str(role) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let from = the_rest.get(3)?;
        let to = the_rest.get(4)?;
        let mut to = (*to).to_string();
        if to == "_" {
            to = String::new();
        }

        let Some(game) = self.games.0.get_mut(&index) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(game_light) = self.games_light.0.get_mut(&index) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let mut attackers_turn_next = true;
        if role == Role::Attacker {
            if *username == game.attacker {
                game.game
                    .read_line(&format!("play attacker {from} {to}"))
                    .map_err(|error| {
                        error!("{error}");
                        error
                    })
                    .ok()?;
                attackers_turn_next = false;

                let message = format!("game {index} play attacker {from} {to}");
                for spectator in game_light.spectators.values() {
                    if let Some(client) = self.clients.get(spectator) {
                        let _ok = client.send(message.clone());
                    }
                }
                let _ok = game.defender_tx.send(message);
            } else {
                return Some((
                    self.clients.get(&index_supplied)?.clone(),
                    false,
                    (*command).to_string(),
                ));
            }
        } else if *username == game.defender {
            game.game
                .read_line(&format!("play defender {from} {to}"))
                .map_err(|error| {
                    error!("{error}");
                    error
                })
                .ok()?;

            let message = format!("game {index} play defender {from} {to}");
            for spectator in game_light.spectators.values() {
                if let Some(client) = self.clients.get(spectator) {
                    let _ok = client.send(message.clone());
                }
            }
            let _ok = game.attacker_tx.send(message);
        } else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        }

        match game.game.status {
            Status::AttackerWins => {
                let accounts = &mut self.accounts.0;
                let (attacker_rating, defender_rating) = if let (Some(attacker), Some(defender)) =
                    (accounts.get(&game.attacker), accounts.get(&game.defender))
                {
                    (attacker.rating.rating, defender.rating.rating)
                } else {
                    panic!("the attacker and defender accounts should exist");
                };

                if let Some(attacker) = accounts.get_mut(&game.attacker) {
                    attacker.wins += 1;

                    if game.rated.into() {
                        attacker
                            .rating
                            .update_rating(defender_rating, &Outcome::Win);
                    }
                }
                if let Some(defender) = accounts.get_mut(&game.defender) {
                    defender.losses += 1;

                    if game.rated.into() {
                        defender
                            .rating
                            .update_rating(attacker_rating, &Outcome::Loss);
                    }
                }

                let message = format!("= game_over {index} attacker_wins");
                let _ok = game.attacker_tx.send(message.clone());
                let _ok = game.defender_tx.send(message.clone());

                for spectator in game_light.spectators.values() {
                    if let Some(sender) = self.clients.get(spectator) {
                        let _ok = sender.send(message.clone());
                    }
                }

                let Some(game) = self.games.0.remove(&index) else {
                    panic!("the game should exist")
                };

                if let Some(game) = self.games_light.0.get_mut(&index) {
                    game.game_over = true;
                }

                if !self.skip_the_data_file {
                    self.append_archived_game(game)
                        .map_err(|err| {
                            error!("{err}");
                        })
                        .ok()?;
                }

                self.save_server();

                return None;
            }
            Status::Draw => {
                // Handled in the draw fn.
            }
            Status::Ongoing => {
                if attackers_turn_next {
                    let _ok = game
                        .attacker_tx
                        .send(format!("game {index} generate_move attacker"));
                } else {
                    let _ok = game
                        .defender_tx
                        .send(format!("game {index} generate_move defender"));
                }
            }
            Status::DefenderWins => {
                let accounts = &mut self.accounts.0;
                let (attacker_rating, defender_rating) = if let (Some(attacker), Some(defender)) =
                    (accounts.get(&game.attacker), accounts.get(&game.defender))
                {
                    (attacker.rating.rating, defender.rating.rating)
                } else {
                    panic!("the attacker and defender accounts should exist");
                };

                if let Some(attacker) = accounts.get_mut(&game.attacker) {
                    attacker.losses += 1;

                    if game.rated.into() {
                        attacker
                            .rating
                            .update_rating(defender_rating, &Outcome::Loss);
                    }
                }
                if let Some(defender) = accounts.get_mut(&game.defender) {
                    defender.wins += 1;

                    if game.rated.into() {
                        defender
                            .rating
                            .update_rating(attacker_rating, &Outcome::Win);
                    }
                }

                let message = format!("= game_over {index} defender_wins");
                let _ok = game.attacker_tx.send(message.clone());
                let _ok = game.defender_tx.send(message.clone());

                for spectator in game_light.spectators.values() {
                    if let Some(sender) = self.clients.get(spectator) {
                        let _ok = sender.send(message.clone());
                    }
                }

                let Some(game) = self.games.0.remove(&index) else {
                    panic!("the game should exist")
                };

                if let Some(game) = self.games_light.0.get_mut(&index) {
                    game.game_over = true;
                }

                if !self.skip_the_data_file {
                    self.append_archived_game(game)
                        .map_err(|err| {
                            error!("{err}");
                        })
                        .ok()?;
                }

                self.save_server();

                return None;
            }
        }

        Some((
            self.clients.get(&index_supplied)?.clone(),
            true,
            (*command).to_string(),
        ))
    }

    fn set_email(
        &mut self,
        index_supplied: usize,
        username: &str,
        command: &str,
        email: Option<&str>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(address) = email else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(account) = self.accounts.0.get_mut(username) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let random_u32 = random::<u32>();
        let email = Email {
            address: address.to_string(),
            code: Some(random_u32),
            username: username.to_string(),
            verified: false,
        };

        info!("{index_supplied} {username} email {}", email.tx());

        let email_send = lettre::Message::builder()
            .from("Hnefatafl Org <no-reply@hnefatafl.org>".parse().ok()?)
            .to(email.to_mailbox()?)
            .subject("Account Verification")
            .header(ContentType::TEXT_PLAIN)
            .body(format!(
                "Dear {username},\nyour email verification code is as follows: {random_u32:x}",
            ))
            .ok()?;

        let credentials = Credentials::new(self.smtp.username.clone(), self.smtp.password.clone());

        let mailer = SmtpTransport::relay(&self.smtp.service)
            .ok()?
            .credentials(credentials)
            .build();

        match mailer.send(&email_send) {
            Ok(_) => {
                info!("email sent to {address} successfully!");

                account.email = Some(email);
                self.save_server();

                let reply = format!("email {address} false");
                Some((self.clients.get(&index_supplied)?.clone(), true, reply))
            }
            Err(err) => {
                let reply = format!("could not send email to {address}");
                error!("{reply}: {err}");

                Some((self.clients.get(&index_supplied)?.clone(), false, reply))
            }
        }
    }

    fn handle_messages(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> anyhow::Result<()> {
        loop {
            if let Some((tx, ok, command)) = self.handle_messages_internal(rx) {
                if ok {
                    tx.send(format!("= {command}"))?;
                } else {
                    tx.send(format!("? {command}"))?;
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn handle_messages_internal(
        &mut self,
        rx: &mpsc::Receiver<(String, Option<mpsc::Sender<String>>)>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let (message, option_tx) = rx.recv().ok()?;
        let index_username_command: Vec<_> = message.split_ascii_whitespace().collect();

        if let (Some(index_supplied), Some(username), Some(command)) = (
            index_username_command.first(),
            index_username_command.get(1),
            index_username_command.get(2),
        ) {
            let index_supplied = index_supplied.parse::<usize>().ok()?;
            let the_rest: Vec<_> = index_username_command.clone().into_iter().skip(3).collect();

            match *command {
                "change_password" => {
                    self.change_password(username, index_supplied, command, the_rest.as_slice())
                }
                "check_update_rd" => {
                    let bool = self.check_update_rd();
                    info!("0 {username} check_update_rd {bool}");
                    None
                }
                "create_account" => self.create_account(
                    username,
                    index_supplied,
                    command,
                    the_rest.as_slice(),
                    option_tx,
                ),
                "decline_game" => self.decline_game(
                    username,
                    index_supplied,
                    (*command).to_string(),
                    the_rest.as_slice(),
                ),
                "delete_account" => {
                    self.delete_account(username, index_supplied);
                    None
                }
                "display_server" => self.display_server(username),
                "draw" => self.draw(index_supplied, command, the_rest.as_slice()),
                "game" => self.game(index_supplied, username, command, the_rest.as_slice()),
                "email" => {
                    self.set_email(index_supplied, username, command, the_rest.first().copied())
                }
                "email_everyone" => {
                    let emails_bcc = self.bcc_mailboxes(username);

                    let subject = the_rest.first()?;
                    let email_string = the_rest.get(1..)?.join(" ").replace("\\n", "\n");

                    info!("{index_supplied} {username} email_everyone");

                    let mut email = lettre::Message::builder();

                    for email_bcc in emails_bcc {
                        email = email.bcc(email_bcc);
                    }

                    let email = email
                        .from("Hnefatafl Org <no-reply@hnefatafl.org>".parse().ok()?)
                        .subject(*subject)
                        .header(ContentType::TEXT_PLAIN)
                        .body(email_string)
                        .ok()?;

                    let credentials =
                        Credentials::new(self.smtp.username.clone(), self.smtp.password.clone());

                    let mailer = SmtpTransport::relay(&self.smtp.service)
                        .ok()?
                        .credentials(credentials)
                        .build();

                    match mailer.send(&email) {
                        Ok(_) => {
                            info!("emails sent successfully!");

                            Some((
                                self.clients.get(&index_supplied)?.clone(),
                                true,
                                (*command).to_string(),
                            ))
                        }
                        Err(err) => {
                            let reply = "could not send emails";
                            error!("{reply}: {err}");

                            Some((
                                self.clients.get(&index_supplied)?.clone(),
                                false,
                                reply.to_string(),
                            ))
                        }
                    }
                }
                "emails_bcc" => {
                    let emails_bcc = self.bcc_send(username);

                    if !emails_bcc.is_empty() {
                        self.clients
                            .get(&index_supplied)?
                            .send(format!("= emails_bcc {emails_bcc}"))
                            .ok()?;
                    }

                    None
                }
                "email_code" => {
                    if let Some(account) = self.accounts.0.get_mut(*username) {
                        if let Some(email) = &mut account.email {
                            if let (Some(code_1), Some(code_2)) = (email.code, the_rest.first()) {
                                if format!("{code_1:x}") == *code_2 {
                                    email.verified = true;

                                    self.clients
                                        .get(&index_supplied)?
                                        .send("= email_code".to_string())
                                        .ok()?;
                                } else {
                                    email.verified = false;

                                    self.clients
                                        .get(&index_supplied)?
                                        .send("? email_code".to_string())
                                        .ok()?;
                                }

                                self.save_server();
                            }
                        }
                    }

                    None
                }
                "email_get" => {
                    if let Some(account) = self.accounts.0.get(*username) {
                        if let Some(email) = &account.email {
                            self.clients
                                .get(&index_supplied)?
                                .send(format!("= email {} {}", email.address, email.verified))
                                .ok()?;
                        }
                    }

                    None
                }
                "email_reset" => {
                    if let Some(account) = self.accounts.0.get_mut(*username) {
                        account.email = None;
                        self.save_server();

                        Some((
                            self.clients.get(&index_supplied)?.clone(),
                            true,
                            (*command).to_string(),
                        ))
                    } else {
                        None
                    }
                }
                "join_game" => self.join_game(
                    username,
                    index_supplied,
                    (*command).to_string(),
                    the_rest.as_slice(),
                ),
                "join_game_pending" => self.join_game_pending(
                    (*username).to_string(),
                    index_supplied,
                    (*command).to_string(),
                    the_rest.as_slice(),
                ),
                "leave_game" => self.leave_game(
                    username,
                    index_supplied,
                    (*command).to_string(),
                    the_rest.as_slice(),
                ),
                "login" => self.login(
                    username,
                    index_supplied,
                    command,
                    the_rest.as_slice(),
                    option_tx,
                ),
                "logout" => self.logout(username, index_supplied, command),
                "new_game" => self.new_game(username, index_supplied, command, the_rest.as_slice()),
                "reset_password" => {
                    let account = self.accounts.0.get_mut(*username)?;
                    if let Some(email) = &account.email {
                        if email.verified {
                            let password = format!("{:x}", random::<u32>());
                            account.password = hash_password(&password)?;

                            let message = lettre::Message::builder()
                                .from("Hnefatafl Org <no-reply@hnefatafl.org>".parse().ok()?)
                                .to(email.to_mailbox()?)
                                .subject("Password Reset")
                                .header(ContentType::TEXT_PLAIN)
                                .body(format!(
                                    "Dear {username},\nyour new password is as follows: {password}",
                                ))
                                .ok()?;

                            let credentials = Credentials::new(
                                self.smtp.username.clone(),
                                self.smtp.password.clone(),
                            );

                            let mailer = SmtpTransport::relay(&self.smtp.service)
                                .ok()?
                                .credentials(credentials)
                                .build();

                            match mailer.send(&message) {
                                Ok(_) => {
                                    info!("email sent to {} successfully!", email.address);
                                    self.save_server();
                                }
                                Err(err) => {
                                    error!("could not send email to {}: {err}", email.address);
                                }
                            }
                        } else {
                            error!("the email address for account {username} is unverified");
                        }
                    } else {
                        error!("no email exists for account {username}");
                    }

                    None
                }
                "resume_game" => self.resume_game(username, index_supplied, command, &the_rest),
                "request_draw" => self.request_draw(username, index_supplied, command, &the_rest),
                "text" => {
                    let timestamp = timestamp();
                    let the_rest = the_rest.join(" ");
                    info!("{index_supplied} {timestamp} {username} text {the_rest}");

                    for tx in &mut self.clients.values() {
                        let _ok = tx.send(format!("= text {timestamp} {username}: {the_rest}"));
                    }

                    None
                }
                "text_game" => self.text_game(username, index_supplied, command, the_rest),
                "watch_game" => self.watch_game(
                    username,
                    index_supplied,
                    (*command).to_string(),
                    the_rest.as_slice(),
                ),
                "=" => None,
                _ => self.clients.get(&index_supplied).map(|channel| {
                    info!("{index_supplied} {username} {command}");
                    (channel.clone(), false, (*command).to_string())
                }),
            }
        } else {
            info!("{index_username_command:?}");
            None
        }
    }

    fn join_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: String,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };

        info!("{index_supplied} {username} join_game {id}");
        let Some(game) = self.games_light.0.get_mut(&id) else {
            panic!("the id must refer to a valid pending game");
        };
        game.challenge_accepted = true;

        let (Some(attacker_tx), Some(defender_tx)) = (game.attacker_channel, game.defender_channel)
        else {
            panic!("the attacker and defender channels must be set")
        };

        for tx in [&attacker_tx, &defender_tx] {
            self.clients
                .get(tx)?
                .send(format!(
                    "= join_game {} {} {} {:?}",
                    game.attacker.clone()?,
                    game.defender.clone()?,
                    game.rated,
                    game.timed,
                ))
                .ok()?;
        }
        let new_game = ServerGame::new(
            self.clients.get(&attacker_tx)?.clone(),
            self.clients.get(&defender_tx)?.clone(),
            game.clone(),
        );

        self.games.0.insert(id, new_game);
        self.clients
            .get(&attacker_tx)?
            .send(format!("game {id} generate_move attacker"))
            .ok()?;

        None
    }

    fn join_game_pending(
        &mut self,
        username: String,
        index_supplied: usize,
        mut command: String,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let channel = self.clients.get(&index_supplied)?;

        let Some(id) = the_rest.first() else {
            return Some((channel.clone(), false, command));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((channel.clone(), false, command));
        };

        info!("{index_supplied} {username} join_game_pending {id}");
        let Some(game) = self.games_light.0.get_mut(&id) else {
            command.push_str(" the id doesn't refer to a pending game");
            return Some((channel.clone(), false, command));
        };

        if game.attacker.is_none() {
            game.attacker = Some(username.clone());
            game.attacker_channel = Some(index_supplied);

            if let Some(channel) = game.defender_channel {
                if let Some(channel) = self.clients.get(&channel) {
                    let _ok = channel.send(format!("= challenge_requested {id}"));
                }
            }
        } else if game.defender.is_none() {
            game.defender = Some(username.clone());
            game.defender_channel = Some(index_supplied);

            if let Some(channel) = game.attacker_channel {
                if let Some(channel) = self.clients.get(&channel) {
                    let _ok = channel.send(format!("= challenge_requested {id}"));
                }
            }
        }
        game.challenger.0 = Some(username);

        command.push(' ');
        command.push_str(the_rest.first()?);

        Some((channel.clone(), true, command))
    }

    fn leave_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        mut command: String,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };

        info!("{index_supplied} {username} leave_game {id}");

        let mut remove = false;
        match self.games_light.0.get_mut(&id) {
            Some(game) => {
                if let Some(attacker) = &game.attacker {
                    if username == attacker {
                        game.attacker = None;
                    }
                }
                if let Some(defender) = &game.defender {
                    if username == defender {
                        game.defender = None;
                    }
                }
                if let Some(challenger) = &game.challenger.0 {
                    if username == challenger {
                        game.challenger.0 = None;
                    }
                }

                game.spectators.remove(username);

                if game.attacker.is_none() && game.defender.is_none() {
                    remove = true;
                }
            }
            None => return Some((self.clients.get(&index_supplied)?.clone(), false, command)),
        }

        if remove {
            self.games_light.0.remove(&id);
        }

        command.push(' ');
        command.push_str(the_rest.first()?);
        Some((self.clients.get(&index_supplied)?.clone(), true, command))
    }

    fn login(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
        option_tx: Option<Sender<String>>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let password_1 = the_rest.join(" ");
        let tx = option_tx?;
        if let Some(account) = self.accounts.0.get_mut(username) {
            // The username is in the database and already logged in.
            if let Some(index_database) = account.logged_in {
                info!("{index_supplied} {username} login failed, {index_database} is logged in");

                Some(((tx), false, (*command).to_string()))
            // The username is in the database, but not logged in yet.
            } else {
                let hash_2 = PasswordHash::try_from(account.password.as_str()).ok()?;
                if let Err(_error) =
                    Argon2::default().verify_password(password_1.as_bytes(), &hash_2)
                {
                    info!("{index_supplied} {username} provided the wrong password");
                    return Some((tx, false, (*command).to_string()));
                }
                info!("{index_supplied} {username} logged in");

                self.clients.insert(index_supplied, tx);
                account.logged_in = Some(index_supplied);

                Some((
                    self.clients.get(&index_supplied)?.clone(),
                    true,
                    (*command).to_string(),
                ))
            }
        // The username is not in the database.
        } else {
            info!("{index_supplied} {username} is not in the database");
            Some((tx, false, (*command).to_string()))
        }
    }

    fn logout(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        // The username is in the database and already logged in.
        if let Some(account) = self.accounts.0.get_mut(username) {
            if let Some(index_database) = account.logged_in {
                if index_database == index_supplied {
                    info!("{index_supplied} {username} logged out");
                    account.logged_in = None;
                    self.clients.remove(&index_database);

                    return None;
                }
            }
        }

        self.clients
            .get(&index_supplied)
            .map(|sender| (sender.clone(), false, (*command).to_string()))
    }

    /// ```sh
    /// <- new_game attacker rated fischer 900000 10
    /// -> = new_game game 6 player-1 _ rated fischer 900000 10 _ false {}
    /// ```
    fn new_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        if the_rest.len() < 5 {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        }

        let role = the_rest.first()?;
        let Ok(role) = Role::from_str(role) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let rated = the_rest.get(1)?;
        let Ok(rated) = Rated::from_str(rated) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let timed = the_rest.get(2)?;
        let minutes = the_rest.get(3)?;
        let add_seconds = the_rest.get(4)?;

        let Ok(timed) = TimeSettings::try_from(vec!["time-settings", timed, minutes, add_seconds])
        else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        info!(
            "{index_supplied} {username} new_game {} {role} {rated} {timed:?}",
            self.game_id
        );
        let game = ServerGameLight::new(
            self.game_id,
            (*username).to_string(),
            rated,
            timed,
            index_supplied,
            role,
        );

        let command = format!("{command} {game:?}");
        self.games_light.0.insert(self.game_id, game);
        self.game_id += 1;

        Some((self.clients.get(&index_supplied)?.clone(), true, command))
    }

    fn resume_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(server_game) = self.games.0.get(&id) else {
            panic!("we must have a board at this point")
        };

        let game = &server_game.game;
        let Ok(board) = ron::ser::to_string(game) else {
            panic!("we should be able to serialize the board")
        };
        let texts = &server_game.texts;
        let Ok(texts) = ron::ser::to_string(&texts) else {
            panic!("we should be able to serialize the texts")
        };

        info!("{index_supplied} {username} watch_game {id}");
        let Some(game_light) = self.games_light.0.get_mut(&id) else {
            panic!("the id must refer to a valid pending game");
        };

        if Some((*username).to_string()) == game_light.attacker {
            if let Some(server_game) = self.games.0.get_mut(&id) {
                server_game.attacker_tx = self.clients.get(&index_supplied)?.clone();
            }
            game_light.attacker_channel = Some(index_supplied);
        } else if Some((*username).to_string()) == game_light.defender {
            if let Some(server_game) = self.games.0.get_mut(&id) {
                server_game.defender_tx = self.clients.get(&index_supplied)?.clone();
            }
            game_light.defender_channel = Some(index_supplied);
        }

        self.clients
            .get(&index_supplied)?
            .send(format!(
                "= resume_game {} {} {} {:?} {board} {texts}",
                game_light.attacker.clone()?,
                game_light.defender.clone()?,
                game_light.rated,
                game_light.timed,
            ))
            .ok()?;

        None
    }

    fn request_draw(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let Some(role) = the_rest.get(1) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(role) = Role::from_str(role) else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        info!("{index_supplied} {username} request_draw {id} {role}");

        let message = format!("request_draw {id} {role}");
        if let Some(game) = self.games.0.get(&id) {
            match role {
                Role::Attacker => {
                    let _ok = game.defender_tx.send(message);
                }
                Role::Roleless => {}
                Role::Defender => {
                    let _ok = game.attacker_tx.send(message);
                }
            }
        }

        Some((
            self.clients.get(&index_supplied)?.clone(),
            true,
            (*command).to_string(),
        ))
    }

    fn save_server(&self) {
        let mut server = self.clone();
        for account in server.accounts.0.values_mut() {
            account.logged_in = None;
        }

        if !self.skip_the_data_file {
            if let Ok(string) =
                ron::ser::to_string_pretty(&server, ron::ser::PrettyConfig::default())
            {
                if !string.trim().is_empty() {
                    let data_file = data_file();

                    if let Ok(mut file) = File::create(&data_file) {
                        if let Err(error) = file.write_all(string.as_bytes()) {
                            error!("{error}");
                        }
                    }
                }
            }
        }
    }

    fn text_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: &str,
        mut the_rest: Vec<&str>,
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((
                self.clients.get(&index_supplied)?.clone(),
                false,
                (*command).to_string(),
            ));
        };

        let timestamp = timestamp();
        let text = the_rest.split_off(1);
        let mut text = text.join(" ");
        text = format!("{timestamp} {username}: {text}");
        info!("{index_supplied} {username} text_game {id} {text}");

        if let Some(game) = self.games.0.get_mut(&id) {
            game.texts.push_front(text.clone());
        }

        text = format!("= text_game {text}");

        if let Some(game) = self.games_light.0.get(&id) {
            if let Some(attacker_channel) = game.attacker_channel {
                if let Some(sender) = self.clients.get(&attacker_channel) {
                    let _ok = sender.send(text.clone());
                }
            }

            if let Some(defender_channel) = game.defender_channel {
                if let Some(sender) = self.clients.get(&defender_channel) {
                    let _ok = sender.send(text.clone());
                }
            }

            for spectator in game.spectators.values() {
                if let Some(sender) = self.clients.get(spectator) {
                    let _ok = sender.send(text.clone());
                }
            }
        }

        None
    }

    fn watch_game(
        &mut self,
        username: &str,
        index_supplied: usize,
        command: String,
        the_rest: &[&str],
    ) -> Option<(mpsc::Sender<String>, bool, String)> {
        let Some(id) = the_rest.first() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };
        let Ok(id) = id.parse::<usize>() else {
            return Some((self.clients.get(&index_supplied)?.clone(), false, command));
        };

        if let Some(game) = self.games_light.0.get_mut(&id) {
            game.spectators.insert(username.to_string(), index_supplied);
        }

        let Some(server_game) = self.games.0.get(&id) else {
            panic!("we must have a board at this point")
        };

        let game = &server_game.game;
        let Ok(board) = ron::ser::to_string(game) else {
            panic!("we should be able to serialize the board")
        };
        let texts = &server_game.texts;
        let Ok(texts) = ron::ser::to_string(&texts) else {
            panic!("we should be able to serialize the texts")
        };

        info!("{index_supplied} {username} watch_game {id}");
        let Some(game) = self.games_light.0.get_mut(&id) else {
            panic!("the id must refer to a valid pending game");
        };

        self.clients
            .get(&index_supplied)?
            .send(format!(
                "= watch_game {} {} {} {:?} {board} {texts}",
                game.attacker.clone()?,
                game.defender.clone()?,
                game.rated,
                game.timed,
            ))
            .ok()?;

        None
    }
}

fn hash_password(password: &str) -> Option<String> {
    let ctx = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    Some(
        ctx.hash_password(password.as_bytes(), &salt)
            .ok()?
            .to_string(),
    )
}

fn init_logger(systemd: bool) {
    let mut builder = Builder::new();

    if systemd {
        builder.format(|formatter, record| {
            writeln!(formatter, "[{}]: {}", record.level(), record.args())
        });
    } else {
        builder.format(|formatter, record| {
            writeln!(
                formatter,
                "{} [{}] ({}): {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S %z"),
                record.level(),
                record.target(),
                record.args()
            )
        });
    }

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
    }

    builder.init();
}

fn timestamp() -> String {
    Local::now().to_utc().format("[%F %T UTC]").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::process::{Child, Stdio};
    use std::thread;
    use std::time::Duration;

    const ADDRESS: &str = "localhost:49152";

    struct Server(Child);

    impl Drop for Server {
        fn drop(&mut self) {
            self.0.kill().unwrap();
        }
    }

    #[test]
    fn capital_letters_fail() {
        let mut accounts = Accounts::default();

        let password = "A".to_string();
        let ctx = Argon2::default();

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = ctx
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let account = Account {
            password: password_hash,
            logged_in: Some(0),
            ..Default::default()
        };

        accounts.0.insert("testing".to_string(), account);
        {
            let account = accounts.0.get_mut("testing").unwrap();

            let salt = SaltString::generate(&mut OsRng);
            let password_hash = ctx
                .hash_password(password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            account.password = password_hash;
        }

        {
            let account = accounts.0.get_mut("testing").unwrap();
            let hash = PasswordHash::try_from(account.password.as_str()).unwrap();

            assert!(
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            );
        }
    }

    #[test]
    fn server_full() -> anyhow::Result<()> {
        std::process::Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg("hnefatafl-server-full")
            .arg("--features")
            .arg("server")
            .output()?;

        let _server = Server(
            std::process::Command::new("./target/debug/hnefatafl-server-full")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .arg("--skip-the-data-file")
                .arg("--skip-advertising-updates")
                .spawn()?,
        );

        thread::sleep(Duration::from_millis(10));

        let mut buf = String::new();

        let mut tcp_1 = TcpStream::connect(ADDRESS)?;
        let mut reader_1 = BufReader::new(tcp_1.try_clone()?);

        tcp_1.write_all(format!("{VERSION_ID} create_account player-1\n").as_bytes())?;
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
            "= new_game game 0 player-1 _ rated fischer 900000 10 _ false {}\n"
        );
        buf.clear();

        let mut tcp_2 = TcpStream::connect(ADDRESS)?;
        let mut reader_2 = BufReader::new(tcp_2.try_clone()?);

        tcp_2.write_all(format!("{VERSION_ID} create_account player-2\n").as_bytes())?;
        reader_2.read_line(&mut buf)?;
        assert_eq!(buf, "= login\n");
        buf.clear();

        tcp_2.write_all(b"join_game_pending 0\n")?;
        reader_2.read_line(&mut buf)?;
        assert_eq!(buf, "= join_game_pending 0\n");
        buf.clear();

        reader_1.read_line(&mut buf)?;
        assert_eq!(buf, "= challenge_requested 0\n");
        buf.clear();

        // Todo: "join_game_pending 0\n" should not be allowed!
        tcp_1.write_all(b"join_game 0\n")?;
        reader_1.read_line(&mut buf)?;
        assert_eq!(
            buf,
            "= join_game player-1 player-2 rated fischer 900000 10\n"
        );
        buf.clear();

        reader_2.read_line(&mut buf)?;
        assert_eq!(
            buf,
            "= join_game player-1 player-2 rated fischer 900000 10\n"
        );
        buf.clear();

        reader_1.read_line(&mut buf)?;
        assert_eq!(buf, "game 0 generate_move attacker\n");
        buf.clear();

        tcp_1.write_all(b"game 0 play attacker resigns _\n")?;
        reader_1.read_line(&mut buf)?;
        assert_eq!(buf, "= game_over 0 defender_wins\n");
        buf.clear();

        reader_2.read_line(&mut buf)?;
        assert_eq!(buf, "game 0 play attacker resigns \n");
        buf.clear();

        reader_2.read_line(&mut buf)?;
        assert_eq!(buf, "= game_over 0 defender_wins\n");
        buf.clear();

        Ok(())
    }
}
