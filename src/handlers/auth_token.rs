use serde_json::Value;
use warp::http;

use crate::{cookie, jwt};

#[derive(Clone)]
pub struct State {
    pub(super) jwt_manager: jwt::Manager,
    pub(super) cookie_manager: cookie::Manager,
    pub(super) decoder: jwt::Decoder,
}

pub struct Builder<JwtManager = (), CookieManager = (), Decoder = ()> {
    jwt_manager: JwtManager,
    cookie_manager: CookieManager,
    decoder: Decoder,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            jwt_manager: (),
            cookie_manager: (),
            decoder: (),
        }
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<JwtManager, CookieManager, Decoder> Builder<JwtManager, CookieManager, Decoder> {
    pub fn jwt_manager(self, value: jwt::Manager) -> Builder<jwt::Manager, CookieManager, Decoder> {
        let Self {
            cookie_manager,
            decoder,
            ..
        } = self;
        Builder {
            jwt_manager: value,
            cookie_manager,
            decoder,
        }
    }

    pub fn cookie_manager(
        self,
        value: cookie::Manager,
    ) -> Builder<JwtManager, cookie::Manager, Decoder> {
        let Self {
            jwt_manager,
            decoder,
            ..
        } = self;
        Builder {
            jwt_manager,
            cookie_manager: value,
            decoder,
        }
    }

    pub fn decoder(self, value: jwt::Decoder) -> Builder<JwtManager, CookieManager, jwt::Decoder> {
        let Self {
            jwt_manager,
            cookie_manager,
            ..
        } = self;
        Builder {
            jwt_manager,
            cookie_manager,
            decoder: value,
        }
    }
}

impl Builder<jwt::Manager, cookie::Manager, jwt::Decoder> {
    pub fn build(self) -> State {
        let Self {
            jwt_manager,
            cookie_manager,
            decoder,
        } = self;
        State {
            jwt_manager,
            cookie_manager,
            decoder,
        }
    }
}

pub(super) async fn unwrap_cookie_from_headers(
    state: &State,
    headers: &http::HeaderMap,
) -> anyhow::Result<Value> {
    let Some(cookie) = headers.get(http::header::COOKIE) else {
        anyhow::bail!("Received no cookie header");
    };
    let cookie = cookie.to_str()?;
    let jwt = state.cookie_manager.from_header_value(cookie)?;
    let claims = state.jwt_manager.decode(&jwt)?.into_inner();
    Ok(claims.custom)
}
