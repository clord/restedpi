use std::error::Error;
//use rppal::i2c::I2c;
use rppal::system::DeviceInfo;

fn main() -> Result<(), Box<dyn Error>> {
    //let i2c = I2c::new()?;

    println!("Going to talk on I2C of {}", DeviceInfo::new()?.model());

    Ok(())
}
