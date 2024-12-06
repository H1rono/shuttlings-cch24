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
pub(super) struct ProperOrder<'a> {
    item: &'a str,
    quantity: u32,
}

impl<'a> ProperOrder<'a> {
    pub(super) fn from_value(value: &'a toml::Value) -> Option<Self> {
        let table = value.as_table()?;
        let item = table.get("item")?.as_str()?;
        let quantity = table.get("quantity")?.as_integer()?;
        let quantity = u32::try_from(quantity).ok()?;
        Some(Self { item, quantity })
    }
}

impl fmt::Display for ProperOrder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { item, quantity } = self;
        write!(f, "{item}: {quantity}")
    }
}
