use std::collections::HashMap;

use crate::{
    bot::{Bot, Node, RoomEvent},
    utils::codeblock_format
};


pub struct Structure {
    reply_room: Option<String>,
}

impl Structure {
    pub fn new() -> Self {
        Self {
            reply_room: None
        }
    }

    pub fn structure(&self, bot: &Bot, children: &HashMap<&str, Vec<String>>) -> String {
        let mut out_lines = Vec::new();

        let mut at_depth_remaining = HashMap::new();

        let mut services = Vec::new();

        // Initialize the services stack and the N elements at depth D hashmap
        for rs in bot.get_root_services().iter() {
            services.push((0, rs.clone()));

            let old_value = at_depth_remaining.get(&0).unwrap_or(&0);
            at_depth_remaining.insert(0, old_value + 1);
        }

        // let mut first = true;
        while !services.is_empty() {
            let (depth, service) = services.pop().unwrap();

            /*
            // Optional. Add blank tree elongations between each child and the first child and the parent
            let mut lines = String::new();
            for i in 1..depth+1 {
                lines += match depths_remaining.get(&i) {
                    Some(remaining) if *remaining > 0 => "  |",
                    _ => "   ",
                };
            }
            if first {
                first = false;
            } else {
                out_lines.push(format!(" {}", &lines));
            }
            */

            // Build up outer level continuation lines which appear left
            // of the child on the same row.
            let mut lines = String::new();
            for i in 0..depth {
                lines += match at_depth_remaining.get(&i) {
                    Some(remaining) if *remaining > 0 => "|  ",
                    _ => "   ",
                };
            }

            out_lines.push(format!("{}+- {}", lines, service));

            *at_depth_remaining.get_mut(&depth).unwrap() -= 1;

            // Potentially create a new depth level to expand with
            // the children of the current node.
            if let Some(children) = children.get(service) {
                for c in children {
                    let new_depth = depth + 1;
                    services.push((new_depth, c));
                    let old_value = at_depth_remaining.get(&new_depth).unwrap_or(&0);
                    at_depth_remaining.insert(new_depth, old_value + 1);
                }
            }
        }

        out_lines.join("\n")
    }
}

fn query(n: &mut dyn Node) -> Box<dyn std::any::Any> {
    let mut kids = Box::new(Vec::new());
    if let Some(children) = n.children() {
        for c in children.iter() {
            kids.push(c.to_string());
        }
    }

    return kids;
}

impl<'a> Node<'a> for Structure {
    fn handle<'b>(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();

        if body.starts_with("structure") {
            self.reply_room = Some(event.room_id.to_string());
            bot.delay_service_query("structure", None, query);
        }
    }

    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn std::any::Any>)>) {
        let mut children = HashMap::new();

        for (node, value) in result {
            let node_children = value.downcast::<Vec<String>>().unwrap();
            children.insert(node, *node_children);
        }

        if let Some(ref room_id) = self.reply_room {
            let raw_msg = self.structure(bot, &children);
            let msg = codeblock_format(&raw_msg);
            bot.say_fmt(room_id, &msg, &raw_msg).ok();

            self.reply_room = None;
        }
    }

    fn description(&self) -> Option<String> {
        Some("structure - outputs ascii visualization of all the configured bot services".to_string())
    }
}
