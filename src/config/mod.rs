pub mod boolean;
pub mod parse;
pub mod sched;
pub mod types;
pub mod value;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use types::LocationValue;

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

    // determines which bus to talk on for i2c
    pub i2cbus: Option<u8>,

    // Location to use as "here" value
    pub lat: f64,
    pub long: f64,

    // The app secret file, a file containing a reasonable length of random bytes.
    // when it changes, all sessions are invalidated.
    // if unset, we'll use the environment variable APP_SECRET for the secret value.
    pub app_secret_path: Option<PathBuf>,

    // tls key and cert in that order
    pub key_and_cert_path: Option<(PathBuf, PathBuf)>,

    // Use path of config file if unset, otherwise, directory containing rpi.sql3
    pub db_path: Option<PathBuf>,

    // Map from username to hashed passwords
    pub users: Option<HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            name: None,
            listen: None,
            i2cbus: None,
            port: None,
            lat: 0.0,
            long: 0.0,
            db_path: None,
            app_secret_path: None,
            key_and_cert_path: None,
            users: None,
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
mod tests {}
