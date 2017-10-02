extern crate rustix;

use std::io::Read;
use std::fs::File;

use rustix::client::MatrixClient;
use rustix::services::*;



/*
fn get_room_events(data: MatrixSync, room_id: &str) {
    for (k,v) in data.rooms.join {
        println!("{} => {:?}", k, v);
    }
}
*/


fn main() {
    let mut m = MatrixClient::new("https://cclub.cs.wmich.edu/");

    let mut password = String::new();
    let mut f = File::open("auth").expect("auth file not found");
    f.read_to_string(&mut password).expect("something went wrong reading file");

    let r = m.login("rustix", password.trim());
    println!("{:?}", r);

    m.set_display_name("rustix");

    let mut b = rustix::bot::Bot::new(&mut m);

    b.register_service("echo", Box::new(Echo::new()));

    b.run();
}
