use anyhow::Context;
use chrono::TimeDelta;
use tracing_subscriber::EnvFilter;
use warp::Filter;
use warp::Reply;

use shuttlings_cch24 as lib;

macro_rules! get_secret {
    ($s:ident.$k:ident) => {
        $s.get(stringify!($k))
            .context(concat!("secret ", stringify!($k), " not set"))
    };
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://{secrets.PG_USER}:{secrets.PG_PASSWORD}@localhost:5432/{secrets.PG_DATABASE}"
    )]
    pool: sqlx::PgPool,
) -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let env_filter = EnvFilter::try_from_default_env()
        .context("from env failed")
        .or_else(|_| get_secret!(secrets.CCH24_LOG).map(EnvFilter::from))
        .unwrap_or_else(|_| "info".into());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let seek_url = get_secret!(secrets.SEEK_URL)?;
    let manifest_keyword = get_secret!(secrets.MANIFEST_KEYWORD)?;
    let milk_bucket = lib::bucket::MilkBucket::builder()
        .full(5.0)
        .initial(0.0)
        .build();
    let jwt_manager = load_jwt_manager(&secrets)?;
    let cookie_manager = load_cookie_manager(&secrets)?;
    let jwt_decoder = load_jwt_decoder(&secrets).await?;
    let quotes_repo = load_quotes_repository(pool).await?;
    let state = lib::routes::State::builder()
        .seek_url(seek_url)
        .manifest_keyword(manifest_keyword)
        .milk_bucket(milk_bucket)
        .jwt_manager(jwt_manager)
        .cookie_manager(cookie_manager)
        .jwt_decoder(jwt_decoder)
        .quotes_repository(quotes_repo)
        .build();
    let _bg_task = tokio::spawn(state.bg_task());
    let route = lib::routes::make(state);
    Ok(route.boxed().into())
}

#[tracing::instrument(skip_all)]
fn load_jwt_manager(secrets: &shuttle_runtime::SecretStore) -> anyhow::Result<lib::jwt::Manager> {
    let issuer = get_secret!(secrets.JWT_ISSUER)
        .inspect_err(|e| tracing::error!(%e))
        .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string());
    let key = get_secret!(secrets.JWT_KEY)?;
    let expires_in: i64 = get_secret!(secrets.JWT_EXPIRES_IN)
        .inspect_err(|e| tracing::error!(%e))
        .unwrap_or_else(|_| "86400".to_string()) // 1 day in seconds
        .parse()?;
    let expires_in = TimeDelta::seconds(expires_in);
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
    let name = get_secret!(secrets.COOKIE_NAME)?;
    // let max_age: i64 = get_secret!(secrets.COOKIE_MAX_AGE)
    //     .inspect_err(|e| tracing::error!(%e))
    //     .unwrap_or_else(|_| "86400".to_string())
    //     .parse()?;
    // let max_age = TimeDelta::seconds(max_age);
    // let domain = secrets.get("COOKIE_DOMAIN");
    // let path = secrets.get("COOKIE_PATH");
    // let secure: bool = secrets
    //     .get("COOKIE_SECURE")
    //     .unwrap_or_else(|| "false".to_string())
    //     .parse()?;
    let builder = lib::cookie::Manager::builder().name(name);
    // let builder = builder.max_age(max_age);
    // let builder = if let Some(d) = domain {
    //     builder.domain(d)
    // } else {
    //     builder
    // };
    // let builder = if let Some(p) = path {
    //     builder.path(p)
    // } else {
    //     builder
    // };
    // let builder = if secure { builder.secure() } else { builder };
    Ok(builder.build())
}

#[tracing::instrument(skip_all)]
async fn load_jwt_decoder(
    secrets: &shuttle_runtime::SecretStore,
) -> anyhow::Result<lib::jwt::Decoder> {
    let pem_path = get_secret!(secrets.JWT_PEM_FILE)?;
    let pem = tokio::fs::read(pem_path)
        .await
        .context("failed to read pem file")?;
    let decoder = lib::jwt::Decoder::builder().pem(pem).build();
    Ok(decoder)
}

#[tracing::instrument(skip_all)]
async fn load_quotes_repository(pool: sqlx::PgPool) -> anyhow::Result<lib::quotes::Repository> {
    let repo = lib::quotes::Repository::builder().pool(pool).build();
    repo.migrate().await?;
    Ok(repo)
}
