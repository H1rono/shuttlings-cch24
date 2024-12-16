use std::sync::Arc;

mod manager;

pub use manager::PercentDecodeError;

#[derive(Clone)]
pub struct Manager {
    inner: Arc<manager::Inner>,
}
