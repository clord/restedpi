pub mod bmp085;
pub mod bus;
pub mod error;
pub mod mcp23017;
pub mod mcp9808;
pub mod util;

use crate::config::value::Unit;
use std::result;

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
pub trait Sensor {
    fn read_sensor(&self, unit: Unit) -> Result<f64>;
}

/// Generically read or write to devices that act as gpio pins
pub trait Switch {
    fn pin_count(&self) -> usize;
    fn set_direction(&mut self, index: usize, dir: Direction) -> Result<()>;
    fn write_switch(&mut self, index: usize, value: bool) -> Result<()>;
    fn read_switch(&mut self, index: usize) -> Result<bool>;
}

/// Generically read or write a device
pub trait Device {
    fn reset(&self) -> Result<()>;
    fn address(&self) -> Result<bus::Address>;

    fn sensors(&self) -> Result<[Box<dyn Sensor>]>;
    fn switches(&self) -> Result<[Box<dyn Switch>]>;
}

/// Represent all common results of i2c
pub type Result<T> = result::Result<T, error::Error>;
