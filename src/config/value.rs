use crate::app::state::State;
use crate::config::sched;
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

    // Current value of a sensor i of device x
    Sensor(String, usize, Unit),

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
pub fn evaluate(app: &State, expr: &Value) -> f64 {
    match expr {
        Value::Const(a) => *a,

        Value::Sensor(name, index, unit) => match app.read_sensor(name, *index) {
            Ok(value) => {
                if *unit == value.1 {
                    value.0
                } else {
                    error!(
                        "type mismatch on sensor. provided {:?}, user demands {:?}",
                        value.1, *unit
                    );
                    std::f64::NAN
                }
            }
            std::result::Result::Err(e) => {
                error!("Failed to read sensor: {}", e);
                std::f64::NAN
            }
        },

        Value::Sub(a, b) => evaluate(app, a) - evaluate(app, b),
        Value::Add(a, b) => evaluate(app, a) + evaluate(app, b),
        Value::Mul(a, b) => evaluate(app, a) * evaluate(app, b),

        Value::OffsetForLong { long } => sched::exact_offset_hrs(evaluate(app, long)),

        Value::HourAngleSunrise { lat, doy } => {
            sched::hour_angle_sunrise(evaluate(app, lat), sched::noon_decl_sun(evaluate(app, doy)))
                .to_degrees()
        }

        Value::NoonSunDeclinationAngle { doy } => sched::noon_decl_sun(evaluate(app, doy)),

        Value::HoursOfDaylight { lat, doy } => {
            sched::day_length_hrs(evaluate(app, lat), evaluate(app, doy))
        }

        Value::HourOfSunset { lat, long, doy } => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate(app, doy);
            let h = sched::hour_angle_sunrise(
                evaluate(app, lat).to_radians(),
                sched::noon_decl_sun(doy_ev),
            )
            .to_degrees()
                / 15.0;
            let exact_offset = sched::exact_offset_hrs(evaluate(app, long));
            let solar_offset = (12.0 + h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            let local = solar_dt.with_timezone(&dt.timezone());
            local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0
        }

        Value::HourOfSunrise { lat, long, doy } => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate(app, doy);
            let h = sched::hour_angle_sunrise(
                evaluate(app, lat).to_radians(),
                sched::noon_decl_sun(doy_ev),
            )
            .to_degrees()
                / 15.0;

            let exact_offset = sched::exact_offset_hrs(evaluate(app, long));
            debug!("ha: {}, sn: {}", h, exact_offset);
            let solar_offset = (12.0 - h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            debug!("solar: {}", solar_dt);
            let local = solar_dt.with_timezone(&dt.timezone());
            debug!("local: {} ({:?})", local, dt.timezone());
            local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0
        }

        Value::HourOfDay => {
            let dt: DateTime<Local> = app.current_dt();
            dt.hour() as f64 + (dt.minute() as f64 / 60.0f64) + (dt.second() as f64 / 3600.0f64)
        }

        Value::Year => {
            let dt: DateTime<Local> = app.current_dt();
            dt.year() as f64
        }

        Value::MonthOfYear => {
            let dt: DateTime<Local> = app.current_dt();
            dt.month() as f64
        }

        Value::DayOfMonth => {
            let dt: DateTime<Local> = app.current_dt();
            dt.day() as f64
        }

        Value::DayOfYear => {
            let dt: DateTime<Local> = app.current_dt();
            let hour_ratio = (dt.hour() as f64)
                + (dt.minute() as f64 / 60.0f64)
                + (dt.second() as f64 / 3600.0f64);
            dt.ordinal() as f64 + (hour_ratio / 24.0)
        }

        Value::WeekDayFromMonday => {
            let dt: DateTime<Local> = app.current_dt();
            dt.weekday().number_from_monday() as f64
        }

        Value::Lerp(a, t, b) => {
            let tev = evaluate(app, t);
            let aev = evaluate(app, a);
            let bev = evaluate(app, b);
            aev * (1f64 - tev) + bev * tev
        }

        Value::Linear(a, x, b) => evaluate(app, a) * evaluate(app, x) + evaluate(app, b),

        Value::Trunc(x) => evaluate(app, x).trunc(),

        Value::Inverse(v) => 1.0f64 / evaluate(app, v),
    }
}
