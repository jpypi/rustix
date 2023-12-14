use std::any::Any;

use crate::{bot::{Bot, Node, RoomEvent}, utils::codeblock_format};

pub struct Configure {
    reply_id: Option<String>,
}

impl Configure {
    pub fn new() -> Self {
        Self {
            reply_id: None
        }
    }
}

impl<'a> Node<'a> for Configure {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();

        if let Some(args) = body.strip_prefix("node config") {
            if let Some((target, command)) = args.trim().split_once(' ') {
                let cmd = command.to_string();
                let room_id = event.room_id.to_string();
                let from = event.from.to_string();
                let rev = event.raw_event.clone();
                bot.delay_service_query("nodectl",
                                        Some(target.to_string()),
                                        move |b, n| {
                                            let ev = RoomEvent {
                                                room_id: &room_id,
                                                from: &from,
                                                raw_event: (&rev).clone()
                                            };
                                            n.configure(b, &cmd, ev);
                                            Box::new(0)
                                        });
            }
        }

        if let Some(command) = body.strip_prefix("node help") {
            let target = command.trim();
            bot.delay_service_query("nodectl",
                                    Some(target.to_string()),
                                    move |_, n| {
                                        Box::new(n.configure_description())
                                    });
            self.reply_id = Some(event.room_id.to_string());
        }
    }

    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn Any>)>) {
        if let Some(ref rid) = self.reply_id {
            if let Some((_, value)) = result.into_iter().next() {
                let mut opt_v = value.downcast::<Option<String>>().unwrap();
                if let Some(v) = opt_v.take() {
                    bot.client().send_msg_fmt(rid, &codeblock_format(&v), &v).ok();
                } else {
                    bot.client().send_msg(rid, "No config help found.").ok();
                }
            } else {
                bot.client().send_msg(rid, "Node not found.").ok();
            }
        }

        self.reply_id = None;
    }

    fn description(&self) -> Option<String> {
        Some("node config <node name> <command> - send command to a specific node\n\
              node help <node name> - get help for configuring node".to_string())
    }
}