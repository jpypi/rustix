extern crate rustix;

use std::io::Read;
use std::fs::File;

use rustix::bot;
use rustix::client::MatrixClient;
use rustix::services::echo::*;
use rustix::services::self_filter::*;
use rustix::services::upvote::*;


fn main() {
    let mut m = MatrixClient::new("https://cclub.cs.wmich.edu/");

    let mut password = String::new();
    let mut f = File::open("auth").expect("auth file not found");
    f.read_to_string(&mut password).expect("something went wrong reading file");

    m.login("rustix", password.trim()).expect("login failed!");
    m.set_display_name("rustix");

    let mut b = bot::Bot::new(&mut m);

    let sf = b.register_service("self_filter", None, Box::new(SelfFilter::new()));
    b.register_service("echo", sf, Box::new(Echo::new()));
    b.register_service("upvote_tracker", sf, Box::new(UpvoteTracker::new()));

    b.run();
}
