mod game;
mod model;

pub use game::{Error as GameError, Team};
pub use model::{Column, Grid, Tile};
use rand::rngs::StdRng;

#[derive(Clone)]
pub struct Game {
    grid: Grid,
    rng: StdRng,
}
