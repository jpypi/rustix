use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};

use super::backend::Backend;

pub struct RankKarma {
    vote_db: Backend,
    karmastats_re: Regex,
    nickstats_re: Regex,
}

impl RankKarma {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
            karmastats_re: Regex::new(r"^kokarma(?: (.+))?$").unwrap(),
            nickstats_re: Regex::new(r"^nickstats(?: (.+))?$").unwrap(),
        }
    }
}

impl<'a> Node<'a> for RankKarma {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
            revent.content["msgtype"] == "m.text" {

            let body = revent.content["body"].as_str().unwrap().trim();

            let mut response = String::new();

            if let Some(captures) = self.karmastats_re.captures(body) {
                if let Some(query) = captures.get(1) {
                    let clean_query = query.as_str().trim();
                    if let Ok(rankings) = self.vote_db.votes_rank(clean_query, 10) {
                        response += &format!("Top upvoters for '{}': ", clean_query);
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
                    response += "All time most downvoted: ";
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

            if let Some(captures) = self.nickstats_re.captures(body) {
                let user_query = match captures.get(1) {
                    Some(query) => query.as_str().trim(),
                    None => &revent.sender,
                };

                if let Ok(rankings) = self.vote_db.user_ranks(user_query, 10) {
                    response += &format!("Most upvoted by {}: ", user_query);
                    for (i, (item, up, down)) in rankings.iter().enumerate() {
                        let item = format!("{}. '{}' with {} (+{}/-{})",
                                           i + 1, item, up - down, up, down);
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
