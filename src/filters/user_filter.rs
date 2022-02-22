use crate::bot::{Bot, Node, RoomEvent};

pub struct UserFilter<'a> {
    children: Vec<&'a str>,
    ignore: Vec<String>,
}

impl<'a> UserFilter<'a> {
    pub fn new(ignore: Vec<String>) -> Self {
        Self {
            children: Vec::new(),
            ignore: ignore,
        }
    }
}

impl<'a> Node<'a> for UserFilter<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if !self.ignore.contains(&event.raw_event.sender) {
            self.propagate_event(bot, &event);
        }
    }
}
