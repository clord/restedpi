use crate::session::{authenticate, AppContext};
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{reject, reply, Rejection, Reply};

pub async fn authentication(
    ctx: AppContext,
    form: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let user = form.get("username").unwrap();
    let pw = form.get("password").unwrap();
    match authenticate(&ctx, user, pw).await {
        Ok(token) => {
            let reply = json!({ "token": token });
            Ok(reply::json(&reply))
        }
        Err(e) => Err(reject::custom(e)),
    }
}

pub async fn get_available_devices(_ctx: AppContext) -> Result<impl Reply, Infallible> {
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
