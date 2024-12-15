use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::connect4::{Game, Team};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PlacePathParam {
    pub(super) team: Team,
    pub(super) col: usize,
}

impl PlacePathParam {
    pub fn new(team: Team, col: usize) -> Self {
        Self { team, col }
    }
}
