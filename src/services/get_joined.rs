use crate::{
    bot::{Bot, Node, RoomEvent},
    utils::codeblock_format
};

use itertools::Itertools;

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
                let joined = bot.client().get_joined();
                match joined {
                    Ok(rooms) => {
                        let room_names = rooms.joined_rooms.iter().map(|r|{
                            let room_name = bot.client().get_room_name(r);
                            match room_name {
                                Ok(name) => name,
                                Err(_) => {
                                    let members = bot.client().get_members(r).unwrap();
                                    format!("{} ({})", r, members.join(", "))
                                },
                            }
                        }).sorted().join("\n");

                        let resp = format!("Currently in rooms:\n{}", room_names);
                        let fmt_resp = codeblock_format(&resp);
                        bot.reply_fmt(&event, &fmt_resp, &resp).ok();
                    }
                    Err(e) => {
                        let resp = format!("{:?}", e);
                        bot.reply(&event, &resp).ok();
                    }
                };
            }
        }
    }
}
