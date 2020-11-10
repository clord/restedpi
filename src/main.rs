extern crate pretty_env_logger;

#[macro_use]
extern crate log;

extern crate bit_array;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate sled;
extern crate typenum;
extern crate warp;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate rust_embed;

use crate::config::Config;
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
    // let x = config::Device{
    //  model: config::Type::MCP23017 {
    //    address: 67,
    //    pin_direction:
    //        [Dir::Out, Dir::Out, Dir::In(false), Dir::Out, Dir::Out, Dir::In(true), Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, Dir::Out, ]
    //  },
    //  name: "device name".to_string(),
    //  description: "Description of device".to_string(),
    //  disabled: None
    // } ;

    // println!("{}", serde_json::to_string(&x).expect("foo"));

    if env::var_os("LOG").is_none() {
        // Set `RUST_LOG=restedpi=debug` to see debug logs,
        env::set_var("LOG", "restedpi=info");
        info!("defaulting to info level logging. RUST_LOG='restedpi=info'");
    }
    pretty_env_logger::init_custom_env("LOG");

    // let server_name = match DeviceInfo::new() {
    //     Ok(model) => model.model().to_string(),
    //     Err(e) => {
    //         warn!("reading model: {}", e);
    //         "Unknown".to_string()
    //     }
    // };

    // If ~/.config/restedpi/startup.json exists, use as config,
    let config_dir_config_file = dirs::config_dir().map(|x| {
        let mut y = x.clone();
        y.push("restedpi");
        y.push("startup.json");
        y
    });

    // otherwise use /etc/restedpi/startup.json
    let etc_dir_config_file = std::path::PathBuf::from("/etc/restedpi/startup.json");

    let config_file = {
        match config_dir_config_file {
            Some(path) => {
                if path.exists() {
                    path
                } else {
                    etc_dir_config_file
                }
            }
            None => etc_dir_config_file,
        }
    };

    let contents = match fs::read_to_string(config_file) {
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

    let api = webapp::filters::api(app);
    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

    let serve = warp::serve(
        api.with(warp::log("restedpi::access"))
            .recover(webapp::handle_rejection),
    );
    if let Some((key_path, cert_path)) = key_and_cert {
        info!("RestedPi listening: https://{}", addr);
        serve
            .tls()
            .cert_path(cert_path)
            .key_path(key_path)
            .run(addr)
            .await
    } else {
        error!("Missing keys in configuration; can't start in TLS mode. set key_and_cert_path");
        return ();
    }
}
