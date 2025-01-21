use regex::Regex;

use crate::{bot::{Bot, Node, RoomEvent}, utils::codeblock_format};

use super::backend::Backend;

pub struct RankKarma {
    vote_db: Backend,
    karmastats_re: Regex,
    badkarmastats_re: Regex,
    nickstats_re: Regex,
    badnickstats_re: Regex,
}

impl RankKarma {
    pub fn new() -> Self {
        Self {
            vote_db: Backend::new(),
            karmastats_re: Regex::new(r"^(?:karmastats|ks)(?: (.+))?$").unwrap(),
            badkarmastats_re: Regex::new(r"^(?:badkarmastats|bks)(?: (.+))?$").unwrap(),
            nickstats_re: Regex::new(r"^(?:nickstats|ns)(?: (.+))?$").unwrap(),
            badnickstats_re: Regex::new(r"^(?:badnickstats|bns)(?: (.+))?$").unwrap(),
        }
    }
}

#[allow(clippy::ptr_arg)]
fn ranking_reply(bot: &Bot, event: &RoomEvent, header: &str, ranks: &Vec<String>) {
    let plain_responsee = format!("{} {}", header, ranks.join("; "));
    let fancy_msg = format!("{}\n{}", header, ranks.join("\n"));
    let fancy_response = codeblock_format(&fancy_msg);
    bot.reply_fmt(event, &fancy_response, &plain_responsee).ok();
}

impl<'a> Node<'a> for RankKarma {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap().trim();

            if let Some(captures) = self.karmastats_re.captures(body) {
                if let Some(query) = captures.get(1) {
                    let clean_query = query.as_str().trim();
                    if let Ok(rankings) = self.vote_db.votes_rank(clean_query, 10, true) {
                        let ranks = rankings.iter().enumerate().map(|(i, (user, up, down))| {
                            format!("{}. {} with {} (+{}/-{})", i + 1, user, up - down, up, down)
                        }).collect();

                        let header = format!("Top upvoters for '{}':", clean_query);
                        ranking_reply(bot, &event, &header, &ranks);
                    }
                } else if let Ok(rankings) = self.vote_db.voteables_rank_desc(10) {
                    let header = "All time most upvoted:";
                    let ranks = rankings.iter().enumerate().map(|(i, r)| {
                        format!("{}. '{}' with {} (+{}/-{})", i + 1, r.value, r.total_up - r.total_down, r.total_up, r.total_down)
                    }).collect();
                    ranking_reply(bot, &event, header, &ranks);
                }
            }


            if let Some(captures) = self.badkarmastats_re.captures(body) {
                if let Some(query) = captures.get(1) {
                    let clean_query = query.as_str().trim();
                    if let Ok(rankings) = self.vote_db.votes_rank(clean_query, 10, false) {
                        let header = format!("Top downvoters for '{}':", clean_query);
                        let ranks = rankings.iter().enumerate().map(|(i, (user, up, down))| {
                             format!("{}. {} with {} (+{}/-{})", i + 1, user, up - down, up, down)
                        }).collect();

                        ranking_reply(bot, &event, &header, &ranks);
                    }
                } else if let Ok(rankings) = self.vote_db.voteables_rank_asc(10) {
                    let header = "All time most downvoted:";
                    let ranks = rankings.iter().enumerate().map(|(i, r)| {
                        format!("{}. '{}' with {} (+{}/-{})", i + 1, r.value, r.total_up - r.total_down, r.total_up, r.total_down)
                    }).collect();

                    ranking_reply(bot, &event, header, &ranks);
                }
            }

            if let Some(captures) = self.nickstats_re.captures(body) {
                let user_query = match captures.get(1) {
                    Some(query) => {
                        let q = query.as_str().trim();
                        match bot.uid_from_displayname(q) {
                            Ok(r) => r,
                            Err(crate::errors::Error::Generic(_)) => {
                                bot.reply(&event, &format!("Unable to determine user id for: {}", q)).ok();
                                return;
                            },
                            Err(e) => {
                                println!("{:?}", e);
                                return;
                            }
                        }
                    },
                    None => revent.sender.clone(),
                };

                if let Ok(rankings) = self.vote_db.user_ranks(&user_query, 10) {
                    if rankings.is_empty() {
                        bot.reply(&event, &format!("{} has not upvoted anything", user_query)).ok();
                    } else {
                        let header = format!("Most upvoted by {}:", user_query);
                        let ranks = rankings.iter().enumerate().map(|(i, (item, up, down))| {
                            format!("{}. '{}' with {} (+{}/-{})", i + 1, item, up - down, up, down)
                        }).collect();

                        ranking_reply(bot, &event, &header, &ranks);
                    }
                }
            }

            if let Some(captures) = self.badnickstats_re.captures(body) {
                let user_query = match captures.get(1) {
                    Some(query) => {
                        let q = query.as_str().trim();
                        match bot.uid_from_displayname(q) {
                            Ok(r) => r,
                            Err(crate::errors::Error::Generic(_)) => {
                                bot.reply(&event, &format!("Unable to determine user id for: {}", q)).ok();
                                return;
                            },
                            Err(e) => {
                                println!("{:?}", e);
                                return;
                            }
                        }
                    },
                    None => revent.sender.clone(),
                };

                if let Ok(rankings) = self.vote_db.user_ranks_asc(&user_query, 10) {
                    if rankings.is_empty() {
                        bot.reply(&event, &format!("{} has not downvoted anything", user_query)).ok();
                    } else {
                        let header = format!("Most downvoted by {}:", user_query);
                        let ranks = rankings.iter().enumerate().map(|(i, (item, up, down))| {
                            format!("{}. '{}' with {} (+{}/-{})", i + 1, item, up - down, up, down)
                        }).collect();

                        ranking_reply(bot, &event, &header, &ranks);
                    }
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("karma rankings:\n\
              \tkarmastats (alt: ks) <optional thing> - View kings of karma. Providing no argument will rank all things.\n\
              \tbadkarmastats (alt: bks) <optional thing> - View peasants of karma.\n\
              \tnickstats (alt: ns) <optional user id> - View ranking of things user has given karma.\n\
              \tbadnickstats (alt: bns) <optional user id> - View ranking of things user has given karma... but from the other end.".to_string())
    }
}
