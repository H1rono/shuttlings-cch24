use std::{borrow::Cow, sync::Arc};

use anyhow::Context;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder {
    seek_url: Option<String>,
    manifest_keyword: Option<String>,
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
            ..self
        }
    }

    pub fn manifest_keyword<'s, S>(self, value: S) -> Self
    where
        S: Into<Cow<'s, str>>,
    {
        let manifest_keyword = value.into().into_owned();
        Self {
            manifest_keyword: Some(manifest_keyword),
            ..self
        }
    }

    pub fn build(self) -> anyhow::Result<super::State> {
        use crate::handlers::{manifest, seek};

        let Self {
            seek_url,
            manifest_keyword,
        } = self;
        let seek_url = seek_url.context("state seek_url not set")?;
        let manifest_keyword = manifest_keyword.context("state manifest_keyword not set")?;
        let seek_state = seek::State::builder().seek_url(seek_url).build();
        let manifest_state = manifest::State::builder()
            .manifest_keyword(manifest_keyword)
            .build();
        let state = super::State {
            seek: Arc::new(seek_state),
            manifest: Arc::new(manifest_state),
        };
        Ok(state)
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
