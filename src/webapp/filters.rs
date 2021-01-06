use super::handlers;
use super::{SharedAppState, WebSession};
use crate::app::channel::AppChannel;
use std::convert::Infallible;
use warp::http::header::{HeaderMap, HeaderValue};
use warp::{any, get, header, path, post, reply, Filter, Rejection, Reply};

fn with_session() -> impl Filter<Extract = (WebSession,), Error = Rejection> + Clone {
    any().and(header::<WebSession>("authorization"))
}

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
                .or(available_devices(app.clone())),
        )
        .or(auth(app))
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
        .and(with_session())
        .and(with_app(app))
        .and_then(handlers::get_available_devices)
        .with(reply::with::headers(short_cache_header))
}

fn auth(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("auth")
        .and(post())
        .and(with_app(app))
        .and(warp::body::form())
        .and_then(handlers::authentication)
}

fn devices(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    devices_list(app.clone())
        .or(devices_update(app.clone()))
        .or(devices_delete(app))
}

fn devices_list(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices")
        .and(get())
        .and(with_session())
        .and(with_app(app))
        .and_then(handlers::list_devices)
}

fn devices_update(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices" / String)
        .and(warp::put())
        .and(with_session())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(handlers::add_or_replace_device)
}

fn devices_delete(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices" / String)
        .and(warp::delete())
        .and(with_session())
        .and(with_app(app))
        .and_then(handlers::remove_device)
}
