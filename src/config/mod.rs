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
        pins: HashMap<usize, Pin>,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Pin {
    Input {
        /**
         * Indicates the pin is reading a device that is active low, hence, invert the input.
         */
        active_low: bool,
    },
    Output {
        /**
         * Indicates the pin is controlling a device that is active low, hence, invert the output.
         */
        active_low: bool,

        /**
         * Some: expression that is evaluated each tick to determine what the pin should be set to
         * None: Value set by user via actions.
         */
        value: Option<BoolExpr>,

        /**
         * Some: (and value is Some) if expr is true, `value` will not be computed.
         */
        disable_automatic: Option<BoolExpr>,
    },
}

impl Pin {
    pub fn is_active_low(&self) -> bool {
        match self {
            Pin::Output { active_low, .. } => *active_low,
            Pin::Input { active_low, .. } => *active_low,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub model: Type,
    pub name: String,
    pub description: String,
    pub disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen: Option<String>,
    pub port: Option<u16>,
    pub database: Option<PathBuf>,
    pub devices: Option<Vec<Device>>,
}
