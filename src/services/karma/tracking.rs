use std::collections::HashMap;

use regex::Regex;
use toml::Value;

use crate::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

#[derive(Deserialize)]
struct Config {
    max_per_message: i32,
}

pub struct KarmaTracker {
    vote_db: Backend,
    re: Regex,
    bot_prefix: String,
    max_per_message: i32,
}

impl KarmaTracker {
    pub fn new(bot_prefix: String, config: Option<&Value>) -> Self {
        let mut max_per_message = 10;
        if let Some(value) = config {
            let cfg: Config = value.clone().try_into().expect("Bad karma config");
            max_per_message = cfg.max_per_message;
        }

        Self {
            vote_db: Backend::new(),
            re: Regex::new(r"([^\- ]+|\(.+?\))(\+\+|--)").unwrap(),
            bot_prefix,
            max_per_message,
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
        if body.starts_with(&self.bot_prefix) {
            return;
        }

        let mut votes: HashMap<String, VoteCount> = HashMap::new();

        for cap in self.re.captures_iter(body) {
            let ent = cap[1].trim_matches(|c| c == '(' || c == ')').to_string();
            let e = votes.entry(ent).or_insert(VoteCount{up: 0, down: 0});

            if &cap[2] == "++" {
                e.up += 1;
            }

            if &cap[2] == "--" {
                e.down += 1;
            }

            // Limit max karma per line to help prevent spam
            if (e.up + e.down) >= self.max_per_message {
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
