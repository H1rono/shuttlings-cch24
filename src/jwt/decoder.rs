use std::sync::Arc;

use bytes::Bytes;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde_json::Value;

use super::{Decoder, DecoderError};

#[derive(Clone)]
pub(super) struct Inner {
    pem: Bytes,
}

#[derive(Default)]
pub struct Builder<Pem = ()> {
    pem: Pem,
}

impl<Pem> Builder<Pem> {
    pub fn pem(self, value: impl Into<Bytes>) -> Builder<Bytes> {
        Builder { pem: value.into() }
    }
}

impl Builder<Bytes> {
    pub fn build(self) -> Decoder {
        let Self { pem } = self;
        let inner = Inner { pem };
        Decoder {
            inner: Arc::new(inner),
        }
    }
}

impl Decoder {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn decode(&self, jwt: &str) -> Result<Value, DecoderError> {
        let header = jsonwebtoken::decode_header(jwt).map_err(DecoderError::DecodeHeaderFailed)?;
        let key = self.decoding_key_of_alg(header.alg)?;
        let validation = Validation::new(header.alg);
        let value = jsonwebtoken::decode(jwt, &key, &validation)
            .map_err(DecoderError::DecodePayloadFailed)?;
        Ok(value.claims)
    }

    fn decoding_key_of_alg(&self, alg: Algorithm) -> Result<DecodingKey, DecoderError> {
        use Algorithm::{EdDSA, ES256, ES384, RS256, RS384, RS512};
        // RSA, EC, ED
        let key = match alg {
            RS256 | RS384 | RS512 => DecodingKey::from_rsa_pem(&self.inner.pem),
            ES256 | ES384 => DecodingKey::from_ec_pem(&self.inner.pem),
            EdDSA => DecodingKey::from_ed_pem(&self.inner.pem),
            _ => return Err(DecoderError::unsupported_alg()),
        }
        .map_err(DecoderError::LoadKeyFailed)?;
        Ok(key)
    }
}
