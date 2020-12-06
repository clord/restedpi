use std::io::{self, BufRead, Write};
use librpi::config::parse;
use log::error;
use std::env;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;


fn main() {
    if env::var_os("LOG").is_none() {
        // Set `RUST_LOG=restedpi=debug` to see debug logs,
        env::set_var("LOG", "restedpi=info");
        info!("defaulting to info level logging. RUST_LOG='restedpi=info'");
    }
    pretty_env_logger::init_custom_env("LOG");
    let stdin = io::stdin();
    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        match stdin.lock().lines().next() {
            Some(Ok(ref l)) => {
                if l.trim().is_empty() {
                    continue;
                }
                match parse::bool_expr(l) {
                    Ok(r) => println!("Result: {:?}", r),
                    Err(e) => error!("Unable to evaluate expression: {:?}", e)
                }
            }
            _ => break
        }
    }
}
