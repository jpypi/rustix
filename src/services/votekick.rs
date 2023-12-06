use std::{time::{Duration, Instant}, collections::{HashMap, HashSet}};

use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};


struct KickVote {
    start: Instant,
    voters: HashSet<String>,
}


pub struct Votekick {
    votes: HashMap<String, KickVote>,
    votes_required: usize,
    wait: Duration,
    votekick_re: Regex,
}

impl<'a> Votekick {
    // If no value is provided, default to false
    pub fn new(votes_required: usize, wait_minutes: u64) -> Self {
        Self {
            votes: HashMap::new(),
            votes_required,
            wait: Duration::new(wait_minutes * 60, 0),
            votekick_re: Regex::new(r"^votekick(?: (.+))?$").unwrap(),
        }
    }

    fn vote_user(&mut self, source: &str, target: &str) -> &KickVote {
        let item = self.votes.get_mut(target);

        if let Some(v) = item {
            v.voters.insert(source.to_string());
            self.votes.get(target).unwrap()
        } else {
            let kv = KickVote {
                start: std::time::Instant::now(),
                voters: hashset![source.to_string()],
            };

            self.votes.insert(target.to_string(), kv);
            self.votes.get(target).unwrap()
        }
    }
}

impl<'a> Node<'a> for Votekick {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap();

            if let Some(captures) = self.votekick_re.captures(body) {
                match captures.get(1) {
                    Some(query) => {
                        let query_str = query.as_str().trim();
                        let uid = match bot.uid_from_displayname(query_str) {
                            Ok(r) => r,
                            Err(crate::errors::Error::Generic(_)) => {
                                bot.reply(&event, &format!("Unable to determine user id for: {}", query_str)).ok();
                                return;
                            },
                            Err(e) => {
                                println!("{:?}", e);
                                return;
                            }
                        };

                        let w = self.wait.clone();
                        let vote_res = self.vote_user(&revent.sender, &uid);
                        let cur_votes = vote_res.voters.len();
                        let waited = Instant::now() - vote_res.start;
                        if waited > w {
                            self.votes.remove(&uid);
                            bot.reply(&event, &format!("Votekick for {} expired.", uid)).ok();
                        } else if cur_votes >= self.votes_required {
                            self.votes.remove(&uid);
                            bot.kick(event.room_id, &uid, Some("Votekicked")).ok();
                        } else if cur_votes == 1 {
                            bot.reply(&event, &format!("Votekick started for {} - votes remaining: {}, time remaining: {}",
                                                       uid,
                                                       self.votes_required - 1,
                                                       render_dur(w - waited))).ok();
                        } else {
                            bot.reply(&event, &format!("Votes remaining to votekick {}: {}, time remaining: {}",
                                                       uid,
                                                       self.votes_required - cur_votes,
                                                       render_dur(w - waited))).ok();
                        }
                    },
                    None => {
                        bot.reply(&event, "Please specify a user to vote kick.").ok();
                    },
                };
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("votekick <user> - Vote to kick a user.".to_string())
    }
}


fn render_dur(duration: Duration) -> String{
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let render_seconds = seconds % 60;

    format!("{}:{}", minutes, render_seconds)
}