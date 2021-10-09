use super::SharedAppState;
use crate::graphql::create_schema;
use crate::session::{AppContext, WebSession};
use juniper_warp::{make_graphql_filter, playground_filter};
use warp::{http::Response, any, get, header, post, Filter, Rejection, Reply};

fn with_app(
    app: SharedAppState,
) -> impl Filter<Extract = (AppContext,), Error = Rejection> + Clone {
    any()
        .and(header::optional::<WebSession>("authorization"))
        .map(move |t| AppContext::new(app.clone(), t))
}

// pub async fn configured_devices(app: SharedAppState) -> Result<impl Reply, Rejection> {
async fn metrics_handler(app: AppContext) -> Result<impl Reply, Rejection> {
    let mut response: Response<String> = Response::default();
    let b = response.body_mut();
    b.push_str("# HELP input_value The current value of inputs\n");
    b.push_str("# TYPE gauge\n");
    for inp in app.channel().all_inputs().await? {
        let v = inp.value(&app).await;
        let name = inp.name();
        let unit = v.unit()?;
        let value = v.value()?;
        b.push_str(&format!("input_value{{name=\"{name}\", unit=\"{unit:?}\"}} {value}\n", name = name, unit = unit, value = value));
    }

    b.push_str("# HELP output_value The current value of outputs\n");
    b.push_str("# TYPE gauge\n");
    for op in app.channel().all_outputs().await? {
        let v = op.value(&app).await;
        let name = op.name();
        let unit = v.unit()?;
        let value = v.value()?;
        b.push_str(&format!("output_value{{name=\"{name}\", unit=\"{unit:?}\"}} {value}", name = name, unit = unit, value = value));
    }

    Ok(response)
}

pub fn graphql_api(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path("graphql")
        .and(
            post()
                .and(make_graphql_filter(create_schema(), with_app(app.clone()).boxed()))
                .or(get().and(playground_filter("/graphql", None))),
        )
        .or(warp::path("metrics").and(get()).and(with_app(app)).and_then(metrics_handler))
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
