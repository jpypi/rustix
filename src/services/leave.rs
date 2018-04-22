use ::bot::{Bot, Node, RoomEvent};

pub struct Leave {
}

impl Leave {
    pub fn new() -> Self {
        Leave {
        }
    }
}

impl<'a> Node<'a> for Leave {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = &revent.content["body"].as_str().unwrap();
        if body.starts_with("leave ") {
            let room_name = &body[6..];
            println!("leaving: {}", room_name);
            if let Err(e) = bot.leave_public(room_name) {
                let resp = format!("Could not leave room: {}",
                                   &body[5..]);
                bot.reply(&event, &resp);
            }
        }
    }
}
