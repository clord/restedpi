use crate::app::state::State;
use crate::config::types::BoolExpr;
use crate::config::value::evaluate as evaluate_value;
use crate::error::Result;
use async_recursion::async_recursion;

/// A very basic parser that evaluates an expression for truth. Can refer to values.
#[async_recursion]
pub async fn evaluate(app: &State, expr: &BoolExpr) -> Result<bool> {
    match expr {
        BoolExpr::Equal(_s, a, b) => {
            Ok(evaluate_value(app, a).await? == evaluate_value(app, b).await?)
        }
        BoolExpr::EqualPlusOrMinus(_s, a, b, c) => Ok((evaluate_value(app, a).await?
            - evaluate_value(app, b).await?)
            .abs()
            < evaluate_value(app, c).await?),
        BoolExpr::MoreThan(_s, a, b) => {
            Ok(evaluate_value(app, a).await? > evaluate_value(app, b).await?)
        }
        BoolExpr::LessThanOrEq(_s, a, b) => {
            Ok(evaluate_value(app, a).await? <= evaluate_value(app, b).await?)
        }
        BoolExpr::MoreThanOrEq(_s, a, b) => {
            Ok(evaluate_value(app, a).await? >= evaluate_value(app, b).await?)
        }
        BoolExpr::LessThan(_s, a, b) => {
            Ok(evaluate_value(app, a).await? < evaluate_value(app, b).await?)
        }
        BoolExpr::Between(_s, a, b, c) => Ok(evaluate_value(app, a).await?
            <= evaluate_value(app, b).await?
            && evaluate_value(app, b).await? <= evaluate_value(app, c).await?),
        BoolExpr::Const(_s, a) => Ok(*a),
        BoolExpr::EqBool(_s, a, b) => Ok(evaluate(app, a).await? == evaluate(app, b).await?),
        BoolExpr::And(_s, a, b) => Ok(evaluate(app, a).await? && evaluate(app, b).await?),
        BoolExpr::Or(_s, a, b) => Ok(evaluate(app, a).await? || evaluate(app, b).await?),
        BoolExpr::Xor(_s, a, b) => Ok(evaluate(app, a).await? ^ evaluate(app, b).await?),
        BoolExpr::Not(_s, b) => Ok(!(evaluate(app, b).await?)),
        BoolExpr::ReadBooleanInput(_s, input_id) => app.read_input_bool(input_id).await,
    }
}
