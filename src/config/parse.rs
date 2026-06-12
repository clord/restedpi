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
    use super::*;

    #[test]
    fn valid_expressions_parse() {
        let cases = [
            "true",
            "false",
            "some_input",
            "a and b or not c",
            "(a or b) and not c",
            "not (a or b)",
            "true eq false",
            "hour_of_day(now) between 7 and 10",
            "read(foo, degC) > 21.5",
            "read(foo, kpa) <= 101.3",
            "read(foo, bool) == 1",
            "lerp(1, 0.5, 2) == 1.5",
            "plus/minus 0.5, read(foo, degC) == 21",
            "hour_of_sunrise(here, now) < hour_of_day(now)",
            "hour_of_sunset(49.2 degN 123.1 degW, now) > 18",
            "1 + 2 * 3 == 7",
            "(8 - 2) / 3 != 1",
            "day_of_year(now) >= 100",
            "month_of_year(2021-06-01T12:00:00) == 6",
        ];
        for case in cases {
            assert!(
                bool_expr(case).is_ok(),
                "expected {:?} to parse, got {:?}",
                case,
                bool_expr(case)
            );
        }
    }

    #[test]
    fn invalid_expressions_fail() {
        let cases = [
            "",
            "and",
            "true and",
            "or true",
            "21.5",
            "read(foo)",
            "read(foo, degc) > 21.5",
            "hour_of_day() > 3",
            "1 between 2",
            "(true",
            "== 5",
            "lerp(1, 2) == 1.5",
            "plus/minus 0.5 read(foo, degC) == 21",
        ];
        for case in cases {
            assert_eq!(
                bool_expr(case),
                Err(Error::ParseError),
                "expected {:?} to fail to parse",
                case
            );
        }
    }
}
