use chrono::{DateTime, Local, NaiveDateTime};
use diesel_derive_enum::DbEnum;
use lrpar::Span;
use serde_derive::{Deserialize, Serialize};

#[derive(Copy, Clone, DbEnum, Serialize, Deserialize, PartialEq, Debug, juniper::GraphQLEnum)]
pub enum Unit {
    Boolean,
    DegC,
    KPa,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LocationValue {
    Here,
    LatLong(f64, f64),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DateTimeValue {
    Now,
    SpecificDT(NaiveDateTime), // use local timezone of server
    SpecificDTZ(DateTime<Local>),
}

/// A source of f64 values, usable in expressions
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    // Some constant
    Const(f64),

    // angle of the sun (declination at noon, in radians)
    NoonSunDeclinationAngle(DateTimeValue),

    // hour-angle of sun at sunrise at a given lat and doy
    HourAngleSunrise(LocationValue, DateTimeValue),

    // How many hours of daylight are in day-of-year at latitude
    HoursOfDaylight(LocationValue, DateTimeValue),

    // Give local time of sunrise and sunset in local time hours
    HourOfSunrise(LocationValue, DateTimeValue),

    HourOfSunset(LocationValue, DateTimeValue),

    // Hour-offset (negative for west, positive for east) of a given longnitude
    HourOffset(LocationValue),

    // fractional minutes since start of this hour
    MinuteOfHour(DateTimeValue),

    // hour of day since midnight of this day
    HourOfDay(DateTimeValue),

    // Day of year, with fractional
    DayOfYear(DateTimeValue),

    // Mon=1, ..., Sun=7
    WeekDayFromMonday(DateTimeValue),

    // 2018, 2019...
    Year(DateTimeValue),

    // 1=Jan, 2=Feb
    MonthOfYear(DateTimeValue),

    // 1, 2, ... 30, 31
    DayOfMonth(DateTimeValue),

    // Current value of an input
    ReadInput(String, Unit),

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
    // y = x / y
    Div(Box<Value>, Box<Value>),

    // y = 1/x, x != 0
    Inverse(Box<Value>),

    // remove any floating point values (round-to-zero)
    Trunc(Box<Value>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum BoolExpr {
    Equal(Span, Value, Value),
    EqualPlusOrMinus(Span, Value, Value, Value),
    MoreThanOrEq(Span, Value, Value),
    LessThanOrEq(Span, Value, Value),
    MoreThan(Span, Value, Value),
    LessThan(Span, Value, Value),
    Between(Span, Value, Value, Value),
    Const(Span, bool),
    EqBool(Span, Box<BoolExpr>, Box<BoolExpr>),
    And(Span, Box<BoolExpr>, Box<BoolExpr>),
    Or(Span, Box<BoolExpr>, Box<BoolExpr>),
    Xor(Span, Box<BoolExpr>, Box<BoolExpr>),
    Not(Span, Box<BoolExpr>),
    ReadBooleanInput(Span, String),
}
