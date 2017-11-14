extern crate rustix;

use std::io::Read;
use std::fs::File;

use rustix::bot;
use rustix::client::MatrixClient;
use rustix::services::echo::*;
use rustix::services::self_filter::*;
use rustix::services::karma::*;
use rustix::services::quote::*;
use rustix::services::timecube::Timecube;
use rustix::services::prefix::Prefix;
use rustix::services::choose::Choose;


fn main() {
    let mut m = MatrixClient::new("https://cclub.cs.wmich.edu/");

    let mut password = String::new();
    let mut f = File::open("auth").expect("auth file not found");
    f.read_to_string(&mut password).expect("something went wrong reading file");

    m.login("rustix", password.trim()).expect("login failed!");
    m.set_display_name("rustix").unwrap();

    let mut b = bot::Bot::new(&mut m);

    let sf = b.register_service("self_filter", None, Box::new(SelfFilter::new()));
    let pf = b.register_service("prefix", sf, Box::new(Prefix::new()));
    b.register_service("echo", pf, Box::new(Echo::new()));
    b.register_service("choose", pf, Box::new(Choose::new()));

    b.register_service("show_karma", pf, Box::new(show_karma::ShowKarma::new()));
    b.register_service("karma_tracker", sf, Box::new(KarmaTracker::new()));

    b.register_service("add_quote", pf, Box::new(add_quote::AddQuote::new()));

    b.register_service("timecube", pf, Box::new(Timecube::new()));

    b.run();
}
