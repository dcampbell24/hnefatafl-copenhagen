use crate::{
    color::Color,
    game::Game,
    play::{Plae, Play, Vertex},
    status::Status,
};

pub trait AI {
    fn generate_move(&mut self, game: &Game) -> Option<Plae>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
