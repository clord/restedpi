pub mod error;
pub mod i2c_io;
pub mod mcp23017;

use std::result;

pub type Result<T> = result::Result<T, error::Error>;
