use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

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

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorType {
    MCP9808 { address: u16 },
    BMP085 { address: u16, mode: SamplingMode },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SwitchPin {
    pub direction: String,
    pub active_low: bool,
    pub schedule: Option<BoolExpr>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SwitchType {
    MCP23017 {
        address: u16,
        bank0: HashMap<usize, SwitchPin>,
        bank1: HashMap<usize, SwitchPin>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Switch {
    pub description: String,
    pub device: SwitchType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sensor {
    pub description: String,
    pub device: SensorType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen: Option<String>,
    pub port: Option<u16>,
    pub sensors: Option<HashMap<String, Sensor>>,
    pub switches: Option<HashMap<String, Switch>>,
}

pub enum ParseUnitError {
    NotKnown,
}

impl FromStr for Unit {
    type Err = ParseUnitError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "degc" => Ok(Unit::DegC),
            "kpa" => Ok(Unit::KPa),
            _ => Err(ParseUnitError::NotKnown),
        }
    }
}
