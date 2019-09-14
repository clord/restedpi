pub mod error;
pub mod bus;
pub mod mcp23017;
pub mod mcp9808;
pub mod bmp085;
pub mod util;

use std::result;

pub type Result<T> = result::Result<T, error::Error>;
