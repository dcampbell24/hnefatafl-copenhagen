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
* play takes `play COLOR VERTEX_FROM VERTEX_TO`, the color is whose turn it is
* `showboard` -> `show_board`
* `time_settings none` | `time_settings fischer MINUTES ADD_SECONDS_AFTER_EACH_MOVE`
* `final_status_list` -> `final_status` = `black_wins` | `draw` | `ongoing` | `white_wins`

## Rules

The attackers are black and the defenders white.

### 1. Starting Board Position

```plain
  ┌───────────┐
11│■..○○○○○..■│
10│.....○.....│
 9│...........│
 8│○....●....○│
 7│○...●●●...○│
 6│○○.●●▲●●.○○│
 5│○...●●●...○│
 4│○....●....○│
 3│...........│
 2│.....○.....│
 1│■..○○○○○..■│
  └───────────┘
   ABCDEFGHJKL
```

### 2. First Turn

The attackers move first.

### 3. Movement

You can move to the edge of the board or another piece orthogonally:

```plain
  ┌───────────┐
11│■.........■│
10│...........│
 9│...........│
 8│...........│
 7│...........│
 6│.....■.....│
 5│.....↑.....│
 4│....←●→....│
 3│.....↓.....│
 2│...........│
 1│■.........■│
  └───────────┘
   ABCDEFGHJKL
```

## 4. Capture

All pieces except the king are captured if sandwiched between two enemy
pieces, or between an enemy piece and a restricted square. A piece is only
captured if the trap is closed by the aggressor's move, it is therefore
permitted to move in between two enemy pieces. The king may take part in
captures.

```plain
  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐
11│■.........■│ │■.........■│ │■.........■│ │■.........■│ │■ø........■│ │■.........■│
10│...........│ │...........│ │...........│ │...........│ │..↑........│ │...........│
 9│...........│ │...........│ │...........│ │...........│ │..○........│ │...........│
 8│...........│ │...........│ │...........│ │...........│ │...........│ │...........│
 7│...........│ │...........│ │...........│ │...........│ │...........│ │...........│
 6│...○.■.....│ │.....■.....│ │.....▲.....│ │..▲..■.....│ │.....■.....│ │.....■.....│
 5│...ø.......│ │.....ø.....│ │.....●.....│ │..↓........│ │.●.●.......│ │...........│
 4│.○ø.ø○.....│ │.○→→→......│ │.○→→→......│ │...........│ │..↑........│ │...........│
 3│...↑.......│ │...........│ │...........│ │..ø........│ │..○........│ │...........│
 2│...○.......│ │...........│ │...........│ │..●........│ │...........│ │...........│
 1│■.........■│ │■.........■│ │■.........■│ │■.........■│ │■.........■│ │■.........■│
  └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL
```
