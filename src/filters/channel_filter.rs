use crate::bot::{Bot, Node, RoomEvent};

pub struct ChannelFilter<'a> {
    children: Vec<&'a str>,
    channels: Vec<String>,
    allow: bool,
}

impl<'a> ChannelFilter<'a> {
    pub fn new(channels: Vec<String>, allow: bool) -> Self {
        Self {
            children: Vec::new(),
            channels,
            allow,
        }
    }
}

impl<'a> Node<'a> for ChannelFilter<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let contains = self.channels.contains(&event.room_id.to_string());

        if (self.allow && contains) || (!self.allow && !contains) {
            self.propagate_event(bot, &event);
        }
    }
}
