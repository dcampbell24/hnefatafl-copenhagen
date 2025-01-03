# Copenhagan Hnefatafl

An engine similar to the [Go Text Protocol].

This engine follows the [Copenhagen Hnefatafl] rules.

[Go Text Protocol]: https://www.lysator.liu.se/~gunnar/gtp/gtp2-spec-draft2/gtp2-spec.html
[Copenhagen Hnefatafl]: (https://aagenielsen.dk/copenhagen_rules.php)

## Differences from the Go Text Protocol

* The character set is UTF-8.
* clear_board -> reset_board
* genmove -> generate_move
* generate_move returns "color vertex_from vertex_to"
* We keep track of whose turn it is.
* play takes "play vertex_from vertex_to", the color is whose turn it is
* showboard -> show_board
