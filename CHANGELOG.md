# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- hnefatafl-server-full: don't delete server_game_light when a game ends so you
  can still send messages.
- hnefatafl-client: don't connect via TCP till you have to.
- hnefatafl-client: fix a bug where the player ends a game with PlayFrom set.
- hnefatafl-client: reset the new game settings every time you enter them.
- hnefatafl-client: default to timed and rated.
- hnefatafl-server-full: archive games to file and load from file.

## 1.2.3 - 2025-06-07

- Update the `Message` documentation.

## 1.2.2 - 2025-06-06

- document `Message`.
- hnefatafl-client: add Icelandic and Icelandic Runic.

## [1.2.1] - 2025-06-03

- hnefatafl-client: update the swords (to be a dagger) and shield icon.
- hnefatafl-client: finish the translations.

## [1.2.0] - 2025-06-02

- hnefatafl-client: limit the passwords to length 32 and usernames to length 16.
- hnefatafl-client: change the game layout.
- hnefatafl-client: add the argument `--tiny-window`.
- hnefatafl-client: add i18n.
- hnefatafl-client: remove decline draw.
- Always report the errors with ? in javascript.
- hnefatafl-client: handle whitespace in a password better.

## [1.1.4] - 2025-05-23

- Port the engine to javascript and WASM.
- hnefatafl-client: don't limit the valid password characters.
- fix the lowercasing of password bug.

## [1.1.3] - 2025-05-21

- Make dirs a global dependency.
- hnefatafl-client: add on_paste to a bunch of input_texts.
- hnefatafl-client: use text_input.secure().

## [1.1.2] - 2025-05-21 (Yanked)

- hnefatafl-client: limit the valid password characters.

## [1.1.1] - 2025-05-20

- When you change the password make it lowercase (to work around a bug).

## [1.1.0] - 2025-05-20 (Yanked)

- Add `#[serde(default)]` to all of the fields that can be filled in automatically
  and are serialized.
- Add an email everyone feature.
- hnefatafl-client: load the program if their is a ron file error, but show the error.
- Add reset your password email.
- Cleanup ron file handling.
- hnefatafl-client: save the username and theme.
- hnefatafl-client: when `username` is empty, login as a default `user-u32`.
- hnefatafl-server-full: add a timestamp to messages.

## [1.0.0] - 2025-05-05

## Changed

- hnefatafl-client: don't shutdown TCP on redox.
- hnefatafl-client: make users scrollable.
- hnefatafl-client: update to iced 0.14.0-dev.
- hnefatafl-client: make the client useable on Redox.
- hnefatafl-client: don't show the terminal on Windows.
- hnefatafl-client: When the user quits logout gracefully.
- hnefatafl-server-full: Fix Windows, handle error in read_line.
- hnefatafl-client: Display light and dark always.
- hnefatafl-client: Use a checkbox for muted.
- hnefatafl-client: Display the icon in the title bar on Linux.

## [0.13.4] - 2025-04-21

- Fix the README.md file.

## [0.13.3] - 2025-04-21 (Yanked)

### Added

- Add a link to the website and point the website to rules.

## [0.13.2] - 2025-04-15

### Added

- hnefatafl-client: the icon to iced.
- hnefatafl-client: a tiny screen size.

### Changed

- hnefatafl-client: sort the players by rating.
- hnefatafl-client: build the sound files into the executable instead of
  packaging them separately.

## [0.13.1] - 2025-03-27

- hnefatafl-client: use Shaping::Advanced on the arrows.

## [0.13.0] - 2025-03-27

### Fixed

- Note when a spectator leaves a game.

### Changed

- switch the edition to 2024 and update the flatpak to match.
- hnefatafl-client: update icons.
- hnefatafl-client: make spectators prettier.

## [0.12.0] - 2025-03-22

### Added

- hnefatafl-client: display the rating of spectators.

### Changed

- Revert the Rust edition.
- hnefatafl-client: automatically resize the board.
- hnefatafl-client: don't mention you're leaving unless you're the attacker or defender.

## [0.11.0] - 2025-03-19

### Changed

- Only white (the defender) cannot repeat a move.
- hnefatafl-client: show the pieces captured.
- hnefatafl-client: make texting prettier.
- hnefatafl-client: make the spectators prettier.

## [0.10.0] - 2025-03-19

### Added

- The ability to delete your account.

### Changed

- Move the passwords into the accounts.
- hnefatafl-client: make the game display prettier.
- Make the board display as capital letters.

### Fixed

- The display of the board.
- hnefatafl-server-full: the player loses if they do not have any moves.

## [0.9.0] - 2025-03-17

- hnefatafl-client: add the letter i to the board.
- hnefatafl-client: fix the numbers spacing.

## [0.8.3] - 2025-03-15

- hnefatafl-client: change the symbols and their size.

## [0.8.2] - 2025-03-14

### Changed

- hnefatafl-client: add a quit button.

### Fixed

- hnefatafl-client: make the board pretty.
- hnefatafl-client: don't unwrap() on sound errors.

## [0.8.1] - 2025-03-12

- Bump the VERSION_ID.
- Make the light / dark buttons into a single button.

## [0.8.0] - 2025-03-11

### Added

- challenge_requested and AI.

### Changed

- hnefatafl-client: show what move we are on.
- End the game on time from the server.
- hnefatafl-client: show captures with an 'X'.
- hnefatafl-client: replace circles and triangle with chess pieces.
- Remove archived games from the `ron` file.

### Fixed

- hnefatafl-server-full: error in parsing resigns.
- hnefatafl-server-full: logout when the user tries to send an empty strings.

## [0.7.0] - 2025-02-23

### Added

- Make the login screen nicer.

## [0.6.1] - 2025-02-19

### Added

- hnefatafl-client: play a game over sound.

### Fixed

- hnefatafl-server-full: fix error where we were removing games before they ended.

## [0.6.0] - 2025-02-19

### Added

- Breaking Change: allow requesting a draw.
- Fix escape forts.
- hnefatafl-client: allow reverting a play_from.
- hnefatafl-client: say when you leave or rejoin a game.
- Allow resuming a game.
- hnefatafl-client: say wether the gme is rated or not.
- hnefatafl-client: say what address you're connected to.
- hnefatafl-client: make the theme configurable.
- Display all users in a game.
- hnefatafl-client: add sound.
- hnefatafl-client: make the board size adjustable.
- hnefatafl-server-full: do less logging when --systemd is passed.
- A link to the website.
- Breaking Change: run update_rd every two months, track when it was last run.
- You're allowed to resign.
- Displays when the game is over.
- The option to leave, accept, or decline a game.

### Changed

- Abort on panic, so that a thread doesn't panic and the program keeps running.

### Fixed

- hnefatafl-client: An error in the turn logic.
- When resuming, joining, or creating a game set the game_id.
- hnefatafl-server-full: fixed game_over error.
- hnefatafl-client: If AI resigns it doesn't crash the client.

## [0.5.3] - 2025-02-08

- Don't set the window size to infinity, it crashes on MacOS.
- Pass the VERSION_ID when you login.

## [0.5.2] - 2025-02-07

### Changed

- Fix errors in logic.

## [0.5.1] - 2025-02-07 (yanked)

### Added

- Check the VERSION_ID and report if it is wrong.
- Make the user pass "login" to login.
- A discourse link to the website.
- hnefatafl-server-full: Throw an error if we encounter a control character or
  the null character.

## [0.5.0] - 2025-02-05

### Added

- Make a users screen.
- hnefatafl-client: Give the option of Hiding the password.
- hnefatafl-client: Do logging.

### Changed

- hnefatafl-client: Improve the GUI.
- Resign if you can't generate a move.

## [0.4.1] - 2025-02-04

### Added

- Add a changelog.

### Changed

- hnefatafl-server-full: Always load the data file if it exists. Use a default location.
- hnefatafl-client: Default to connecting to hnefatafl.org.
- hnefatafl-client: Make users and games scrollable.
- Make all the features dependencies.

[unreleased]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.2.1...main
[1.2.1]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.2.0...v1.2.1
[1.2.0]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.1.4...v1.2.0
[1.1.4]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.1.3...v1.1.4
[1.1.3]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.1.2...v1.1.3
[1.1.2]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.1.1...v1.1.2
[1.1.1]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.1.0...v1.1.1
[1.1.0]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v1.0.0...v1.1.0
[1.0.0]:  https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.13.4...v1.0.0
[0.13.4]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.13.3...v0.13.4
[0.13.3]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.13.2...v0.13.3
[0.13.2]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.8.3...v0.9.0
[0.8.3]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.2...v0.6.0
[0.5.3]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.4.0...v0.4.1
