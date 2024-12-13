use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct State {
    pub(super) seek_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Builder<SeekUrl = ()> {
    pub(super) seek_url: SeekUrl,
}

#[allow(dead_code)]
impl Builder {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<SeekUrl> Builder<SeekUrl> {
    pub fn seek_url<'a, S>(self, value: S) -> Builder<String>
    where
        S: Into<Cow<'a, str>>,
    {
        let seek_url = value.into().into_owned();
        Builder { seek_url }
    }
}

impl Builder<String> {
    pub fn build(self) -> State {
        let Self { seek_url } = self;
        State { seek_url }
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}
