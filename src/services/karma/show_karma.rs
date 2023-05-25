use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

pub struct ShowKarma<'a> {
    vote_db: Backend,
    children: Vec<&'a str>,
}

impl<'a> ShowKarma<'a> {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for ShowKarma<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
            revent.content["msgtype"] == "m.text" {
            let body = revent.content["body"].as_str().unwrap();

            let check_re = Regex::new(r"^karma (.+)").unwrap();

            let mut finds = 0;
            for cap in check_re.captures_iter(body) {
                finds += 1;
                let query = cap[1].trim();
                let res = self.vote_db.get_upvotes(query);
                if let Ok(Some(k)) = res {
                    let positive = (k.total_up as f32/(k.total_up+k.total_down) as f32)*100.0;
                    let total = k.total_up - k.total_down;
                    let response = format!("Karma for '{}': Net karma: {} (+{}/-{}; {:.1}% like it)",
                        query, total, k.total_up, k.total_down, positive
                    );
                    bot.reply(&event, &response).ok();
                } else if let Ok(None) = res {
                    let response = format!("Karma for '{}': Net karma: 0 (+0/-0 0% like it)", query);
                    bot.reply(&event, &response).ok();
                } else {
                    bot.reply(&event, &format!("Error querying karma for '{}'", query)).ok();
                }
            }

            if finds == 0 {
                self.propagate_event(bot, &event);
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("karma <thing> - View aggregated karma stats for something.".to_string())
    }
}
