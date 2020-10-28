pub mod boolean;
pub mod value;

use boolean::BoolExpr;
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
        pins: [Mcp23017PinConfig; 16],
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PinDirection {
    Input,
    Output,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Mcp23017PinConfig {
    /// Whether the pin should be input or output
    pub direction: PinDirection,
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
        input_id: usize,
    },

    /**
     * Read a boolean from the given device
     */
    BoolFromDevice {
        name: String,
        device_id: String,
        input_id: usize,
        active_low: bool,
    },

    /**
     * We can read a single boolean
     */
    BoolFromVariable,

    /**
     * returns the result of an expresion
     */
    ExpressionResult(BoolExpr),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Output {
    /**
     * we can write a boolean value to a given device via name
     */
    BoolToDevice {
        name: String,
        device_id: String,
        output_id: usize,

        // If set to an expression, the system will compute this output every tick,
        // and ignore user interference.
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
    // Where to listen for connections
    pub listen: Option<String>,
    pub port: Option<u16>,

    // where to store state
    pub database: Option<PathBuf>,

    // available devices on this host
    pub devices: HashMap<String, Device>,

    // configured inputs and outputs of system
    pub inputs: HashMap<String, Input>,
    pub outputs: HashMap<String, Output>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            database: None,
            listen: None,
            port: None,
            devices: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    pub fn check_config(&self) -> Vec<ConfigError> {
        let mut errors = Vec::<ConfigError>::new();

        // Checks the whole config for validity,
        // meaning inputs/outputs map to real devices and real device capabilities.
        if self.devices.len() == 0 {
            errors.push(ConfigError::DeviceListEmpty)
        }

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
    DeviceListEmpty,
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
