use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod eval;
pub mod sched;

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum SamplingMode {
    UltraLowPower,
    Standard,
    HighRes,
    UltraHighRes,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Unit {
    DegC,
    KPa,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum SunPosition {
    Set,
    High,
}

/// A source of f64 values, usable in expressions
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Value {
    // Some constant
    Const(f64),

    // angle of the sun (declination at noon, in radians)
    NoonSunDeclinationAngle { doy: Box<Value> },

    // hour-angle of sun at sunrise at a given lat and doy
    HourAngleSunrise { lat: Box<Value>, doy: Box<Value> },

    // How many hours of daylight are in day-of-year at latitude
    HoursOfDaylight { lat: Box<Value>, doy: Box<Value> },

    // hour of day since midnight of this day
    HourOfDay,

    // Day of year, with fractional
    DayOfYear,

    // Mon=1, ..., Sun=7
    WeekDayFromMonday,

    // 2018, 2019...
    Year,

    // 1=Jan, 2=Feb
    MonthOfYear,

    // 1, 2, ... 30, 31
    DayOfMonth,

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

    // y = x + y
    Add(Box<Value>, Box<Value>),
    // y = x - y
    Sub(Box<Value>, Box<Value>),
    // y = x * y
    Mul(Box<Value>, Box<Value>),

    // y = 1/x, x != 0
    Inverse(Box<Value>),

    // remove any floating point values (round-to-zero)
    Trunc(Box<Value>),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
