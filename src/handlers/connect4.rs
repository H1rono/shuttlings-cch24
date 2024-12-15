use tokio::sync::Mutex;

use crate::connect4::Game;

pub struct State {
    pub(super) game: Mutex<Game>,
}

impl Default for State {
    fn default() -> Self {
        let game = Game::default();
        Self {
            game: Mutex::new(game),
        }
    }
}
