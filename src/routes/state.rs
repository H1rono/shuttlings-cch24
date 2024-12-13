use std::{borrow::Cow, future::Future, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder<SeekUrl = (), ManifestKeyword = (), MilkFull = (), MilkInitial = ()> {
    seek_url: SeekUrl,
    manifest_keyword: ManifestKeyword,
    milk_full: MilkFull,
    milk_initial: MilkInitial,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<SeekUrl, ManifestKeyword, MilkFull, MilkInitial>
    Builder<SeekUrl, ManifestKeyword, MilkFull, MilkInitial>
{
    pub fn seek_url<'s, S>(
        self,
        value: S,
    ) -> Builder<String, ManifestKeyword, MilkFull, MilkInitial>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            manifest_keyword,
            milk_full,
            milk_initial,
            ..
        } = self;
        let seek_url = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
        }
    }

    pub fn manifest_keyword<'s, S>(
        self,
        value: S,
    ) -> Builder<SeekUrl, String, MilkFull, MilkInitial>
    where
        S: Into<Cow<'s, str>>,
    {
        let Self {
            seek_url,
            milk_full,
            milk_initial,
            ..
        } = self;
        let manifest_keyword = value.into().into_owned();
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
        }
    }

    pub fn milk_full(self, value: f32) -> Builder<SeekUrl, ManifestKeyword, f32, MilkInitial> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_initial,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full: value,
            milk_initial,
        }
    }

    pub fn milk_initial(self, value: f32) -> Builder<SeekUrl, ManifestKeyword, MilkFull, f32> {
        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            ..
        } = self;
        Builder {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial: value,
        }
    }
}

impl Builder<String, String, f32, f32> {
    pub fn build(self) -> super::State {
        use crate::handlers::{manifest, milk, seek};

        let Self {
            seek_url,
            manifest_keyword,
            milk_full,
            milk_initial,
        } = self;
        let seek_state = seek::State::builder().seek_url(seek_url).build();
        let manifest_state = manifest::State::builder()
            .manifest_keyword(manifest_keyword)
            .build();
        let milk = milk::State::builder()
            .full(milk_full)
            .initial(milk_initial)
            .build();
        super::State {
            seek: Arc::new(seek_state),
            manifest: Arc::new(manifest_state),
            milk: Arc::new(milk),
        }
    }
}

impl super::State {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn bg_task(&self) -> impl Future<Output = ()> + Send + 'static {
        use crate::bucket::{milk, Liters};

        // FIXME: expose configuration
        let rate = milk::RefillRate::per_sec(Liters(1.0));
        self.milk.refill_task(rate)
    }
}
