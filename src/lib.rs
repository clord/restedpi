// Allow clippy warning in generated parser code
#![allow(clippy::unused_unit)]

extern crate tracing;
#[macro_use]
extern crate diesel;
extern crate bit_array;
extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate typenum;
extern crate warp;

#[macro_use]
extern crate lazy_static;

pub mod app;
pub mod auth;
pub mod config;
pub mod error;
pub mod graphql;
pub mod rpi;
pub mod schema;
pub mod session;
pub mod webapp;
