use std::future::Future;
use std::ops::ControlFlow;
use std::time::Duration;

use warp::{http, hyper};

use crate::bucket::{Liters, MilkBucket};

#[derive(Debug, Clone)]
pub struct State {
    pub(super) bucket: MilkBucket,
}

#[derive(Debug, Clone, Default)]
pub struct Builder<Full = (), Initial = ()> {
    full: Full,
    initial: Initial,
}

impl Builder {
    fn new() -> Self {
        Self::default()
    }
}

impl<Full, Initial> Builder<Full, Initial> {
    pub fn full(self, value: f32) -> Builder<f32, Initial> {
        let Self { initial, .. } = self;
        Builder {
            full: value,
            initial,
        }
    }

    pub fn initial(self, value: f32) -> Builder<Full, f32> {
        let Self { full, .. } = self;
        Builder {
            full,
            initial: value,
        }
    }
}

impl Builder<f32, f32> {
    pub fn build(self) -> State {
        let Self { full, initial } = self;
        let bucket = MilkBucket::builder().full(full).initial(initial).build();
        State { bucket }
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

/// refill by amount per duration
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RefillRate {
    amount: Liters,
    duration: Duration,
}

impl State {
    #[tracing::instrument(skip(self))]
    pub fn refill_task(&self, rate: RefillRate) -> impl Future<Output = ()> + Send + 'static {
        let RefillRate { amount, duration } = rate;
        let bucket = self.bucket.clone();
        let mut interval = tokio::time::interval(duration);
        async move {
            loop {
                interval.tick().await;
                bucket.fill_by(amount).await;
            }
        }
    }
}

impl RefillRate {
    pub fn new(amount: Liters, duration: Duration) -> Self {
        Self { amount, duration }
    }

    pub fn per_sec(amount: Liters) -> Self {
        Self::new(amount, Duration::from_secs(1))
    }
}

pub async fn check_bucket(state: &State) -> ControlFlow<super::Response> {
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
