use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod decoder;
mod error;
mod manager;

pub use error::{DecodingError, EncodingError};
pub use manager::{Decoded, Encoded};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    exp: u64,
    iat: u64,
    iss: String,
    pub(crate) custom: Value,
}

#[derive(Clone)]
pub struct Manager {
    inner: Arc<manager::Inner>,
}

#[derive(Clone)]
pub struct Decoder {
    inner: Arc<decoder::Inner>,
}
