use std::any::Any;

use crate::bot::{Bot, Node, RoomEvent};

pub struct Help {
    reply_id: Option<String>,
}

impl Help {
    pub fn new() -> Self {
        Self {
            reply_id: None
        }
    }
}

impl<'a> Node<'a> for Help {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" {
            let body = revent.content["body"].as_str().unwrap();
            if body.starts_with("help") {
                // Save the last event so it can be used to reply with the help text
                self.reply_id = Some(event.room_id.to_string());
                bot.delay_service_query(self, |s| Box::new(s.description()));
            }
        }
    }

    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn Any>)>) {
        let mut help_strings: Vec<String> = Vec::new();

        for (node, value) in result {
            let mut opt_v = value.downcast::<Option<String>>().unwrap();
            if let Some(v) = opt_v.take() {
                help_strings.push(v);
            }
        }

        if let Some(e) = &self.reply_id {
            let response = help_strings.join("\n");
            bot.say(e, &response);
            self.reply_id = None;
        }
    }

    fn description(&self) -> Option<String> {
        Some("help - Get a list of commands or view help for a specific command.".to_string())
    }
}