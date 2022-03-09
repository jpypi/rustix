mod backend;
mod models;
pub mod show_karma;

use std::collections::HashMap;

use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};
use self::backend::Backend;

pub struct KarmaTracker {
    vote_db: Backend,
    re: Regex,
}

impl KarmaTracker {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
            re: Regex::new(r"([^\- ]+|\(.+?\))(\+\+|--)").unwrap(),
        }
    }
}

struct VoteCount {
    up: i32,
    down: i32,
}

impl<'a> Node<'a> for KarmaTracker {
    fn handle(&mut self, _bot: &Bot, event: RoomEvent) {
        let event = event.raw_event;
        let body = event.content["body"].as_str().unwrap();

        // Don't karma based off anything that is a command to the bot
        if body.starts_with("!") {
            return;
        }

        let mut votes: HashMap<String, VoteCount> = HashMap::new();

        for cap in self.re.captures_iter(body) {
            let ent = cap[1].trim_matches(|c| c == '(' || c == ')').to_string();
            let e = votes.entry(ent).or_insert(VoteCount{up: 0, down: 0});

            if &cap[2] == "++" {
                (*e).up += 1;
            }

            if &cap[2] == "--" {
                (*e).down += 1;
            }

            // Limit max karma per line to 10 (to help prevent spam)
            if ((*e).up + (*e).down) >= 10 {
                break;
            }
        }

        for (k, v) in votes.iter() {
            if let Err(e) = self.vote_db.vote(&event.sender, k, v.up, v.down) {
                println!("Error while trying to vote: {:?}", e);
            }
        }
    }
}
