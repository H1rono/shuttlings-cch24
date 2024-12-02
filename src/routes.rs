use std::sync::Arc;

use warp::{Filter, Reply};

use crate::handlers;

mod state;

#[derive(Debug, Clone)]
pub struct State {
    seek: Arc<handlers::seek::State>,
}

pub fn make(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    hello_bird(state.clone()).or(seek(state))
}

pub fn hello_bird(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!().and(warp::get()).map(|| "Hello, bird!")
}

pub fn seek(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!("-1" / "seek")
        .and(warp::get())
        .map(move || Arc::clone(&state.seek))
        .and_then(handlers::seek)
}
