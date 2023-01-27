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
            karmastats_re: Regex::new(r"^karmastats(?: (.+))?$").unwrap(),
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


            if body.starts_with("badkarmastats") {
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
                    Some(query) => {
                        let q = query.as_str().trim();
                        match bot.uid_from_displayname(q) {
                            Ok(r) => r,
                            Err(e) => {
                                bot.reply(&event, &format!("{:?}", e)).ok();
                                return;
                            }
                        }
                    },
                    None => revent.sender.clone(),
                };

                if let Ok(rankings) = self.vote_db.user_ranks(&user_query, 10) {
                    if rankings.len() > 0 {
                        response += &format!("Most upvoted by {}: ", user_query);
                    } else {
                        response = format!("{} has not upvoted anything", user_query);
                    }

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
        Some("karmastats <optional word> - View kings of karma\n\
              badkarmastats - View peasants of karma\n\
              nickstats <optional user id> - view ranking of things user has given karma".to_string())
    }
}
