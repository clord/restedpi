use std::error::Error;
//use rppal::i2c::I2c;
use rppal::system::DeviceInfo;
use warp::{self, path, Filter};

fn main() -> Result<(), Box<dyn Error>> {
    //let i2c = I2c::new()?;
    let serverName = DeviceInfo::new()?.model()

    println!("    Running on {}", serverName);

    let restedpiserver = path!("bmp085" / String).map(|index| serverName)

    Ok(())
}
