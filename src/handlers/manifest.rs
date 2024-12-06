use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Manifest {
    pub(super) package: Package,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Package {
    #[serde(default)]
    pub(super) metadata: Metadata,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct Metadata {
    pub(super) orders: Vec<Order>,
}

pub type Order = toml::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(super) struct ProperOrder {
    item: String,
    quantity: u32,
}

impl ProperOrder {
    pub(super) fn from_value(value: &toml::Value) -> Option<Self> {
        let table = value.as_table()?;
        let item = table.get("item")?.as_str()?.to_string();
        let quantity = table.get("quantity")?.as_integer()?;
        let quantity = u32::try_from(quantity).ok()?;
        Some(Self { item, quantity })
    }
}

impl fmt::Display for ProperOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { item, quantity } = self;
        write!(f, "{item}: {quantity}")
    }
}
