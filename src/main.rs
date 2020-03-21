extern crate pretty_env_logger;

#[macro_use]
extern crate log;

extern crate sled;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate warp;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rust_embed;

use crate::config::Config;
use rppal::system::DeviceInfo;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use warp::{
    filters::path::Tail,
    http::header::{HeaderMap, HeaderValue},
    path, Filter,
};

mod app;
mod config;
mod storage;
mod i2c;
mod webapp;

/// big picture:
/// read configuration and decide what sensors and switches are available. start up application, then
/// start running state machine. finally, present a rest api to the outside world to interact with the
/// application.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=restedpi=debug` to see debug logs,
        env::set_var("RUST_LOG", "restedpi=info");
        info!("defaulting to info level logging. RUST_LOG='restedpi=info'");
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
                database: None,
                listen: None,
                port: None,
                devices: None,
            }
        }
    };

    let db_path = config.database.unwrap_or(std::path::PathBuf::from("rested-pi.db"));

    let listen = config.listen.unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let device_config = config.devices.unwrap_or(Vec::new());

    let mut app_raw = app::new(&db_path).expect("app failed to start");
    for config in device_config.iter() {
        app_raw.add_device(config).expect("pre-configured device to not fail to reset");
    }
    app_raw.reset()?;

    let app_m = Arc::new(Mutex::new(app_raw));
    let app = warp::any().map(move || app_m.clone());

    // Limit incoming body length to 16kb
    const LIMIT: u64 = 1024 * 16;

    let r_config = warp::get2()
        .and(app.clone())
        .and(warp::any().map(move || server_name.clone()))
        .and(path!("config"))
        .map(webapp::server_config);

    let r_config_check = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "config_check"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(webapp::evaluate_config_check);

    let r_eval_bool = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "eval_bool"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(webapp::evaulate_bool_expr);

    let r_eval_value = warp::post2()
        .and(app.clone())
        .and(path!("debug" / "eval_value"))
        .and(warp::body::content_length_limit(LIMIT))
        .and(warp::body::json())
        .and_then(webapp::evaulate_value_expr);

    let r_sensors = warp::get2()
        .and(app.clone())
        .and(path!("sensors"))
        .and_then(webapp::all_sensors);

    let r_device_sensors = warp::get2()
        .and(app.clone())
        .and(path!(String / "sensors"))
        .and_then(webapp::device_sensors);

    let r_read = warp::get2()
        .and(app.clone())
        .and(path!(String / "sensors" / usize))
        .and_then(webapp::read_sensor);

    let r_write = warp::put2()
        .and(app.clone())
        .and(path!(String / "switches" / usize))
        .and(warp::body::json())
        .and_then(webapp::write_switch);

    let r_toggle = warp::post2()
        .and(app.clone())
        .and(path!(String / "switches" / "toggle" / usize))
        .and_then(webapp::toggle_switch);

    let mut nocache_header = HeaderMap::new();
    nocache_header.insert("cache-control", HeaderValue::from_static("no-store"));

    let index_html = warp::get2()
        .and_then(|| webapp::serve("index.html"))
        .with(warp::reply::with::headers(nocache_header));

    let r_static = warp::get2()
        .and(warp::any())
        .and(warp::path::tail())
        .and_then(|tail: Tail| webapp::serve(tail.as_str()));

    let r_available = warp::get2()
        .and(path!("available"))
        .and(app.clone())
        .and_then(webapp::all_devices);

    let r_adding_configured = warp::post2()
        .and(app.clone())
        .and(warp::body::json())
        .and_then(webapp::add_device);

    let r_remove_configured = warp::delete2()
        .and(app.clone())
        .and(warp::path::param())
        .and_then(webapp::remove_device);

    let r_fetching_all_configured = warp::get2()
        .and(app.clone())
        .and_then(webapp::configured_devices);

    let r_update_configured_device = warp::put2()
        .and(app.clone())
        .and(warp::path::param())
        .and(warp::body::json())
        .and_then(webapp::edit_configured_device);

    let r_fetching_configured_device = warp::get2()
        .and(app.clone())
        .and(warp::path::param())
        .and_then(webapp::configured_device);

    let r_configured = warp::path("configured").and(
        r_adding_configured
            .or(r_fetching_configured_device)
            .or(r_update_configured_device)
            .or(r_fetching_all_configured)
            .or(r_remove_configured),
    );

    let r_devices = warp::path("devices").and(
        r_read
            .or(r_toggle)
            .or(r_write)
            .or(r_device_sensors)
            .or(r_available)
            .or(r_configured),
    );

    let api = r_static
        .or(path!("api").and(
            r_config
                .or(r_sensors)
                .or(r_devices)
                .or(r_config_check)
                .or(r_eval_bool)
                .or(r_eval_value),
        ))
        .or(index_html)
        .recover(webapp::customize_error);

    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    info!("RestedPi listening: http://{}", addr);
    warp::serve(api.with(warp::log("restedpi::access"))).run(addr);

    Ok(())
}
