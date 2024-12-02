use std::{convert::Infallible, sync::Arc};

use warp::{http, hyper};

pub(crate) mod seek;

pub async fn seek(state: Arc<seek::State>) -> Result<http::Response<hyper::Body>, Infallible> {
    let res = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, state.seek_url.clone())
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}
