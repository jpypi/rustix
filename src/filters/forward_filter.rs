use std::collections::HashMap;

use crate::bot::{Bot, Node, RoomEvent};

pub struct ForwardFilter<'a> {
    children: Vec<&'a str>,
    channels: HashMap<String, u64>,
    threshold: u64,
}

impl<'a> ForwardFilter<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            channels: HashMap::new(),
            threshold: 2,
        }
    }
}

impl<'a> Node<'a> for ForwardFilter<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let last_channel_time = *self.channels.get(event.room_id).unwrap_or(&0);
        if let Some(event_time) = event.raw_event.origin_server_ts {
            if event_time > last_channel_time.saturating_sub(self.threshold) {
                self.channels.insert(event.room_id.to_string(), event_time);
                self.propagate_event(bot, &event)
            } else {
            }
        } else {
            self.propagate_event(bot, &event)
        }
    }
}
