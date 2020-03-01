pub mod boolean;
pub mod value;

use boolean::BoolExpr;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
pub use value::Unit;

pub mod sched;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum SamplingMode {
    UltraLowPower,
    Standard,
    HighRes,
    UltraHighRes,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum SunPosition {
    Set,
    High,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Type {
    MCP9808,
    BMP085 {
        mode: SamplingMode,
    },
    MCP23017 {
        bank0: HashMap<usize, SwitchPin>,
        bank1: HashMap<usize, SwitchPin>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SwitchPin {
    pub direction: String,
    pub active_low: bool,
    pub schedule: Option<BoolExpr>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub model: Type,
    pub name: String,
    pub description: String,
    pub address: u16,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen: Option<String>,
    pub port: Option<u16>,
    pub database_path: Option<String>,
    pub devices: Option<Vec<Device>>,
}
