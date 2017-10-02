use matrix_types::Event;
use bot::{Bot, Node};

pub struct Echo {}

impl Echo {
    pub fn new() -> Self {
        Self{}
    }
}

impl Node for Echo {
    fn parent(&self) -> Option<&str> {
        None
    }

    fn children(&self) -> Vec<&str> {
        vec!()
    }

    fn register_child(&mut self, name: &str) {
    }

    fn handle(&self, bot: &Bot, event: &Event) {
        self.propagate_event(bot, event);
    }
}
