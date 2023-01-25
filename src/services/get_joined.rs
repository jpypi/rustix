use crate::bot::{Bot, Node, RoomEvent};


pub struct GetJoined;
impl GetJoined {
    pub fn new() -> Self {
        GetJoined
    }
}

impl<'a> Node<'a> for GetJoined {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if event.is_normal() {
            let body = &event.raw_event.content["body"].as_str().unwrap();
            if body.starts_with("joined") {
                match bot.get_joined() {
                    Ok(rooms) => {
                        let resp = format!("Currently in rooms: {}",
                                        rooms.joined_rooms.join(", "));
                        bot.reply(&event, &resp).ok();
                    },
                    Err(e) => {
                        let resp = format!("{:?}", e);
                        bot.reply(&event, &resp).ok();
                    },
                };
            }
        }
    }
}
