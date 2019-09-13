pub mod error;
pub mod i2c_io;
pub mod mcp23017;
pub mod bmp085;

use std::result;

pub type Result<T> = result::Result<T, error::Error>;
pub type I2cBus = i2c_io::I2cBus;
