use warp::{Filter, Reply};

pub fn make() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> {
    hello_bird()
}

pub fn hello_bird() -> impl Filter<Extract = (&'static str,), Error = warp::Rejection> {
    warp::get().map(|| "Hello, bird!")
}
