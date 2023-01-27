use crate::bot::{Bot, Node, RoomEvent};
use url::Url;

pub struct SelfFilter<'a> {
    children: Vec<&'a str>,
    sender: String
}

impl<'a> SelfFilter<'a> {
    pub fn new(username: &str, server: &str) -> Self {
        let url = Url::parse(server).unwrap();
        let sender = format!("@{}:{}", username, url.host_str().unwrap());
        println!("Ignoring messages sent by self ({})", sender);
        Self {
            children: Vec::new(),
            sender,
        }
    }
}

impl<'a> Node<'a> for SelfFilter<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if revent.sender != self.sender {
            self.propagate_event(bot, &event);
        }
    }
}
