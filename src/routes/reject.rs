use std::str::Utf8Error;

use warp::{http, hyper, reject, Rejection};

#[derive(Debug, thiserror::Error)]
#[error("non-utf8 body encoding")]
pub struct InvalidBodyEncoding {
    #[from]
    source: Utf8Error,
}

impl reject::Reject for InvalidBodyEncoding {}

impl InvalidBodyEncoding {
    pub fn wrap_into_reject(source: Utf8Error) -> Rejection {
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
        self.recover_with(|_| String::new()).await
    }
}
