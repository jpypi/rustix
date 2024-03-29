use crate::bot::{Bot, Node, RoomEvent};


pub struct Logger<'a> {
    children: Vec<&'a str>
}

impl<'a> Logger<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for Logger<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if event.is_normal() {
            let body = &revent.content["body"].as_str().unwrap();
            let sender = &revent.sender;

            println!("<{}> | {}", sender, body);
        }

        self.propagate_event(bot, &event);
    }
}
