use std::fmt;
use std::sync::Arc;
use std::{borrow::Cow, time::Duration};

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde_json::Value;

use super::{Claims, DecodingError, EncodingError, Manager};

#[derive(Clone)]
pub(super) struct Inner {
    issuer: String,
    #[allow(unused)]
    raw_key: String,
    enc_key: EncodingKey,
    dec_key: DecodingKey,
    expires_in: Duration,
}

pub struct Builder<Issuer = (), Key = (), ExpiresIn = ()> {
    issuer: Issuer,
    key: Key,
    expires_in: ExpiresIn,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            issuer: (),
            key: (),
            expires_in: (),
        }
    }
}

impl<Issuer, Key, ExpiresIn> Builder<Issuer, Key, ExpiresIn> {
    pub fn issuer<'a, S>(self, value: S) -> Builder<String, Key, ExpiresIn>
    where
        S: Into<Cow<'a, str>>,
    {
        let Self {
            key, expires_in, ..
        } = self;
        let issuer: String = value.into().into_owned();
        Builder {
            issuer,
            key,
            expires_in,
        }
    }

    pub fn key<'a, S>(self, value: S) -> Builder<Issuer, String, ExpiresIn>
    where
        S: Into<Cow<'a, str>>,
    {
        let Self {
            issuer, expires_in, ..
        } = self;
        let key: String = value.into().into_owned();
        Builder {
            issuer,
            key,
            expires_in,
        }
    }

    pub fn expires_in(self, value: Duration) -> Builder<Issuer, Key, Duration> {
        let Self { issuer, key, .. } = self;
        let expires_in = value;
        Builder {
            issuer,
            key,
            expires_in,
        }
    }
}

impl Builder<String, String, Duration> {
    pub fn build(self) -> Manager {
        let Self {
            issuer,
            key,
            expires_in,
        } = self;
        let enc_key = EncodingKey::from_secret(key.as_bytes());
        let dec_key = DecodingKey::from_secret(key.as_bytes());
        let inner = Inner {
            issuer,
            raw_key: key,
            enc_key,
            dec_key,
            expires_in,
        };
        Manager {
            inner: Arc::new(inner),
        }
    }
}

impl Manager {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub const ALGORITHM: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS256;

    pub fn encode(&self, value: Value) -> Result<Encoded, EncodingError> {
        let iat = jsonwebtoken::get_current_timestamp();
        let exp = iat + self.inner.expires_in.as_secs();
        let iss = self.inner.issuer.clone();
        let claims = Claims {
            exp,
            iat,
            iss,
            custom: value,
        };
        self.encode_claims(&claims)
    }

    pub fn encode_claims(&self, claims: &Claims) -> Result<Encoded, EncodingError> {
        use jsonwebtoken::{encode, Header};

        let header = Header::new(Self::ALGORITHM);
        let encoded = encode(&header, claims, &self.inner.enc_key)?;
        Ok(Encoded(encoded))
    }

    pub fn decode(&self, token: &str) -> Result<Decoded, DecodingError> {
        use jsonwebtoken::{decode, TokenData, Validation};

        let validation = Validation::new(Self::ALGORITHM);
        let TokenData { claims, .. } = decode(token, &self.inner.dec_key, &validation)?;
        Ok(Decoded(claims))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Encoded(String);

impl fmt::Display for Encoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Encoded {
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Decoded(Claims);

impl Decoded {
    pub fn into_inner(self) -> Claims {
        self.0
    }
}
