use crate::bot::{Bot, Node, RoomEvent};

pub struct MessageTypeFilter<'a> {
    children: Vec<&'a str>,
}

impl<'a> MessageTypeFilter<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for MessageTypeFilter<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" {
            self.propagate_event(bot, &event);
        }
    }
}

impl<'a> Default for MessageTypeFilter<'a> {
    fn default() -> Self {
        Self::new()
    }
}