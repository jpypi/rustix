use crate::bot::{Bot, Node, RoomEvent};

pub struct Admin<'a> {
    children: Vec<&'a str>,
    hard_admins: Vec<String>,
}

impl<'a> Admin<'a> {
    pub fn new(hard_admins: Vec<String>) -> Self {
        Self {
            children: Vec::new(),
            hard_admins,
        }
    }
}

impl<'a> Node<'a> for Admin<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if self.hard_admins.contains(&event.raw_event.sender) {
            self.propagate_event(bot, &event);
        }
    }
}
