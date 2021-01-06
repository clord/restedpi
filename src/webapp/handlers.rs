#![feature(duration_saturating_ops)]

use super::WebSession;
use crate::app::channel::AppChannel;
use crate::auth::{password, token};
use crate::error::Error;

use rppal::system::DeviceInfo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

const TOKEN_DURATION: u64 = 60 * 60 * 24 * 660;

pub async fn authentication(
    app: AppChannel,
    form: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let secret =
        std::env::var("APP_SECRET").expect("Failed to read APP_SECRET environment variable");
    let user = form.get("username").unwrap();
    let pw = form.get("password").unwrap();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    match app.hash_for(user) {
        Some(user_hash) => match password::verify(pw, user_hash) {
            Ok(false) => Err(reject::custom(Error::UserNotFound)),
            Ok(true) => match token::make_token(
                WebSession {
                    user: user.clone(),
                    expires: since_the_epoch
                        .checked_add(Duration::new(TOKEN_DURATION, 0))
                        .unwrap()
                        .as_secs(),
                },
                &secret,
            ) {
                Ok(token) => {
                    let reply = json!({ "token": token });
                    Ok(reply::json(&reply))
                }
                Err(e) => {
                    error!("Error generating token: {:?}", e);
                    Err(reject::custom(Error::TokenIssue))
                }
            },
            Err(e) => {
                error!("Password issue: {}", e);
                Err(reject::custom(Error::PasswordIssue))
            }
        },
        None => Err(reject::custom(Error::UserNotFound)),
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
