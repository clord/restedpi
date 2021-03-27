use crate::app::state::State;
use crate::config::sched;
use crate::config::{DateTimeValue, LocationValue, Unit, Value};
use crate::error::Result;
use async_recursion::async_recursion;
use chrono::offset::LocalResult;
use chrono::prelude::*;
use chrono::Duration;
use std::str::FromStr;
use tracing::error;

pub enum ParseUnitError {
    NotKnown,
}

fn doy_for_dt(dt: DateTime<Local>) -> f64 {
    dt.ordinal0() as f64
        + (dt.hour() as f64 / 24.0f64)
        + ((dt.minute() as f64 / 24.0f64) / 60.0f64)
        + ((dt.second() as f64 / 24.0f64) / 3600.0f64)
}

fn lat_for_loc(app: &State, location: &LocationValue) -> f64 {
    match location {
        LocationValue::Here => app.lat(),
        LocationValue::LatLong(lat, _) => *lat,
    }
}

fn long_for_loc(app: &State, location: &LocationValue) -> f64 {
    match location {
        LocationValue::Here => app.long(),
        LocationValue::LatLong(_, long) => *long,
    }
}

fn dt_for_datetime(app: &State, datetime: &DateTimeValue) -> DateTime<Local> {
    match datetime {
        DateTimeValue::Now => app.current_dt(),
        DateTimeValue::SpecificDTZ(v) => *v,
        DateTimeValue::SpecificDate(v) => match Local.from_local_date(v) {
            LocalResult::None => {
                error!("invalid date {:?}", v);
                Local.timestamp(0, 0)
            }
            LocalResult::Single(s) => s.and_hms(0, 0, 0),
            LocalResult::Ambiguous(s, x) => {
                error!("ambiguous date {:?} {:?} {:?}", v, s, x);
                s.and_hms(0, 0, 0)
            }
        },
        DateTimeValue::SpecificDT(v) => match Local.from_local_datetime(v) {
            LocalResult::None => {
                error!("invalid date {:?}", v);
                Local.timestamp(0, 0)
            }
            LocalResult::Single(s) => s,
            LocalResult::Ambiguous(s, x) => {
                error!("ambiguous date {:?} {:?} {:?}", v, s, x);
                s
            }
        },
    }
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

/// An evaluator for value expressions.
#[async_recursion]
pub async fn evaluate(app: &State, expr: &Value) -> Result<f64> {
    let res = match expr {
        Value::Const(a) => Ok(*a),

        Value::ReadInput(input_name, unit) => {
            let value = app.read_input_value(input_name).await?;
            if *unit == value.1 {
                Ok(value.0)
            } else {
                Err(crate::error::Error::UnitError(format!("{:?}", unit)))
            }
        }
        Value::Sub(a, b) => Ok(evaluate(app, a).await? - evaluate(app, b).await?),
        Value::Add(a, b) => Ok(evaluate(app, a).await? + evaluate(app, b).await?),
        Value::Mul(a, b) => Ok(evaluate(app, a).await? * evaluate(app, b).await?),
        Value::Div(a, b) => Ok(evaluate(app, a).await? / evaluate(app, b).await?),

        Value::HourOffset(location) => Ok(sched::exact_offset_hrs(long_for_loc(app, location))),

        Value::HourAngleSunrise(location, datetime) => Ok(sched::hour_angle_sunrise(
            lat_for_loc(app, location),
            sched::noon_decl_sun(doy_for_dt(dt_for_datetime(app, datetime))),
        )
        .to_degrees()),

        Value::NoonSunDeclinationAngle(datetime) => Ok(sched::noon_decl_sun(doy_for_dt(
            dt_for_datetime(app, datetime),
        ))),

        Value::HoursOfDaylight(location, datetime) => Ok(sched::day_length_hrs(
            lat_for_loc(app, location),
            doy_for_dt(dt_for_datetime(app, datetime)),
        )),

        Value::HourOfSunset(location, datetime) => {
            let dt = dt_for_datetime(app, datetime);
            let doy_ev = doy_for_dt(dt);
            let lat = lat_for_loc(app, location);
            let long = long_for_loc(app, location);
            let h = sched::hour_angle_sunrise(lat.to_radians(), sched::noon_decl_sun(doy_ev))
                .to_degrees()
                / 15.0;
            let exact_offset = sched::exact_offset_hrs(long);
            let solar_offset = (12.0 + h / 2.0) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            let local = solar_dt.with_timezone(&dt.timezone());
            Ok(local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0)
        }

        Value::HourOfSunrise(location, datetime) => {
            let dt = dt_for_datetime(app, datetime);
            let doy_ev = doy_for_dt(dt);
            let lat = lat_for_loc(app, location);
            let long = long_for_loc(app, location);

            let h = sched::hour_angle_sunrise(lat.to_radians(), sched::noon_decl_sun(doy_ev))
                .to_degrees()
                / 15.0;

            let exact_offset = sched::exact_offset_hrs(long);
            let solar_offset = (12.0 - h / 2.0) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32)
                .yo(dt.year(), doy_ev as u32)
                .and_hms(0, 0, 0)
                + Duration::seconds(solar_offset as i64);
            let local = solar_dt.with_timezone(&dt.timezone());
            Ok(local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0)
        }

        Value::MinuteOfHour(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(dt.minute() as f64 + (dt.second() as f64 / 3600.0f64))
        }
        Value::HourOfDay(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(
                dt.hour() as f64
                    + (dt.minute() as f64 / 60.0f64)
                    + (dt.second() as f64 / 3600.0f64),
            )
        }

        Value::Year(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(dt.year() as f64)
        }

        Value::MonthOfYear(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(dt.month() as f64)
        }

        Value::DayOfMonth(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(dt.day() as f64)
        }

        Value::DayOfYear(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            let hour_ratio = (dt.hour() as f64)
                + (dt.minute() as f64 / 60.0f64)
                + (dt.second() as f64 / 3600.0f64);
            Ok(dt.ordinal() as f64 + (hour_ratio / 24.0))
        }

        Value::WeekDayFromMonday(vdt) => {
            let dt = dt_for_datetime(app, vdt);
            Ok(dt.weekday().number_from_monday() as f64)
        }

        Value::Lerp(a, t, b) => {
            let tev = evaluate(app, t).await?;
            let aev = evaluate(app, a).await?;
            let bev = evaluate(app, b).await?;
            Ok(aev * (1f64 - tev) + bev * tev)
        }

        Value::Linear(a, x, b) => {
            Ok(evaluate(app, a).await? * evaluate(app, x).await? + evaluate(app, b).await?)
        }
        Value::Trunc(x) => Ok(evaluate(app, x).await?.trunc()),
        Value::Inverse(v) => Ok(1.0f64 / evaluate(app, v).await?),
    };
    res
}
