use std::convert::Infallible;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::ControlFlow;
use std::sync::Arc;

use warp::{http, hyper};

use crate::bucket::Liters;

// MARK: mod

pub(crate) mod auth_token;
pub(crate) mod connect4;
pub(crate) mod ipv4_dest;
pub(crate) mod ipv4_key;
pub(crate) mod ipv6_dest;
pub(crate) mod ipv6_key;
pub(crate) mod manifest;
pub(crate) mod milk;
pub(crate) mod quotes;
pub(crate) mod seek;

type Response<B = hyper::Body> = http::Response<B>;

// MARK: seek

pub async fn seek(state: Arc<seek::State>) -> Result<Response, Infallible> {
    let res = http::Response::builder()
        .status(http::StatusCode::FOUND)
        .header(http::header::LOCATION, state.seek_url.clone())
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}

// MARK: ipv4

macro_rules! ipv4_octets_zip_with {
    ($f:path => ($l:expr, $r:expr)) => {
        [
            $f($l[0], $r[0]),
            $f($l[1], $r[1]),
            $f($l[2], $r[2]),
            $f($l[3], $r[3]),
        ]
    };
}

pub async fn ipv4_dest(query: ipv4_dest::Query) -> Result<Response, Infallible> {
    let (from, key) = query.octets();
    let dest = ipv4_octets_zip_with!(u8::wrapping_add => (from, key));
    let dest = Ipv4Addr::from(dest);
    let body = hyper::Body::from(format!("{dest}"));
    let res = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

pub async fn ipv4_key(query: ipv4_key::Query) -> Result<Response, Infallible> {
    let (from, to) = query.octets();
    let key = ipv4_octets_zip_with!(u8::wrapping_sub => (to, from));
    let key = Ipv4Addr::from(key);
    let body = hyper::Body::from(format!("{key}"));
    let res = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

// MARK: ipv6

pub async fn ipv6_dest(query: ipv6_dest::Query) -> Result<Response, Infallible> {
    let (from, key) = query.to_bits();
    let dest = from ^ key;
    let dest = Ipv6Addr::from_bits(dest);
    let body = hyper::Body::from(format!("{dest}"));
    let res = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

pub async fn ipv6_key(query: ipv6_key::Query) -> Result<Response, Infallible> {
    let (from, to) = query.to_bits();
    let key = to ^ from;
    let key = Ipv6Addr::from_bits(key);
    let body = hyper::Body::from(format!("{key}"));
    let res = http::Response::builder()
        .status(http::StatusCode::OK)
        .body(body)
        .unwrap();
    Ok(res)
}

// MARK: manifest

pub async fn manifest_order(
    state: Arc<manifest::State>,
    manifest: manifest::Manifest,
) -> Result<Response, Infallible> {
    use manifest::ProperOrder;

    if !manifest::manifest_key_included(&state, &manifest) {
        let body = "Magic keyword not provided".to_string();
        let res = Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body(hyper::Body::from(body))
            .unwrap();
        return Ok(res);
    }
    tracing::info!(?manifest);
    let orders = manifest
        .package
        .as_ref()
        .and_then(|p| p.metadata.as_ref())
        .and_then(ProperOrder::from_value)
        .unwrap_or_default();
    let orders = orders
        .iter()
        .map(ProperOrder::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    let status = if orders.is_empty() {
        http::StatusCode::NO_CONTENT
    } else {
        http::StatusCode::OK
    };
    let body = hyper::Body::from(orders);
    let res = Response::builder().status(status).body(body).unwrap();
    Ok(res)
}

// MARK: milk factory

pub async fn request_milk(state: Arc<milk::State>) -> Result<Response, Infallible> {
    let flow = milk::check_bucket(Arc::clone(&state)).await;
    let _ = state.bucket.withdraw_by(Liters(1.0)).await;
    if let ControlFlow::Break(res) = flow {
        tracing::error!("rate limit reached");
        return Ok(res);
    }
    let body = hyper::Body::from("Milk withdrawn\n".to_string());
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(body)
        .unwrap();
    Ok(res)
}

pub async fn convert_milk_unit(
    state: Arc<milk::State>,
    request: bytes::Bytes,
) -> Result<Response, milk::Error> {
    let flow = milk::check_bucket(Arc::clone(&state)).await;
    let _ = state.bucket.withdraw_by(Liters(1.0)).await;
    if let ControlFlow::Break(res) = flow {
        tracing::error!("rate limit reached");
        return Ok(res);
    }
    let request = std::str::from_utf8(&request)?;
    let request: milk::Unit = serde_json::from_str(request)?;
    let response = request.convert();
    let body = serde_json::to_string(&response);
    let (status, content_type, body) = match body {
        Ok(b) => (
            http::StatusCode::OK,
            "application/json",
            hyper::Body::from(b),
        ),
        Err(e) => {
            let err = &e as &dyn std::error::Error;
            tracing::error!(err, "failed to serialize unit {request:?}");
            (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "plain/text",
                hyper::Body::empty(),
            )
        }
    };
    let res = Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, content_type)
        .body(body)
        .unwrap();
    Ok(res)
}

pub async fn refill_milk(state: Arc<milk::State>) -> Result<Response, Infallible> {
    state.bucket.fulfill().await;
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}

// MARK: connect4

pub async fn connect4_board(state: Arc<connect4::State>) -> Result<Response, Infallible> {
    let game = state.game.lock().await;
    let body = game.display_with_status().to_string();
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

pub async fn connect4_reset(state: Arc<connect4::State>) -> Result<Response, Infallible> {
    let mut game = state.game.lock().await;
    game.reset();
    let body = game.display_with_status().to_string();
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

#[tracing::instrument(skip(state))]
pub async fn connect4_place(
    state: Arc<connect4::State>,
    param: connect4::PlacePathParam,
) -> Result<Response, Infallible> {
    use crate::connect4::GameError;

    let connect4::PlacePathParam { team, col } = param;
    let mut game = state.game.lock().await;
    let (status, body) = if let Err(err) = game.pile(team, col) {
        tracing::info!(err = &err as &dyn std::error::Error, "placement failed");
        match err {
            e @ GameError::InvalidColumn(_) => (http::StatusCode::BAD_REQUEST, e.to_string()),
            _ => (
                http::StatusCode::SERVICE_UNAVAILABLE,
                game.display_with_status().to_string(),
            ),
        }
    } else {
        tracing::info!("placed successfully");
        (http::StatusCode::OK, game.display_with_status().to_string())
    };
    let res = http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

pub async fn connect4_random_board(state: Arc<connect4::State>) -> Result<Response, Infallible> {
    let mut game = state.game.lock().await;
    game.random_board();
    let body = game.display_with_status().to_string();
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "plain/text")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

// MARK: jwt

#[tracing::instrument(skip_all)]
pub async fn jwt_wrap(
    state: Arc<auth_token::State>,
    payload: serde_json::Value,
) -> Result<Response, Infallible> {
    let auth_token::State {
        jwt_manager,
        cookie_manager,
        ..
    } = &*state;
    let jwt = match jwt_manager.encode(payload) {
        Ok(jwt) => jwt,
        Err(e) => {
            tracing::error!(err = &e as &dyn std::error::Error, "failed to encode JWT");
            let res = Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(hyper::Body::empty())
                .unwrap();
            return Ok(res);
        }
    };
    tracing::info!("successfully encoded JWT");
    let jwt = jwt.into_inner();
    let set_cookie = cookie_manager.to_header_value(&jwt);
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::SET_COOKIE, set_cookie.to_string())
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn jwt_unwrap(
    state: Arc<auth_token::State>,
    headers: http::HeaderMap,
) -> Result<Response, Infallible> {
    let value = auth_token::unwrap_cookie_from_headers(&state, &headers).await;
    let value = match value {
        Ok(v) => v,
        Err(e) => {
            tracing::info!(err = %e);
            let res = Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body(hyper::Body::empty())
                .unwrap();
            return Ok(res);
        }
    };
    tracing::info!("successfully decoded JWT from cookie");
    let body = serde_json::to_string(&value).unwrap();
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn jwt_decode(
    state: Arc<auth_token::State>,
    body: bytes::Bytes,
) -> Result<Response, Infallible> {
    use crate::jwt::DecoderError;
    use jsonwebtoken::errors::ErrorKind as JwtErrorKind;

    let value = match auth_token::decode_with_pem(&state, body).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(?e);
            let status = match e.downcast_ref::<DecoderError>() {
                Some(e @ DecoderError::LoadKeyFailed(_)) => {
                    tracing::error!(err = e as &dyn std::error::Error);
                    http::StatusCode::INTERNAL_SERVER_ERROR
                }
                Some(DecoderError::DecodePayloadFailed(e)) => {
                    if let JwtErrorKind::InvalidSignature = e.kind() {
                        http::StatusCode::UNAUTHORIZED
                    } else {
                        http::StatusCode::BAD_REQUEST
                    }
                }
                _ => http::StatusCode::BAD_REQUEST,
            };
            let res = Response::builder()
                .status(status)
                .body(hyper::Body::empty())
                .unwrap();
            return Ok(res);
        }
    };
    tracing::info!("successfully decoded with pem");
    let body = serde_json::to_string(&value).unwrap();
    let res = Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(hyper::Body::from(body))
        .unwrap();
    Ok(res)
}

// MARK: quotes

#[tracing::instrument(skip_all)]
pub async fn quotes_reset(state: Arc<quotes::State>) -> Result<Response, Infallible> {
    let status = match state.repository.reset().await {
        Ok(()) => {
            tracing::info!("quotes reseted");
            http::StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                err = &e as &dyn std::error::Error,
                "resetting quotes failed"
            );
            http::StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    let res = Response::builder()
        .status(status)
        .body(hyper::Body::empty())
        .unwrap();
    Ok(res)
}

#[tracing::instrument(skip(state))]
pub async fn quotes_cite(
    state: Arc<quotes::State>,
    param: quotes::CitePathParam,
) -> Result<Response, Infallible> {
    let res = match quotes::find_and_serialize_cite(&state, param).await {
        Ok(body) => Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(body))
            .unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(hyper::Body::empty())
            .unwrap(),
    };
    Ok(res)
}

#[tracing::instrument(skip(state))]
pub async fn quotes_remove(
    state: Arc<quotes::State>,
    param: quotes::RemovePathParam,
) -> Result<Response, Infallible> {
    let quotes::RemovePathParam { id } = param;
    let res = match state.repository.delete_one(id).await {
        Ok(Some(quote)) => {
            tracing::info!("Removed one quote");
            serde_json::to_string(&quote).map_err(|e| {
                tracing::error!(
                    err = &e as &dyn std::error::Error,
                    "Failed to serialize quote"
                );
                http::StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        Ok(None) => {
            tracing::info!("No matching quote found");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!(
                err = &e as &dyn std::error::Error,
                "Failed to delete one quote"
            );
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    let res = match res {
        Ok(body) => Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(body))
            .unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(hyper::Body::empty())
            .unwrap(),
    };
    Ok(res)
}

#[tracing::instrument(skip(state, body))]
pub async fn quotes_undo(
    state: Arc<quotes::State>,
    param: quotes::UndoPathParam,
    body: quotes::UndoBody,
) -> Result<Response, Infallible> {
    let res = match state.undo_aka_update(param, body).await {
        Ok(Some(quote)) => {
            tracing::info!("Updated one quote");
            serde_json::to_string(&quote).map_err(|e| {
                tracing::error!(
                    err = &e as &dyn std::error::Error,
                    ?quote,
                    "Failed to serialize quote"
                );
                http::StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        Ok(None) => {
            tracing::info!("No matching quote found");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!(
                err = &e as &dyn std::error::Error,
                "Failed to update one quote"
            );
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    let res = match res {
        Ok(body) => Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(body))
            .unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(hyper::Body::empty())
            .unwrap(),
    };
    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn quotes_draft(
    state: Arc<quotes::State>,
    body: quotes::DraftBody,
) -> Result<Response, Infallible> {
    let res = match state.repository.create(body.into()).await {
        Ok(quote) => {
            tracing::info!("Created one quote");
            serde_json::to_string(&quote).map_err(|e| {
                tracing::error!(
                    err = &e as &dyn std::error::Error,
                    ?quote,
                    "Failed to serialize quote"
                );
                http::StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        Err(e) => {
            tracing::error!(
                err = &e as &dyn std::error::Error,
                "Failed to create a quote"
            );
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    let res = match res {
        Ok(body) => Response::builder()
            .status(http::StatusCode::CREATED)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(body))
            .unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(hyper::Body::empty())
            .unwrap(),
    };
    Ok(res)
}

#[tracing::instrument(skip_all)]
pub async fn quotes_list(
    state: Arc<quotes::State>,
    query: quotes::ListQuery,
) -> Result<Response, Infallible> {
    use crate::quotes::ops::ListError;

    let quotes::ListQuery { token: next_token } = query;
    let res = match state.repository.list(next_token.as_deref()).await {
        Ok(Some(b)) => {
            tracing::info!("Listed quotes");
            serde_json::to_string(&b).map_err(|e| {
                tracing::error!(
                    err = &e as &dyn std::error::Error,
                    "Failed to serialize response"
                );
                http::StatusCode::INTERNAL_SERVER_ERROR
            })
        }
        Ok(None) => {
            tracing::info!(next_token, "No matching quote against next_token found");
            Err(http::StatusCode::BAD_REQUEST)
        }
        Err(ListError::Token(e)) => {
            tracing::info!(
                err = &e as &dyn std::error::Error,
                "Bad next_token provided"
            );
            Err(http::StatusCode::BAD_REQUEST)
        }
        Err(ListError::Database(e)) => {
            tracing::error!(err = &e as &dyn std::error::Error, "Failed to list quotes");
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
    let res = match res {
        Ok(body) => Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(hyper::Body::from(body))
            .unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(hyper::Body::empty())
            .unwrap(),
    };
    Ok(res)
}
