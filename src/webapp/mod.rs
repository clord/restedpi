use std::borrow::Cow;
use crate::app;
use mime_guess::from_path;
use serde_json::{json};
use warp::{ http::Response, Reply, Rejection, reply};

// We have to share the app state since warp uses a thread pool
pub type SharedAppState = std::sync::Arc<std::sync::Mutex<app::AppState>>;

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;

pub extern fn serve(path: &str) -> Result<impl Reply, Rejection> {
  let mime = from_path(path).first_or_octet_stream();
  let asset: Option<Cow<'static, [u8]>> = Asset::get(path);
  let file = asset.ok_or_else(|| warp::reject::not_found())?;

  Ok(Response::builder()
      .header("content-type", mime.to_string())
      .body(file)
    )
}

// GET /devices/available
//
// available devices that can be configured on this system
// (not configured devices)
pub extern fn all_devices(
    _app: SharedAppState
) -> Result<impl Reply, Rejection> {
    let reply = json!({ "result": [
        { "name": "BMP085"
        , "device": "/api/devices/available/bmp085"
        , "description": "High accuracy temperature and pressure"
        , "datasheet": "https://www.sparkfun.com/datasheets/Components/General/BST-BMP085-DS000-05.pdf"
        , "bus": "I2C"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +85°C  ±0.1°C" }, {"type": "pressure", "range": "300 to 1100hPa"}]
        },
        { "name": "MCP23017"
        , "device": "/api/devices/available/mcp23017"
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
        , "device": "/api/devices/available/mcp9808"
        , "description": "High-accuracy temperature sensor"
        , "datasheet": "http://ww1.microchip.com/downloads/en/DeviceDoc/25095A.pdf"
        , "bus": "I2C"
        , "sensors": [{ "type": "temperature", "range": "-40°C to +125°C ±0.5°C" }]
        }
    ] });
    Ok(reply::json(&reply))
}


// GET /devices/configured
//
// configured devices in the system
pub extern fn configured_devices(
    _app: SharedAppState
) -> Result<impl Reply, Rejection> {
    let reply = json!({ "result": [
        { "name": "Configured Device 1"
        , "description": "A User-entered description for the device"
        , "device": "/api/devices/available/mcp9808"
        , "status": "ok"
        , "config": {
            "bus": {
                "type": "i2c",
                "address": "23"
            },
        }
        }
    ] });
    Ok(reply::json(&reply))
}

pub extern fn add_device(
    _app: SharedAppState
) -> Result<impl Reply, Rejection> {
    let reply = json!({
        "result": []
    });
    Ok(reply::json(&reply))
}

