extern crate pretty_env_logger;

#[macro_use]
extern crate log;

extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate warp;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rust_embed;

use crate::config::boolean::{evaluate as evaluate_bool, BoolExpr};
use crate::config::value::{evaluate as evaluate_val, Value};
use crate::config::Config;
use i2c::error::Error;
use rppal::system::DeviceInfo;
use serde_json::json;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::{
    filters::path::Tail,
    http::header::{HeaderMap, HeaderValue},
    http::StatusCode,
    path, Filter,
};

mod app;
mod config;
mod i2c;
mod webapp;

// GET /api/config
fn server_config(_app: webapp::SharedAppState, server_name: String) -> impl warp::Reply {
    let reply = json!({ "serverConfig":
    {"deviceName": format!("restedpi on {}", server_name),
    }});
    warp::reply::json(&reply)
}

// POST /api/debug/check_config
fn evaluate_config_check(
    _app: webapp::SharedAppState,
    expr: config::Config,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("config: {:?}", expr);
    //let app_l = app.lock().expect("failure");
    Ok(warp::reply::json(&expr))
}

// POST /api/debug/eval_bool
fn evaulate_bool_expr(
    app: webapp::SharedAppState,
    expr: BoolExpr,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("boolean evaluate: {:?}", expr);
    let reply_bool = evaluate_bool(&mut app.lock().expect("failure"), &expr);
    let reply = json!({ "result": reply_bool });
    Ok(warp::reply::json(&reply))
}

// POST /api/debug/eval_value
fn evaulate_value_expr(
    app: webapp::SharedAppState,
    expr: Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("value evaluate: {:?}", expr);
    let reply_value = evaluate_val(&mut app.lock().expect("failure"), &expr);
    let reply = json!({ "result": reply_value });
    Ok(warp::reply::json(&reply))
}

// GET /sensors
fn all_sensors(_app: webapp::SharedAppState) -> Result<impl warp::Reply, warp::Rejection> {
    let reply = json!({ "result": [] });
    Ok(warp::reply::json(&reply))
}

// GET /device/:name/sensor/read/:index
fn read_sensor(
    app: webapp::SharedAppState,
    device: String,
    index: usize,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.read_sensor(device, index) {
        Ok(reply) => Ok(warp::reply::json(&reply)),
        Err(_e) => Err(warp::reject::not_found()),
    }
}

/// big picture:
/// read configuration and decide what sensors and switches are available. start up application, then
/// start running state machine. finally, present a rest api to the outside world to interact with the
/// application.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=restedpi=debug` to see debug logs,
        env::set_var("RUST_LOG", "restedpi=info");
        info!("defaulting to info level logging. RUST_LOG='restedpi=info'");
    }
    pretty_env_logger::init();
    let server_name = match DeviceInfo::new() {
        Ok(model) => model.model().to_string(),
        Err(e) => {
            warn!("reading model: {}", e);
            "Unknown".to_string()
        }
    };
    let contents = match fs::read_to_string("config.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("error reading config: {}", e);
            "".to_string()
        }
    };

    let config: config::Config = match serde_json::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("error parsing config: {}", e);
            Config {
                database_path: None,
                listen: None,
                port: None,
                devices: None,
            }
        }
    };

    let listen = config.listen.unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let device_config = config.devices.unwrap_or(Vec::new());

    let mut app_raw = app::new();
    for config in device_config.iter() {
        app_raw.add_device(config);
    }

    let app_m = Arc::new(Mutex::new(app_raw));
    let app = warp::any().map(move || app_m.clone());

    // Limit incoming body length to 16kb
    const LIMIT: u64 = 1024 * 16;

    let r_config = warp::get2()
        .and(app.clone())
        .and(warp::any().map(move || server_name.clone()))
        .and(path!("config"))
        .map(server_config);

    let r_config_check = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "config_check"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaluate_config_check);

    let r_eval_bool = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "eval_bool"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaulate_bool_expr);

    let r_eval_value = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "eval_value"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaulate_value_expr);

    let r_sensors = warp::get2()
        .and(app.clone())
        .and(path!("sensors"))
        .and_then(all_sensors);

    let r_read = warp::get2()
        .and(app.clone())
        .and(path!(String / "sensor" / usize))
        .and_then(read_sensor);

    let mut nocache_header = HeaderMap::new();
    nocache_header.insert("cache-control", HeaderValue::from_static("no-store"));

    let index_html = warp::get2()
        .and_then(|| webapp::serve("index.html"))
        .with(warp::reply::with::headers(nocache_header));

    let r_static = warp::get2()
        .and(warp::any())
        .and(warp::path::tail())
        .and_then(|tail: Tail| webapp::serve(tail.as_str()));

    let r_available = warp::get2()
        .and(path!("available"))
        .and(app.clone())
        .and_then(move |app| webapp::all_devices(app));

    let r_adding_configured = warp::post2()
        .and(app.clone())
        .and(warp::body::json())
        .and_then(move |app, body| webapp::add_device(app, body));

    let r_remove_configured = warp::delete2()
        .and(app.clone())
        .and(warp::path::param())
        .and_then(move |app, name| webapp::remove_device(app, name));

    let r_fetching_configured = warp::get2()
        .and(app.clone())
        .and_then(move |app| webapp::configured_devices(app));

    let r_configured = warp::path("configured").and(
        r_adding_configured
            .or(r_fetching_configured)
            .or(r_remove_configured),
    );

    let r_devices = warp::path("devices").and(r_read.or(r_available.or(r_configured)));

    let api = r_static
        .or(path!("api").and(
            r_config
                .or(r_sensors)
                .or(r_devices)
                .or(r_config_check)
                .or(r_eval_bool)
                .or(r_eval_value),
        ))
        .or(index_html)
        .recover(customize_error);

    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    info!("RestedPi listening: http://{}", addr);
    warp::serve(api.with(warp::log("restedpi::access"))).run(addr);

    Ok(())
}

/// Produce a json-compatible error report
fn customize_error(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(err) = err.find_cause::<Error>() {
        let code = match err {
            Error::Io(_) => 1001,
            Error::InvalidPinDirection => 1008,
            Error::I2cError(_) => 1016,
            Error::NonExistant(_) => 1017,
            Error::OutOfBounds(_) => 1019,
            Error::RecvError(_) => 1020,
            Error::SendError(_) => 1024,
        };
        let message = err.to_string();

        let json = json!({ "code": code, "message": message });

        Ok(warp::reply::with_status(
            json.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Err(err)
    }
}
