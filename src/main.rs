use std::error::Error;
use rppal::i2c::I2c;
use rppal::system::DeviceInfo;
use warp::{self, path, http::Response, Filter};
use serde_json::json;


fn main() -> Result<(), Box<dyn Error>> {
    let server_name = DeviceInfo::new()?.model();

    println!("** Running on {}", server_name);

    match I2c::new() {

        Ok(i2c) => {

            let greeting = warp::path::end().map(move || {
                    Response::builder().header("content-type", "application/json").body(
                        json!({
                            "server": format!("{}", server_name)
                        }).to_string()
                    )
                }
                );

            let  bmp085 =  path!("bmp085" / String).map(|index| {
                // TODO: load data from bmp085 device at specified address 
                "Hello World"
            });

            let routes = warp::get2().and(greeting.or(bmp085));

            warp::serve(routes).run(([0,0,0,0], 3030));

        }
        Err(err) => {
            println!("ERROR: The I2C bus connected to pins 3 and 5 is disabled by default.");
            println!("       You can enable it through `sudo raspi-config`, or by manually adding `dtparam=i2c_arm=on` to `/boot/config.txt`. ");
            println!("       Remember to reboot the Raspberry Pi afterwards.")
        }
    }

    Ok(())
}
