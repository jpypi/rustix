use crate::bot::{Bot, Node, RoomEvent};

pub struct Configure {
}

impl Configure {
    pub fn new() -> Self {
        Self {  }
    }
}

impl<'a> Node<'a> for Configure {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();

        if let Some(args) = body.strip_prefix("node config") {
            if let Some((target, command)) = args.trim().split_once(' ') {
                let cmd = command.to_string();
                let room_id = event.room_id.to_string();
                let rev = event.raw_event.clone();
                bot.delay_service_query("nodectl",
                                        Some(target.to_string()),
                                        move |n| {
                                            let rid = &room_id;
                                            let rev_ptr = &rev;
                                            let ev = RoomEvent { room_id: rid, raw_event: rev_ptr.clone() };
                                            n.configure(&cmd, ev);
                                            Box::new(0)
                                        });
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("node config <node name> <command> - send command to a specific node".to_string())
    }
}