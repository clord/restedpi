use super::SharedAppState;
use crate::graphql::create_schema;
use crate::session::{AppContext, WebSession};
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::subscriptions::make_ws_filter;
use juniper_warp::{make_graphql_filter, playground_filter};
use std::sync::Arc;
use tracing::warn;
use warp::{Filter, Rejection, Reply, any, get, header, http::Response, post};

fn with_app(
    app: SharedAppState,
) -> impl Filter<Extract = (AppContext,), Error = Rejection> + Clone {
    any()
        .and(header::optional::<WebSession>("authorization"))
        .map(move |t| AppContext::new(app.clone(), t))
}

/// Escape a string for use as a Prometheus label value.
///
/// Per the Prometheus text exposition format, label values must escape
/// backslash (`\\`), double-quote (`\"`), and newline (`\n`).
fn escape_label_value(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            other => escaped.push(other),
        }
    }
    escaped
}

async fn metrics_handler(app: AppContext) -> Result<impl Reply, Rejection> {
    let mut response: Response<String> = Response::default();
    let b = response.body_mut();

    let mut failed_inputs: Vec<String> = Vec::new();
    b.push_str("# HELP input_value The current value of inputs\n");
    b.push_str("# TYPE input_value gauge\n");
    for inp in app.channel().all_inputs().await? {
        let v = inp.value(&app).await;
        let name = inp.name();
        match (v.unit(), v.value()) {
            (Ok(unit), Ok(value)) => {
                let name = escape_label_value(name);
                let unit = escape_label_value(&format!("{unit:?}"));
                b.push_str(&format!(
                    "input_value{{name=\"{name}\", unit=\"{unit}\"}} {value}\n",
                ));
            }
            (Err(e), Ok(_)) | (Ok(_), Err(e)) | (Err(e), Err(_)) => {
                warn!("metrics: skipping input '{name}', reading failed: {e}");
                failed_inputs.push(escape_label_value(name));
            }
        }
    }

    b.push('\n');
    b.push_str("# HELP input_read_failure Inputs whose most recent reading failed\n");
    b.push_str("# TYPE input_read_failure gauge\n");
    for name in &failed_inputs {
        b.push_str(&format!("input_read_failure{{name=\"{name}\"}} 1\n"));
    }

    let mut failed_outputs: Vec<String> = Vec::new();
    b.push('\n');
    b.push_str("# HELP output_value The current value of outputs\n");
    b.push_str("# TYPE output_value gauge\n");
    for op in app.channel().all_outputs().await? {
        let v = op.value(&app).await;
        let name = op.name();
        match (v.unit(), v.value()) {
            (Ok(unit), Ok(value)) => {
                let name = escape_label_value(name);
                let unit = escape_label_value(&format!("{unit:?}"));
                b.push_str(&format!(
                    "output_value{{name=\"{name}\", unit=\"{unit}\"}} {value}\n",
                ));
            }
            (Err(e), Ok(_)) | (Ok(_), Err(e)) | (Err(e), Err(_)) => {
                warn!("metrics: skipping output '{name}', reading failed: {e}");
                failed_outputs.push(escape_label_value(name));
            }
        }
    }

    b.push('\n');
    b.push_str("# HELP output_read_failure Outputs whose most recent reading failed\n");
    b.push_str("# TYPE output_read_failure gauge\n");
    for name in &failed_outputs {
        b.push_str(&format!("output_read_failure{{name=\"{name}\"}} 1\n"));
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

#[cfg(test)]
mod tests {
    use super::escape_label_value;

    #[test]
    fn passes_through_plain_strings() {
        assert_eq!(escape_label_value("living room temp"), "living room temp");
    }

    #[test]
    fn escapes_backslash() {
        assert_eq!(escape_label_value(r"a\b"), r"a\\b");
    }

    #[test]
    fn escapes_double_quote() {
        assert_eq!(escape_label_value(r#"say "hi""#), r#"say \"hi\""#);
    }

    #[test]
    fn escapes_newline() {
        assert_eq!(escape_label_value("line1\nline2"), r"line1\nline2");
    }

    #[test]
    fn escapes_combined_sequences() {
        assert_eq!(escape_label_value("\\\"\n"), r#"\\\"\n"#);
    }

    #[test]
    fn empty_string_is_unchanged() {
        assert_eq!(escape_label_value(""), "");
    }
}
