# Copenhagan Hnefatafl

[![github]][github-link]&ensp;[![crates-io]][crates-io-link]&ensp;[![docs-rs]][docs-rs-link]

[github]: https://img.shields.io/badge/github-8da0cb?logo=github
[github-link]: https://github.com/dcampbell24/hnefatafl-copenhagen
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?logo=rust
[crates-io-link]: https://crates.io/crates/hnefatafl-copenhagen
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?logo=docs.rs
[docs-rs-link]: https://docs.rs/hnefatafl-copenhagen

**WARNING: THIS IS A WORK IN PROGESS** all the rules have not been implemented
nor has it been thoroughly tested yet.

An engine similar to the [Go Text Protocol].

This engine follows the [Copenhagen Hnefatafl] rules.

[Go Text Protocol]: https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html
[Copenhagen Hnefatafl]: https://aagenielsen.dk/copenhagen_rules.php

## Differences from the Go Text Protocol

* The character set is UTF-8.
* `clear_board` -> `reset_board`
* `genmove` -> `generate_move`
* `generate_move` returns `COLOR VERTEX_FROM VERTEX_TO`
* We keep track of whose turn it is.
* play takes `play VERTEX_FROM VERTEX_TO`, the color is whose turn it is
* `showboard` -> `show_board`
* `time_settings none` | `time_settings fischer MINUTES ADD_SECONDS_AFTER_EACH_MOVE`
