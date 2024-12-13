use anyhow::Context;
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
    let state = lib::routes::State::builder()
        .seek_url(seek_url)
        .manifest_keyword(manifest_keyword)
        .milk_full(5.0)
        .milk_initial(0.0)
        .build();
    let _bg_task = tokio::spawn(state.bg_task());
    let route = lib::routes::make(state);
    Ok(route.boxed().into())
}
