use crate::config::BoolExpr;

pub fn evaluate(expr: BoolExpr) -> bool {
    match expr {
    Equal(a, b) => a == b,
    EqualPlusOrMinus(a, b, c) => false,
    MoreThan(a, b) => a > b,
    LessThan(a, b) => a < b,
    Between(a, b, c) => a <= b && b <= c,
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Not(Box<BoolExpr>),
    }
}

