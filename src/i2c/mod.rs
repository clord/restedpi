pub mod bmp085;
pub mod bus;
pub mod error;
pub mod mcp23017;
pub mod mcp9808;
pub mod util;

use crate::config::value::Unit;
use serde_derive::{Deserialize, Serialize};
use std::result;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub enum DeviceType {
    Bmp085,
    Mcp23017,
    Mcp9808,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub enum Pullup {
    On,
    Off,
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub enum Direction {
    Output,
    Input(Pullup),
}

/// Generically read or write a device
pub trait Device {
    fn reset(&mut self) -> Result<()>;
    fn address(&self) -> bus::Address;
    fn status(&self) -> String;
    fn name(&self) -> &str;
    fn description(&self) -> String;
    fn device_type(&self) -> DeviceType;

    // Sensor
    fn sensor_count(&self) -> usize;
    fn read_sensor(&self, index: usize) -> Result<(f64, Unit)>;

    // Switch
    fn switch_count(&self) -> usize;
    fn set_direction(&mut self, index: usize, dir: Direction) -> Result<()>;
    fn switch_direction(&mut self, index: usize) -> Result<Direction>;
    fn write_switch(&mut self, index: usize, value: bool) -> Result<()>;
    fn read_switch(&mut self, index: usize) -> Result<bool>;
}

/// Represent all common results of i2c
pub type Result<T> = result::Result<T, error::Error>;
