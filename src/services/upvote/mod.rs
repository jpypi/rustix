mod backend;
mod schema;
mod models;

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

/*
 * user (+4/-1), user2 (+3/-2)
 */

impl<'a> Node<'a> for UpvoteTracker {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let event = event.raw_event;
        if event.type_ == "m.room.message" &&
            event.content["msgtype"] == "m.text" {
            let re = Regex::new(r"([^ ]+|\(.+?\))\+\+").unwrap();
            for cap in re.captures_iter(event.content["body"].as_str().unwrap()) {
                let ent = &cap[1].trim_matches(|c| c == '(' || c == ')');
                println!("found karma for: {}", ent);

                self.vote_db.vote(&event.sender, ent, 1, 0);
            }
        }
    }
}
