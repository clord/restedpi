use crate::error::Result;
use crate::error::Error;
use lrlex::lrlex_mod;
use lrpar::lrpar_mod;

lrlex_mod!("config/config.l");
lrpar_mod!("config/config.y");

pub use config_y::{Value, Unit, DateTimeValue, LocationValue, BoolExpr};

pub fn bool_expr(as_str: &str) -> Result<BoolExpr> {
    let lexerdef = config_l::lexerdef();
    let lexer = lexerdef.lexer(as_str);
    let (res, errs) = config_y::parse(&lexer);
    if errs.len() > 0 {
        for e in errs {
            warn!("{}", e.pp(&lexer, &config_y::token_epp));
        }
        return Err(Error::ParseError);
    }
    match res {
        Some(Ok(e)) => Ok(e),
        _ => Err(Error::ParseError)
    }
}

