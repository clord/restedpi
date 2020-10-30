use super::handlers;
use super::SharedAppState;
use crate::app::channel::AppChannel;
use std::convert::Infallible;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::{any, get, path, reply, Filter, Rejection, Reply};

fn with_app(
    app: SharedAppState,
) -> impl Filter<Extract = (AppChannel,), Error = Infallible> + Clone {
    any().map(move || app.clone().lock().expect("locked state").clone())
}

pub fn api(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("api")
        .and(
            devices(app.clone())
                .or(about_server())
                .or(available_devices(app)),
        )
        .or(static_filter())
        .or(static_index_html())
}

fn static_index_html() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get().and_then(|| super::static_serve("index.html"))
}

fn about_server() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get()
        .and(path!("about-server"))
        .and_then(handlers::server_name)
}

fn static_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get()
        .and(warp::path::tail())
        .and_then(super::static_serve_tail)
}

fn available_devices(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let mut short_cache_header = HeaderMap::new();
    short_cache_header.insert(
        "cache-control",
        HeaderValue::from_static("private, max-age=4"),
    );
    warp::path!("available-devices")
        .and(get())
        .and(with_app(app))
        .and_then(handlers::get_available_devices)
        .with(reply::with::headers(short_cache_header))
}

fn devices(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    devices_list(app.clone())
        .or(devices_create(app.clone()))
        .or(devices_update(app.clone()))
        .or(devices_delete(app))
}

fn devices_list(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices")
        .and(get())
        .and(with_app(app))
        .and_then(handlers::list_devices)
}

fn devices_create(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices")
        .and(warp::post())
        .and(with_app(app))
        .and_then(handlers::list_devices)
}

fn devices_update(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices")
        .and(warp::put())
        .and(with_app(app))
        .and_then(handlers::list_devices)
}

fn devices_delete(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices")
        .and(warp::delete())
        .and(with_app(app))
        .and_then(handlers::list_devices)
}
