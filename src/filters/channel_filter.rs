use std::{collections::HashSet, iter::FromIterator};
use itertools::Itertools;

use crate::{state, utils::TrimMatch, bot::{Bot, Node, RoomEvent}};


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

        if !(self.allow ^ contains) {
            self.propagate_event(bot, &event);
        }
    }

    fn configure(&mut self, bot: &Bot, command: &str, event: RoomEvent) {
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
        } else if command.starts_with("status") {
            let chan_list = self.channels.iter().join(", ");
            let allow = match self.allow { true => "allow", false => "deny" };
            bot.reply(&event, &format!("{allow}: {chan_list}")).ok();
        } else if let Some(arg) = command.trim_match(&["allow", "deny"]) {
            match arg {
                "allow" => self.allow = true,
                "deny" => self.allow = false,
                _ => panic!("Invalid argument passed to channel filter configure allow/deny."),
            };
        }
    }

    fn configure_description(&self) -> Option<String> {
        Some("add <\"here\" | channel id> - add channel to filter list\n\
              rm  <\"here\" | channel id> - remove channel from filter list\n\
              allow  - change filter mode to only allow in configured channels\n\
              deny   - change filter mode to deny configured channels\n\
              status - view the current configuration state of the filter".to_string())
    }

    fn on_load(&mut self, service_name: &str) -> Result<(), String>{
        if let Some(state) = state::load_state(service_name) {
            let mut real_channels = state.as_str();
            if let Some((allow, channels)) = state.split_once("|") {
                match allow.parse() {
                    Ok(v) => self.allow = v,
                    Err(_) => return Err("Channel filter allow state value should parse to bool".to_string()),
                };
                real_channels = channels;
            }

            for c in real_channels.split(",") {
                self.channels.insert(c.to_string());
            }
        }

        Ok(())
    }

    fn on_exit(&self, service_name: &str) {
        state::save_state(service_name, &format!("{}|{}", self.allow, self.channels.iter().join(",")));
    }
}
