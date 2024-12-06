use std::fmt;

use serde::{Deserialize, Serialize};

pub type Manifest = cargo_manifest::Manifest<PackageMetadata>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct PackageMetadata {
    #[serde(default)]
    pub(super) orders: Orders,
}

pub(super) type Orders = Vec<ProperOrder>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(super) struct ProperOrder {
    item: String,
    quantity: u32,
}

impl fmt::Display for ProperOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { item, quantity } = self;
        write!(f, "{item}: {quantity}")
    }
}
