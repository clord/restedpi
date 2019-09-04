use i2c_bus::{i2c_io, mcp23017};
use rppal::system::DeviceInfo;
use serde_json::json;
use std::error::Error;
use std::sync::{Arc, Mutex};
use warp::{self, http::Response, path, Filter};

mod i2c_bus;

fn main() -> Result<(), Box<dyn Error>> {
    let server_name = DeviceInfo::new()?.model();
    let i2c = i2c_io::start();
    let d_mcp23017 = Arc::new(Mutex::new(mcp23017::Device::configure(0, i2c.clone())?));
    println!("** Running on {}", server_name);

    let r_greeting = warp::get2().and(warp::path::end()).map(move || {
        Response::builder()
            .header("content-type", "application/json")
            .body(json!({ "server": format!("{}", server_name) }).to_string())
    });

    let r_mcp23017 = warp::put2()
        .and(path!("mcp23017" / String / usize / bool))
        .map(move |banknum, pinnum, value| {
            let bank = if banknum == "B" {
                mcp23017::Bank::B
            } else {
                mcp23017::Bank::A
            };
            let pin = match mcp23017::ordinal_pin(pinnum) {
                Some(pin) => pin,
                None => mcp23017::Pin::Pin0,
            };

            let mut data = d_mcp23017.lock().unwrap();
            data.set_pin(bank, pin, value).expect("problem setting pin");
            Response::builder()
                .header("content-type", "application/json")
                .body(json!({ "value": value }).to_string())
        });

    let r_bmp085 = warp::get2().and(path!("bmp085" / String)).map(|index| {
        // TODO: load data from bmp085 device at specified address
        "Hello World"
    });

    let routes = r_greeting.or(r_bmp085).or(r_mcp23017);

    warp::serve(routes).run(([0, 0, 0, 0], 3030));

    Ok(())
}
