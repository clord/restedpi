use warp::{http::Response, Rejection, Reply};

pub async fn handler() -> Result<impl Reply, Rejection> {
    // TODO: I'm working on getting metrics working for grafana. something like:
    // https://blog.logrocket.com/using-prometheus-metrics-in-a-rust-web-service/
    @NOT READY
    Ok(Response::builder().body("OK"))
}
