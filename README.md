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

[Go Text Protocol]: https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html

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

From [Copenhagen Hnefatafl] with minor changes.

`X`: attacker  
`x`: captured attacker  
`K`: king  
`k`: captured king  
`O`: defender  
`o`: captured defender  
`R`: restricted space  

[Copenhagen Hnefatafl]: https://aagenielsen.dk/copenhagen_rules.php

### 1. Starting Board Position

```plain
  ┌───────────┐
11│R..XXXXX..R│
10│.....X.....│
 9│...........│
 8│X....O....X│
 7│X...OOO...X│
 6│XX.OOKOO.XX│
 5│X...OOO...X│
 4│X....O....X│
 3│...........│
 2│.....X.....│
 1│R..XXXXX..R│
  └───────────┘
   ABCDEFGHJKL
```

### 2. First Turn

The attackers move first.

### 3. Movement

You can move to the edge of the board or another piece orthogonally:

```plain
  ┌───────────┐
11│R.........R│
10│...........│
 9│...........│
 8│...........│
 7│...........│
 6│.....R.....│
 5│.....↑.....│
 4│....←O→....│
 3│.....↓.....│
 2│...........│
 1│R.........R│
  └───────────┘
   ABCDEFGHJKL
```

## 4. Capture

All pieces except the king are captured if sandwiched between two enemy
pieces, or between an enemy piece and a restricted square. A piece is only
captured if the trap is closed by the aggressor's move, it is therefore
permitted to move in between two enemy pieces. The king may take part in
captures.

### Captures

```plain
  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│ │R.........R│ │Ro........R│
10│...........│ │...........│ │...........│ │..↑........│
 9│...........│ │...........│ │...........│ │..X........│
 8│...........│ │...........│ │...........│ │...........│
 7│...........│ │...........│ │...........│ │...........│
 6│.....R.....│ │...X.R.....│ │..K..R.....│ │.....R.....│
 5│.....o.....│ │...o.......│ │..↓........│ │...........│
 4│.X→→→......│ │.Xo.oX.....│ │...........│ │...........│
 3│...........│ │...↑.......│ │..x........│ │...........│
 2│...........│ │...X.......│ │..O........│ │...........│
 1│R.........R│ │R.........R│ │R.........R│ │R.........R│
  └───────────┘ └───────────┘ └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL
```

### Doesn't Capture

```plain
  ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│
10│...........│ │...........│
 9│...........│ │...........│
 8│...........│ │...........│
 7│...........│ │...........│
 6│.....K.....│ │.....R.....│
 5│.....O.....│ │.O.O.......│
 4│.X→→→......│ │..↑........│
 3│...........│ │..X........│
 2│...........│ │...........│
 1│R.........R│ │R.........R│
  └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL
```

### Shield Wall

A row of two or more taflmen along the board edge may be captured together, by
bracketing the whole group at both ends, as long as every member of the row has
an enemy taflman directly in front of him.

A corner square may stand in for one of the bracketing pieces at one end of the
row. The king may take part in the capture, either as part of the shield wall
or as a bracketing piece. If the king plus one or more defenders are attacked
with a shield wall, the attack will capture the defenders but not the king.

```plain
  ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│
10│...........│ │...........│
 9│...........│ │...........│
 8│...........│ │...........│
 7│...........│ │...........│
 6│.....R.....│ │.....R.....│
 5│...........│ │...........│
 4│...........│ │...........│
 3│..O........│ │...........│
 2│..↓OOO.....│ │........XX.│
 1│R..xxxO...R│ │R....X→.KoR│
  └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL
```

## 5. Restricted squares

Restricted squares may only be occupied by the king. The central restricted
square is called the throne. It is allowed for the king to re-enter the throne,
and all pieces may pass through the throne when it is empty.

Restricted squares are hostile, which means they can replace one of the two
pieces taking part in a capture. The throne is always hostile to the attackers,
but only hostile to the defenders when it is empty.

The four corner squares are also restricted and hostile, just like the throne.
The board edge is _NOT_ hostile.

```plain
  ┌───────────┐
11│R.........R│
10│...........│
 9│...........│
 8│...........│
 7│...........│
 6│.....R.....│
 5│...........│
 4│...........│
 3│...........│
 2│...........│
 1│R.........R│
  └───────────┘
   ABCDEFGHJKL
```

## 6. King's Side Win (Defenders)

If the king reaches any corner square, the king has escaped and his side wins.

```plain
  ┌───────────┐
11│R.........R│
10│...........│
 9│...........│
 8│...........│
 7│...........│
 6│.....R.....│
 5│...........│
 4│...........│
 3│...........│
 2│...........│
 1│K.........R│
  └───────────┘
   ABCDEFGHJKL
```

NOT IMPLEMENTED YET BELOW THIS LINE

---

### Exit Forts

The defenders also win if the king has contact with the board edge, is able to
move, and it is impossible for the attackers to break the fort.

```plain
  ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│
10│...........│ │...........│
 9│...........│ │...........│
 8│...........│ │...........│
 7│...........│ │...........│
 6│.....R.....│ │.....R.....│
 5│...........│ │...........│
 4│...........│ │...........│
 3│...........│ │....OO.....│
 2│....OO.....│ │....O.O....│
 1│R..OK.O...R│ │R...OKO...R│
  └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL
```

## 7. Attackers Win

The attackers win if they can capture the king.

The king is captured when the attackers surround him on all four cardinal
points, except when he is next to the throne.

If on a square next to the throne, the attackers must occupy the three remaining
squares around him.

The king cannot be captured on the board edge.

### The King is Captured

```plain
  ┌───────────┐ ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│ │R.........R│
10│...........│ │...........│ │...........│
 9│...........│ │...........│ │...........│
 8│...........│ │...........│ │...........│
 7│.....X.....│ │...........│ │...........│
 6│....XkX....│ │.....R.....│ │.....R.....│
 5│.....X.....│ │....XkX....│ │...........│
 4│...........│ │.....X.....│ │....X......│
 3│...........│ │...........│ │...XkX.....│
 2│...........│ │...........│ │....X......│
 1│R.........R│ │R.........R│ │R.........R│
  └───────────┘ └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL   ABCDEFGHJKL
```

If the attackers surround the king and _ALL_ remaining defenders with an
unbroken ring, then they win, as they have prevented the king from escaping.

```plain
  ┌───────────┐
11│R.........R│
10│...........│
 9│.....XXX...│
 8│....X...XX.│
 7│...X.o....X│
 6│..X..R....X│
 5│.X.o...o..X│
 4│.X..ok...X.│
 3│..X...o.X..│
 2│...XXX.X...│
 1│R.....X...R│
  └───────────┘
   ABCDEFGHJKL
```

### The King is Not Captured

```plain
  ┌───────────┐ ┌───────────┐
11│R.........R│ │R.........R│
10│...........│ │...........│
 9│...........│ │...........│
 8│...........│ │...........│
 7│...........│ │...........│
 6│.....R.....│ │.....R.....│
 5│...........│ │...........│
 4│...........│ │...........│
 3│...........│ │...........│
 2│....X......│ │.X.........│
 1│R..XKX....R│ │RKX.......R│
  └───────────┘ └───────────┘
   ABCDEFGHJKL   ABCDEFGHJKL
```

## 8. Perpetual Repetitions

Perpetual repetitions are forbidden. Any perpetual repetition results in a loss
for white.

### Added Rule

If a move would repeat a board position it is not allowed.

## 9. Automatic Loss

If a player cannot move, he loses the game.

## 10. Draw

If it is not possible to end the game, for example because both sides have too
few pieces left, it is a draw.
