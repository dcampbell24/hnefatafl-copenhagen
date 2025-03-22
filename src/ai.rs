use std::cmp::{max, min};

use chrono::Utc;
use rand::{rngs::OsRng, RngCore};

use crate::{
    board::Board,
    color::Color,
    game::Game,
    play::{Plae, Play, Vertex},
    role::Role,
    space::Space,
    status::Status,
};

pub trait AI {
    fn generate_move(&mut self, game: &Game) -> Option<Plae>;
}

#[derive(Clone, Debug, Default)]
pub struct AiBanal;

impl AI for AiBanal {
    fn generate_move(&mut self, game: &Game) -> Option<Plae> {
        if game.status != Status::Ongoing {
            return None;
        }
        let mut game_clone = game.clone();

        for x_from in 0..11 {
            for y_from in 0..11 {
                for x_to in 0..11 {
                    for y_to in 0..11 {
                        let play = Plae::Play(Play {
                            color: game.turn.clone(),
                            from: Vertex {
                                x: x_from,
                                y: y_from,
                            },
                            to: Vertex { x: x_to, y: y_to },
                        });

                        if game_clone.play(&play).is_ok() {
                            return Some(play);
                        }
                    }
                }
            }
        }

        match game.turn {
            Color::Black => Some(Plae::BlackResigns),
            Color::Colorless => None,
            Color::White => Some(Plae::WhiteResigns),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AiBasic {
    pub depth: u64,
    // seconds
    pub time_to_move: i64,
}

impl Default for AiBasic {
    fn default() -> Self {
        Self {
            depth: 4,
            time_to_move: 15,
        }
    }
}

impl AI for AiBasic {
    fn generate_move(&mut self, game: &Game) -> Option<Plae> {
        if game.status != Status::Ongoing {
            return None;
        }

        self.minimax_search(game)
    }
}

impl AiBasic {
    fn minimax_search(&mut self, game: &Game) -> Option<Plae> {
        let cutoff_time = Utc::now().timestamp() + self.time_to_move;
        let alpha = i32::MIN;
        let beta = i32::MAX;

        let (value, play) = match Role::try_from(&game.turn).ok()? {
            Role::Attacker => self.min_value(game, alpha, beta, cutoff_time, 0),
            Role::Defender => self.max_value(game, alpha, beta, cutoff_time, 0),
        };

        println!("value: {value}");
        play
    }

    fn max_value(
        &mut self,
        game: &Game,
        mut alpha: i32,
        beta: i32,
        cutoff_time: i64,
        depth: u64,
    ) -> (i32, Option<Plae>) {
        if Utc::now().timestamp() > cutoff_time
            || depth > self.depth
            || game.status != Status::Ongoing
        {
            return (game.utility(), None);
        }

        let (mut value, mut play_1) = (i32::MIN, None);
        for play_2 in game.all_legal_plays() {
            let mut game = game.clone();
            game.play(&play_2).unwrap();
            let (value_new, _play) = self.min_value(&game, alpha, beta, cutoff_time, depth + 1);

            if value_new > value {
                (value, play_1) = (value_new, Some(play_2));
                alpha = max(alpha, value);
            }

            if value >= beta {
                return (value, play_1);
            }
        }

        (value, play_1)
    }

    fn min_value(
        &mut self,
        game: &Game,
        alpha: i32,
        mut beta: i32,
        cutoff_time: i64,
        depth: u64,
    ) -> (i32, Option<Plae>) {
        if Utc::now().timestamp() > cutoff_time
            || depth > self.depth
            || game.status != Status::Ongoing
        {
            return (game.utility(), None);
        }

        let (mut value, mut play_1) = (i32::MAX, None);
        for play_2 in game.all_legal_plays() {
            let mut game = game.clone();
            game.play(&play_2).unwrap();
            let (value_new, _play) = self.max_value(&game, alpha, beta, cutoff_time, depth + 1);

            if value_new < value {
                (value, play_1) = (value_new, Some(play_2));
                beta = min(beta, value);
            }
            if value <= alpha {
                return (value, play_1);
            }
        }

        (value, play_1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZobristTable {
    /// Bitstrings representing piece placement
    piece_bits: [[u64; 3]; 11 * 11],
    /// Bitstring to use used when it's the defender's move.
    defender_to_move_bits: u64,
}

impl Default for ZobristTable {
    fn default() -> Self {
        let mut rng = OsRng;

        let mut hashes = [[0; 3]; 121];
        for hash in &mut hashes {
            *hash = [rng.next_u64(), rng.next_u64(), rng.next_u64()];
        }

        Self {
            piece_bits: hashes,
            defender_to_move_bits: rng.next_u64(),
        }
    }
}

impl ZobristTable {
    #[must_use]
    pub fn hash(&self, board: &Board, side_to_play: Role) -> u64 {
        let mut hash = 0u64;

        if side_to_play == Role::Defender {
            hash ^= self.defender_to_move_bits;
        }

        for (i, space) in board.spaces.iter().enumerate() {
            if space != &Space::Empty {
                let j = space.index();
                hash ^= self.piece_bits[i][j];
            }
        }

        hash
    }
}
