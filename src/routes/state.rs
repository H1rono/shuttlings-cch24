use std::{borrow::Cow, sync::Arc};

use anyhow::Context;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder {
    seek_url: Option<String>,
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
        let Self { seek_url } = self;
        let seek_state = crate::handlers::seek::State::builder()
            .set_seek_url(seek_url)
            .build()
            .context("failed to build seek state")?;
        let state = super::State {
            seek: Arc::new(seek_state),
        };
        Ok(state)
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
