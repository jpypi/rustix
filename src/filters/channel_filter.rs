use std::{collections::HashSet, iter::FromIterator};
use itertools::Itertools;

use crate::utils::TrimMatch;

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
        if let Some(mut add_args) = command.strip_prefix("add ") {
            if add_args == "here" {
                add_args = event.room_id;
            }
            self.channels.insert(add_args.to_string());
        } else if let Some(mut rm_args) = command.strip_prefix("rm ") {
            if rm_args == "here" {
                rm_args = event.room_id
            }
            self.channels.remove(rm_args);
        } else if let Some(arg) = command.trim_match(&["allow", "deny"]) {
            match arg {
                "allow" => self.allow = true,
                "deny" => self.allow = false,
                _ => panic!("Invalid argument passed to channel filter configure allow/deny."),
            };
        }
    }

    fn on_load(&mut self, service_name: &str) {
        let saved_state = utils::load_state(service_name);
        if let Some(state) = saved_state {
            let mut real_channels = state.as_str();
            if let Some((allow, channels)) = state.split_once("|") {
                self.allow = allow.parse().expect("Invalid value for channel filter allow state");
                real_channels = channels;
            }

            for c in real_channels.split(",") {
                self.channels.insert(c.to_string());
            }
        }
    }

    fn on_exit(&self, service_name: &str) {
        utils::save_state(service_name, &format!("{}|{}", self.allow, self.channels.iter().join(",")));
    }
}
