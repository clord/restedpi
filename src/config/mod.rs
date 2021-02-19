#![feature(async_closure)]
pub mod boolean;
pub mod parse;
pub mod sched;
pub mod value;

use serde_derive::{Deserialize, Serialize};
use crate::session::AppContext;
use std::collections::HashMap;
use std::path::PathBuf;
use juniper::{GraphQLEnum, GraphQLUnion, GraphQLObject, graphql_object};
pub use parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug, GraphQLEnum)]
pub enum SamplingMode {
    UltraLowPower,
    Standard,
    HighRes,
    UltraHighRes,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug, GraphQLEnum)]
pub enum SunPosition {
    Set,
    High,
}

#[derive(Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct MCP9808Config {
    pub address: i32
}

#[derive(Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct BMP085Config {
    pub address: i32,
    pub mode: SamplingMode,
}

#[derive(Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct Directions {
     pub p0: Dir,
     pub p1: Dir,
     pub p2: Dir,
     pub p3: Dir,
     pub p4: Dir,
     pub p5: Dir,
     pub p6: Dir,
     pub p7: Dir,
}
impl Directions {
    pub fn new() -> Self {
        Directions { 
            p0: Dir::OutH,
            p1: Dir::OutH,
            p2: Dir::OutH,
            p3: Dir::OutH,
            p4: Dir::OutH,
            p5: Dir::OutH,
            p6: Dir::OutH,
            p7: Dir::OutH,
        }
    }
    pub fn get(&self, pin: usize) -> &Dir {
        match pin % 8 {
           0 =>  &self.p0,
           1 =>  &self.p1,
           2 =>  &self.p2,
           3 =>  &self.p3,
           4 =>  &self.p4,
           5 =>  &self.p5,
           6 =>  &self.p6,
           7 =>  &self.p7,
           _ =>  &self.p0,
        }
    }
    pub fn get_mut(&mut self, pin: usize) -> &mut Dir {
        match pin % 8 {
           0 => &mut self.p0,
           1 => &mut self.p1,
           2 => &mut self.p2,
           3 => &mut self.p3,
           4 => &mut self.p4,
           5 => &mut self.p5,
           6 => &mut self.p6,
           7 => &mut self.p7,
           _ => &mut self.p0,
        }
    }
}

#[derive( Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct MCP23017Config {
        pub address: i32,
        pub bank_a: Directions,
        pub bank_b: Directions,
}

#[derive(Copy, Serialize, Deserialize, GraphQLUnion, PartialEq, Clone, Debug)]
#[serde(tag = "name")]
pub enum Type {
    MCP9808(MCP9808Config),
    BMP085(BMP085Config),
    MCP23017(MCP23017Config),
}

#[derive(Serialize, Deserialize, GraphQLEnum, PartialEq, PartialOrd, Copy, Clone, Debug)]
pub enum Dir {
    // Active High output
    OutH,
    // Active Low output
    OutL,
    In,
    InWithPD,
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

#[graphql_object]
impl Device {
    pub fn model(&self) -> Type {
        self.model
    } 
    pub fn name(&self) -> &str {
        self.name.as_str()
    } 
    pub fn disabled(&self) -> Option<bool> {
        self.disabled
    } 
    pub fn description(&self) -> &str {
        self.description.as_str()
    } 
}
#[derive(Serialize, Deserialize, GraphQLObject, Debug, PartialEq, Clone)]
pub struct InputValue {
    pub value: f64,
    pub unit: Unit
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Input {
    pub name: String,
    pub input_id: Option<String>,
    pub device_id: String,
    pub device_input_id: u32,
    pub unit: Unit,
}

#[graphql_object(context = AppContext)]
impl Input {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn input_id(&self) -> Option<&String> {
        self.input_id.as_ref()
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }

    pub async fn device(&self, context: &AppContext) -> Option<Device> {
        context.channel().get_device_config(&self.device_id).await.ok().map(|(cfg,_, _)| cfg)
    }

    pub async fn bool_value(&self, context: &AppContext) -> Option<bool> {

        match self.input_id.as_ref() {
         Some(id) =>     context.channel().read_boolean(id).await.ok()
                ,
             None => None
        }
    }
    pub async fn value(&self, context: &AppContext) -> Option<InputValue> {
        match self.input_id.as_ref() {
         Some(id) =>     context.channel().read_value(id).await.ok().map(|(value, unit)| 
                 InputValue {value, unit}
                ),
             None => None
        }
    }
}


/**
 * we can write a boolean value to a given device via name
 */
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Output {
    pub name: String,
    pub output_id: Option<String>,
    pub device_id: String,
    pub device_output_id: u32,
    pub active_low: Option<bool>,
    pub unit: Unit,

    // If set to an expression, the system will compute this output every state change and write it to the output
    pub on_when: Option<String>,
}

#[graphql_object(context = AppContext)]
impl Output {
    pub fn output_id(&self) -> Option<&String> {
        self.output_id.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }

    pub async fn device(&self, context: &AppContext) -> Option<Device> {
        context.channel().get_device_config(&self.device_id).await.ok().map(|(cfg,_, _)| cfg)
    }

    pub fn active_low(&self) -> Option<bool> {
        self.active_low
    }

    pub fn on_when(&self) -> Option<String> {
        self.on_when.clone()
    }

    pub async fn value(&self, context: &AppContext) -> Option<bool> {
        match self.output_id.as_ref() {
            Some(oid) => context.channel().current_output_value(oid).await.ok(),
            None => None,
        }
    }
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

    // Map from username to hashed passwords
    pub users: Option<HashMap<String, String>>,
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
mod tests {
    // use crate::config::boolean;
    // use crate::config::{Config, Device, Input, Output, Type};
    // use std::path::PathBuf;
}
