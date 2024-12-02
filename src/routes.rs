use std::sync::Arc;

use warp::{Filter, Reply};

use crate::handlers;

mod state;

#[derive(Debug, Clone)]
pub struct State {
    seek: Arc<handlers::seek::State>,
}

pub fn make(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    hello_bird(state.clone())
        .or(seek(state.clone()))
        .or(ipv4_dest(state))
}

fn hello_bird(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!().and(warp::get()).map(|| "Hello, bird!")
}

fn seek(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!("-1" / "seek")
        .and(warp::get())
        .map(move || Arc::clone(&state.seek))
        .and_then(handlers::seek)
}

fn ipv4_dest(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let query = warp::query::<handlers::ipv4_dest::Query>();
    warp::path!("2" / "dest")
        .and(warp::get())
        .and(query)
        .and_then(handlers::ipv4_dest)
}
