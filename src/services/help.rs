use std::any::Any;

use crate::bot::{Bot, Node, RoomEvent};

use super::utils::codeblock_format;

pub struct Help {
    reply_id: Option<String>,
}

impl Help {
    pub fn new() -> Self {
        Self { reply_id: None }
    }
}

impl<'a> Node<'a> for Help {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap();
            if body.starts_with("help") {
                // Save the last event so it can be used to reply with the help text
                self.reply_id = Some(event.room_id.to_string());
                bot.delay_service_query("help", |s| Box::new(s.description()));
            }
        }
    }

    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn Any>)>) {
        let mut help_strings: Vec<String> = Vec::new();

        for (_, value) in result {
            let mut opt_v = value.downcast::<Option<String>>().unwrap();
            if let Some(v) = opt_v.take() {
                help_strings.push(v);
            }
        }

        if let Some(e) = &self.reply_id {
            let response = help_strings.join("\n");
            let message = codeblock_format(&response);
            bot.say_fmt(e, &message, &response).ok();
            self.reply_id = None;
        }
    }

    fn description(&self) -> Option<String> {
        Some("help - Get a list of commands or view help for a specific command.".to_string())
    }
}
