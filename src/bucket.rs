use std::sync::Arc;

pub mod milk;
mod unit;

#[derive(Debug, Clone)]
pub struct MilkBucket {
    inner: Arc<milk::Inner>,
}

pub use unit::{Gallons, Liters, Litres, Pints};
