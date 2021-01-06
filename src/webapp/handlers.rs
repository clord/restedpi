use super::WebSession;
use crate::app::channel::AppChannel;
use rppal::system::DeviceInfo;
use std::collections::HashMap;
use crate::auth::{password, token};
use std::convert::Infallible;
use warp::{reject, reply, Rejection, Reply};

use serde_json::json;

pub async fn list_devices(_session: WebSession, app: AppChannel) -> Result<impl Reply, Rejection> {
    match app.all_devices() {
        Ok(r) => Ok(reply::json(&r)),
        Err(e) => Err(reject::custom(e)),
    }
}

pub async fn add_or_replace_device(
    device_id: String,
    _session: WebSession,
    device: crate::config::Device,
    app: AppChannel,
) -> Result<impl Reply, Rejection> {
    match app.add_or_replace_device(device_id, device) {
        Ok(r) => Ok(reply::json(&r)),
        Err(e) => Err(reject::custom(e)),
    }
}

pub async fn remove_device(
    device_id: String,
    _session: WebSession,
    app: AppChannel,
) -> Result<impl Reply, Rejection> {
    match app.remove_device(device_id) {
        Ok(r) => Ok(reply::json(&r)),
        Err(e) => Err(reject::custom(e)),
    }
}

pub async fn server_name() -> Result<impl Reply, Infallible> {
    let server_name = match DeviceInfo::new() {
        Ok(model) => model.model().to_string(),
        Err(e) => {
            warn!("reading model: {}", e);
            "Unknown".to_string()
        }
    };

    let reply = json!({ "server-name": server_name });
    Ok(reply::json(&reply))
}

fn hash_for(user: &str) -> String {
    // look in config for user's hash
    return "".to_string()
}

pub async fn authentication(
    app: AppChannel,
    form: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let secret = std::env::var("APP_SECRET").expect("Failed to read APP_SECRET environment variable");
    let user = form.get("username").unwrap();
    let pw = form.get("password").unwrap();
    match password::verify(pw, &hash_for(user)) {
        Ok(true) => {
            match token::make_token(WebSession { version: 1 }, &secret) {
                Ok(token) => {
                    let reply = json!({ "token": token });
                    Ok(reply::json(&reply))
                }, 
                Err(_e) => {
                    Err(reject::reject())
                }
            }
        }, _ => {
            Err(reject::reject())
        }
    }
}

pub async fn get_available_devices(
    _session: WebSession,
    _app: AppChannel,
) -> Result<impl Reply, Infallible> {
    let reply = json!([
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
    ]);
    Ok(reply::json(&reply))
}
