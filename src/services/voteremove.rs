use std::{
    thread,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    collections::{HashMap, HashSet}
};

use regex::Regex;

use crate::{bot::{Bot, Node, RoomEvent}, state};
use crate::config::RemovalMode;

struct Vote {
    start: Instant,
    voters: HashSet<String>,
}


pub struct Voteremove {
    votes: Arc<Mutex<HashMap<String, Vote>>>,
    votes_required: usize,
    timeout: Duration,
    mode: RemovalMode,
    votekick_re: Regex,
    voteban_re: Regex,
}


impl Voteremove {
    // If no value is provided, default to false
    pub fn new(votes_required: usize, wait_minutes: u64, mode: RemovalMode) -> Self {
        Self {
            votes: Arc::new(Mutex::new(HashMap::new())),
            votes_required,
            timeout: Duration::new(wait_minutes * 60, 0),
            mode,
            votekick_re: Regex::new(r"^votekick(?: (.+))?$").unwrap(),
            voteban_re: Regex::new(r"^voteban(?: (.+))?$").unwrap(),
        }
    }

    fn vote_user(&mut self, bot: &Bot, event: &RoomEvent, source: &str, target: &str) {
        let mut votes = self.votes.lock().expect("Poisoned");

        let item = votes.get_mut(target);
        if let Some(v) = item {
            v.voters.insert(source.to_string());
        } else {
            let t_sleep = self.timeout;
            let t_votes = Arc::clone(&self.votes);
            let t_target = target.to_string();
            let t_room_id = event.room_id.to_string();
            let t_mode = self.mode.as_str().to_string();
            let t_client = bot.arc_client();

            thread::spawn(move || {
                thread::sleep(t_sleep);
                let mut votes_map = t_votes.lock().expect("Poisoned");
                if votes_map.remove(&t_target).is_some() {
                    let mut client = t_client.write().unwrap();
                    client.send_msg(&t_room_id, &format!("Vote to {} {} expired.", &t_mode, &t_target)).ok();
                }
            });

            let kv = Vote {
                start: std::time::Instant::now(),
                voters: hashset![source.to_string()],
            };

            votes.insert(target.to_string(), kv);
        }
    }

    fn mode_re(&self) -> &Regex {
        match self.mode {
            RemovalMode::Kick => &self.votekick_re,
            RemovalMode::Ban => &self.voteban_re,
        }
    }
}

impl<'a> Node<'a> for Voteremove {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap();

            if let Some(captures) = self.mode_re().captures(body) {
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

                        self.vote_user(bot, &event, &revent.sender, &uid);
                        let mut vl = self.votes.lock().expect("Poisoned");
                        if let Some(vote_res) = vl.get(&uid) {
                            let cur_votes = vote_res.voters.len();
                            let waited = Instant::now() - vote_res.start;

                            if cur_votes == self.votes_required {
                                vl.remove(&uid);
                                match self.mode {
                                    RemovalMode::Kick => bot.client().kick(event.room_id, &uid, Some("Votekicked")),
                                    RemovalMode::Ban => bot.client().ban(event.room_id, &uid, Some("Votebanned"))
                                }.ok();
                            } else {
                                let mode = match self.mode {
                                    RemovalMode::Kick => "Votekick",
                                    RemovalMode::Ban => "Voteban",
                                };
                                bot.reply(&event, &format!("{} for {} - {} of {} votes needed, time remaining: {}.",
                                                        mode,
                                                        uid,
                                                        cur_votes,
                                                        self.votes_required,
                                                        render_dur(self.timeout - waited))).ok();
                            }
                        }
                    },
                    None => {
                        bot.reply(&event, "Please specify a user to vote for removal.").ok();
                    },
                };
            }
        }
    }

    fn configure(&mut self, bot: &Bot, command: &str, event: RoomEvent) {
        if let Some(votes_args) = command.strip_prefix("votes ")  {
            match votes_args.parse() {
                Ok(value) => self.votes_required = value,
                Err(msg) => { bot.reply(&event, &format!("Error: Could not set number of required votes - {}", msg)).ok(); }
            };
        } else if let Some(time_args) = command.strip_prefix("time ")  {
            match time_args.parse() {
                Ok(value) => self.timeout = Duration::from_secs(value),
                Err(msg) => { bot.reply(&event, &format!("Error: Could not set vote timeout (seconds) - {}", msg)).ok(); },
            };
        } else if let Some(mode_args) = command.strip_prefix("mode ")  {
            match mode_args {
                "kick" => self.mode = RemovalMode::Kick,
                "ban" => self.mode = RemovalMode::Ban,
                x => { bot.reply(&event, &format!("Invalid mode passed to mode: {}", x)).ok(); },
            }
        } else if command.starts_with("status") {
            let mode = match self.mode { RemovalMode::Kick => "kick", RemovalMode::Ban => "ban" };
            bot.reply(&event, &format!("mode: {} - votes: {} - timeout (seconds): {}",
                                       mode,
                                       self.votes_required,
                                       self.timeout.as_secs())).ok();
        }
    }

    fn configure_description(&self) -> Option<String> {
        Some("votes <int>            - Set the number of votes required for success.\n\
              time  <int>            - Set the number of seconds before vote expires.\n\
              mode  <\"kick\" | \"ban\"> - Change mode between kick or ban.\n\
              status                 - View the current configuration of the node.".to_string())
    }

    fn description(&self) -> Option<String> {
        let mode = self.mode.as_str();
        Some(format!("vote{mode} <user> - Vote to {mode} a user. ({} votes in {})", self.votes_required, render_dur(self.timeout)))
    }

    fn on_load(&mut self, service_name: &str) -> Result<(), String> {
        if let Some(state) = state::load_state(service_name) {
            let mut values = state.as_str().split('|');
            self.mode = match values.next() {
                Some("kick") => RemovalMode::Kick,
                Some("ban") => RemovalMode::Ban,
                Some(_) => return Err("Invalid removal mode specified in voteremove state".to_string()),
                None => return Err("Invalid voteremove state".to_string()),
            };

            self.votes_required = match values.next() {
                Some(v) => match v.parse(){
                    Ok(p) => p,
                    Err(_) => return Err("Voteremove votes required state value should parse to usize".to_string()),
                },
                None => return Err("Invalid voteremove state".to_string()),
            };

            self.timeout = match values.next() {
                Some(v) => match v.parse(){
                    Ok(p) => Duration::from_secs(p),
                    Err(_) => return Err("Voteremove timeout state value should parse to u64".to_string()),
                },
                None => return Err("Invalid voteremove state".to_string()),
            };
        }

        Ok(())
    }

    fn on_exit(&self, service_name: &str) {
        state::save_state(service_name, &format!("{}|{}|{}", self.mode.as_str(), self.votes_required, self.timeout.as_secs()));
    }
}


fn render_dur(duration: Duration) -> String{
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let render_seconds = seconds % 60;

    format!("{:02}:{:02}", minutes, render_seconds)
}