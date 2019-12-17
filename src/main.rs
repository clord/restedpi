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
use warp::{http::StatusCode, filters::path::Tail, path, Filter};

mod app;
mod config;
mod webapp;
mod i2c;

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
    let app_l = app.lock().expect("failure");
    let reply_bool = evaluate_bool(&app_l, &expr);
    let reply = json!({ "result": reply_bool });
    Ok(warp::reply::json(&reply))
}

// POST /api/debug/eval_value
fn evaulate_value_expr(
    app: webapp::SharedAppState,
    expr: Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("value evaluate: {:?}", expr);
    let app_l = app.lock().expect("failure");
    let reply_value = evaluate_val(&app_l, &expr);
    let reply = json!({ "result": reply_value });
    Ok(warp::reply::json(&reply))
}

// GET /sensors
fn all_sensors(
    _app: webapp::SharedAppState
) -> Result<impl warp::Reply, warp::Rejection> {
    //let app_l = app.lock().expect("failure");
    let reply = json!({ "result": [] });
    Ok(warp::reply::json(&reply))
}

// GET /sensors/:name/:unit
fn read_sensor(
    app: webapp::SharedAppState,
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

    let r_sensor = warp::get2()
        .and(app.clone())
        .and(path!("sensors" / String / Unit))
        .and_then(read_sensor);

    let index_html = warp::get2()
        .and_then(|| webapp::serve("index.html"));

    let r_static = warp::get2()
        .and(warp::path("static"))
        .and(warp::path::tail())
        .and_then(|tail: Tail| webapp::serve(tail.as_str()));

    let r_available =
        warp::get2()
        .and(path!("available"))
        .and(app.clone())
        .and_then(move |app| webapp::all_devices(app));

    let r_adding_configured =
        warp::post2()
        .and(app.clone())
        .and_then(move |app| webapp::add_device(app));

    let r_fetching_configured =
        warp::get2()
        .and(app.clone())
        .and_then(move |app| webapp::configured_devices(app));

    let r_configured =
        warp::path("configured")
        .and(r_adding_configured.or(r_fetching_configured));

    let r_devices =
        warp::path("devices")
        .and(r_available.or(r_configured));


    let api = r_static
        .or(path!("api").and(
                r_config
                .or(r_sensor)
                .or(r_sensors)
                .or(r_devices)
                .or(r_config_check)
                .or(r_eval_bool)
                .or(r_eval_value)))
        .or(index_html)
        .recover(customize_error);

    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    info!("RestedPi listening: http://{}", addr);
    warp::serve(api.with(warp::log("restedpi_rust::access"))).run(addr);

    Ok(())
}

/// Produce a json-compatible error report
fn customize_error(err: warp::Rejection)
    -> Result<impl warp::Reply, warp::Rejection> {
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
