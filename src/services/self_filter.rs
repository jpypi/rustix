use matrix_types::Event;
use bot::{Bot, Node};

pub struct SelfFilter<'a> {
    children: Vec<&'a str>,
}

impl<'a> SelfFilter<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for SelfFilter<'a> {
    fn parent(&self) -> Option<&'static str> {
        None
    }

    fn children(&self) -> &Vec<&'a str> {
        &self.children
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: Event) {
        if event.type_ == "m.room.message" &&
           event.content["msgtype"] == "m.text" &&
           event.sender != "@rustix:cclub.cs.wmich.edu" {

            self.propagate_event(bot, event);
        }
    }
}
