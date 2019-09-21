pub mod bmp085;
pub mod bus;
pub mod error;
pub mod mcp23017;
pub mod mcp9808;
pub mod util;

use std::result;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Unit {
    // TODO: Maybe investigate other libraries, like dimensioned
    DegC,
    KPa,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Pullup {
    On,
    Off,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
pub enum Direction {
    Output,
    Input(Pullup),
}

/// generically read devices that can act as sensors
pub trait Sensor : Send {
    fn reset(&self) -> Result<()>;
    fn read_sensor(&self, unit: Unit) -> Result<(f64, Unit)>;
}

/// Generically read or write to devices that act as gpio pins
pub trait Switch : Send {
    fn reset(&mut self) -> Result<()>;
    fn pin_count(&self) -> usize;
    fn set_direction(&mut self, index: usize, dir: Direction) -> Result<()>;
    fn write_switch(&mut self, index: usize, value: bool) -> Result<()>;
    fn read_switch(&mut self, index: usize) -> Result<bool>;
}

/// Represent all common results of i2c
pub type Result<T> = result::Result<T, error::Error>;
