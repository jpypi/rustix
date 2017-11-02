use serde_json::value::Value;

use bot::{Bot, Node, RoomEvent};

pub struct Prefix<'a> {
    children: Vec<&'a str>,
}

impl<'a> Prefix<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for Prefix<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, mut event: RoomEvent) {
        if event.raw_event.type_ == "m.room.message" &&
           event.raw_event.content["msgtype"] == "m.text" {
            if event.raw_event.content["body"].as_str().unwrap().starts_with("!") {
                event.raw_event.content["body"] =
                    Value::String(event.raw_event.content["body"].as_str().unwrap()[1..].to_string().clone());
                self.propagate_event(bot, &event);
            }
        }
    }
}
