use regex::Regex;

use ::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

pub struct ShowKarma {
    vote_db: Backend,
}

impl ShowKarma {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new()
        }
    }
}

impl<'a> Node<'a> for ShowKarma {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = event.raw_event;
        if revent.type_ == "m.room.message" &&
            revent.content["msgtype"] == "m.text" {
            let body = revent.content["body"].as_str().unwrap();

            let up_re = Regex::new(r"^karma (.+)").unwrap();

            for cap in up_re.captures_iter(body) {
                let query = cap[1].trim();
                if let Some(k) = self.vote_db.get_upvotes(query) {
                    let positive = (k.total_up as f32/(k.total_up+k.total_down) as f32)*100.0;
                    let total = k.total_up - k.total_down;
                    let response = format!("{} has {} karma (+{}/-{}) {:.2}% like it",
                        query, total, k.total_up, k.total_down, positive
                    );
                    bot.reply(&event, &response);
                }
            }
        }
    }
}
