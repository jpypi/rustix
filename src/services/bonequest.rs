use std::time::Duration;

use reqwest;
use regex::Regex;
use rand::seq::IteratorRandom;
use html_escape;

use crate::bot::{Bot, Node, RoomEvent};

const RANDOM_BQ_URL: &str = "https://www.bonequest.com/random/?_";


pub struct Bonequest {
    dialog_regex: Regex,
}


impl Bonequest {
    pub fn new() -> Self {
        Self {
            dialog_regex: Regex::new("(?s)<div style=\"margin-left: 40px; margin-right: 40px; font-family: monospace; white-space: pre-wrap; text-align: left;\">(.+?)</div>").unwrap()
        }
    }

    fn get_line(&self) -> Result<String, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let res = client.get(RANDOM_BQ_URL).timeout(Duration::new(30, 0))
                        .send()?
                        .text();

        let mut rng = rand::thread_rng();

        res.and_then(|r| {
            if let Some(captures) = self.dialog_regex.captures(&r) {
                if let Some(content) = captures.get(1) {
                    let lines = content.as_str().split("\n");
                    let line = lines.choose(&mut rng)
                                    .and_then(|l| l.split(": ")
                                                   .nth(1));
                    return match line {
                        Some(l) => Ok(l.to_owned()),
                        None => Ok("Error".to_owned()),
                    }
                }
            }

            Ok("".to_owned())
        })
    }
}

impl<'a> Node<'a> for Bonequest {
   fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = &revent.content["body"].as_str().unwrap();

        if body.starts_with("bq") {
            bot.indicate_typing(&event.room_id, Some(Duration::from_secs(10))).ok();

            match self.get_line() {
                Ok(l) => {
                    let escaped = html_escape::decode_html_entities(&l);
                    bot.reply(&event, &escaped).ok();
                },
                Err(e) => {
                    if e.is_timeout() {
                        bot.reply(&event, &"bq timed out").ok();
                    } else {
                        println!("{:?}", e);
                    }
                }
            }

            bot.indicate_typing(&event.room_id, None).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("bq - Fetch a random Bonequest line".to_string())
    }
}