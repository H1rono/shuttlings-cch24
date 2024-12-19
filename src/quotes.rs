use std::sync::Arc;

pub mod model;
pub mod ops;
pub mod repository;

#[must_use]
#[derive(Clone)]
pub struct Repository {
    inner: Arc<repository::Inner>,
}
