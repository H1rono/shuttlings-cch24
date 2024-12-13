use std::sync::Arc;

mod milk;

#[derive(Debug, Clone)]
pub struct MilkBucket {
    inner: Arc<milk::Inner>,
}

pub use milk::Pack as MilkPack;
