use std::sync::Arc;

use tokio::sync::Mutex;

use super::MilkBucket;

// MARK: Inner

#[derive(Debug)]
pub(super) struct Inner {
    full: f32,
    filled: Mutex<f32>,
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

pub struct Pack(f32);

impl MilkBucket {
    pub async fn fill_by(&self, liters: f32) {
        let mut filled = self.inner.filled.lock().await;
        let current = *filled;
        let after = f32::max(current + liters, self.inner.full);
        *filled = after;
    }

    pub async fn withdraw_by(&self, request_liters: f32) -> Pack {
        let mut filled = self.inner.filled.lock().await;
        let current = *filled;
        if current >= request_liters {
            *filled = current - request_liters;
            Pack(request_liters)
        } else {
            Pack(0.0)
        }
    }
}
