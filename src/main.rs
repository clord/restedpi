extern crate pretty_env_logger;

#[macro_use]
extern crate log;

extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate sled;
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
use warp::Filter;

mod app;
mod auth;
mod config;
mod i2c;
mod storage;
mod webapp;

/// big picture:
/// read configuration and decide what sensors and switches are available. start up application, then
/// start running state machine. finally, present a rest api to the outside world to interact with the
/// application.
#[tokio::main]
async fn main() {
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
            warn!("invalid config file: {}", e);
            "".to_string()
        }
    };

    let config = match serde_json::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("error parsing config, using defaults. error: {}", e);
            Config::new()
        }
    };

    let listen = config.listen.clone().unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let key_and_cert = config.key_and_cert_path.clone();

    let app = app::channel::start_app(config).expect("app failed to start");
    let app = Arc::new(Mutex::new(app));

    // Limit incoming body length to 16kb
 //   const LIMIT: u64 = 1024 * 16;


    // let r_available_devices = warp::get()
    //     .and(path!("available-devices"))
    //     .and(app.clone())
    //     .map(webapp::available_devices);

    // let r_adding_configured = warp::post()
    //     .and(app.clone())
    //     .and(warp::body::json())
    //     .map(webapp::add_device);

    // let r_remove_configured = warp::delete()
    //     .and(app.clone())
    //     .and(warp::path::param())
    //     .map(webapp::remove_device);

    // let r_fetching_all_configured = warp::get()
    //     .and(app.clone())
    //     .map(webapp::configured_devices);

    // let r_update_configured_device = warp::put()
    //     .and(app.clone())
    //     .and(warp::path::param())
    //     .and(warp::body::json())
    //     .map(webapp::edit_configured_device);

    // let r_fetching_configured_device = warp::get()
    //     .and(app.clone())
    //     .and(warp::path::param())
    //     .map(webapp::configured_device);

    // let r_configured = warp::path("configured").and(
    //     r_adding_configured
    //         .or(r_fetching_configured_device)
    //         .or(r_update_configured_device)
    //         .or(r_fetching_all_configured)
    //         .or(r_remove_configured),
    // );

    // let r_devices = warp::path("devices").and(
    //         // .or(r_configured),
    // );
    //

    let api = webapp::filters::api(app);
    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    let serve = warp::serve(
        api.with(warp::log("restedpi::access"))
            .recover(webapp::handle_rejection),
    );
    if let Some((key_path, cert_path)) = key_and_cert {
        info!("RestedPi listening: https://{}", addr);
        serve.tls().cert_path(cert_path).key_path(key_path).run(addr).await
    } else {
        error!("Missing keys in configuration; can't start in TLS mode. set key_and_cert_path");
        return ()
    }
}

