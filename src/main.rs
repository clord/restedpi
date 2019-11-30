extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate warp;

#[macro_use]
extern crate rust_embed;

use crate::config::Config;
use crate::config::value::{Value, Unit, evaluate as evaluate_val};
use crate::config::boolean::{BoolExpr, evaluate as evaluate_bool};
use i2c::error::Error;
use rppal::system::DeviceInfo;
use serde_json::{json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, filters::path::Tail, path, Filter, Rejection, Reply};

mod app;
mod config;
mod webapp;
mod i2c;

// We have to share the app state since warp uses a thread pool
type SharedAppState = std::sync::Arc<std::sync::Mutex<app::AppState>>;

// GET /
fn greeting(_app: SharedAppState, server_name: String) -> impl warp::Reply {
    let reply = json!({ "server": format!("restedpi on {}", server_name) });
    warp::reply::json(&reply)
}

// POST /api/debug/check_config
fn evaluate_config_check(
    _app: SharedAppState,
    expr: config::Config,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("config: {:?}", expr);
    //let app_l = app.lock().expect("failure");
    Ok(warp::reply::json(&expr))
}

// POST /api/debug/eval_bool
fn evaulate_bool_expr(
    app: SharedAppState,
    expr: BoolExpr,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("boolean evaluate: {:?}", expr);
    let app_l = app.lock().expect("failure");
    let reply_bool = evaluate_bool(&app_l, &expr);
    let reply = json!({ "result": reply_bool });
    Ok(warp::reply::json(&reply))
}

// POST /api/debug/eval_value
fn evaulate_value_expr(
    app: SharedAppState,
    expr: Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("value evaluate: {:?}", expr);
    let app_l = app.lock().expect("failure");
    let reply_value = evaluate_val(&app_l, &expr);
    let reply = json!({ "result": reply_value });
    Ok(warp::reply::json(&reply))
}

// GET /devices
//
// available devices that can be configured on this system
// (not configured devices)
fn all_devices(
    _app: SharedAppState
) -> Result<impl warp::Reply, warp::Rejection> {
    let reply = json!({ "result": [
        { "name": "BMP085"
        , "description": "High accuracy temperature and pressure"
        , "datasheet": "https://www.sparkfun.com/datasheets/Components/General/BST-BMP085-DS000-05.pdf"
        , "bus": "I2C"
        , "create": "/api/devices/create/bmp085"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +85°C  ±0.1°C" }, {"type": "pressure", "range": "300 to 1100hPa"}]
        },
        { "name": "MCP23017"
        , "description": " 16-port GPIO Expander"
        , "datasheet": "http://ww1.microchip.com/downloads/en/DeviceDoc/20001952C.pdf"
        , "bus": "I2C"
        , "create": "/api/devices/create/mcp23017"
        , "switches":
             [ { "name": "pin0" }
             , { "name": "pin1" }
             , { "name": "pin2" }
             , { "name": "pin3" }
             , { "name": "pin4" }
             , { "name": "pin5" }
             , { "name": "pin6" }
             , { "name": "pin7" }
             , { "name": "pin8" }
             , { "name": "pin9" }
             , { "name": "pin10" }
             , { "name": "pin11" }
             , { "name": "pin12" }
             , { "name": "pin13" }
             , { "name": "pin14" }
             , { "name": "pin15" }
             ]
        },
        { "name": "MCP9808"
        , "description": "High-accuracy temperature sensor"
        , "datasheet": "http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf"
        , "bus": "I2C"
        , "create": "/api/devices/create/mcp9808"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +125°C ±0.5°C" }]
        }
    ] });
    Ok(warp::reply::json(&reply))
}

// GET /sensors
fn all_sensors(
    _app: SharedAppState
) -> Result<impl warp::Reply, warp::Rejection> {
    //let app_l = app.lock().expect("failure");
    let reply = json!({ "result": [] });
    Ok(warp::reply::json(&reply))
}

// GET /sensors/:name/:unit
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
        .and(path!("api" / "about"))
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

    let r_devices = warp::get2()
        .and(app.clone())
        .and(path!("api" / "devices"))
        .and_then(all_devices);

    let r_sensors = warp::get2()
        .and(app.clone())
        .and(path!("api" / "sensors"))
        .and_then(all_sensors);

    let r_sensor = warp::get2()
        .and(app.clone())
        .and(path!("api" / "sensors" / String / Unit))
        .and_then(read_sensor);

    let index_html = warp::get2()
        .and_then(|| webapp::serve("index.html"));

    let r_static = warp::get2()
        .and(warp::path("static"))
        .and(warp::path::tail())
        .and_then(|tail: Tail| webapp::serve(tail.as_str()))
        .with(warp::log("restedpi-static"));

    let api = r_static
        .or(r_greeting)
        .or(r_sensor)
        .or(r_sensors)
        .or(r_devices)
        .or(r_config_check)
        .or(r_eval_bool)
        .or(r_eval_value)
        .or(index_html)
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
