use crate::app::AppState;
use crate::config::sched;
use crate::config::{BoolExpr, Value};
use chrono::prelude::*;
use chrono::Duration;

pub fn evaluate_value(app: &AppState, expr: &Value) -> f64 {
    match expr {
        Value::Const(a) => *a,

        Value::Sensor(name, unit) => match app.read_sensor(name.to_string(), *unit) {
            Ok(value) => value,
            std::result::Result::Err(e) => {
                error!("Failed to read sensor: {}", e);
                std::f64::NAN
            }
        },

        Value::Sub(a, b) => evaluate_value(app, a) - evaluate_value(app, b),
        Value::Add(a, b) => evaluate_value(app, a) + evaluate_value(app, b),
        Value::Mul(a, b) => evaluate_value(app, a) * evaluate_value(app, b),

        Value::OffsetForLong { long } =>
            sched::exact_offset_hrs(
                evaluate_value(app, long)
            ),

        Value::HourAngleSunrise { lat, doy } =>
            sched::hour_angle_sunrise(
                evaluate_value(app, lat),
                sched::noon_decl_sun(evaluate_value(app, doy)),
            )
            .to_degrees(),

        Value::NoonSunDeclinationAngle { doy } =>
            sched::noon_decl_sun(evaluate_value(app, doy)),

        Value::HoursOfDaylight { lat, doy } => {
            sched::day_length_hrs(
                evaluate_value(app, lat),
                evaluate_value(app, doy))
        }

        Value::HourOfSunset {lat, long, doy} => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate_value(app, doy);
            let h = sched::hour_angle_sunrise(
                evaluate_value(app, lat),
                sched::noon_decl_sun(doy_ev),
            ).to_degrees() / 15.0;
            let exact_offset = sched::exact_offset_hrs(evaluate_value(app, long));
            let solar_offset =  (h + 2.0*h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset * 3600.0) as i32).yo(dt.year(), doy_ev as u32).and_hms(0,0,0) + Duration::seconds(solar_offset as i64);
            let local = solar_dt.with_timezone(&dt.timezone());
            local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0
        }

        Value::HourOfSunrise {lat, long, doy} => {
            let dt: DateTime<Local> = app.current_dt();
            let doy_ev = evaluate_value(app, doy);
            let h = sched::hour_angle_sunrise(
                evaluate_value(app, lat).to_radians(),
                sched::noon_decl_sun(doy_ev),
            ).to_degrees() / 15.0;

            let exact_offset = sched::exact_offset_hrs(evaluate_value(app, long));
            debug!("ha: {}, sn: {}", h, exact_offset);
            let solar_offset = (h) * 3600.0;
            let solar_dt = FixedOffset::east((exact_offset  * 3600.0) as i32).yo(dt.year(), doy_ev as u32).and_hms(0,0,0) + Duration::seconds(solar_offset as i64);
            debug!("solar: {}", solar_dt);
            let local = solar_dt.with_timezone(&dt.timezone());
            debug!("local: {} ({:?})", local, dt.timezone());
            local.hour() as f64 + local.minute() as f64 / 60.0 + local.second() as f64 / 3600.0
        }

        Value::HourOfDay  => {
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
            let hour_ratio = (dt.hour() as f64 )
                + (dt.minute() as f64 / 60.0f64)
                + (dt.second() as f64 / 3600.0f64);
            dt.ordinal() as f64 + (hour_ratio / 24.0)
        }

        Value::WeekDayFromMonday => {
            let dt: DateTime<Local> = app.current_dt();
            dt.weekday().number_from_monday() as f64
        }

        Value::Lerp(a, t, b) => {
            let tev = evaluate_value(app, t);
            let aev = evaluate_value(app, a);
            let bev = evaluate_value(app, b);
            aev * (1f64 - tev) + bev * tev
        }

        Value::Linear(a, x, b) => {
            evaluate_value(app, a) * evaluate_value(app, x) + evaluate_value(app, b)
        }

        Value::Trunc(x) => evaluate_value(app, x).trunc(),

        Value::Inverse(v) => 1.0f64 / evaluate_value(app, v),
    }
}

pub fn evaluate_bool(app: &AppState, expr: &BoolExpr) -> bool {
    match expr {
        BoolExpr::Equal(a, b) => evaluate_value(app, a) == evaluate_value(app, b),
        BoolExpr::EqualPlusOrMinus(a, b, c) => {
            (evaluate_value(app, a) - evaluate_value(app, b)).abs() < evaluate_value(app, c)
        }
        BoolExpr::MoreThan(a, b) => evaluate_value(app, a) > evaluate_value(app, b),
        BoolExpr::LessThan(a, b) => evaluate_value(app, a) < evaluate_value(app, b),
        BoolExpr::Between(a, b, c) => {
            evaluate_value(app, a) <= evaluate_value(app, b)
                && evaluate_value(app, b) <= evaluate_value(app, c)
        }
        BoolExpr::And(a, b) => evaluate_bool(app, &*a) && evaluate_bool(app, &*b),
        BoolExpr::Or(a, b) => evaluate_bool(app, &*a) || evaluate_bool(app, &*b),
        BoolExpr::Not(b) => !evaluate_bool(app, &*b),
    }
}
