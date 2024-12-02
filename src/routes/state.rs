use std::{borrow::Cow, sync::Arc};

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Inner {
    pub(super) seek_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder {
    pub(super) seek_url: Option<String>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn seek_url<'s, S>(self, value: S) -> Self
    where
        S: Into<Cow<'s, str>>,
    {
        let seek_url = value.into().into_owned();
        Self {
            seek_url: Some(seek_url),
        }
    }

    pub fn build(self) -> anyhow::Result<super::State> {
        let seek_url = self.seek_url.context("seek_url of state not set")?;
        let inner = Inner { seek_url };
        let state = super::State {
            inner: Arc::new(inner),
        };
        Ok(state)
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
