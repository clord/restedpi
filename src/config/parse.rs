use crate::error::Error;
use crate::error::Result;
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;
use tracing::{span,warn,Level, trace, instrument};

lrlex_mod!("config/config.l");
lrpar_mod!("config/config.y");

pub use config_y::{BoolExpr, DateTimeValue, LocationValue, Unit, Value};

#[instrument(skip(as_str))]
pub fn bool_expr(as_str: &str) -> Result<BoolExpr> {
    let span = span!(Level::TRACE, "bool expression parse");
    let _e = span.enter();
    trace!("start");
    let lexerdef = config_l::lexerdef();
    let lexer = lexerdef.lexer(as_str);
    let (res, errs) = config_y::parse(&lexer);
    if errs.len() > 0 {
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
