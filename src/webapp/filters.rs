use super::SharedAppState;
use crate::graphql::create_schema;
use crate::session::{AppContext, WebSession};
use juniper_warp::{make_graphql_filter, playground_filter};

use warp::{any, get, header, post, Filter, Rejection, Reply};

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

pub fn api(_app: SharedAppState) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    static_filter().or(static_index_html())
}

fn static_index_html() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get().and_then(|| super::static_serve("index.html"))
}

fn static_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    get()
        .and(warp::path::tail())
        .and_then(super::static_serve_tail)
}
