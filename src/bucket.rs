use std::sync::Arc;

mod milk;
mod unit;

#[derive(Debug, Clone)]
pub struct MilkBucket {
    inner: Arc<milk::Inner>,
}

pub use milk::Pack as MilkPack;
pub use unit::{Gallons, Liters};
