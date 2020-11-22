pub mod boolean;
pub mod value;

pub use boolean::BoolExpr;
use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;
pub use value::Unit;

pub mod sched;

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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Output {
    /**
     * we can write a boolean value to a given device via name
     */
    BoolToDevice {
        name: String,
        device_id: String,
        device_output_id: usize,
        active_low: Option<bool>,

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
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Config {
    // name of device (defaults to device name)
    pub name: Option<String>,

    // Where to listen for connections
    pub listen: Option<String>,
    pub port: Option<u16>,

    // tls key and cert in that order
    pub key_and_cert_path: Option<(PathBuf, PathBuf)>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            name: None,
            listen: None,
            port: None,
            key_and_cert_path: None,
        }
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
        pin_id: usize,
    }, // could check that i2c addresses are valid
}

#[cfg(test)]
mod tests {
    use crate::config::boolean;
    use crate::config::{Config, Device, Input, Output, Type};
    use std::path::PathBuf;

    static CONFIG: &str = r#"{
  "listen": "0.0.0.0",
  "key_and_cert_path": [
    "/etc/restedpi/rip.z.odago.ca.key.pem",
    "/etc/restedpi/rip.z.odago.ca.cert.pem"
  ],
}"#;
    static DEVICES: &str = r#"{
    "barom1": {
      "name": "barometer and temp",
      "description": "",
      "model": {
        "name": "BMP085",
        "mode": "HighRes",
        "address": 119
      }
    },
    "temp1": {
      "name": "temp 1",
      "description": "",
      "model": {
        "name": "MCP9808",
        "address": 24
      }
    },
    "temp2": {
      "name": "temp 2",
      "description": "",
      "model": {
        "name": "MCP9808",
        "address": 25
      }
    },
    "temp3": {
      "name": "temp 3",
      "description": "",
      "model": {
        "name": "MCP9808",
        "address": 26
      }
    },
    "main-mcp": {
      "name": "main mcp",
      "description": "Main Switchbank",
      "model": {
        "name": "MCP23017",
        "address": 32,
        "pin_direction": [
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL",
          "OutL"
        ],
        "disabled": null
      }
    }
}"#;
    static OUTPUTS: &str = r#"{
    "lights-1": {
      "BoolToDevice": {
        "name": "Main Lights",
        "device_id": "main-mcp",
        "device_output_id": 3,
        "automation": {
          "And": [
            {
              "And": [
                {
                  "Not": {
                    "Between": [
                      {
                        "Add": [
                          {
                            "Const": 2.5
                          },
                          {
                            "HourOfSunrise": {
                              "lat": {
                                "Const": 54.2779
                              },
                              "long": {
                                "Const": -110.7399
                              },
                              "doy": "DayOfYear"
                            }
                          }
                        ]
                      },
                      "HourOfDay",
                      {
                        "Sub": [
                          {
                            "HourOfSunset": {
                              "lat": {
                                "Const": 54.2779
                              },
                              "long": {
                                "Const": -110.7399
                              },
                              "doy": "DayOfYear"
                            }
                          },
                          {
                            "Const": 2.5
                          }
                        ]
                      }
                    ]
                  }
                },
                {
                  "Between": [
                    {
                      "Const": 6.5
                    },
                    "HourOfDay",
                    {
                      "Const": 22.6
                    }
                  ]
                }
              ]
            },
            {
              "LessThan": [
                {
                  "HoursOfDaylight": {
                    "lat": {
                      "Const": 54.2679
                    },
                    "doy": "DayOfYear"
                  }
                },
                {
                  "Const": 14
                }
              ]
            }
          ]
        }
      }
    },
    "entry-1": {
      "BoolToDevice": {
        "name": "Entryway light",
        "device_id": "main-mcp",
        "device_output_id": 5,
        "automation": {
          "Const": false
        }
      }
    }
}"#;
}
