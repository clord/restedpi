
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use crate::app::AppState;
use crate::config::sched;
use crate::config::value::{Value, evaluate as evaluate_value};
use chrono::prelude::*;
use chrono::Duration;

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

/// A very basic parser that evaluates an expression for truth. Can refer to values.
pub fn evaluate(app: &AppState, expr: &BoolExpr) -> bool {
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
        BoolExpr::And(a, b) => evaluate(app, &*a) && evaluate(app, &*b),
        BoolExpr::Or(a, b) => evaluate(app, &*a) || evaluate(app, &*b),
        BoolExpr::Not(b) => !evaluate(app, &*b),
    }
}
