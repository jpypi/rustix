use std::any::Any;

use crate::bot::{Bot, Node, RoomEvent};

use crate::utils::codeblock_format;

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
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if let Some(mut service_name) = body.strip_prefix("help") {
            service_name = service_name.trim();
            let mut target = None;
            if !service_name.is_empty() {
                target = Some(service_name.to_string());
            }

            // Save the last event so it can be used to reply with the help text
            self.reply_id = Some(event.room_id.to_string());
            bot.delay_service_query("help", target, |s| Box::new(s.description()));
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

        help_strings.sort();

        if let Some(e) = &self.reply_id {
            // Exit early. Avoids sending useless response.
            if help_strings.is_empty() {
                bot.say(e, &format!("No help found.")).ok();
                return;
            }

            let response = help_strings.join("\n");
            let message = codeblock_format(&response);
            bot.say_fmt(e, &message, &response).ok();
            self.reply_id = None;
        }
    }

    fn description(&self) -> Option<String> {
        Some("help <?service name> - Get a list of commands or view help for a specific command.".to_string())
    }
}
