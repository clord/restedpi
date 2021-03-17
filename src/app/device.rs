use crate::session::AppContext;
use juniper::{graphql_object, FieldError, FieldResult, GraphQLEnum, GraphQLObject, GraphQLUnion};
pub use crate::config::parse::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
pub struct MCP9808Config {
    pub address: i32,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug, GraphQLEnum)]
pub enum SamplingMode {
    UltraLowPower,
    Standard,
    HighRes,
    UltraHighRes,
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
            0 => &self.p0,
            1 => &self.p1,
            2 => &self.p2,
            3 => &self.p3,
            4 => &self.p4,
            5 => &self.p5,
            6 => &self.p6,
            7 => &self.p7,
            _ => &self.p0,
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

#[derive(Copy, Clone, GraphQLObject, Serialize, Deserialize, PartialEq, Debug)]
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

