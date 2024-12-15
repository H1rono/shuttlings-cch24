use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! newtype {
    (
        $(#[$m:meta])*
        $v:vis struct $t:ident($vi:vis $ti:ty);
    ) => {
        $(#[$m])*
        $v struct $t($vi $ti);

        impl $t {
            pub fn new(inner: $ti) -> Self {
                Self(inner)
            }

            pub fn into_inner(self) -> $ti {
                self.0
            }

            pub fn as_inner(&self) -> &$ti {
                &self.0
            }

            pub fn as_inner_mut(&mut self) -> &mut $ti {
                &mut self.0
            }
        }

        impl std::convert::From<$ti> for $t {
            fn from(value: $ti) -> $t {
                $t(value)
            }
        }

        impl std::convert::From<$t> for $ti {
            fn from(value: $t) -> $ti {
                value.into_inner()
            }
        }

        impl std::convert::AsRef<$ti> for $t {
            fn as_ref(&self) -> &$ti {
                &self.0
            }
        }

        impl std::convert::AsMut<$ti> for $t {
            fn as_mut(&mut self) -> &mut $ti {
                &mut self.0
            }
        }
    };

    (
        $(#[$m:meta])*
        $v:vis struct $t:ident($vi:vis $ti:ty);
        impl without new;
    ) => {
        $(#[$m])*
        $v struct $t($vi $ti);

        impl $t {
            pub fn into_inner(self) -> $ti {
                self.0
            }

            pub fn as_inner(&self) -> &$ti {
                &self.0
            }

            pub fn as_inner_mut(&mut self) -> &mut $ti {
                &mut self.0
            }
        }

        impl std::convert::From<$ti> for $t {
            fn from(value: $ti) -> $t {
                $t(value)
            }
        }

        impl std::convert::From<$t> for $ti {
            fn from(value: $t) -> $ti {
                value.into_inner()
            }
        }

        impl std::convert::AsRef<$ti> for $t {
            fn as_ref(&self) -> &$ti {
                &self.0
            }
        }

        impl std::convert::AsMut<$ti> for $t {
            fn as_mut(&mut self) -> &mut $ti {
                &mut self.0
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tile {
    #[default]
    Empty,
    Cookie,
    Milk,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("⬛"),
            Self::Cookie => f.write_str("🍪"),
            Self::Milk => f.write_str("🥛"),
        }
    }
}

newtype! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct Column(pub(super) [Tile; 4]);
}

newtype! {
    #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize, Serialize)]
    #[serde(transparent)]
    pub struct Grid(pub(super) Vec<Column>);
    impl without new;
}

#[derive(Clone, Copy)]
pub struct Row<'a> {
    grid: &'a Grid,
    at: usize,
}

impl Row<'_> {
    pub fn tile_at(&self, at: usize) -> Option<&Tile> {
        self.grid.0.get(at)?.0.get(self.at)
    }
}

impl fmt::Display for Row<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..4 {
            write!(f, "{}", self.tile_at(i).unwrap())?;
        }
        Ok(())
    }
}

impl Grid {
    pub(super) fn row_at(&self, at: usize) -> Option<Row<'_>> {
        if at <= 3 {
            Some(Row { grid: self, at })
        } else {
            None
        }
    }

    pub(super) fn rows(&self) -> impl Iterator<Item = Row<'_>> + '_ {
        (0..4).flat_map(|i| self.row_at(i))
    }

    pub(super) fn col_at(&self, at: usize) -> Option<&Column> {
        self.0.get(at)
    }

    pub(super) fn cols(&self) -> impl Iterator<Item = &'_ Column> {
        (0..4).flat_map(|i| self.col_at(i))
    }
}
