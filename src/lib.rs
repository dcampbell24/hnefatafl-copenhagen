pub mod board;
pub mod color;
pub mod game;
pub mod message;
pub mod play;
pub mod space;
pub mod status;
pub mod time;

#[cfg(test)]
mod tests {
    use board::STARTING_POSITION;
    use game::Game;

    use super::*;

    #[test]
    fn starting_position() {
        let game = Game::default();
        assert_eq!(game.board, STARTING_POSITION.into());
    }
}
