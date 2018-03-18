use ::bot::{Bot, Node, RoomEvent};

pub struct Join {
}

impl Join {
    pub fn new() -> Self {
        Join {
        }
    }
}

impl<'a> Node<'a> for Join {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = &revent.content["body"].as_str().unwrap();
        //let sender = &revent.sender;
        if body.starts_with("join ") {
            let room_id = &body[5..];
            if let Err(e) = bot.join_public(room_id) {
                let resp = format!("Could not join: {}", &body[5..]);
                bot.reply(&event, &resp);
            }
        }
    }
}
