# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Run update_rd every two months.
- You're allowed to resign.
- Displays when the game is over.
- The option to leave, accept, or decline a game.

### Changed

- hnefatafl-server-full: Reduce the information logged when run under systemd.

### Fixed

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

[unreleased]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.3...main
[0.5.3]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/dcampbell24/hnefatafl-copenhagen/compare/v0.4.0...v0.4.1
