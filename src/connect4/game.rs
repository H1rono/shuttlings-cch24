use std::fmt;

use serde::{Deserialize, Serialize};

use super::{Column, Game, Grid, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    Cookie,
    Milk,
}

impl From<Team> for Tile {
    fn from(value: Team) -> Self {
        match value {
            Team::Cookie => Tile::Cookie,
            Team::Milk => Tile::Milk,
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tile = Tile::from(*self);
        fmt::Display::fmt(&tile, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Playing,
    NoWinner,
    Wins(Team),
}

impl Status {
    fn collect_lines(lines: impl IntoIterator<Item = LineStatus>) -> Self {
        lines.into_iter().fold(Self::NoWinner, |s, l| match (s, l) {
            (s @ Self::Wins(_), _) => s,
            (_, LineStatus::OnlyCookie) => Self::Wins(Team::Cookie),
            (_, LineStatus::OnlyMilk) => Self::Wins(Team::Milk),
            (_, LineStatus::NotFilled) => Self::Playing,
            (s, _) => s,
        })
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum LineStatus {
    #[default]
    Initial,
    OnlyCookie,
    OnlyMilk,
    NotFilled,
    Mixed,
}

impl LineStatus {
    fn accept_tile(self, tile: Tile) -> Self {
        match (self, tile) {
            (_, Tile::Empty) => Self::NotFilled,
            (Self::Initial | Self::OnlyCookie, Tile::Cookie) => Self::OnlyCookie,
            (Self::Initial | Self::OnlyMilk, Tile::Milk) => Self::OnlyMilk,
            _ => Self::Mixed,
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("Invalid column {0}")]
    InvalidColumn(usize),
    #[error("Column {0} already fulfilled")]
    ColumnFulfilled(usize),
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.grid.rows() {
            writeln!(f, "⬜{row}⬜")?;
        }
        writeln!(f, "⬜⬜⬜⬜⬜⬜")
    }
}

impl Game {
    pub fn new() -> Self {
        let grid = Grid(vec![Column::default(); 4]);
        Self { grid }
    }

    pub fn reset(&mut self) {
        self.grid.0.fill(Column::default());
    }

    pub fn status(&self) -> Status {
        let mut rows = [LineStatus::Initial; 4];
        let mut cols = [LineStatus::Initial; 4];
        // (left-top to right-bottom, left-bottom to right-top)
        let mut diagonal = (LineStatus::Initial, LineStatus::Initial);
        for (c, col) in self.grid.cols().map(|c| c.into_inner()).enumerate() {
            let cs = &mut cols[c];
            for (rs, t) in rows.iter_mut().zip(col) {
                *rs = rs.accept_tile(t);
                *cs = cs.accept_tile(t);
            }

            let (d1, d2) = &mut diagonal;
            *d1 = d1.accept_tile(col[c]);
            *d2 = d2.accept_tile(col[3 - c]);
        }

        let lines = [diagonal.0, diagonal.1].into_iter().chain(rows).chain(cols);
        Status::collect_lines(lines)
    }

    pub fn display_with_status(&self) -> DisplayWithStatus<'_> {
        DisplayWithStatus(self)
    }

    pub fn pile(&mut self, team: Team, col: usize) -> Result<(), Error> {
        let column = self
            .grid
            .as_inner_mut()
            .get_mut(col)
            .ok_or(Error::InvalidColumn(col))?;
        let tile = column
            .as_inner_mut()
            .iter_mut()
            .rev()
            .find(|t| **t == Tile::Empty)
            .ok_or(Error::ColumnFulfilled(col))?;
        *tile = team.into();
        Ok(())
    }
}

pub struct DisplayWithStatus<'a>(&'a Game);

impl fmt::Display for DisplayWithStatus<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)?;
        match self.0.status() {
            Status::Playing => Ok(()),
            Status::Wins(team) => writeln!(f, "{team} wins!"),
            Status::NoWinner => writeln!(f, "No winner."),
        }
    }
}
