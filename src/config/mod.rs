pub mod boolean;
pub mod parse;
pub mod sched;
pub mod value;

use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

pub use parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum SamplingMode {
    UltraLowPower,
    Standard,
    HighRes,
    UltraHighRes,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum SunPosition {
    Set,
    High,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(tag = "name")]
pub enum Type {
    MCP9808 {
        address: u16,
    },
    BMP085 {
        address: u16,
        mode: SamplingMode,
    },
    MCP23017 {
        address: u16,
        pin_direction: [Dir; 16],
    },
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub enum Dir {
    // Active High output
    OutH,
    // Active Low output
    OutL,
    In(bool),
}

/**
 * Data for devices
 */
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Device {
    pub model: Type,
    pub name: String,
    pub description: String,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Input {
    pub name: String,
    pub device_id: String,
    pub device_input_id: u32,
    pub unit: Unit,
}

/**
 * we can write a boolean value to a given device via name
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Output {
    pub name: String,
    pub device_id: String,
    pub device_output_id: u32,
    pub active_low: Option<bool>,
    pub unit: Unit,

    // If set to an expression, the system will compute this output every state change and write it to the output
    pub on_when: Option<String>,
}

/**
 * Top level configuration of the system
 */
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    // name of device (defaults to device name)
    pub name: Option<String>,

    // Where to listen for connections
    pub listen: Option<String>,
    pub port: Option<u16>,

    // Location to use as "here" value
    pub lat: f64,
    pub long: f64,

    // tls key and cert in that order
    pub key_and_cert_path: Option<(PathBuf, PathBuf)>,

    // List of allowed user passwords
    pub user_keys: Option<Vec<String>>
}

impl Config {
    pub fn new() -> Self {
        Config {
            name: None,
            listen: None,
            port: None,
            lat: 0.0,
            long: 0.0,
            key_and_cert_path: None,
            user_keys: None,
        }
    }

    pub fn here(&self) -> LocationValue {
        LocationValue::LatLong(self.lat, self.long)
    }

    pub fn check_config(&self) -> Vec<ConfigError> {
        let errors = Vec::<ConfigError>::new();
        return errors;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IORef {
    InputRef { input_id: String },
    OutputRef { output_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissingReason {
    Missing,
    Disabled,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigError {
    DuplicateIoId {
        io_id: IORef,
    },
    DuplicateDeviceId {
        device_id: String,
    },
    IORefersToMissingOrDisabledDevice {
        io: IORef,
        device_id: String,
        reason: MissingReason,
    },
    IORefersToNonExistantDevicePin {
        io: IORef,
        pin_id: u32,
    }, // could check that i2c addresses are valid
}

#[cfg(test)]
mod tests {
    // use crate::config::boolean;
    // use crate::config::{Config, Device, Input, Output, Type};
    // use std::path::PathBuf;
}
