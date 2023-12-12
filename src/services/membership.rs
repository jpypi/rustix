use crate::bot::{Bot, Node, RoomEvent};


pub struct Join;

impl Join {
    pub fn new() -> Self {
        Join
    }
}

impl<'a> Node<'a> for Join {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if let Some(room_name) = body.strip_prefix("join ") {
            if bot.join_public(room_name).is_err() {
                let resp = format!("Could not join: {}", room_name);
                bot.reply(&event, &resp).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("join <room name> - Command the bot to join a room.".to_string())
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
        if let Some(channel) = body.strip_prefix("leave") {
            let room_name = &channel.trim_start();
            if room_name.is_empty() {
                bot.client().leave(event.room_id).ok();
            }

            if bot.leave_public(room_name).is_err() {
                let resp = format!("Could not leave room: {}", room_name);
                bot.reply(&event, &resp).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("leave <optional room name> - Command the bot to leave a room.".to_string())
    }
}


pub struct EmptyCleanup;

impl EmptyCleanup {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> Node<'a> for EmptyCleanup {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if body.starts_with("emptycleanup") {
            if let Ok(joined) = bot.client().get_joined() {
                for room_id in joined.joined_rooms {
                    bot.client().get_members(&room_id)
                       .map(|m| {
                            if m.len() == 1 {
                                bot.client().leave(&room_id).ok();
                            }
                       }).ok();
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("emptycleanup - Leave all rooms where bot is the only member.".to_string())
    }
}


pub struct AcceptInvite;

impl AcceptInvite {
    pub fn new() -> Self {
        AcceptInvite
    }
}

impl<'a> Node<'a> for AcceptInvite {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.member" && event.from == "invite" {
            if let Some(value) = revent.content.get("membership") {
                if value.is_string() && value.as_str().unwrap() == "invite" {
                    println!("Joining room {} via invitation from {}", &event.room_id, revent.sender);
                    bot.client().join(event.room_id).ok();
                }
            }
        }
    }
}
