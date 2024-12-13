use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use super::{Liters, MilkBucket};

// MARK: Inner

#[derive(Debug)]
pub(super) struct Inner {
    full: Liters,
    filled: Mutex<Liters>,
}

// MARK: Builder

pub struct Builder<Full = (), Initial = ()> {
    full: Full,
    initial: Initial,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            full: (),
            initial: (),
        }
    }
}

impl Builder {
    fn new() -> Self {
        Self::default()
    }
}

impl MilkBucket {
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl<Full, Initial> Builder<Full, Initial> {
    pub fn full(self, value: f32) -> Builder<Liters, Initial> {
        let Self { initial, .. } = self;
        Builder {
            full: Liters(value),
            initial,
        }
    }

    pub fn initial(self, value: f32) -> Builder<Full, Liters> {
        let Self { full, .. } = self;
        Builder {
            full,
            initial: Liters(value),
        }
    }
}

impl Builder<Liters, Liters> {
    pub fn build(self) -> MilkBucket {
        let Self { full, initial } = self;
        let filled = Mutex::new(initial);
        let inner = Inner { filled, full };
        MilkBucket {
            inner: Arc::new(inner),
        }
    }
}

// MARK: op with pack

pub struct Pack(Liters);

impl MilkBucket {
    pub async fn available(&self) -> Liters {
        *self.inner.filled.lock().await
    }

    pub async fn is_empty(&self) -> bool {
        self.available().await.0 <= 0.0
    }

    pub async fn is_full(&self) -> bool {
        self.available().await.0 >= self.inner.full.0
    }

    pub async fn fill_by<L>(&self, liters: L)
    where
        L: Into<Liters>,
    {
        let liters: Liters = liters.into();
        let mut filled = self.inner.filled.lock().await;
        let current = *filled;
        let after = f32::min(current.0 + liters.0, self.inner.full.0);
        *filled = Liters(after);
    }

    pub async fn fulfill(&self) {
        let mut filled = self.inner.filled.lock().await;
        *filled = self.inner.full;
    }

    #[tracing::instrument(skip(self))]
    pub async fn withdraw_by(&self, request_liters: Liters) -> Pack {
        let mut filled = self.inner.filled.lock().await;
        let after = filled.0 - request_liters.0;
        if after >= 0.0 {
            tracing::info!(after, "milk withdrawn");
            *filled = Liters(after);
            Pack(request_liters)
        } else {
            Pack(Liters(0.0))
        }
    }
}

impl Pack {
    pub fn inner(self) -> Liters {
        self.0
    }
}

// MARK: refill task

/// refill by amount per duration
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RefillRate {
    amount: Liters,
    duration: Duration,
}

impl MilkBucket {
    #[tracing::instrument(skip(self))]
    pub fn refill_task(self, rate: RefillRate) -> impl Future<Output = ()> + Send + 'static {
        let RefillRate { amount, duration } = rate;
        let mut interval = tokio::time::interval(duration);
        async move {
            loop {
                interval.tick().await;
                self.fill_by(amount).await;
                tracing::debug!("tick");
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
