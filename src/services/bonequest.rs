use std::time::Duration;

use reqwest;
use regex::Regex;
use rand::seq::IteratorRandom;
use html_escape;
use toml::Value;

use crate::bot::{Bot, Node, RoomEvent};

const RANDOM_BQ_URL: &str = "https://www.bonequest.com/random/?_";


pub struct Bonequest {
    dialog_regex: Regex,
    profanity: Vec<String>
}

#[derive(Deserialize)]
struct Config {
    profanity: Vec<String>
}

impl Bonequest {
    pub fn new(config: &Value) -> Self {
        let cfg: Config = config.clone().try_into().expect("Bad bonequest config.");
        Self {
            dialog_regex: Regex::new("(?s)<div class=\"acre\" style=\"margin-left: 40px; margin-right: 40px; font-family: monospace; white-space: pre-wrap; text-align: left;\">(.+?)</div>").unwrap(),
            profanity: cfg.profanity.iter().map(|p| p.to_lowercase()).collect(),
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
                                    .and_then(|l| l.split(": ").nth(1));
                    return match line {
                        Some(l) => Ok(l.to_owned()),
                        None => Ok("Line parsing error".to_owned()),
                    }
                }
            }

            Ok("Regex error".to_owned())
        })
    }
}

impl<'a> Node<'a> for Bonequest {
   fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = &revent.content["body"].as_str().unwrap();

        if body.starts_with("bq") {
            bot.indicate_typing(&event.room_id, Some(Duration::from_secs(10))).ok();

            'attempts: for _ in 0..10 {
                match self.get_line() {
                    Ok(l) => {
                        let escaped = html_escape::decode_html_entities(&l);
                        let lowered = escaped.to_lowercase();
                        for word in self.profanity.iter() {
                            if lowered.contains(word) {
                                continue 'attempts;
                            }
                        }
                        bot.reply(&event, &escaped).ok();
                        break;
                    },
                    Err(e) => {
                        if e.is_timeout() {
                            bot.reply(&event, &"bq timed out").ok();
                        } else {
                            println!("{:?}", e);
                        }
                        break;
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