extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
extern crate warp;

use crate::config::Unit;
use crate::config::Config;
use i2c::error::Error;
use rppal::system::DeviceInfo;
use serde_json::{from_str, json};
use std::collections::HashMap;
use std::env;
use std::fs;
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

// POST /api/debug/eval
fn evaulate_bool_expr(
    app: SharedAppState,
    expr: config::BoolExpr,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("config evaluate: {:?}", expr);
    let reply: bool = config::eval::evaluate(expr);
    Ok(warp::reply::json(&reply))
}

// GET /sensor/:name/:unit
fn read_sensor(
    app: SharedAppState,
    sensor: String,
    unit: Unit,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("sensor evaluate: {}", sensor);
    match app.lock().expect("failure").read_sensor(sensor, unit) {
        Ok(reply) => Ok(warp::reply::json(&reply)),
        Err(_e) => Err(warp::reject::not_found())
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
                listen: "127.0.0.1".to_string(),
                port: None,
                sensors: HashMap::new(),
                switches: HashMap::new(),
            }
        }
    };

    info!(
        "starting up... device: '{}'; port {}",
        server_name, port
    );

    let app_raw = app::start()?;
    let app_m = Arc::new(Mutex::new(app_raw));
    let app = warp::any().map(move || app_m.clone());

    let json_body = warp::body::content_length_limit(1024 * 16).and(warp::body::json());

    let r_greeting = warp::get2()
        .and(app.clone())
        .and(warp::any().map(move || server_name.clone()))
        .and(warp::path::end())
        .map(greeting);

    let r_eval = warp::post2()
        .and(app.clone())
        .and(path!("api" / "debug" / "eval"))
        .and(json_body)
        .and_then(evaulate_bool_expr);

    let r_sensors = warp::get2()
        .and(app.clone())
        .and(path!("api" / "sensors" / String / Unit))
        .and_then(read_sensor);

    let api = r_greeting
        .or(r_sensors)
        .or(r_eval)
        .recover(customize_error)
        .with(warp::log("restedpi"));

    warp::serve(api).run((
        [0, 0, 0, 0],
        port
    ));

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
            Error::UnsupportedUnit(_) => 1019,
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
        // Could be a NOT_FOUND, or any other internal error... here we just
        // let warp use its default rendering.
        Err(err)
    }
}

// let r_mcp23017_get =
//     warp::get2()
//         .and(path!("mcp23017" / String / usize))
//         .map(move |banknum, pinnum| {
//             let bank = if banknum == "B" {
//                 mcp23017::Bank::B
//             } else {
//                 mcp23017::Bank::A
//             };

//             let pin = match mcp23017::ordinal_pin(pinnum) {
//                 Some(pin) => pin,
//                 None => mcp23017::Pin::Pin0,
//             };

//             let mut data = d_mcp23017_1.lock().unwrap();

//             let value = data.get_pin(bank, pin).expect("problem getting pin");

//             Response::builder()
//                 .header("content-type", "application/json")
//                 .body(json!({ "value": value }).to_string())
//         });

// let r_mcp23017_put = warp::put2()
//     .and(path!("mcp23017" / String / usize / bool))
//     .map(move |banknum, pinnum, value| {
//         let bank = if banknum == "B" {
//             mcp23017::Bank::B
//         } else {
//             mcp23017::Bank::A
//         };
//         let pin = match mcp23017::ordinal_pin(pinnum) {
//             Some(pin) => pin,
//             None => mcp23017::Pin::Pin0,
//         };

//         let mut data = d_mcp23017_2.lock().unwrap();

//         data.set_pin(bank, pin, value).expect("problem setting pin");

//         Response::builder()
//             .header("content-type", "application/json")
//             .body(json!({ "value": value }).to_string())
//     });

// let r_mcp9808 = warp::get2()
//     .and(path!("mcp9808" / usize))
//     .map(move |index| {
//         let device_mx = match index {
//             0 => Arc::clone(&d_mcp9808_0),
//             1 => Arc::clone(&d_mcp9808_1),
//             2 => Arc::clone(&d_mcp9808_2),
//             _ => panic!("bad index"),
//         };

//         let device = device_mx.lock().unwrap();

//         let temperature: f32 = device.read_temp().expect("failed to read temperature");

//         Response::builder()
//             .header("content-type", "application/json")
//             .body(json!({ "temperature": temperature }).to_string())
//     });

// let r_bmp085 = warp::get2().and(path!("bmp085" / usize)).map(move |index| {
//     if index != 0 {
//         panic!("bad index")
//     }
//     let device = d_bmp085.lock().unwrap();
//     let pressure: f32 = device.pressure_kpa().expect("failed to read pressure");
//     let temperature: f32 = device
//         .temperature_in_c()
//         .expect("failed to read temperature");
//     Response::builder()
//         .header("content-type", "application/json")
//         .body(json!({ "temperature": temperature, "pressure": pressure }).to_string())
// });
