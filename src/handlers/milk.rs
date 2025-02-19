use std::future::Future;
use std::ops::ControlFlow;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use warp::{http, hyper};

use crate::bucket::{milk, Gallons, Liters, Litres, MilkBucket, Pints};

#[derive(Debug, Clone)]
pub struct State {
    pub(super) bucket: MilkBucket,
}

#[derive(Debug, Clone, Default)]
pub struct Builder<Bucket = ()> {
    bucket: Bucket,
}

impl Builder {
    fn new() -> Self {
        Self::default()
    }
}

impl<Bucket> Builder<Bucket> {
    pub fn bucket(self, value: MilkBucket) -> Builder<MilkBucket> {
        Builder { bucket: value }
    }
}

impl Builder<MilkBucket> {
    pub fn build(self) -> State {
        let Self { bucket } = self;
        State { bucket }
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl State {
    pub fn refill_task(&self, rate: milk::RefillRate) -> impl Future<Output = ()> + Send + 'static {
        self.bucket.clone().refill_task(rate)
    }
}

pub async fn check_bucket(state: Arc<State>) -> ControlFlow<super::Response> {
    if !state.bucket.is_empty().await {
        return ControlFlow::Continue(());
    }
    let body = hyper::Body::from("No milk available\n".to_string());
    let res = super::Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(body)
        .unwrap();
    ControlFlow::Break(res)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    // US
    Liters(f32),
    Gallons(f32),
    // UK
    Litres(f32),
    Pints(f32),
}

impl From<Liters> for Unit {
    fn from(value: Liters) -> Self {
        Self::Liters(value.0)
    }
}

impl From<Gallons> for Unit {
    fn from(value: Gallons) -> Self {
        Self::Gallons(value.0)
    }
}

impl From<Litres> for Unit {
    fn from(value: Litres) -> Self {
        Self::Litres(value.0)
    }
}

impl From<Pints> for Unit {
    fn from(value: Pints) -> Self {
        Self::Pints(value.0)
    }
}

impl Unit {
    pub(super) fn convert(self) -> Self {
        match self {
            Self::Liters(l) => Liters(l).gallons().into(),
            Self::Gallons(g) => Gallons(g).liters().into(),
            Self::Litres(l) => Litres(l).pints().into(),
            Self::Pints(p) => Pints(p).litres().into(),
        }
    }
}
