use crate::app;
use crate::config::boolean::{evaluate as evaluate_bool, BoolExpr};
use crate::config::value::{evaluate as evaluate_val, Value};
use crate::i2c::device::{Device, Status};
use crate::i2c::error::Error;
use mime_guess::from_path;
use serde_derive::Serialize;
use serde_json::json;
use std::borrow::Cow;

use warp::filters::path::Tail;
use warp::{http::Response, http::StatusCode, reject, reply, Rejection, Reply};
pub mod slugify;

use std::sync::{Arc, Mutex};
// We have to share the app state since warp uses a thread pool
type SharedAppState = Arc<Mutex<app::channel::AppChannel>>;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub async fn static_serve(path: &str) -> Result<impl Reply, Rejection> {
    let asset_a: Option<Cow<'static, [u8]>> = Asset::get(path);
    let mime = from_path(path).first_or_octet_stream();
    match asset_a {
        Some(file) => Ok(Response::builder()
            .header("content-type", mime.to_string())
            .body(file)),
        None => {
            let path_b: String = format!("{}/index.js", path);
            let asset_b: Option<Cow<'static, [u8]>> = Asset::get(path_b.as_str());
            let mime = from_path(path_b).first_or_octet_stream();
            match asset_b {
                Some(file) => Ok(Response::builder()
                    .header("content-type", mime.to_string())
                    .body(file)),
                None => {
                    // let asset_b: Option<Cow<'static, [u8]>> = Asset::get("index.html");
                    // let mime = from_path("index.html").first_or_octet_stream();
                    // match asset_b {
                    //     Some(file) => Ok(Response::builder()
                    //         .header("content-type", mime.to_string())
                    //         .body(file)),
                         Err(reject::not_found())
                }
            }
        }
    }
}
pub async fn static_serve_tail(path: Tail) -> Result<impl Reply, Rejection> {
    static_serve(path.as_str()).await
}

/// These are our API handlers, the ends of each filter chain.
/// Notice how thanks to using `Filter::and`, we can define a function
/// with the exact arguments we'd expect from each filter in the chain.
/// No tuples are needed, it's auto flattened for the functions.
mod handlers {
    use crate::app::channel::AppChannel;
    use serde_json::json;
    use std::convert::Infallible;
    use warp::{reply, Reply};

    pub async fn list_devices(app: AppChannel) -> Result<impl Reply, Infallible> {
        let reply = json!([]);
        Ok(reply::json(&reply))
    }

    pub async fn get_configuration(app: AppChannel) -> Result<impl Reply, Infallible> {
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
}

pub mod filters {
    use super::handlers;
    use super::SharedAppState;
    use crate::app::channel::AppChannel;
    use std::convert::Infallible;
    use warp::filters::path::Tail;
    use warp::http::header::{HeaderMap, HeaderValue};
    use warp::{any, get, path, reply, Filter, Rejection, Reply};

    fn with_app(
        app: SharedAppState,
    ) -> impl Filter<Extract = (AppChannel,), Error = Infallible> + Clone {
        any().map(move || app.clone().lock().expect("locked state").clone())
    }

    pub fn api(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        path("api")
            .and(devices(app.clone()).or(configuration(app)))
            .or(static_filter())
            .or(static_index_html())
    }

    fn static_index_html() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        get().and_then(|| super::static_serve("index.html"))
    }

    fn static_filter() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        get()
            .and(warp::path::tail())
            .and_then(super::static_serve_tail)
    }

    fn configuration(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let mut short_cache_header = HeaderMap::new();
        short_cache_header.insert(
            "cache-control",
            HeaderValue::from_static("private, max-age=4"),
        );
        warp::path!("configuration")
            .and(get())
            .and(with_app(app))
            .and_then(handlers::get_configuration)
            .with(reply::with::headers(short_cache_header))
    }

    fn devices(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        devices_list(app.clone())
            .or(devices_create(app.clone()))
            .or(devices_update(app.clone()))
            .or(devices_delete(app))
    }

    fn devices_list(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("devices")
            .and(get())
            .and(with_app(app))
            .and_then(handlers::list_devices)
    }

    fn devices_create(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("devices")
            .and(warp::post())
            .and(with_app(app))
            .and_then(handlers::list_devices)
    }

    fn devices_update(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("devices")
            .and(warp::put())
            .and(with_app(app))
            .and_then(handlers::list_devices)
    }

    fn devices_delete(
        app: SharedAppState,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!("devices")
            .and(warp::delete())
            .and(with_app(app))
            .and_then(handlers::list_devices)
    }
}

// pub fn device_as_json(device: &Device) -> (config::Device, Status) {
//     (device.config(), device.status())
// }

// pub fn devices_as_json(mut app: AppChannel) -> serde_json::value::Value {
//     let d: HashMap<String, (config::Device, Status)> = app
//         .all_devices()
//         .into_iter()
//         .map(|(name, device)| (name.to_owned(), device_as_json(&device)))
//         .collect();
//     json!(d)
// }

// GET /devices/configured
//
// configured devices in the system
// pub async fn configured_devices(app: SharedAppState) -> Result<impl Reply, Rejection> {
//     let app = {app.lock().expect("failure").clone()};
//     let reply = devices_as_json(app);
//     Ok(reply::json(&reply))
// }

// PUT /devices/configured/:name
//
// update some configured device in the system
// pub fn edit_configured_device(
//     app: SharedAppState,
//     name: String,
//     new_config: config::Device,
// ) -> Result<impl Reply, Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.edit_device(&name, &new_config) {
//         Ok(d) => {
//             let reply = device_as_json(&d);
//             Ok(reply::json(&reply))
//         }
//         Err(Error::NonExistant(_)) => Err(warp::reject::not_found()),
//         Err(e) => Err(warp::reject::custom(e)),
//     }
// }

// GET /devices/configured/:name
//
// configured device in the system
// pub fn configured_device(app: SharedAppState, name: String) -> Result<impl Reply, Rejection> {
//     let app = {app.lock().expect("failure").clone()};
//     match app.device(&name) {
//         Ok(d) => {
//             let reply = device_as_json(&d);
//             Ok(reply::json(&reply))
//         }
//         Err(Error::NonExistant(_)) => Err(warp::reject::not_found()),
//         Err(e) => Err(warp::reject::custom(e)),
//     }
// }

// pub fn remove_device(app: SharedAppState, name: String) -> Result<impl Reply, Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     app.remove_device(&name);
//     let reply = devices_as_json(app);
//     Ok(reply::json(&reply))
// }

// pub fn add_device(app: SharedAppState, device: config::Device) -> Result<impl Reply, Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.add_device(&device) {
//         Err(e) => Err(warp::reject::custom(e)),
//         Ok(()) => {
//             let reply = devices_as_json(app);
//             Ok(reply::json(&reply))
//         }
//     }
// }

//// GET /api/config
//pub fn server_config(_app: SharedAppState, server_name: String) -> impl warp::Reply {
//    let reply = json!(
//        {"serverConfig": {
//        "deviceName": format!("restedpi on {}", server_name),
//        }
//    });
//    warp::reply::json(&reply)
//}

//// POST /api/debug/check_config
//pub fn evaluate_config_check(
//    _app: SharedAppState,
//    expr: config::Config,
//) -> Result<impl warp::Reply, warp::Rejection> {
//    debug!("config: {:?}", expr);
//    //let app = app.lock().expect("failure");
//    Ok(warp::reply::json(&expr))
//}

// POST /api/debug/eval_bool
// pub fn evaulate_bool_expr(
//     app: SharedAppState,
//     expr: BoolExpr,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     debug!("boolean evaluate: {:?}", expr);
//     let reply_bool = evaluate_bool(&mut app, &expr);
//     let reply = json!({ "result": reply_bool });
//     Ok(warp::reply::json(&reply))
// }

// // POST /api/debug/eval_value
// pub fn evaulate_value_expr(
//     app: SharedAppState,
//     expr: Value,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     debug!("value evaluate: {:?}", expr);
//     let reply_value = evaluate_val(&mut app.lock().expect("failure"), &expr);
//     let reply = json!({ "result": reply_value });
//     Ok(warp::reply::json(&reply))
// }

// #[derive(Debug, Clone, Serialize)]
// #[serde(tag = "status")]
// enum SensorResult {
//     Ok { value: f64, unit: config::Unit },
//     Err { error: String },
// }

// GET /sensors
// pub fn all_sensors(app: SharedAppState) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     let mut sensor_values: HashMap<&str, Vec<SensorResult>> = HashMap::new();

//     for (name, device) in app.devices() {
//         for i in 0..device.sensor_count() {
//             let res = match device.read_sensor(i) {
//                 Ok((f, u)) => SensorResult::Ok { value: f, unit: u },
//                 Err(e) => SensorResult::Err {
//                     error: format!("Sensor Error: {:#?}", e),
//                 },
//             };
//             sensor_values
//                 .entry(name)
//                 .and_modify(|e| e.push(res.clone()))
//                 .or_insert(vec![res]);
//         }
//     }
//     let reply = json!(sensor_values);
//     Ok(warp::reply::json(&reply))
// }

// POST /device/:name/toggle/:index
// pub fn toggle_switch(
//     app: SharedAppState,
//     device: String,
//     index: usize,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.switch_toggle(device, index) {
//         Ok(reply) => Ok(warp::reply::json(&reply)),
//         Err(_e) => Err(warp::reject::not_found()),
//     }
// }

// PUT /device/:name/switch/:index
// pub fn write_switch(
//     app: SharedAppState,
//     device: String,
//     index: usize,
//     body: bool,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.switch_set(device, index, body) {
//         Ok(reply) => Ok(warp::reply::json(&reply)),
//         Err(_e) => Err(warp::reject::not_found()),
//     }
// }

// GET /device/:name/sensors
// pub fn device_sensors(
//     app: SharedAppState,
//     device: String,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.sensor_count(&device) {
//         Ok(count) => {
//             let range: Vec<crate::i2c::Result<(f64, crate::config::value::Unit)>> = (0..count)
//                 .into_iter()
//                 .map(|index| app.read_sensor(device.clone(), index))
//                 .collect();
//             Ok(warp::reply::json(&range))
//         }
//         Err(e) => Err(warp::reject::custom(e)),
//     }
// }

// GET /device/:name/sensors/:index
// pub fn read_sensor(
//     app: SharedAppState,
//     device: String,
//     index: usize,
// ) -> Result<impl warp::Reply, warp::Rejection> {
//     let mut app = {app.lock().expect("failure").clone()};
//     match app.read_sensor(device, index) {
//         Ok(reply) => Ok(warp::reply::json(&reply)),
//         Err(_e) => Err(warp::reject::not_found()),
//     }
// }

/// Produce a json-compatible error report
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    if let Some(err) = err.find::<Error>() {
        let code = match err {
            Error::IoError(_) => 1001,
            Error::InvalidPinDirection => 1008,
            Error::I2cError(_) => 1016,
            Error::NonExistant(_) => 1017,
            Error::OutOfBounds(_) => 1019,
            Error::RecvError(_) => 1020,
            Error::UnitError(_) => 1021,
            Error::SendError(_) => 1024,
            Error::StorageError(_) => 1120,
            Error::EncodingError(_) => 1121,
            Error::InputNotFound(_) => 1150,
            Error::OutputNotFound(_) => 1151,
        };

        let message = err.to_string();
        error!("Error code {}: {}", code, message);
        let json = json!({ "type": "error", "code": code, "message": message });
        Ok(reply::with_status(
            json.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else if err.is_not_found() {
        let json = json!({ "type": "error", "code": 404, "message": "Not found" });
        Ok(reply::with_status(json.to_string(), StatusCode::NOT_FOUND))
    } else {
        error!("unhandled error: {:?}", err);
        let json = json!({ "type": "error", "code": 500, "message": "Internal Server Error" });
        Ok(reply::with_status(
            json.to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
