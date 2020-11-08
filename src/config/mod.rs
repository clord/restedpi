pub mod boolean;
pub mod value;

pub use boolean::BoolExpr;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
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
        // true for input, false for output
        pin_direction: [Dir; 16],
    },
}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub enum Dir {
    Out,
    In(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub model: Type,
    pub name: String,
    pub description: String,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Input {
    /**
     * Read a float from the given device (with a unit)
     */
    FloatWithUnitFromDevice {
        name: String,
        device_id: String,
        device_input_id: usize,
    },

    /**
     * Read a boolean from the given device
     */
    BoolFromDevice {
        name: String,
        device_id: String,
        device_input_id: usize,
        active_low: bool,
    },

    /**
     * We can read a single boolean
     */
    BoolFromVariable,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Output {
    /**
     * we can write a boolean value to a given device via name
     */
    BoolToDevice {
        name: String,
        device_id: String,
        device_output_id: usize,

        // If set to an expression, the system will compute this output every tick and write it to the output
        automation: Option<BoolExpr>,
    },

    /**
     * We can write a boolean that can be retrieved at a later time
     */
    BoolToVariable,
}

/**
 * Top level configuration of the system
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    // name of device (defaults to device name)
    pub name: Option<String>,

    // Where to listen for connections
    pub listen: Option<String>,
    pub port: Option<u16>,

    // tls key and cert in that order
    pub key_and_cert_path: Option<(PathBuf, PathBuf)>,

    // where to store state
    pub database: Option<PathBuf>,

    // available devices on this host
    pub devices: Option<HashMap<String, Device>>,

    // configured inputs and outputs of system
    pub inputs: Option<HashMap<String, Input>>,
    pub outputs: Option<HashMap<String, Output>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            name: None,
            database: None,
            listen: None,
            port: None,
            key_and_cert_path: None,
            devices: None,
            inputs: None,
            outputs: None,
        }
    }

    pub fn check_config(&self) -> Vec<ConfigError> {
        let mut errors = Vec::<ConfigError>::new();
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
        pin_id: usize,
    }, // could check that i2c addresses are valid
}
