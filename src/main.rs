use anyhow::Context;
use chrono::TimeDelta;
use tracing_subscriber::EnvFilter;
use warp::Filter;
use warp::Reply;

use shuttlings_cch24 as lib;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let env_filter = EnvFilter::try_from_default_env()
        .context("from env failed")
        .or_else(|_| {
            secrets
                .get("CCH24_LOG")
                .context("secret CCH24_LOG not set")
                .map(EnvFilter::from)
        })
        .unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let seek_url = secrets.get("SEEK_URL").context("secret SEEK_URL not set")?;
    let manifest_keyword = secrets
        .get("MANIFEST_KEYWORD")
        .context("secret MANIFEST_KEYWORD not set")?;
    let jwt_manager = load_jwt_manager(&secrets)?;
    let cookie_manager = load_cookie_manager(&secrets)?;
    let state = lib::routes::State::builder()
        .seek_url(seek_url)
        .manifest_keyword(manifest_keyword)
        .milk_full(5.0)
        .milk_initial(0.0)
        .jwt_manager(jwt_manager)
        .cookie_manager(cookie_manager)
        .build();
    let _bg_task = tokio::spawn(state.bg_task());
    let route = lib::routes::make(state);
    Ok(route.boxed().into())
}

#[tracing::instrument(skip_all)]
fn load_jwt_manager(secrets: &shuttle_runtime::SecretStore) -> anyhow::Result<lib::jwt::Manager> {
    let issuer = secrets
        .get("JWT_ISSUER")
        .context("secret JWT_ISSUER not set")
        .inspect_err(|e| tracing::error!(%e))
        .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string());
    let key = secrets.get("JWT_KEY").context("secret JWT_KEY not set")?;
    let expires_in: i64 = secrets
        .get("JWT_EXPIRES_IN")
        .context("secret JWT_EXPIRES_IN not set")
        .inspect_err(|e| tracing::error!(%e))
        .unwrap_or_else(|_| "86400".to_string()) // 1 day in seconds
        .parse()?;
    let expires_in = chrono::TimeDelta::seconds(expires_in);
    let manager = lib::jwt::Manager::builder()
        .issuer(issuer)
        .key(key)
        .expires_in(expires_in)
        .build();
    Ok(manager)
}

#[tracing::instrument(skip_all)]
fn load_cookie_manager(
    secrets: &shuttle_runtime::SecretStore,
) -> anyhow::Result<lib::cookie::Manager> {
    let name = secrets
        .get("COOKIE_NAME")
        .context("secret COOKIE_NAME not set")?;
    let max_age: i64 = secrets
        .get("COOKIE_MAX_AGE")
        .context("secret COOKIE_MAX_AGE not set")
        .inspect_err(|e| tracing::error!(%e))
        .unwrap_or_else(|_| "86400".to_string())
        .parse()?;
    let max_age = TimeDelta::seconds(max_age);
    let domain = secrets.get("COOKIE_DOMAIN");
    let path = secrets.get("COOKIE_PATH");
    let secure: bool = secrets
        .get("COOKIE_SECURE")
        .unwrap_or_else(|| "false".to_string())
        .parse()?;
    let builder = lib::cookie::Manager::builder().name(name).max_age(max_age);
    let builder = if let Some(d) = domain {
        builder.domain(d)
    } else {
        builder
    };
    let builder = if let Some(p) = path {
        builder.path(p)
    } else {
        builder
    };
    let builder = if secure { builder.secure() } else { builder };
    Ok(builder.build())
}
