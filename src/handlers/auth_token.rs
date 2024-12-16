use serde_json::Value;
use warp::http;

use crate::{cookie, jwt};

#[derive(Clone)]
pub struct State {
    pub(super) jwt_manager: jwt::Manager,
    pub(super) cookie_manager: cookie::Manager,
}

pub struct Builder<JwtManager = (), CookieManager = ()> {
    jwt_manager: JwtManager,
    cookie_manager: CookieManager,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            jwt_manager: (),
            cookie_manager: (),
        }
    }
}

impl State {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl<JwtManager, CookieName> Builder<JwtManager, CookieName> {
    pub fn jwt_manager(self, value: jwt::Manager) -> Builder<jwt::Manager, CookieName> {
        let Self { cookie_manager, .. } = self;
        Builder {
            jwt_manager: value,
            cookie_manager,
        }
    }

    pub fn cookie_manager(self, value: cookie::Manager) -> Builder<JwtManager, cookie::Manager> {
        let Self { jwt_manager, .. } = self;
        Builder {
            jwt_manager,
            cookie_manager: value,
        }
    }
}

impl Builder<jwt::Manager, cookie::Manager> {
    pub fn build(self) -> State {
        let Self {
            jwt_manager,
            cookie_manager,
        } = self;
        State {
            jwt_manager,
            cookie_manager,
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
    let value = state.cookie_manager.from_header_value(cookie)?;
    let value = serde_json::from_str(&value)?;
    Ok(value)
}
