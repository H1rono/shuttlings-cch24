use std::{str::FromStr, sync::Arc};

use warp::{http, hyper, Filter, Reply};

use crate::handlers;

mod json;
mod reject;
mod state;
mod toml;

use self::reject::InvalidBodyEncoding;

#[derive(Clone)]
pub struct State {
    seek: Arc<handlers::seek::State>,
    manifest: Arc<handlers::manifest::State>,
    milk: Arc<handlers::milk::State>,
    connect4: Arc<handlers::connect4::State>,
    auth_token: Arc<handlers::auth_token::State>,
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
        .or(refill_milk(state.clone()))
        .or(connect4_board(state.clone()))
        .or(connect4_reset(state.clone()))
        .or(connect4_place(state.clone()))
        .or(connect4_random_board(state.clone()))
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
        .and(json::header())
        .and(warp::body::bytes())
        .and_then(|m, b| async move {
            use handlers::milk::Error;
            match handlers::convert_milk_unit(m, b).await {
                Ok(res) => Ok(res),
                Err(Error::Utf8Error(e)) => Err(InvalidBodyEncoding::wrap_into_reject(e)),
                Err(Error::JsonError(e)) => Err(json::RejectJson::wrap_into_reject(e)),
            }
        })
        .recover(json::recover);
    let s = state.clone();
    let request_milk = warp::any()
        .map(move || Arc::clone(&s.milk))
        .and_then(handlers::request_milk);
    warp::path!("9" / "milk")
        .and(warp::post())
        .and(Filter::or(convert_unit, request_milk))
}

fn refill_milk(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::path!("9" / "refill")
        .and(warp::post())
        .map(move || Arc::clone(&state.milk))
        .and_then(handlers::refill_milk)
}

fn connect4_board(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { connect4, .. } = state;
    warp::path!("12" / "board")
        .and(warp::get())
        .map(move || Arc::clone(&connect4))
        .and_then(handlers::connect4_board)
}

fn connect4_reset(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { connect4, .. } = state;
    warp::path!("12" / "reset")
        .and(warp::post())
        .map(move || Arc::clone(&connect4))
        .and_then(handlers::connect4_reset)
}

fn connect4_place(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum Team {
        Cookie,
        Milk,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("<Team as FromStr>::Err")]
    struct TeamFromStrError;

    impl FromStr for Team {
        type Err = TeamFromStrError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "cookie" => Ok(Self::Cookie),
                "milk" => Ok(Self::Milk),
                _ => Err(TeamFromStrError),
            }
        }
    }

    impl From<Team> for crate::connect4::Team {
        fn from(value: Team) -> Self {
            match value {
                Team::Cookie => Self::Cookie,
                Team::Milk => Self::Milk,
            }
        }
    }

    let State { connect4, .. } = state;
    warp::path!("12" / "place" / String / String)
        .map(move |t: String, c: String| {
            (
                Arc::clone(&connect4),
                Team::from_str(&t),
                usize::from_str(&c),
            )
        })
        .untuple_one()
        .and_then(|connect4, t, c| async move {
            let (team, col) = match (t, c) {
                (Ok(t), Ok(c)) => (crate::connect4::Team::from(t), usize::wrapping_sub(c, 1)),
                e => {
                    tracing::info!("bad request: {e:?}");
                    let res = http::Response::builder()
                        .status(http::StatusCode::BAD_REQUEST)
                        .body(hyper::Body::empty())
                        .unwrap();
                    return Ok(res);
                }
            };
            let param = handlers::connect4::PlacePathParam::new(team, col);
            handlers::connect4_place(connect4, param).await
        })
}

fn connect4_random_board(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { connect4, .. } = state;
    warp::path!("12" / "random-board")
        .and(warp::get())
        .map(move || Arc::clone(&connect4))
        .and_then(handlers::connect4_random_board)
}
