use std::convert::Infallible;
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
        .map(move || Arc::clone(&state.seek))
        .and_then(handlers::seek)
}
