use crate::app::state::State;
use crate::config::value::evaluate as evaluate_value;
use crate::error::Result;
use crate::config::BoolExpr;

/// A very basic parser that evaluates an expression for truth. Can refer to values.
pub fn evaluate(app: &State, expr: &BoolExpr) -> Result<bool> {
    match expr {
        BoolExpr::Equal(_s, a, b) => Ok(evaluate_value(app, a)? == evaluate_value(app, b)?),
        BoolExpr::EqualPlusOrMinus(_s, a, b, c) => {
            Ok((evaluate_value(app, a)? - evaluate_value(app, b)?).abs() < evaluate_value(app, c)?)
        }
        BoolExpr::MoreThan(_s, a, b) => Ok(evaluate_value(app, a)? > evaluate_value(app, b)?),
        BoolExpr::LessThanOrEq(_s, a, b) => Ok(evaluate_value(app, a)? <= evaluate_value(app, b)?),
        BoolExpr::MoreThanOrEq(_s, a, b) => Ok(evaluate_value(app, a)? >= evaluate_value(app, b)?),
        BoolExpr::LessThan(_s, a, b) => Ok(evaluate_value(app, a)? < evaluate_value(app, b)?),
        BoolExpr::Between(_s, a, b, c) => Ok(evaluate_value(app, a)? <= evaluate_value(app, b)?
            && evaluate_value(app, b)? <= evaluate_value(app, c)?),
        BoolExpr::Const(_s, a) => Ok(*a),
        BoolExpr::EqBool(_s, a, b) => Ok(evaluate(app, &*a)? == evaluate(app, &*b)?),
        BoolExpr::And(_s, a, b) => Ok(evaluate(app, &*a)? && evaluate(app, &*b)?),
        BoolExpr::Or(_s, a, b) => Ok(evaluate(app, &*a)? || evaluate(app, &*b)?),
        BoolExpr::Xor(_s, a, b) => Ok(evaluate(app, &*a)? ^ evaluate(app, &*b)?),
        BoolExpr::Not(_s, b) => Ok(!(evaluate(app, &*b)?)),
        BoolExpr::ReadBooleanInput(_s, input_id) => app.read_input_bool(input_id),
    }
}
