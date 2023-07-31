use std::{collections::HashSet, iter::FromIterator};
use itertools::Itertools;

use crate::{
    bot::{Bot, Node, RoomEvent},
    utils
};

pub struct ChannelFilter<'a> {
    children: Vec<&'a str>,
    channels: HashSet<String>,
    allow: bool,
}

impl<'a> ChannelFilter<'a> {
    pub fn new(channels: Vec<String>, allow: bool) -> Self {
        Self {
            children: Vec::new(),
            channels: HashSet::from_iter(channels.iter().cloned()),
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

    fn configure(&mut self, command: &str, event: RoomEvent) {
        if let Some(add_args) = command.strip_prefix("add ") {
            if add_args == "here" {
                self.channels.insert(event.room_id.to_string());
            } else {
                self.channels.insert(add_args.to_string());
            }

        } else if let Some(add_args) = command.strip_prefix("rm ") {
            if add_args == "here" {
                self.channels.remove(event.room_id);
            } else {
                self.channels.remove(add_args);
            }
        }
    }

    fn on_load(&mut self, service_name: &str) {
        let saved_state = utils::load_state(service_name);
        if let Some(state) = saved_state {
            for c in state.split(",") {
                self.channels.insert(c.to_string());
            }
        }
    }

    fn on_exit(&self, service_name: &str) {
        utils::save_state(service_name, &self.channels.iter().join(","));
    }
}
