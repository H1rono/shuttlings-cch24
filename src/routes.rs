use std::convert::Infallible;
use std::sync::Arc;

use warp::{http, hyper, Filter, Reply};

mod state;

#[derive(Debug, Clone)]
pub struct State {
    inner: Arc<state::Inner>,
}

pub fn make(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    hello_bird(state.clone()).or(seek(state))
}

fn with_state(state: State) -> impl Filter<Extract = (State,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

pub fn hello_bird(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!()
        .and(warp::get())
        .and(with_state(state))
        .map(|_s| "Hello, bird!")
}

pub fn seek(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!("-1" / "seek")
        .and(warp::get())
        .and(with_state(state))
        .and_then(seek_fn)
}

async fn seek_fn(state: State) -> Result<http::Response<hyper::Body>, Infallible> {
    let res = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, state.inner.seek_url.clone())
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}
