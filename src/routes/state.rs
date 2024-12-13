use std::{borrow::Cow, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder<SeekUrl = (), ManifestKeyword = ()> {
    seek_url: SeekUrl,
    manifest_keyword: ManifestKeyword,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<SeekUrl, ManifestKeyword> Builder<SeekUrl, ManifestKeyword> {
    pub fn seek_url<'s, S>(self, value: S) -> Builder<String, ManifestKeyword>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            manifest_keyword, ..
        } = self;
        let seek_url = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
        }
    }

    pub fn manifest_keyword<'s, S>(self, value: S) -> Builder<SeekUrl, String>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self { seek_url, .. } = self;
        let manifest_keyword = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
        }
    }
}

impl Builder<String, String> {
    pub fn build(self) -> super::State {
        use crate::handlers::{manifest, seek};

        let Self {
            seek_url,
            manifest_keyword,
        } = self;
        let seek_state = seek::State::builder().seek_url(seek_url).build();
        let manifest_state = manifest::State::builder()
            .manifest_keyword(manifest_keyword)
            .build();
        super::State {
            seek: Arc::new(seek_state),
            manifest: Arc::new(manifest_state),
        }
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
