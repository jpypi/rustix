extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate regex;
extern crate dotenv;
extern crate rand;
extern crate chrono;
extern crate http;
extern crate url;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate diesel;

mod matrix_types;
mod errors;
mod utils;
mod state;

pub mod config;
pub mod client;
pub mod bot;

pub mod services;
pub mod filters;
