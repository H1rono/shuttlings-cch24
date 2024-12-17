use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, TimeDelta, Utc};

use super::Manager;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Secure;

impl fmt::Display for Secure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secure")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SameSite {
    Strict,
    Lax,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lax => f.write_str("Lax"),
            Self::Strict => f.write_str("Strict"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct HttpOnly;

impl fmt::Display for HttpOnly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HttpOnly")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecureSameSite {
    Both(SameSite, Secure),
    SameSite(SameSite),
    SameSiteNone(Secure),
    Secure(Secure),
}

impl fmt::Display for SecureSameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Both(ss, s) => write!(f, "SameSite={ss}; {s}"),
            Self::SameSite(ss) => write!(f, "SameSite={ss}"),
            Self::SameSiteNone(s) => write!(f, "SameSite=None; {s}"),
            Self::Secure(s) => fmt::Display::fmt(s, f),
        }
    }
}

impl SecureSameSite {
    fn secure(self) -> Self {
        match self {
            Self::SameSite(ss) => Self::Both(ss, Secure),
            sss => sss,
        }
    }

    fn secure_option(s: Option<Self>) -> Self {
        match s {
            None => Self::Secure(Secure),
            Some(slf) => slf.secure(),
        }
    }

    fn same_site(self, ss: SameSite) -> Self {
        match self {
            Self::Both(_, s) | Self::SameSiteNone(s) | Self::Secure(s) => Self::Both(ss, s),
            Self::SameSite(_) => Self::SameSite(ss),
        }
    }

    fn same_site_option(s: Option<Self>, ss: SameSite) -> Self {
        match s {
            None => Self::SameSite(ss),
            Some(slf) => slf.same_site(ss),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Lifetime {
    Expires(DateTime<Utc>),
    MaxAge(TimeDelta),
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Expires(expires) => {
                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Date
                // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
                let e = expires.format("%a, %d %b %Y %H:%M:%S GMT");
                write!(f, "Expires={e}")
            }
            Self::MaxAge(max_age) => {
                let max_age = max_age.num_seconds();
                write!(f, "Max-Age={max_age}")
            }
        }
    }
}

pub(super) struct Inner {
    name: String,
    path: Option<String>,
    secure_same_site: Option<SecureSameSite>,
    http_only: Option<HttpOnly>,
    domain: Option<String>,
    lifetime: Option<Lifetime>,
}

#[derive(Debug, Default, Clone)]
pub struct Builder<Name = ()> {
    name: Name,
    path: Option<String>,
    secure_same_site: Option<SecureSameSite>,
    http_only: Option<HttpOnly>,
    domain: Option<String>,
    lifetime: Option<Lifetime>,
}

impl<Name> Builder<Name> {
    pub fn name<'a, S>(self, value: S) -> Builder<String>
    where
        S: Into<Cow<'a, str>>,
    {
        Builder {
            name: value.into().into_owned(),
            path: self.path,
            secure_same_site: self.secure_same_site,
            http_only: self.http_only,
            domain: self.domain,
            lifetime: self.lifetime,
        }
    }

    pub fn path<'a, S>(self, value: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            path: Some(value.into().into_owned()),
            ..self
        }
    }

    pub fn secure(self) -> Self {
        let s = SecureSameSite::secure_option(self.secure_same_site);
        Self {
            secure_same_site: Some(s),
            ..self
        }
    }

    pub fn same_site_none(self) -> Self {
        let s = SecureSameSite::SameSiteNone(Secure);
        Self {
            secure_same_site: Some(s),
            ..self
        }
    }

    pub fn same_site_strict(self) -> Self {
        let s = SecureSameSite::same_site_option(self.secure_same_site, SameSite::Strict);
        Self {
            secure_same_site: Some(s),
            ..self
        }
    }

    pub fn same_site_lax(self) -> Self {
        let s = SecureSameSite::same_site_option(self.secure_same_site, SameSite::Lax);
        Self {
            secure_same_site: Some(s),
            ..self
        }
    }

    pub fn http_only(self) -> Self {
        Self {
            http_only: Some(HttpOnly),
            ..self
        }
    }

    pub fn domain<'a, S>(self, value: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            domain: Some(value.into().into_owned()),
            ..self
        }
    }

    pub fn expires(self, value: DateTime<Utc>) -> Self {
        Self {
            lifetime: Some(Lifetime::Expires(value)),
            ..self
        }
    }

    pub fn max_age(self, value: TimeDelta) -> Self {
        Self {
            lifetime: Some(Lifetime::MaxAge(value)),
            ..self
        }
    }
}

impl Builder<String> {
    pub fn build(self) -> Manager {
        let Self {
            name,
            path,
            secure_same_site,
            http_only,
            domain,
            lifetime,
        } = self;
        let inner = Inner {
            name,
            path,
            secure_same_site,
            http_only,
            domain,
            lifetime,
        };
        Manager {
            inner: Arc::new(inner),
        }
    }
}

pub struct ToHeaderValue<'a> {
    manager: &'a Manager,
    value: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum PercentDecodeError {
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error("No matching cookie entry found")]
    NoMatchingItemFound,
}

impl fmt::Display for ToHeaderValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! attr {
            ( Some($n:ident) => $e:expr ) => {
                if let Some($n) = $n {
                    $e;
                }
            };

            ( $(Some($n:ident) => $e:expr;)+ ) => {
                $( attr! { Some($n) => $e } )+
            };
        }

        // FIXME: apply percent_encoding
        // use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        let Self { manager, value } = self;
        let Inner {
            name,
            path,
            secure_same_site,
            http_only,
            domain,
            lifetime,
        } = &*manager.inner;
        // let value = utf8_percent_encode(value, NON_ALPHANUMERIC);
        write!(f, "{name}={value}")?;
        attr! {
            Some(path) => write!(f, "; Path={path}")?;
            Some(secure_same_site) => write!(f, "; {secure_same_site}")?;
            Some(http_only) => write!(f, "; {http_only}")?;
            Some(domain) => write!(f, "; Domain={domain}")?;
            Some(lifetime) => write!(f, "; {lifetime}")?;
        }
        Ok(())
    }
}

impl Manager {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn to_header_value<'a>(&'a self, value: &'a str) -> ToHeaderValue<'a> {
        ToHeaderValue {
            manager: self,
            value,
        }
    }

    /// **NOTE**: Cookie lifetime is not checked
    pub fn from_header_value(&self, value: &str) -> Result<String, PercentDecodeError> {
        let v = value
            .split(";")
            .flat_map(|i| i.trim().split_once('='))
            .find_map(|(k, v)| (k == self.inner.name).then_some(v))
            .ok_or(PercentDecodeError::NoMatchingItemFound)?;
        Ok(v.to_string())
        // FIXME: apply percent_encoding
        // let v = percent_encoding::percent_decode_str(v).decode_utf8()?;
        // Ok(v.into_owned())
    }
}
