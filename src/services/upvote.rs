use regex::Regex;

use matrix_types::Event;
use bot::{Bot, Node};

pub struct UpvoteTracker<'a> {
    children: Vec<&'a str>,
}

impl<'a> UpvoteTracker<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for UpvoteTracker<'a> {
    fn parent(&self) -> Option<&'static str> {
        Some("self_filter")
    }

    fn children(&self) -> &Vec<&'a str> {
        &self.children
    }

    fn register_child(&mut self, name: &'a str) {
    }

    fn handle(&mut self, bot: &Bot, event: Event) {
        if event.type_ == "m.room.message" &&
            event.content["msgtype"] == "m.text" {
            let re = Regex::new(r"([^ ]+|\(.+?\))\+\+").unwrap();
            for cap in re.captures_iter(event.content["body"].as_str().unwrap()) {
                println!("found karma for: {}", &cap[1].trim_matches(|c| c == '(' || c == ')'));
            }
        }
    }
}
