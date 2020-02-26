use crate::app;
use crate::config;
use mime_guess::from_path;
use serde_json::json;
use std::borrow::Cow;
use std::collections::HashMap;
use warp::{http::Response, reply, Rejection, Reply};
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
pub extern "C" fn all_devices(_app: SharedAppState) -> Result<impl Reply, Rejection> {
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
    let d: serde_json::Value = app
        .devices()
        .into_iter()
        .map(|(name, device)| {
            json!(
            { "name": device.name()
            , "address": device.address()
            , "type": device.device_type()
            , "description": device.description()
            , "status": device.status()
            })
        })
        .collect();
    return json!(d);
}

// GET /devices/configured
//
// configured devices in the system
pub extern "C" fn configured_devices(app: SharedAppState) -> Result<impl Reply, Rejection> {
    let mut app_l = app.lock().expect("failure");
    let reply = devices_as_json(app_l);
    Ok(reply::json(&reply))
}

pub extern "C" fn add_device(
    app: SharedAppState,
    devices: HashMap<String, config::Device>,
) -> Result<impl Reply, Rejection> {
    let mut app_l = app.lock().expect("failure");
    app_l.add_devices_from_config(devices);
    let reply = devices_as_json(app_l);
    Ok(reply::json(&reply))
}
