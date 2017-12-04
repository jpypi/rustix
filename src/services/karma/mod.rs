mod backend;
mod models;
pub mod show_karma;

use std::collections::HashMap;

use regex::Regex;

use ::bot::{Bot, Node, RoomEvent};
use self::backend::Backend;

pub struct KarmaTracker {
    vote_db: Backend,
}

impl KarmaTracker {
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

impl<'a> Node<'a> for KarmaTracker {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let event = event.raw_event;
        if event.type_ == "m.room.message" &&
            event.content["msgtype"] == "m.text" {
            let body = event.content["body"].as_str().unwrap();

            // Don't karma based off anything that is a command to the bot
            if body.starts_with("!") {
                return;
            }

            let mut votes: HashMap<String, VoteCount> = HashMap::new();

            let re = Regex::new(r"([^\- ]+|\(.+?\))(\+\+|--)").unwrap();

            for cap in re.captures_iter(body) {
                let ent = cap[1].trim_matches(|c| c == '(' || c == ')').to_string();
                let e = votes.entry(ent).or_insert(VoteCount{up: 0, down: 0});

                if &cap[2] == "++" {
                    (*e).up += 1;
                }

                if &cap[2] == "--" {
                    (*e).down += 1;
                }
            }

            for (k, v) in votes.iter() {
                println!("{} up: {} down: {}", &k, v.up, v.down);
                self.vote_db.vote(&event.sender, k, v.up, v.down);
            }
        }
    }
}
