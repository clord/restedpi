use crate::app::state::State;
use crate::config::sched;
use crate::i2c::Result;
use chrono::prelude::*;
use chrono::Duration;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Copy, Clone, Serialize, PartialEq, Deserialize, Debug)]
pub enum Unit {
    Boolean,
    DegC,
    KPa,
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

/// A source of f64 values, usable in expressions
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Value {
    // Some constant
    Const(f64),

    // angle of the sun (declination at noon, in radians)
    NoonSunDeclinationAngle {
        doy: Box<Value>,
    },

    // hour-angle of sun at sunrise at a given lat and doy
    HourAngleSunrise {
        lat: Box<Value>,
        doy: Box<Value>,
    },

    // How many hours of daylight are in day-of-year at latitude
    HoursOfDaylight {
        lat: Box<Value>,
        doy: Box<Value>,
    },

    // Give local time of sunrise and sunset in local time hours
    HourOfSunrise {
        lat: Box<Value>,
        long: Box<Value>,
        doy: Box<Value>,
    },
    HourOfSunset {
        lat: Box<Value>,
        long: Box<Value>,
        doy: Box<Value>,
    },

    // Hour-offset (negative for west, positive for east) of a given longnitude
    OffsetForLong {
        long: Box<Value>,
    },

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

    // y = 1/x, x != 0
    Inverse(Box<Value>),

    // remove any floating point values (round-to-zero)
    Trunc(Box<Value>),
}

/// An evaluator for value expressions.
pub fn evaluate(app: &State, expr: &Value) -> Result<f64> {
    match expr {
        Value::Const(a) => Ok(*a),

        Value::ReadInput(input_id, unit) => {
            let value = app.read_input_value(input_id)?;
            if *unit == value.1 {
                Ok(value.0)
            } else {
                Err(crate::i2c::error::Error::UnitError(*unit))
            }
        }
        Value::Sub(a, b) => Ok(evaluate(app, a)? - evaluate(app, b)?),
        Value::Add(a, b) => Ok(evaluate(app, a)? + evaluate(app, b)?),
        Value::Mul(a, b) => Ok(evaluate(app, a)? * evaluate(app, b)?),

        Value::OffsetForLong { long } => Ok(sched::exact_offset_hrs(evaluate(app, long)?)),

        Value::HourAngleSunrise { lat, doy } => Ok(sched::hour_angle_sunrise(
            evaluate(app, lat)?,
            sched::noon_decl_sun(evaluate(app, doy)?),
        )
        .to_degrees()),

        Value::NoonSunDeclinationAngle { doy } => Ok(sched::noon_decl_sun(evaluate(app, doy)?)),

        Value::HoursOfDaylight { lat, doy } => Ok(sched::day_length_hrs(
            evaluate(app, lat)?,
            evaluate(app, doy)?,
        )),

        Value::HourOfSunset { lat, long, doy } => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate(app, doy)?;
            let h = sched::hour_angle_sunrise(
                evaluate(app, lat)?.to_radians(),
                sched::noon_decl_sun(doy_ev),
            )
            .to_degrees()
                / 15.0;
            let exact_offset = sched::exact_offset_hrs(evaluate(app, long)?);
            let solar_offset = (12.0 + h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            let local = solar_dt.with_timezone(&dt.timezone());
            Ok(local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0)
        }

        Value::HourOfSunrise { lat, long, doy } => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate(app, doy)?;
            let h = sched::hour_angle_sunrise(
                evaluate(app, lat)?.to_radians(),
                sched::noon_decl_sun(doy_ev),
            )
            .to_degrees()
                / 15.0;

            let exact_offset = sched::exact_offset_hrs(evaluate(app, long)?);
            debug!("ha: {}, sn: {}", h, exact_offset);
            let solar_offset = (12.0 - h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            debug!("solar: {}", solar_dt);
            let local = solar_dt.with_timezone(&dt.timezone());
            debug!("local: {} ({:?})", local, dt.timezone());
            Ok(local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0)
        }

        Value::HourOfDay => {
            let dt: DateTime<Local> = app.current_dt();
            Ok(
                dt.hour() as f64
                    + (dt.minute() as f64 / 60.0f64)
                    + (dt.second() as f64 / 3600.0f64),
            )
        }

        Value::Year => {
            let dt: DateTime<Local> = app.current_dt();
            Ok(dt.year() as f64)
        }

        Value::MonthOfYear => {
            let dt: DateTime<Local> = app.current_dt();
            Ok(dt.month() as f64)
        }

        Value::DayOfMonth => {
            let dt: DateTime<Local> = app.current_dt();
            Ok(dt.day() as f64)
        }

        Value::DayOfYear => {
            let dt: DateTime<Local> = app.current_dt();
            let hour_ratio = (dt.hour() as f64)
                + (dt.minute() as f64 / 60.0f64)
                + (dt.second() as f64 / 3600.0f64);
            Ok(dt.ordinal() as f64 + (hour_ratio / 24.0))
        }

        Value::WeekDayFromMonday => {
            let dt: DateTime<Local> = app.current_dt();
            Ok(dt.weekday().number_from_monday() as f64)
        }

        Value::Lerp(a, t, b) => {
            let tev = evaluate(app, t)?;
            let aev = evaluate(app, a)?;
            let bev = evaluate(app, b)?;
            Ok(aev * (1f64 - tev) + bev * tev)
        }

        Value::Linear(a, x, b) => Ok(evaluate(app, a)? * evaluate(app, x)? + evaluate(app, b)?),

        Value::Trunc(x) => Ok(evaluate(app, x)?.trunc()),

        Value::Inverse(v) => Ok(1.0f64 / evaluate(app, v)?),
    }
}
