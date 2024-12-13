#![allow(dead_code)]

use bytes::Bytes;
use serde::de::DeserializeOwned;
use warp::{http, hyper, reject, reply::Reply, Filter, Rejection};

use super::reject::InvalidBodyEncoding;

#[derive(Debug, thiserror::Error)]
#[error("could not deserialize request body as toml")]
pub struct RejectToml {
    #[from]
    source: toml::de::Error,
}

impl reject::Reject for RejectToml {}

impl RejectToml {
    pub fn wrap_into_reject(source: toml::de::Error) -> Rejection {
        reject::custom(Self::from(source))
    }

    pub async fn recover_with<F>(&self, message: F) -> http::Response<hyper::Body>
    where
        F: FnOnce(&Self) -> String,
    {
        {
            let error = self as &(dyn std::error::Error);
            tracing::error!(error);
        }
        let message = message(self);
        http::Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body(hyper::Body::from(message))
            .unwrap()
    }

    pub async fn recover(&self) -> http::Response<hyper::Body> {
        self.recover_with(|s| format!("{s}")).await
    }
}

pub fn header() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::filters::header::exact("content-type", "application/toml")
}

pub fn body<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: DeserializeOwned + Send,
{
    warp::filters::body::bytes().and_then(deserialize_toml::<T>)
}

async fn deserialize_toml<T>(body: Bytes) -> Result<T, Rejection>
where
    T: DeserializeOwned + Send,
{
    let s = std::str::from_utf8(&body).map_err(InvalidBodyEncoding::wrap_into_reject)?;
    let t = toml::from_str(s).map_err(RejectToml::wrap_into_reject)?;
    Ok(t)
}

pub async fn recover(error: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(e) = error.find::<InvalidBodyEncoding>() {
        let reply = e.recover().await;
        return Ok(reply);
    }
    if let Some(e) = error.find::<RejectToml>() {
        let reply = e.recover().await;
        return Ok(reply);
    }
    Err(error)
}

pub fn toml_body<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: DeserializeOwned + Send,
{
    Filter::and(header(), body::<T>())
}
