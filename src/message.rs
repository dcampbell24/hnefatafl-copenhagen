use std::str::FromStr;

use anyhow::Context;

use crate::{
    play::{Plae, Vertex},
    role::Role,
    time,
};

/// hnefatafl-text-protocol binary and javascript pkg
///
/// The engine is line orientated and takes UTF-8 encoded text. The engine takes the below commands
/// and returns `= response` on success and `? error_message` on failure. If only comments and
/// whitespace or the empty string are passed the engine ignores the input and requests another
/// string. Comments are any text following `#` on a line.
///
/// Valid **ROLE** strings are `a`, `attacker`, `d`, and `defender`. Case does not matter.
///
/// Valid **TO** and **FROM** coordinates are a letter, uppercase or lowercase, `A` though `K`
/// followed by a number `1` through `11`. For example, `A1`.
///
/// **MILLISECONDS** and **ADD_SECONDS** are numbers.
///
/// In order to run the javascript pkg:
///
/// ```sh
/// cargo install wasm-pack
/// make js
/// ```
///
/// Then copy the pkg folder to your web browser's site folder. For example with Apache on Debian:
///
/// ```sh
/// sudo mkdir --parent /var/www/html/pkg
/// sudo cp -r pkg /var/www/html
/// ```
///
/// Or if you installed the package via npm:
///
/// ```sh
/// sudo mkdir --parent /var/www/html/pkg
/// sudo cp ~/node_modules/hnefatafl-copenhagen/* /var/www/html/pkg
/// ```
///
/// Then load the javascript on a webpage:
///
/// ```sh
/// cat << EOF > /var/www/html/index.html
/// <!DOCTYPE html>
/// <html>
/// <head>
///     <title>Copenhagen Hnefatafl</title>
/// </head>
/// <body>
///     <h1>Copenhagen Hnefatafl</h1>
///     <script type="module">
///         import init, { Game } from '../pkg/hnefatafl_copenhagen.js';
///
///         init().then(() => {
///             const game = new Game();
///             const output = game.read_line_js("show_board");
///             console.log(output);
///         });
///     </script>
/// </body>
/// </html>
/// EOF
/// ```
#[allow(clippy::doc_markdown)]
#[derive(Debug, Clone)]
pub enum Message {
    /// The empty string or only comments and whitespace was passed.
    Empty,

    /// `final_status`
    ///
    /// Returns `attacker_wins` or `draw` or `ongoing` or `defender_wins`.
    FinalStatus,

    /// `generate_move`
    ///
    /// Returns `play ROLE FROM TO`.
    GenerateMove,

    /// `known_command STRING`
    ///
    /// Returns a boolean signifying whether the engine knows the command.
    KnownCommand(String),

    /// `list_commands`
    ///
    /// Lists all of the known commands, each separated by a newline.
    ListCommands,

    /// `name`
    ///
    /// Prints the name of the package.
    Name,

    /// `play ROLE FROM TO` | `play ROLE resign`
    ///
    /// Plays a move and returns **CAPTURES**, where **CAPTURES** has the format `A2 C2 ...`.
    Play(Plae),

    /// `play_from`
    ///
    /// Returns a **ROLE** followed by all the valid **FROM** squares.
    PlayFrom,

    /// `play_to ROLE FROM`
    ///
    /// Returns all the valid **TO** squares.
    PlayTo((Role, Vertex)),

    /// `protocol_version`
    ///
    /// Prints the version of the Hnefatafl Text Protocol.
    ProtocolVersion,

    /// `quit`
    ///
    /// quits the engine.
    Quit,

    /// `reset_board`
    ///
    /// Sets the board to the starting position.
    ResetBoard,

    /// `show_board`
    ///
    /// Displays the board
    ShowBoard,

    /// `time_settings un-timed` | `time_settings fischer MILLISECONDS ADD_SECONDS`
    ///
    /// Choose the time settings. For fischer time **MILLISECONDS** is the starting time and
    /// **ADD_SECONDS** is how much time to add after each move. **ADD_SECONDS** may be zero, in
    /// which case the time settings are really absolute time.
    TimeSettings(time::TimeSettings),

    /// `version`
    ///
    /// Displays the package version.
    Version,
}

pub static COMMANDS: [&str; 14] = [
    "final_status",
    "generate_move",
    "known_command",
    "list_commands",
    "name",
    "play",
    "play_from",
    "play_to",
    "protocol_version",
    "quit",
    "reset_board",
    "show_board",
    "time_settings",
    "version",
];

impl FromStr for Message {
    type Err = anyhow::Error;

    fn from_str(message: &str) -> anyhow::Result<Self> {
        let args: Vec<&str> = message.split_whitespace().collect();

        if args.is_empty() {
            return Ok(Self::Empty);
        }

        match *args.first().unwrap() {
            "final_status" => Ok(Self::FinalStatus),
            "generate_move" => Ok(Self::GenerateMove),
            "known_command" => Ok(Self::KnownCommand(
                (*args.get(1).context("expected: known_command COMMAND")?).to_string(),
            )),
            "list_commands" => Ok(Self::ListCommands),
            "name" => Ok(Self::Name),
            "play" => {
                let play = Plae::try_from(args)?;
                Ok(Self::Play(play))
            }
            "play_from" => Ok(Self::PlayFrom),
            "play_to" => {
                if let (Some(role), Some(vertex)) = (args.get(1), args.get(2)) {
                    let role = Role::from_str(role)?;
                    let vertex = Vertex::from_str(vertex)?;
                    Ok(Self::PlayTo((role, vertex)))
                } else {
                    Err(anyhow::Error::msg("expected: play_to role vertex"))
                }
            }
            "protocol_version" => Ok(Self::ProtocolVersion),
            "quit" => Ok(Self::Quit),
            "reset_board" => Ok(Self::ResetBoard),
            "show_board" => Ok(Self::ShowBoard),
            "time_settings" => {
                let time_settings = time::TimeSettings::try_from(args)?;
                Ok(Self::TimeSettings(time_settings))
            }
            "version" => Ok(Self::Version),
            text => {
                if text.trim().is_empty() {
                    Ok(Self::Empty)
                } else {
                    Err(anyhow::Error::msg(format!("unrecognized command: {text}")))
                }
            }
        }
    }
}
