mod backend;
mod schema;
mod models;
pub mod show_karma;

use std::collections::HashMap;

use regex::Regex;

use ::bot::{Bot, Node, RoomEvent};
use self::backend::Backend;

pub struct UpvoteTracker {
    vote_db: Backend,
}

impl UpvoteTracker {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new()
        }
    }
}

struct VoteCount {
    up: i32,
    down: i32,
}

impl<'a> Node<'a> for UpvoteTracker {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let event = event.raw_event;
        if event.type_ == "m.room.message" &&
            event.content["msgtype"] == "m.text" {
            let up_re = Regex::new(r"([^ ]+|\(.+?\))\+\+").unwrap();
            let dn_re = Regex::new(r"([^ ]+|\(.+?\))--").unwrap();

            let body = event.content["body"].as_str().unwrap();
            //println!("found karma for: {}", ent);

            let mut votes: HashMap<String, VoteCount> = HashMap::new();

            for cap in up_re.captures_iter(body) {
                let ent = cap[1].trim_matches(|c| c == '(' || c == ')').to_string();
                let e = votes.entry(ent).or_insert(VoteCount{up: 0, down: 0});
                (*e).up += 1;
            }

            for cap in dn_re.captures_iter(body) {
                let ent = cap[1].trim_matches(|c| c == '(' || c == ')').to_string();
                let e = votes.entry(ent).or_insert(VoteCount{up: 0, down: 0});
                (*e).down += 1;
            }

            for (k, v) in votes.iter() {
                println!("{} up: {} down: {}", &k, v.up, v.down);
                self.vote_db.vote(&event.sender, k, v.up, v.down);
            }
        }
    }
}
