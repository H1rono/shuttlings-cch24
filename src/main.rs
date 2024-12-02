use warp::Filter;
use warp::Reply;

mod routes;

#[shuttle_runtime::main]
async fn main() -> shuttle_warp::ShuttleWarp<(impl Reply,)> {
    let route = routes::make();
    Ok(route.boxed().into())
}
