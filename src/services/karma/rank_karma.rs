use crate::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

pub struct RankKarma {
    vote_db: Backend,
}

impl RankKarma {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
        }
    }
}

impl<'a> Node<'a> for RankKarma {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
            revent.content["msgtype"] == "m.text" {
            let body = revent.content["body"].as_str().unwrap().trim();
            if body.starts_with("kkarma") {
                if let Ok(rankings) = self.vote_db.voteables_rank_desc(10) {
                    let mut response = String::new();
                    for (i, r) in rankings.iter().enumerate() {
                        let item = format!("{}. '{}' with {} (+{}/-{});",
                                           i+1, r.value, r.total_up - r.total_down,
                                           r.total_up, r.total_down);
                        if i > 0 {
                            response += " ";
                        }
                        response += &item;
                    }

                    bot.reply(&event, &response).ok();
                }
            }

            if body.starts_with("lkarma") {
                if let Ok(rankings) = self.vote_db.voteables_rank_asc(10) {
                    let mut response = String::new();
                    for (i, r) in rankings.iter().enumerate() {
                        let item = format!("{}. '{}' with {} (+{}/-{});",
                                           i+1, r.value, r.total_up - r.total_down,
                                           r.total_up, r.total_down);
                        if i > 0 {
                            response += " ";
                        }
                        response += &item;
                    }

                    bot.reply(&event, &response).ok();
                }
            }


        }
    }

    fn description(&self) -> Option<String> {
        Some("kkarma - View karma rank".to_string())
    }
}
