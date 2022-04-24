use regex::Regex;

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

            let check_re = Regex::new(r"^kokarma(?: (.+))?$").unwrap();
            let mut response = String::new();

            if let Some(captures) = check_re.captures(body) {
                if let Some(query) = captures.get(1) {
                    if let Ok(rankings) = self.vote_db.votes_rank(query.as_str().trim(), 10) {
                        response += &format!("Top upvoters for '{}': ", query.as_str().trim());
                        for (i, (user, up, down)) in rankings.iter().enumerate() {
                            let item = format!("{}. {} with {} (+{}/-{})",
                                               i + 1, user, up - down, up, down);
                            if i > 0 {
                                response += "; ";
                            }
                            response += &item;
                        }

                        bot.reply(&event, &response).ok();
                    }
                } else if let Ok(rankings) = self.vote_db.voteables_rank_desc(10) {
                    response += "All time most upvoted: ";
                    for (i, r) in rankings.iter().enumerate() {
                        let item = format!("{}. '{}' with {} (+{}/-{})",
                                           i + 1, r.value, r.total_up - r.total_down,
                                           r.total_up, r.total_down);
                        if i > 0 {
                            response += "; ";
                        }
                        response += &item;
                    }

                    bot.reply(&event, &response).ok();
                }
            }


            if body.starts_with("pokarma") {
                if let Ok(rankings) = self.vote_db.voteables_rank_asc(10) {
                    for (i, r) in rankings.iter().enumerate() {
                        let item = format!("{}. '{}' with {} (+{}/-{})",
                                           i + 1, r.value, r.total_up - r.total_down,
                                           r.total_up, r.total_down);
                        if i > 0 {
                            response += "; ";
                        }
                        response += &item;
                    }

                    bot.reply(&event, &response).ok();
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("kokarma <optional word> - View kings of karma\npokarma - View peasants of karma".to_string())
    }
}
