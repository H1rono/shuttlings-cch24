use std::sync::Arc;

use warp::{Filter, Reply};

use crate::handlers;

mod json;
mod reject;
mod state;
mod toml;

use self::reject::InvalidBodyEncoding;

#[derive(Debug, Clone)]
pub struct State {
    seek: Arc<handlers::seek::State>,
    manifest: Arc<handlers::manifest::State>,
    milk: Arc<handlers::milk::State>,
}

pub fn make(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    hello_bird(state.clone())
        .or(seek(state.clone()))
        .or(ipv4_dest(state.clone()))
        .or(ipv4_key(state.clone()))
        .or(ipv6_dest(state.clone()))
        .or(ipv6_key(state.clone()))
        .or(manifest_order(state.clone()))
        .or(milk_factory(state.clone()))
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

fn ipv4_key(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let query = warp::query::<handlers::ipv4_key::Query>();
    warp::path!("2" / "key")
        .and(warp::get())
        .and(query)
        .and_then(handlers::ipv4_key)
}

fn ipv6_dest(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let query = warp::query::<handlers::ipv6_dest::Query>();
    warp::path!("2" / "v6" / "dest")
        .and(warp::get())
        .and(query)
        .and_then(handlers::ipv6_dest)
}

fn ipv6_key(
    _state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let query = warp::query::<handlers::ipv6_key::Query>();
    warp::path!("2" / "v6" / "key")
        .and(warp::get())
        .and(query)
        .and_then(handlers::ipv6_key)
}

fn manifest_order(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!("5" / "manifest")
        .and(warp::post())
        .map(move || Arc::clone(&state.manifest))
        .and(self::toml::toml_body())
        .and_then(handlers::manifest_order)
        .recover(|r: warp::Rejection| async move {
            use self::toml::RejectToml;
            if let Some(e) = r.find::<InvalidBodyEncoding>() {
                let reply = e.recover_with(|_| "Invalid manifest".to_string()).await;
                return Ok(reply);
            }
            if let Some(e) = r.find::<RejectToml>() {
                let reply = e.recover_with(|_| "Invalid manifest".to_string()).await;
                return Ok(reply);
            }
            Err(r)
        })
}

fn milk_factory(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let s = state.clone();
    let convert_unit = warp::any()
        .map(move || Arc::clone(&s.milk))
        .and(json::json_body::<handlers::milk::Unit>())
        .and_then(handlers::convert_milk_unit)
        .recover(json::recover);
    let s = state.clone();
    let request_milk = warp::any()
        .map(move || Arc::clone(&s.milk))
        .and_then(handlers::request_milk);
    warp::path!("9" / "milk")
        .and(warp::post())
        .and(Filter::or(convert_unit, request_milk))
}
