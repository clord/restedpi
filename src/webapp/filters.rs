use super::SharedAppState;
use crate::graphql::create_schema;
use crate::session::{AppContext, WebSession};
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::subscriptions::make_ws_filter;
use juniper_warp::{make_graphql_filter, playground_filter};
use std::sync::Arc;
use warp::{Filter, Rejection, Reply, any, get, header, http::Response, post};

fn with_app(
    app: SharedAppState,
) -> impl Filter<Extract = (AppContext,), Error = Rejection> + Clone {
    any()
        .and(header::optional::<WebSession>("authorization"))
        .map(move |t| AppContext::new(app.clone(), t))
}

async fn metrics_handler(app: AppContext) -> Result<impl Reply, Rejection> {
    let mut response: Response<String> = Response::default();
    let b = response.body_mut();
    b.push_str("# HELP input_value The current value of inputs\n");
    b.push_str("# TYPE input_value gauge\n");
    for inp in app.channel().all_inputs().await? {
        let v = inp.value(&app).await;
        let name = inp.name();
        let unit = v.unit()?;
        let value = v.value()?;
        b.push_str(&format!(
            "input_value{{name=\"{name}\", unit=\"{unit:?}\"}} {value}\n",
        ));
    }

    b.push('\n');
    b.push_str("# HELP output_value The current value of outputs\n");
    b.push_str("# TYPE output_value gauge\n");
    for op in app.channel().all_outputs().await? {
        let v = op.value(&app).await;
        let name = op.name();
        let unit = v.unit()?;
        let value = v.value()?;
        b.push_str(&format!(
            "output_value{{name=\"{name}\", unit=\"{unit:?}\"}} {value}\n",
        ));
    }
    b.push('\n');

    Ok(response)
}

pub fn graphql_api(
    app: SharedAppState,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let schema = Arc::new(create_schema());

    // WebSocket subscription endpoint
    let subscriptions = {
        let app_clone = app.clone();
        warp::path("subscriptions").and(make_ws_filter(schema.clone(), move |_| {
            let ctx = AppContext::new(app_clone.clone(), None);
            async move { Ok::<_, std::convert::Infallible>(ConnectionConfig::new(ctx)) }
        }))
    };

    // GraphQL query/mutation endpoint
    let graphql = warp::path("graphql").and(
        post()
            .and(make_graphql_filter(schema, with_app(app.clone()).boxed()))
            .or(get().and(playground_filter("/graphql", Some("/subscriptions")))),
    );

    // Metrics endpoint
    let metrics = warp::path("metrics")
        .and(get())
        .and(with_app(app))
        .and_then(metrics_handler);

    // Serve index.html for root and any other path (SPA-style)
    let index = get().and_then(super::serve_index);

    subscriptions.or(graphql).or(metrics).or(index)
}
