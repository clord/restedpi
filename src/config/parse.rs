use crate::config::types::BoolExpr;
use crate::error::Error;
use crate::error::Result;
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use tracing::{Level, instrument, span, trace, warn};

lrlex_mod!("config/config.l");
lrpar_mod!("config/config.y");

#[instrument(skip(as_str))]
pub fn bool_expr(as_str: &str) -> Result<BoolExpr> {
    let span = span!(Level::TRACE, "bool expression parse");
    let _e = span.enter();
    trace!("start");
    let lexerdef = config_l::lexerdef();
    let lexer = lexerdef.lexer(as_str);
    let (res, errs) = config_y::parse(&lexer);
    if !errs.is_empty() {
        for e in errs {
            warn!("{}", e.pp(&lexer, &config_y::token_epp));
        }
        return Err(Error::ParseError);
    }
    trace!("done");
    match res {
        Some(Ok(e)) => Ok(e),
        _ => Err(Error::ParseError),
    }
}

#[cfg(test)]
mod tests {
    use super::bool_expr;
    use crate::config::types::{BoolExpr, DateTimeValue, Value};

    #[test]
    fn parses_between() {
        let expr = bool_expr("hour_of_day(now) between 7 and 10").expect("should parse");
        match expr {
            BoolExpr::Between(_, low, tested, high) => {
                assert_eq!(low, Value::Const(7.0));
                assert_eq!(tested, Value::HourOfDay(DateTimeValue::Now));
                assert_eq!(high, Value::Const(10.0));
            }
            other => panic!("expected Between, got {:?}", other),
        }
    }

    #[test]
    fn parses_lerp_comparison() {
        let expr = bool_expr("lerp(1, 0.5, 2) > 1").expect("should parse");
        match expr {
            BoolExpr::MoreThan(_, lhs, rhs) => {
                assert_eq!(
                    lhs,
                    Value::Lerp(
                        Box::new(Value::Const(1.0)),
                        Box::new(Value::Const(0.5)),
                        Box::new(Value::Const(2.0)),
                    )
                );
                assert_eq!(rhs, Value::Const(1.0));
            }
            other => panic!("expected MoreThan, got {:?}", other),
        }
    }

    #[test]
    fn parses_xor_keyword() {
        let expr = bool_expr("a xor b").expect("should parse");
        match expr {
            BoolExpr::Xor(_, lhs, rhs) => match (*lhs, *rhs) {
                (BoolExpr::ReadBooleanInput(_, l), BoolExpr::ReadBooleanInput(_, r)) => {
                    assert_eq!(l, "a");
                    assert_eq!(r, "b");
                }
                other => panic!("expected ReadBooleanInput operands, got {:?}", other),
            },
            other => panic!("expected Xor, got {:?}", other),
        }
    }

    #[test]
    fn parses_xor_symbol() {
        let expr = bool_expr("true ^ false").expect("should parse");
        match expr {
            BoolExpr::Xor(_, lhs, rhs) => match (*lhs, *rhs) {
                (BoolExpr::Const(_, l), BoolExpr::Const(_, r)) => {
                    assert!(l);
                    assert!(!r);
                }
                other => panic!("expected Const operands, got {:?}", other),
            },
            other => panic!("expected Xor, got {:?}", other),
        }
    }

    #[test]
    fn xor_shares_or_tier_left_associative() {
        // Same precedence tier as `or`, left-associative:
        // a xor b or c => Or(Xor(a, b), c)
        let expr = bool_expr("a xor b or c").expect("should parse");
        match expr {
            BoolExpr::Or(_, lhs, rhs) => {
                match *lhs {
                    BoolExpr::Xor(_, _, _) => {}
                    other => panic!("expected Xor on the left, got {:?}", other),
                }
                match *rhs {
                    BoolExpr::ReadBooleanInput(_, r) => assert_eq!(r, "c"),
                    other => panic!("expected ReadBooleanInput, got {:?}", other),
                }
            }
            other => panic!("expected Or, got {:?}", other),
        }
    }

    #[test]
    fn parses_trunc() {
        let expr = bool_expr("trunc(3.7) == 3").expect("should parse");
        match expr {
            BoolExpr::Equal(_, lhs, rhs) => {
                assert_eq!(lhs, Value::Trunc(Box::new(Value::Const(3.7))));
                assert_eq!(rhs, Value::Const(3.0));
            }
            other => panic!("expected Equal, got {:?}", other),
        }
    }

    #[test]
    fn parses_inverse() {
        let expr = bool_expr("inverse(4) == 0.25").expect("should parse");
        match expr {
            BoolExpr::Equal(_, lhs, rhs) => {
                assert_eq!(lhs, Value::Inverse(Box::new(Value::Const(4.0))));
                assert_eq!(rhs, Value::Const(0.25));
            }
            other => panic!("expected Equal, got {:?}", other),
        }
    }

    #[test]
    fn parses_linear() {
        let expr = bool_expr("linear(2, 3, 1) == 7").expect("should parse");
        match expr {
            BoolExpr::Equal(_, lhs, rhs) => {
                assert_eq!(
                    lhs,
                    Value::Linear(
                        Box::new(Value::Const(2.0)),
                        Box::new(Value::Const(3.0)),
                        Box::new(Value::Const(1.0)),
                    )
                );
                assert_eq!(rhs, Value::Const(7.0));
            }
            other => panic!("expected Equal, got {:?}", other),
        }
    }
}
