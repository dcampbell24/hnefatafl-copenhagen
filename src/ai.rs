use std::fmt;

use rand::{RngCore, rngs::OsRng};

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

#[derive(Clone, Debug, Default)]
pub struct AiBasic;

impl AI for AiBasic {
    fn generate_move(&mut self, game: &Game) -> Option<Plae> {
        if game.status != Status::Ongoing {
            return None;
        }

        let mut tree = Tree::new(game);
        tree.expand_all();
        let tree_node = &tree.0[0];

        // println!("{}", tree);

        for i in tree_node.children.as_ref().unwrap() {
            let node = &tree.0[*i];
            if node.game.status == Status::Ongoing {
                if let Node::Play(play) = &node.inner {
                    return Some(Plae::Play(play.clone()));
                }
            }
        }

        match Role::try_from(&game.turn).unwrap() {
            Role::Attacker => Some(Plae::BlackResigns),
            Role::Defender => Some(Plae::WhiteResigns),
        }
    }
}

#[derive(Clone, Debug)]
struct Tree(Vec<NodeStruct>);

impl Tree {
    fn new(game: &Game) -> Self {
        Tree(vec![NodeStruct {
            inner: Node::Root,
            up: None,
            game: game.clone(),
            children: None,
            utility: 0,
        }])
    }

    fn expand_all(&mut self) {
        let length = self.0.len();
        let mut offset = 0;
        let mut new_nodes = Vec::new();
        for (i, node) in self.0.iter_mut().enumerate() {
            if node.children.is_none() && node.game.status == Status::Ongoing {
                let legal_moves = node.game.all_legal_moves();

                let mut plays = Vec::new();
                for from in legal_moves.moves.keys() {
                    for to in &legal_moves.moves[from] {
                        plays.push(Play {
                            color: legal_moves.color.clone(),
                            from: from.clone(),
                            to: to.clone(),
                        });
                    }
                }

                let mut children = Vec::new();
                for (j, play) in plays.iter().enumerate() {
                    let mut game = node.game.clone();
                    let turn = game.turn.clone();
                    let captures = game.play(&Plae::Play(play.clone())).unwrap();

                    let mut node = NodeStruct {
                        inner: Node::Play(play.clone()),
                        up: Some(i),
                        game,
                        children: None,
                        utility: 0,
                    };

                    match turn {
                        Color::Black => node.utility -= i32::try_from(captures.0.len()).unwrap(),
                        Color::Colorless => {}
                        Color::White => node.utility += i32::try_from(captures.0.len()).unwrap(),
                    }
                    node.utility += node.utility();

                    new_nodes.push(node);

                    children.push(length + offset + j);
                }
                offset += children.len();
                node.children = Some(children);
            }
        }

        for node in new_nodes {
            self.0.push(node);
        }
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for node in &self.0 {
            write!(f, "[")?;
            match &node.up {
                Some(node) => write!(f, "{node}")?,
                None => write!(f, "_")?,
            }
            write!(f, "; ")?;
            match &node.children {
                Some(nodes) => write!(f, "{nodes:?}")?,
                None => write!(f, "_")?,
            }
            write!(f, "] ")?;
        }

        writeln!(f)
    }
}

#[derive(Clone, Debug)]
enum Node {
    Root,
    Play(Play),
}

#[derive(Clone, Debug)]
struct NodeStruct {
    inner: Node,
    up: Option<usize>,
    game: Game,
    children: Option<Vec<usize>>,
    utility: i32,
}

impl NodeStruct {
    #[must_use]
    fn utility(&self) -> i32 {
        let mut utility = 0;

        if self.game.exit_one() {
            utility += 100;
        }

        utility
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
