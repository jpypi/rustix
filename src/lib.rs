extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate regex;
extern crate dotenv;
extern crate rand;
extern crate chrono;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;

mod matrix_types;
mod errors;
pub mod client;
pub mod bot;

pub mod services;
