use std::sync::Arc;

pub mod model;
mod ops;
mod repository;

#[must_use]
#[derive(Clone)]
pub struct Repository {
    inner: Arc<repository::Inner>,
}
