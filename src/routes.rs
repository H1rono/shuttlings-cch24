use std::error::Error as StdError;
use std::str::FromStr;
use std::sync::Arc;

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
    quotes: Arc<handlers::quotes::State>,
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
        .or(jwt_wrap(state.clone()))
        .or(jwt_unwrap(state.clone()))
        .or(jwt_decode(state.clone()))
        .or(quotes(state.clone()))
        .with(warp::filters::trace::request())
}

macro_rules! error_bad_request {
    (
        $result:expr;
        $($p:pat => $ok:expr),+
    ) => {
        match $result {
            $( $p => $ok, )+
            Err(e) => {
                tracing::info!(err = &e as &dyn StdError, "bad request");
                let res = http::Response::builder()
                    .status(http::StatusCode::BAD_REQUEST)
                    .body(hyper::Body::empty())
                    .unwrap();
                return Ok(res);
            }
        }
    };
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

fn jwt_wrap(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { auth_token, .. } = state;
    warp::path!("16" / "wrap")
        .and(warp::post())
        .map(move || Arc::clone(&auth_token))
        .and(json::json_body::<serde_json::Value>())
        .and_then(handlers::jwt_wrap)
}

fn jwt_unwrap(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { auth_token, .. } = state;
    warp::path!("16" / "unwrap")
        .and(warp::get())
        .map(move || Arc::clone(&auth_token))
        .and(warp::header::headers_cloned())
        .and_then(handlers::jwt_unwrap)
}

fn jwt_decode(
    state: State,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { auth_token, .. } = state;
    warp::path!("16" / "decode")
        .and(warp::post())
        .map(move || Arc::clone(&auth_token))
        .and(warp::body::bytes())
        .and_then(handlers::jwt_decode)
}

fn quotes(state: State) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    let State { quotes, .. } = state;
    let use_state = warp::any().map(move || Arc::clone(&quotes));
    let reset = warp::path!("19" / "reset")
        .and(warp::post())
        .and(use_state.clone())
        .and_then(handlers::quotes_reset);
    let cite = warp::path!("19" / "cite" / String)
        .and(warp::get())
        .map(|id: String| id.parse().map(handlers::quotes::CitePathParam::new))
        .and(use_state.clone())
        .and_then(|param, state| async move {
            error_bad_request!(
                param;
                Ok(p) => handlers::quotes_cite(state, p).await
            )
        });
    let remove = warp::path!("19" / "remove" / String)
        .and(warp::delete())
        .map(|id: String| id.parse().map(handlers::quotes::RemovePathParam::new))
        .and(use_state.clone())
        .and_then(|param, state| async move {
            error_bad_request!(
                param;
                Ok(p) => handlers::quotes_remove(state, p).await
            )
        });
    let undo = warp::path!("19" / "undo" / String)
        .and(warp::put())
        .map(|id: String| id.parse().map(handlers::quotes::UndoPathParam::new))
        .and(use_state.clone())
        .and(json::json_body::<handlers::quotes::UndoBody>())
        .and_then(|param, state, body| async move {
            error_bad_request!(
                param;
                Ok(p) => handlers::quotes_undo(state, p, body).await
            )
        });
    let draft = warp::path!("19" / "draft")
        .and(warp::post())
        .and(use_state.clone())
        .and(json::json_body::<handlers::quotes::DraftBody>())
        .and_then(handlers::quotes_draft);
    reset.or(cite).or(remove).or(undo).or(draft)
}
