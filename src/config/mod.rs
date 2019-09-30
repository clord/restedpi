use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Unit {
    DegC,
    KPa,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SwitchType {
    MCP23017,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SunPosition {
    Set,
    High,
}

/// A source of f64 values, usable in expressions
#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    // Some constant
    Const(f64),

    // ratio 0..1 of daylight hours passed by today
    DaylightProgress,

    // angle of the sun (declination at noon, in radians)
    NoonSunDeclinationAngle,

    // How many hours of daylight are in this day
    HoursOfDaylight,

    // Hour of day since midnight of this day
    HourOfDay,

    // Current value of a named sensor
    Sensor(String, Unit),

    // linear interpolation  A * (1 - t) + B * t
    // where:
    //         A           t∈0..1      B
    Lerp(Box<Value>, Box<Value>, Box<Value>),

    // Transform y = Ax + b
    // where:
    //           A           x           b
    Linear(Box<Value>, Box<Value>, Box<Value>),

    // y = 1/x, x != 0
    Inverse(Box<Value>)

}

#[derive(Serialize, Deserialize, Debug)]
pub enum BoolExpr {
    Equal(Value, Value),
    EqualPlusOrMinus(Value, Value, Value),
    MoreThan(Value, Value),
    LessThan(Value, Value),
    Between(Value, Value, Value),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SensorType {
    MCP9808,
    BMP085,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwitchPin {
    pub direction: String,
    pub active_low: bool,
    pub schedule: Option<BoolExpr>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Switch {
    pub device: SwitchType,
    pub description: String,
    pub address: u16,
    pub pins: HashMap<String, SwitchPin>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sensor {
    pub description: String,
    pub device: SensorType,
    pub mode: Option<String>,
    pub address: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub listen: String,
    pub port: Option<u16>,
    pub sensors: HashMap<String, Sensor>,
    pub switches: HashMap<String, Switch>,
}
