use super::handlers;
use super::SharedAppState;
use crate::graphql::create_schema;
use crate::session::{AppContext, WebSession};
use juniper_warp::{make_graphql_filter, playground_filter};

use warp::http::header::{HeaderMap, HeaderValue};
use warp::{any, get, header, path, post, reply, Filter, Rejection, Reply};

fn with_app(
    app: SharedAppState,
) -> impl Filter<Extract = (AppContext,), Error = Rejection> + Clone {
    any()
        .and(header::optional::<WebSession>("authorization"))
        .map(move |t| AppContext::new(app.clone(), t))
}

pub fn graphql_api(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("graphql")
        .and(
            post()
                .and(make_graphql_filter(create_schema(), with_app(app).boxed()))
                .or(get().and(playground_filter("/graphql", None))),
        )
        .or(static_filter())
        .or(static_index_html())
}

pub fn api(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path("api")
        .and(auth(app.clone()).or(available_devices(app)))
        .or(static_filter())
        .or(static_index_html())
}

fn static_index_html() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get().and_then(|| super::static_serve("index.html"))
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

fn auth(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("authenticate")
        .and(post())
        .and(with_app(app))
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::form())
        .and_then(handlers::authentication)
}
