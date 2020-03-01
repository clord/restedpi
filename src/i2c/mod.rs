pub mod bmp085;
pub mod bus;
pub mod device;
pub mod error;
pub mod mcp23017;
pub mod mcp9808;
pub mod util;

/// Represent all common results of i2c
pub type Result<T> = std::result::Result<T, error::Error>;
