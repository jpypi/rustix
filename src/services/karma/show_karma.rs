use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

pub struct ShowKarma {
    vote_db: Backend,
    karma_re: Regex,
}

impl ShowKarma {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
            karma_re: Regex::new(r"^karma (.+)").unwrap(),
        }
    }
}

impl<'a> Node<'a> for ShowKarma {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if event.is_normal() {
            let body = &event.raw_event.content["body"].as_str().unwrap();

            for cap in self.karma_re.captures_iter(body) {
                let query = cap[1].trim();
                let res = match self.vote_db.get_upvotes(query) {
                    Ok(Some(k)) => {
                        let positive = (k.total_up as f32/(k.total_up+k.total_down) as f32)*100.0;
                        let total = k.total_up - k.total_down;
                        format!("Karma for '{}': Net karma: {} (+{}/-{}; {:.1}% like it)",
                            query, total, k.total_up, k.total_down, positive
                        )
                    },
                    Ok(None) => format!("Karma for '{}': Net karma: 0 (+0/-0 0% like it)", query),
                    _ => format!("Error querying karma for '{}'", query),
                };
                bot.reply(&event, &res).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("karma <thing> - View aggregated karma stats for something.".to_string())
    }
}
