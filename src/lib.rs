extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate maplit;

mod matrix_types;
mod errors;
pub mod client;
pub mod bot;

pub mod services;
