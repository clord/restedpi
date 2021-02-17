use crate::error::Error;
use crate::session::{authenticate, AppContext};
use rppal::system::DeviceInfo;
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{reject, reply, Rejection, Reply};

fn do_result<T: serde::Serialize>(input: Result<T, Error>) -> Result<impl Reply, Rejection> {
    match input {
        Ok(r) => Ok(reply::json(&r)),
        Err(e) => Err(reject::custom(e)),
    }
}

pub async fn list_devices(ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().all_devices())
}

pub async fn add_or_replace_device(
    device_id: String,
    device: crate::config::Device,
    ctx: AppContext,
) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().add_or_replace_device(device_id, device))
}

pub async fn remove_device(device_id: String, ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().remove_device(device_id))
}

pub async fn read_input(input_id: String, ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().read_value(input_id))
}

pub async fn list_outputs(ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().all_outputs())
}

pub async fn add_or_replace_output(
    output_id: String,
    output: crate::config::Output,
    ctx: AppContext,
) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().add_or_replace_output(output_id, output))
}

pub async fn remove_output(output_id: String, ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().remove_output(output_id))
}

pub async fn list_inputs(ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().all_inputs())
}

pub async fn add_or_replace_input(
    input_id: String,
    input: crate::config::Input,
    ctx: AppContext,
) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().add_or_replace_input(input_id, input))
}

pub async fn remove_input(input_id: String, ctx: AppContext) -> Result<impl Reply, Rejection> {
    do_result(ctx.channel().remove_input(input_id))
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

pub async fn authentication(
    ctx: AppContext,
    form: HashMap<String, String>,
) -> Result<impl Reply, Rejection> {
    let user = form.get("username").unwrap();
    let pw = form.get("password").unwrap();
    match authenticate(ctx, user, pw).await {
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
