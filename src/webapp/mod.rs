use crate::app;
use crate::config;
use crate::config::boolean::{evaluate as evaluate_bool, BoolExpr};
use crate::config::value::{evaluate as evaluate_val, Value};
use crate::i2c::device::Status;
use crate::i2c::error::Error;
use mime_guess::from_path;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use warp::{http::Response, http::StatusCode, reply, Rejection, Reply};
pub mod slugify;

// We have to share the app state since warp uses a thread pool
pub type SharedAppState = std::sync::Arc<std::sync::Mutex<app::State>>;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub extern "C" fn serve(path: &str) -> Result<impl Reply, Rejection> {
    let asset_a: Option<Cow<'static, [u8]>> = Asset::get(path);
    if asset_a.is_some() {
        let mime = from_path(path).first_or_octet_stream();
        let file = asset_a.ok_or_else(|| warp::reject::not_found())?;
        Ok(Response::builder()
            .header("content-type", mime.to_string())
            .body(file))
    } else {
        let path_b: String = format!("{}/index.js", path);
        let asset_b: Option<Cow<'static, [u8]>> = Asset::get(path_b.as_str());
        let mime = from_path(path_b.as_str()).first_or_octet_stream();
        let file = asset_b.ok_or_else(|| warp::reject::not_found())?;
        Ok(Response::builder()
            .header("content-type", mime.to_string())
            .body(file))
    }
}

// GET /devices/available
//
// available devices that can be configured on this system
// (not configured devices)
pub fn all_devices(_app: SharedAppState) -> Result<impl Reply, Rejection> {
    let reply = json!({ "result": [
        { "name": "BMP085"
        , "device": "/api/devices/available/Bmp085"
        , "description": "High accuracy temperature and pressure"
        , "datasheet": "https://www.sparkfun.com/datasheets/Components/General/BST-BMP085-DS000-05.pdf"
        , "bus": "I2C"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +85°C  ±0.1°C" }, {"type": "pressure", "range": "300 to 1100hPa"}]
        },
        { "name": "MCP23017"
        , "device": "/api/devices/available/Mcp23017"
        , "description": " 16-port GPIO Expander"
        , "datasheet": "http://ww1.microchip.com/downloads/en/DeviceDoc/20001952C.pdf"
        , "bus": "I2C"
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
        , "device": "/api/devices/available/Mcp9808"
        , "description": "High-accuracy temperature sensor"
        , "datasheet": "http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf"
        , "bus": "I2C"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +125°C ±0.5°C" }]
        }
    ] });
    Ok(reply::json(&reply))
}

pub fn devices_as_json(app: std::sync::MutexGuard<app::State>) -> serde_json::value::Value {
    let d: HashMap<String, (config::Device, Status)> = app
        .devices()
        .into_iter()
        .map(|(name, device)| (name.to_string(), (device.config(), device.status())))
        .collect();
    return json!(d);
}

// GET /devices/configured
//
// configured devices in the system
pub fn configured_devices(app: SharedAppState) -> Result<impl Reply, Rejection> {
    let app_l = app.lock().expect("failure");
    let reply = devices_as_json(app_l);
    Ok(reply::json(&reply))
}

pub fn remove_device(app: SharedAppState, name: String) -> Result<impl Reply, Rejection> {
    let mut app_l = app.lock().expect("failure");
    app_l.remove_device(&name);
    let reply = devices_as_json(app_l);
    Ok(reply::json(&reply))
}

pub fn add_device(app: SharedAppState, device: config::Device) -> Result<impl Reply, Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.add_device(&device) {
        Err(e) => Err(warp::reject::custom(e)),
        Ok(()) => {
            let reply = devices_as_json(app_l);
            Ok(reply::json(&reply))
        }
    }
}

// GET /api/config
pub fn server_config(_app: SharedAppState, server_name: String) -> impl warp::Reply {
    let reply = json!({ "serverConfig":
    {"deviceName": format!("restedpi on {}", server_name),
    }});
    warp::reply::json(&reply)
}

// POST /api/debug/check_config
pub fn evaluate_config_check(
    _app: SharedAppState,
    expr: config::Config,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("config: {:?}", expr);
    //let app_l = app.lock().expect("failure");
    Ok(warp::reply::json(&expr))
}

// POST /api/debug/eval_bool
pub fn evaulate_bool_expr(
    app: SharedAppState,
    expr: BoolExpr,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("boolean evaluate: {:?}", expr);
    let reply_bool = evaluate_bool(&mut app.lock().expect("failure"), &expr);
    let reply = json!({ "result": reply_bool });
    Ok(warp::reply::json(&reply))
}

// POST /api/debug/eval_value
pub fn evaulate_value_expr(
    app: SharedAppState,
    expr: Value,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("value evaluate: {:?}", expr);
    let reply_value = evaluate_val(&mut app.lock().expect("failure"), &expr);
    let reply = json!({ "result": reply_value });
    Ok(warp::reply::json(&reply))
}

// GET /sensors
pub fn all_sensors(_app: SharedAppState) -> Result<impl warp::Reply, warp::Rejection> {
    let reply = json!({ "result": [] });
    Ok(warp::reply::json(&reply))
}

// POST /device/:name/toggle/:index
pub fn toggle_switch(
    app: SharedAppState,
    device: String,
    index: usize,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.switch_toggle(device, index) {
        Ok(reply) => Ok(warp::reply::json(&reply)),
        Err(_e) => Err(warp::reject::not_found()),
    }
}

// PUT /device/:name/switch/:index
pub fn write_switch(
    app: SharedAppState,
    device: String,
    index: usize,
    body: bool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.switch_set(device, index, body) {
        Ok(reply) => Ok(warp::reply::json(&reply)),
        Err(_e) => Err(warp::reject::not_found()),
    }
}

// GET /device/:name/sensors
pub fn device_sensors(
    app: SharedAppState,
    device: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.sensor_count(&device) {
        Ok(count) => {
            let range: Vec<crate::i2c::Result<(f64, crate::config::value::Unit)>> = (0..count)
                .into_iter()
                .map(|index| app_l.read_sensor(device.clone(), index))
                .collect();
            Ok(warp::reply::json(&range))
        }
        Err(e) => Err(warp::reject::custom(e)),
    }
}

// GET /device/:name/sensors/:index
pub fn read_sensor(
    app: SharedAppState,
    device: String,
    index: usize,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app_l = app.lock().expect("failure");
    match app_l.read_sensor(device, index) {
        Ok(reply) => Ok(warp::reply::json(&reply)),
        Err(_e) => Err(warp::reject::not_found()),
    }
}

/// Produce a json-compatible error report
pub fn customize_error(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(err) = err.find_cause::<Error>() {
        let code = match err {
            Error::IoError(_) => 1001,
            Error::InvalidPinDirection => 1008,
            Error::I2cError(_) => 1016,
            Error::NonExistant(_) => 1017,
            Error::OutOfBounds(_) => 1019,
            Error::RecvError(_) => 1020,
            Error::UnitError(_) => 1021,
            Error::SendError(_) => 1024,
        };
        let message = err.to_string();

        let json = json!({ "type": "error", "code": code, "message": message });

        Ok(warp::reply::with_status(
            json.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        Err(err)
    }
}
