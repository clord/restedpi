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
        .and(
            devices(app.clone())
                .or(inputs(app.clone()))
                .or(outputs(app.clone()))
                .or(auth(app.clone()))
                .or(available_devices(app)),
        )
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
        .and(with_app(app))
        .and_then(handlers::list_devices)
}

fn devices_update(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(handlers::add_or_replace_device)
}

fn devices_delete(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("devices" / String)
        .and(warp::delete())
        .and(with_app(app))
        .and_then(handlers::remove_device)
}

fn inputs(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    input_list(app.clone())
        .or(input_update(app.clone()))
        .or(input_delete(app.clone()))
        .or(read_input(app))
}

fn input_list(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("inputs")
        .and(get())
        .and(with_app(app))
        .and_then(handlers::list_inputs)
}

fn input_update(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("inputs" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(handlers::add_or_replace_input)
}

fn input_delete(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("inputs" / String)
        .and(warp::delete())
        .and(with_app(app))
        .and_then(handlers::remove_input)
}

fn read_input(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("inputs" / String / "read")
        .and(warp::get())
        .and(with_app(app))
        .and_then(handlers::read_input)
}

fn outputs(app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    output_list(app.clone())
        .or(output_update(app.clone()))
        .or(output_delete(app))
}

fn output_list(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("outputs")
        .and(get())
        .and(with_app(app))
        .and_then(handlers::list_outputs)
}

fn output_update(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("outputs" / String)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(handlers::add_or_replace_output)
}

fn output_delete(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("outputs" / String)
        .and(warp::delete())
        .and(with_app(app))
        .and_then(handlers::remove_output)
}
