mod game;
mod model;

pub use game::{Error as GameError, Team};
pub use model::{Column, Grid, Tile};

#[derive(Clone)]
pub struct Game {
    grid: Grid,
}
