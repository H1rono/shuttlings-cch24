use anyhow::Context;
use warp::Filter;
use warp::Reply;

mod routes;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let seek_url = secrets.get("SEEK_URL").context("secret SEEK_URL not set")?;
    let state = routes::State::builder().seek_url(seek_url).build()?;
    let route = routes::make(state);
    Ok(route.boxed().into())
}
