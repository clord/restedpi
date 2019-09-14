use i2c_bus::{i2c_io, bmp085, mcp9808, mcp23017};
use rppal::system::DeviceInfo;
use serde_json::json;
use std::error::Error;
use std::sync::{Arc, Mutex};
use warp::{self, http::Response, path, Filter};

mod i2c_bus;

fn main() -> Result<(), Box<dyn Error>> {
    let server_name = DeviceInfo::new()?.model();
    let i2c = i2c_io::start();

    let d_mcp9808_0 = Arc::new(Mutex::new(mcp9808::Device::new(0x18u16, i2c.clone())?));
    let d_mcp9808_1 = Arc::new(Mutex::new(mcp9808::Device::new(0x19u16, i2c.clone())?));
    let d_mcp9808_2 = Arc::new(Mutex::new(mcp9808::Device::new(0x1au16, i2c.clone())?));
    let d_mcp23017_1 = Arc::new(Mutex::new(mcp23017::Device::new(0x20u16, i2c.clone())?));
    let d_mcp23017_2 = Arc::clone(&d_mcp23017_1);
    let d_bmp085 = Arc::new(Mutex::new(bmp085::Device::new(0x77u16, i2c.clone(), bmp085::SamplingMode::HighRes)?));

    println!("** Running on {}", server_name);

    let r_greeting = warp::get2().and(warp::path::end()).map(move || {
        Response::builder()
            .header("content-type", "application/json")
            .body(json!({ "server": format!("{}", server_name) }).to_string())
    });

    let r_mcp23017_get = warp::get2()
        .and(path!("mcp23017" / String / usize))
        .map(move |banknum, pinnum| {
            let bank = if banknum == "B" {
                mcp23017::Bank::B
            } else {
                mcp23017::Bank::A
            };

            let pin = match mcp23017::ordinal_pin(pinnum) {
                Some(pin) => pin,
                None => mcp23017::Pin::Pin0,
            };

            let mut data = d_mcp23017_1.lock().unwrap();

            let value = data.get_pin(bank, pin).expect("problem getting pin");

            Response::builder()
                .header("content-type", "application/json")
                .body(json!({ "value": value }).to_string())
        });

    let r_mcp23017_put = warp::put2()
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

            let mut data = d_mcp23017_2.lock().unwrap();

            data.set_pin(bank, pin, value).expect("problem setting pin");

            Response::builder()
                .header("content-type", "application/json")
                .body(json!({ "value": value }).to_string())
        });

    let r_mcp9808 = warp::get2().and(path!("mcp9808" / usize)).map(move |index| {
        let device_mx = match index {
            0 => Arc::clone(&d_mcp9808_0),
            1 => Arc::clone(&d_mcp9808_1),
            2 => Arc::clone(&d_mcp9808_2),
            _ => panic!("bad index")
        };

        let device = device_mx.lock().unwrap();

        let temperature: f32 = device.read_temp().expect("failed to read temperature");

        Response::builder()
            .header("content-type", "application/json")
            .body(json!({ "temperature": temperature }).to_string())
    });

    let r_bmp085 = warp::get2().and(path!("bmp085" / usize)).map(move |index| {
        if index != 0 {
            panic!("bad index")
        }
        let device = d_bmp085.lock().unwrap();
        let pressure: f32 = device.pressure_kpa().expect("failed to read pressure");
        let temperature: f32 = device.temperature_in_c().expect("failed to read temperature");
        Response::builder()
            .header("content-type", "application/json")
            .body(json!({ "temperature": temperature, "pressure": pressure }).to_string())
    });

    let routes = r_greeting.or(r_bmp085).or(r_mcp9808).or(r_mcp23017_get).or(r_mcp23017_put);

    warp::serve(routes).run(([0, 0, 0, 0], 3030));

    Ok(())
}
