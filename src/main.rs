extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate warp;

use crate::config::Config;
use crate::config::Unit;
use i2c::{bmp085, error::Error, mcp23017::Bank, mcp23017::Pin};
use rppal::system::DeviceInfo;
use serde_json::{from_str, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, path, Filter, Rejection, Reply};

mod app;
mod config;
mod i2c;

// We have to share the app state since warp uses a thread pool
type SharedAppState = std::sync::Arc<std::sync::Mutex<app::AppState>>;

// GET /
fn greeting(app: SharedAppState, server_name: String) -> impl warp::Reply {
    let reply = json!({ "server": format!("restedpi on {}", server_name) });
    warp::reply::json(&reply)
}

// POST /api/debug/check_config
fn evaluate_config_check(
    app: SharedAppState,
    expr: config::Config,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("config: {:?}", expr);
    let app_l = app.lock().expect("failure");
    Ok(warp::reply::json(&expr))
}
// POST /api/debug/eval_bool
fn evaulate_bool_expr(
    app: SharedAppState,
    expr: config::BoolExpr,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("boolean evaluate: {:?}", expr);
    let app_l = app.lock().expect("failure");
    let reply = config::eval::evaluate_bool(&app_l, &expr);
    Ok(warp::reply::json(&reply))
}

// POST /api/debug/eval_value
fn evaulate_value_expr(
    app: SharedAppState,
    expr: config::Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("value evaluate: {:?}", expr);
    let app_l = app.lock().expect("failure");
    let reply = config::eval::evaluate_value(&app_l, &expr);
    Ok(warp::reply::json(&reply))
}

// GET /sensor/:name/:unit
fn read_sensor(
    app: SharedAppState,
    sensor: String,
    unit: Unit,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("sensor evaluate: {}", sensor);
    let app_l = app.lock().expect("failure");
    match app_l.read_sensor(sensor, unit) {
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
        // Set `RUST_LOG=restedpi_rust=debug` to see debug logs,
        env::set_var("RUST_LOG", "restedpi_rust=info");
        info!("defaulting to info level logging. RUST_LOG='restedpi_rust=info'");
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
                listen: None,
                port: None,
                sensors: None,
                switches: None,
            }
        }
    };

    let listen = config.listen.unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let sensor_config = config.sensors.unwrap_or(HashMap::new());
    let switch_config = config.switches.unwrap_or(HashMap::new());

    let app_raw = app::start(sensor_config, switch_config)?;

    let app_m = Arc::new(Mutex::new(app_raw));
    let app = warp::any().map(move || app_m.clone());

    // Limit incoming body length to 16kb
    const LIMIT: u64 = 1024 * 16;

    let r_greeting = warp::get2()
        .and(app.clone())
        .and(warp::any().map(move || server_name.clone()))
        .and(warp::path::end())
        .map(greeting);

    let r_config_check = warp::post2()
        .and(app.clone())
        .and(path!("api" / "debug" / "config_check"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaluate_config_check);

    let r_eval_bool = warp::post2()
        .and(app.clone())
        .and(path!("api" / "debug" / "eval_bool"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaulate_bool_expr);

    let r_eval_value = warp::post2()
        .and(app.clone())
        .and(path!("api" / "debug" / "eval_value"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(evaulate_value_expr);

    let r_sensors = warp::get2()
        .and(app.clone())
        .and(path!("api" / "sensors" / String / Unit))
        .and_then(read_sensor);

    let api = r_greeting
        .or(r_sensors)
        .or(r_config_check)
        .or(r_eval_bool)
        .or(r_eval_value)
        .with(warp::log("restedpi"))
        .recover(customize_error);

    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    info!("RestedPi listening: http://{}", addr);
    warp::serve(api).run(addr);

    Ok(())
}

/// Produce a json-compatible error report
fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err.find_cause::<Error>() {
        let code = match err {
            Error::Io(_) => 1001,
            Error::InvalidPinIndex => 1004,
            Error::InvalidPinDirection => 1008,
            Error::I2cError(_) => 1016,
            Error::NonExistant(_) => 1017,
            Error::UnsupportedUnit(_) => 1019,
            Error::RecvError(_) => 1020,
            Error::AppSendError(_) => 1023,
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
