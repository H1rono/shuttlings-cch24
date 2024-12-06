use std::borrow::Cow;
use std::fmt;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct State {
    pub(super) manifest_keyword: String,
}

impl State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash, Deserialize, Serialize)]
pub struct Builder {
    manifest_keyword: Option<String>,
}

#[allow(dead_code)]
impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_manifest_keyword(self, value: Option<String>) -> Self {
        Self {
            manifest_keyword: value,
        }
    }

    pub fn manifest_keyword<'s, S>(self, value: S) -> Self
    where
        S: Into<Cow<'s, str>>,
    {
        let manifest_keyword = value.into().into_owned();
        Self {
            manifest_keyword: Some(manifest_keyword),
        }
    }

    pub fn build(self) -> anyhow::Result<State> {
        let manifest_keyword = self.manifest_keyword.context("manifest_keyword not set")?;
        let state = State { manifest_keyword };
        Ok(state)
    }
}

pub type Manifest = cargo_manifest::Manifest;

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

impl ProperOrder {
    pub(super) fn from_value(value: &toml::Value) -> Option<Vec<Self>> {
        let orders = value
            .as_table()?
            .get("orders")?
            .as_array()?
            .iter()
            .filter_map(|o| {
                let table = o.as_table()?;
                let item = table.get("item")?.as_str()?.to_string();
                let quantity = table.get("quantity")?.as_integer()?;
                let quantity = u32::try_from(quantity).ok()?;
                Some(Self { item, quantity })
            })
            .collect();
        Some(orders)
    }
}

impl fmt::Display for ProperOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { item, quantity } = self;
        write!(f, "{item}: {quantity}")
    }
}

pub(super) fn manifest_key_included(state: &State, manifest: &Manifest) -> bool {
    let keyword = &state.manifest_keyword;
    let manifest_keywords = manifest.package.as_ref().and_then(|p| p.keywords.as_ref());
    let Some(manifest_keywords) = manifest_keywords else {
        return false;
    };
    let Some(manifest_keywords) = manifest_keywords.as_ref().as_local() else {
        // TODO
        return false;
    };
    manifest_keywords.contains(keyword)
}
