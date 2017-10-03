use regex::Regex;

use bot::{Bot, Node, RoomEvent};

pub struct UpvoteTracker {
}

impl UpvoteTracker {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Node<'a> for UpvoteTracker {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let event = event.raw_event;
        if event.type_ == "m.room.message" &&
            event.content["msgtype"] == "m.text" {
            let re = Regex::new(r"([^ ]+|\(.+?\))\+\+").unwrap();
            for cap in re.captures_iter(event.content["body"].as_str().unwrap()) {
                println!("found karma for: {}", &cap[1].trim_matches(|c| c == '(' || c == ')'));
            }
        }
    }
}
