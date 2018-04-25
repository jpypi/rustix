use ::bot::{Bot, Node, RoomEvent};


pub struct Join;

impl Join {
    pub fn new() -> Self {
        Join
    }
}

impl<'a> Node<'a> for Join {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if body.starts_with("join ") {
            let room_name = &body[5..];
            if let Err(e) = bot.join_public(room_name) {
                let resp = format!("Could not join: {}", room_name);
                bot.reply(&event, &resp);
            }
        }
    }
}


pub struct Leave;

impl Leave {
    pub fn new() -> Self {
        Leave
    }
}

impl<'a> Node<'a> for Leave {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if body.starts_with("leave") {
            let room_name = &body[5..].trim_left();
            if room_name.len() == 0 {
                bot.leave(&event.room_id);
            }

            if let Err(e) = bot.leave_public(room_name) {
                let resp = format!("Could not leave room: {}", room_name);
                bot.reply(&event, &resp);
            }
        }
    }
}
