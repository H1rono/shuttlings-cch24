use std::borrow::Cow;

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct State {
    pub(super) seek_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder {
    pub(super) seek_url: Option<String>,
}

#[allow(dead_code)]
impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_seek_url(self, value: Option<String>) -> Self {
        Self { seek_url: value }
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

    pub fn build(self) -> anyhow::Result<State> {
        let seek_url = self.seek_url.context("seek_url not set")?;
        let state = State { seek_url };
        Ok(state)
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
