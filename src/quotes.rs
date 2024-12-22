use std::sync::Arc;

pub mod model;
pub mod ops;
pub mod repository;
mod shorten;

pub use shorten::Error as TokenError;

#[must_use]
#[derive(Clone)]
pub struct Repository {
    inner: Arc<repository::Inner>,
}
