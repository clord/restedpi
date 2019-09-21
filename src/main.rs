use i2c::{bus, error::Error};
use rppal::system::DeviceInfo;
use serde_json::json;
use warp::{self, http::Response, http::StatusCode, path, Filter, Rejection, Reply};

mod app;
mod i2c;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_name = DeviceInfo::new()?.model();
    println!("** starting up on {}", server_name);

    let app = app::start();

    println!("** Running");

    let r_greeting = warp::get2().and(warp::path::end()).map(move || {
        Response::builder()
            .header("content-type", "application/json")
            .body(json!({ "server": format!("{}", server_name) }).to_string())
    });

    // let r_mcp23017_get =
    //     warp::get2()
    //         .and(path!("mcp23017" / String / usize))
    //         .map(move |banknum, pinnum| {
    //             let bank = if banknum == "B" {
    //                 mcp23017::Bank::B
    //             } else {
    //                 mcp23017::Bank::A
    //             };

    //             let pin = match mcp23017::ordinal_pin(pinnum) {
    //                 Some(pin) => pin,
    //                 None => mcp23017::Pin::Pin0,
    //             };

    //             let mut data = d_mcp23017_1.lock().unwrap();

    //             let value = data.get_pin(bank, pin).expect("problem getting pin");

    //             Response::builder()
    //                 .header("content-type", "application/json")
    //                 .body(json!({ "value": value }).to_string())
    //         });

    // let r_mcp23017_put = warp::put2()
    //     .and(path!("mcp23017" / String / usize / bool))
    //     .map(move |banknum, pinnum, value| {
    //         let bank = if banknum == "B" {
    //             mcp23017::Bank::B
    //         } else {
    //             mcp23017::Bank::A
    //         };
    //         let pin = match mcp23017::ordinal_pin(pinnum) {
    //             Some(pin) => pin,
    //             None => mcp23017::Pin::Pin0,
    //         };

    //         let mut data = d_mcp23017_2.lock().unwrap();

    //         data.set_pin(bank, pin, value).expect("problem setting pin");

    //         Response::builder()
    //             .header("content-type", "application/json")
    //             .body(json!({ "value": value }).to_string())
    //     });

    // let r_mcp9808 = warp::get2()
    //     .and(path!("mcp9808" / usize))
    //     .map(move |index| {
    //         let device_mx = match index {
    //             0 => Arc::clone(&d_mcp9808_0),
    //             1 => Arc::clone(&d_mcp9808_1),
    //             2 => Arc::clone(&d_mcp9808_2),
    //             _ => panic!("bad index"),
    //         };

    //         let device = device_mx.lock().unwrap();

    //         let temperature: f32 = device.read_temp().expect("failed to read temperature");

    //         Response::builder()
    //             .header("content-type", "application/json")
    //             .body(json!({ "temperature": temperature }).to_string())
    //     });

    // let r_bmp085 = warp::get2().and(path!("bmp085" / usize)).map(move |index| {
    //     if index != 0 {
    //         panic!("bad index")
    //     }
    //     let device = d_bmp085.lock().unwrap();
    //     let pressure: f32 = device.pressure_kpa().expect("failed to read pressure");
    //     let temperature: f32 = device
    //         .temperature_in_c()
    //         .expect("failed to read temperature");
    //     Response::builder()
    //         .header("content-type", "application/json")
    //         .body(json!({ "temperature": temperature, "pressure": pressure }).to_string())
    // });

    let sensors_route = warp::get2()
        .and(path!("sensors" / String))
        .map(move |index| Ok("hi"));

    let routes = r_greeting.or(sensors_route);
    //.or(r_bmp085)
    //.or(r_mcp9808)
    //.or(r_mcp23017_get)
    //.or(r_mcp23017_put);

    warp::serve(routes).run(([0, 0, 0, 0], 3030));

    Ok(())
}

fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(err) = err.find_cause::<Error>() {
        let code = match err {
            Error::Io(_) => 1001,
            Error::InvalidPinIndex => 1004,
            Error::InvalidPinDirection => 1008,
            Error::I2cError(_) => 1016,
            Error::UnsupportedUnit(_) => 1019,
            Error::RecvError(_) => 1020,
            Error::SendError(_) => 1024,
        };
        let message = err.to_string();

        let json = json!({ "code": code, "message": message });

        Ok(warp::reply::with_status(
            json.to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else {
        // Could be a NOT_FOUND, or any other internal error... here we just
        // let warp use its default rendering.
        Err(err)
    }
}
