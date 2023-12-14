use serde_json::value::Value;

use crate::bot::{Bot, Node, RoomEvent};

pub struct Prefix<'a> {
    children: Vec<&'a str>,
    prefix: String,
    prefix_n: usize,
}

impl<'a> Prefix<'a> {
    pub fn new(prefix: String) -> Self {
        let len = prefix.len();
        Self {
            children: Vec::new(),
            prefix,
            prefix_n: len,
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
           event.raw_event.content["msgtype"] == "m.text" &&
           event.body().unwrap().starts_with(&self.prefix)
        {
            event.raw_event.content["body"] =
                Value::String(event.body().unwrap()[self.prefix_n..].to_string());
            self.propagate_event(bot, &event);
        }
    }
}
