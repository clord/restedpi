use crate::app::state::State;
use crate::config::value::{evaluate as evaluate_value, Value};
use crate::i2c::Result;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum BoolExpr {
    Equal(Value, Value),
    EqualPlusOrMinus(Value, Value, Value),
    MoreThan(Value, Value),
    LessThan(Value, Value),
    Between(Value, Value, Value),
    EqZero(Value),
    NeqZero(Value),
    Const(bool),
    EqBool(Box<BoolExpr>, Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
    ReadBooleanInput(String),
}

/// A very basic parser that evaluates an expression for truth. Can refer to values.
pub fn evaluate(app: &State, expr: &BoolExpr) -> Result<bool> {
    match expr {
        BoolExpr::EqZero(a) => Ok(evaluate_value(app, a)? == 0.0f64),
        BoolExpr::NeqZero(a) => Ok(evaluate_value(app, a)? != 0.0f64),
        BoolExpr::Equal(a, b) => Ok(evaluate_value(app, a)? == evaluate_value(app, b)?),
        BoolExpr::EqualPlusOrMinus(a, b, c) => {
            Ok((evaluate_value(app, a)? - evaluate_value(app, b)?).abs() < evaluate_value(app, c)?)
        }
        BoolExpr::MoreThan(a, b) => Ok(evaluate_value(app, a)? > evaluate_value(app, b)?),
        BoolExpr::LessThan(a, b) => Ok(evaluate_value(app, a)? < evaluate_value(app, b)?),
        BoolExpr::Between(a, b, c) => Ok(evaluate_value(app, a)? <= evaluate_value(app, b)?
            && evaluate_value(app, b)? <= evaluate_value(app, c)?),
        BoolExpr::Const(a) => Ok(*a),
        BoolExpr::EqBool(a, b) => Ok(evaluate(app, &*a)? == evaluate(app, &*b)?),
        BoolExpr::And(a, b) => Ok(evaluate(app, &*a)? && evaluate(app, &*b)?),
        BoolExpr::Or(a, b) => Ok(evaluate(app, &*a)? || evaluate(app, &*b)?),
        BoolExpr::Not(b) => Ok(!(evaluate(app, &*b)?)),
        BoolExpr::ReadBooleanInput(input_id) => app.read_input_bool(input_id),
    }
}
