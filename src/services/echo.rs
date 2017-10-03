use ::matrix_types::Event;
use ::bot::{Bot, Node};

pub struct Echo<'a> {
    children: Vec<&'a str>
}

impl<'a> Echo<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new()
        }
    }
}

impl<'a> Node<'a> for Echo<'a> {
    fn parent(&self) -> Option<&str> {
        Some("self_filter")
    }

    fn children(&self) -> &Vec<&'a str> {
        &self.children
    }

    fn register_child(&mut self, name: &'a str) {
    }

    fn handle(&mut self, bot: &Bot, event: Event) {
        let r = "!tEQUhDXnBDAeqCAgJk:cclub.cs.wmich.edu";

        if event.type_ == "m.room.message" {
            if event.content["msgtype"] == "m.text" {
                bot.say(r, "HEY MR. MESEEKS");
                let sender = &event.sender;
                let body = &event.content["body"].as_str().unwrap();

                println!("<{}> | {}", sender, body);
            }
        }

        self.propagate_event(bot, event);
    }
}
