# Copenhagan Hnefatafl

[![github]][github-link]&ensp;[![crates-io]][crates-io-link]&ensp;[![docs-rs]][docs-rs-link]

[github]: https://img.shields.io/badge/github-8da0cb?logo=github
[github-link]: https://github.com/dcampbell24/hnefatafl-copenhagen
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?logo=rust
[crates-io-link]: https://crates.io/crates/hnefatafl-copenhagen
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?logo=docs.rs
[docs-rs-link]: https://docs.rs/hnefatafl-copenhagen

An [engine] similar to the [Go Text Protocol], a [client], and a [server].

[engine]: https://github.com/dcampbell24/hnefatafl-copenhagen/blob/main/src/bin/hnefatafl-text-protocol.rs
[Go Text Protocol]: https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html
[client]: https://github.com/dcampbell24/hnefatafl-copenhagen/blob/main/src/bin/hnefatafl-client.rs
[server]: https://github.com/dcampbell24/hnefatafl-copenhagen/blob/main/src/bin/hnefatafl-server.rs

## Build Dependencies (Linux)

ALSA development files are needed to build cpal on Linux (rodio dependency,
hnefatafl-client dependency). These are provided as part of the libasound2-dev
package on Debian and Ubuntu distributions and alsa-lib-devel on Fedora.

Also, the package uses the mold linker. This is provided via the mold package
on Debian, Ubuntu, and Fedora.

## Differences from the Go Text Protocol

* The character set is UTF-8.
* `clear_board` -> `reset_board`
* `genmove` -> `generate_move`
* `generate_move` returns `COLOR VERTEX_FROM VERTEX_TO`
* We keep track of whose turn it is.
* play takes `play COLOR VERTEX_FROM VERTEX_TO` or `play COLOR resigns` and
  returns `= CAPTURES`, where `CAPTURES` has the format `a2 c2 ...`. The color
  is whose turn it is.
* `showboard` -> `show_board`
* `time_settings none` | `time_settings fischer MINUTES ADD_SECONDS_AFTER_EACH_MOVE`
* `final_status_list` -> `final_status` = `black_wins` | `draw` | `ongoing` | `white_wins`

## Website

See the [website](https://hnefatafl.org/) for more information about downloading
and building the software.

## Rules

See the [Rules](https://hnefatafl.org/rules.html) for how to play.
