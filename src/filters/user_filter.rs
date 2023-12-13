use std::{collections::HashSet, iter::FromIterator};

use itertools::Itertools;

use crate::{bot::{Bot, Node, RoomEvent}, utils::TrimMatch, state};

pub struct UserFilter<'a> {
    children: Vec<&'a str>,
    users: HashSet<String>,
    allow: bool,
}

impl<'a> UserFilter<'a> {
    pub fn new(users: Vec<String>, allow: bool) -> Self {
        Self {
            children: Vec::new(),
            users: HashSet::from_iter(users.iter().cloned()),
            allow,
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
        let contains = self.users.contains(&event.raw_event.sender);

        if !(self.allow ^ contains) {
            self.propagate_event(bot, &event);
        }
    }

    fn configure(&mut self, bot: &Bot, command: &str, event: RoomEvent) {
        if let Some(add_args) = command.strip_prefix("add ") {
            if add_args == event.raw_event.sender {
                bot.reply(&event, "WARNING: Adding yourself to a user filter could limit your ability to update the filter. Use `add -f <your user id>` if this is really what you want to do.").ok();
            } else {
                let user_id = add_args.trim_start_matches("-f ");
                self.users.insert(user_id.to_string());
            }
        } else if let Some(rm_args) = command.strip_prefix("rm ") {
            if rm_args == event.raw_event.sender {
                bot.reply(&event, "WARNING: Removing yourself from a user filter could limit your ability to update the filter. Use `add -f <your user id>` if this is really what you want to do.").ok();
            } else {
                let user_id = rm_args.trim_start_matches("-f ");
                self.users.remove(user_id);
            }
        } else if command.starts_with("status") {
            let user_list = self.users.iter().join(", ");
            let allow = match self.allow { true => "allow", false => "deny" };
            bot.reply(&event, &format!("{allow}: {user_list}")).ok();
        } else if let Some(arg) = command.trim_match(&["allow", "deny"]) {
            match arg {
                "allow" => self.allow = true,
                "deny" => self.allow = false,
                _ => panic!("Invalid argument passed to channel filter configure allow/deny."),
            };
        }
    }

    fn configure_description(&self) -> Option<String> {
        Some("add <user id> - add user to filter list\n\
              rm  <user id> - remove user from filter list\n\
              allow  - change filter mode to only allow configured users\n\
              deny   - change filter mode to deny configured users\n\
              status - view the current configuration state of the filter".to_string())
    }

    fn on_load(&mut self, service_name: &str) -> Result<(), String>{
        let saved_state = state::load_state(service_name);
        if let Some(state) = saved_state {
            let mut real_channels = state.as_str();
            if let Some((allow, channels)) = state.split_once("|") {
                match allow.parse() {
                    Ok(v) => self.allow = v,
                    Err(_) => return Err("User filter allow state value should parse to bool".to_string()),
                };
                real_channels = channels;
            }

            for c in real_channels.split(",") {
                self.users.insert(c.to_string());
            }
        }

        Ok(())
    }

    fn on_exit(&self, service_name: &str) {
        state::save_state(service_name, &format!("{}|{}", self.allow, self.users.iter().join(",")));
    }
}
