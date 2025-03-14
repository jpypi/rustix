use std::{collections::HashMap, time::Duration};

use reqwest;
use rand::seq::SliceRandom;
use toml::Value;

use crate::bot::{Bot, Node, RoomEvent};

const BQ_BASE_URL: &str = "https://www.bonequest.com";

#[derive(Deserialize)]
struct BqRandom {
    episodes: Vec<BqEpisode>,
}

#[derive(Deserialize)]
struct BqEpisode {
    quote: String,
}

#[derive(Deserialize)]
struct BqDialog {
    dialog: HashMap<String, Vec<String>>,
}


pub struct Bonequest {
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
            profanity: cfg.profanity.iter().map(|p| p.to_lowercase()).collect(),
        }
    }

    fn rand_character(&self, character: &str) -> Result<String, reqwest::Error> {
        let mut rng = rand::thread_rng();

        let client = reqwest::blocking::Client::new();
        let res = client.get(BQ_BASE_URL.to_string() + "/dialog.json")
                        .header(reqwest::header::USER_AGENT, "rustix-matrix-bot")
                        .timeout(Duration::new(30, 0))
                        .send()?
                        .json::<BqDialog>()?;

        Ok(res.dialog.get(character)
                     .and_then(|lines| lines.choose(&mut rng).map(|s| s.to_string()))
                     .unwrap_or_else(|| format!("Could not find character `{}`.", character)))
    }

    fn get_line(&self) -> Result<String, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let res = client.get(BQ_BASE_URL.to_string() + "/api/v2/quote/random")
                        .header(reqwest::header::USER_AGENT, "rustix-matrix-bot")
                        .timeout(Duration::new(30, 0))
                        .send()?
                        .json::<BqRandom>()?;

        Ok(res.episodes.get(0).map(|e| e.quote.to_string()).unwrap_or("Random quote API failed.".to_string()))
    }
}

impl<'a> Node<'a> for Bonequest {
   fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = &revent.content["body"].as_str().unwrap();

        if let Some(p) = body.strip_prefix("bq") {
            bot.client().indicate_typing(event.room_id, Some(Duration::from_secs(10))).ok();

            let line = if p.starts_with(' ') && p.trim().len() > 0 {
                self.rand_character(p.trim())
            } else {
                self.get_line()
            };

            'attempts: for _ in 0..10 {
                match line {
                    Ok(ref l) => {
                        let lowered = l.to_lowercase();
                        for word in self.profanity.iter() {
                            if lowered.contains(word) {
                                continue 'attempts;
                            }
                        }
                        bot.reply(&event, &l).ok();
                    },
                    Err(e) => {
                        if e.is_timeout() {
                            bot.reply(&event, "bq timed out").ok();
                        } else {
                            println!("{:?}", e);
                        }
                    }
                }
                break;
            }

            bot.client().indicate_typing(event.room_id, None).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("bq <optional character name> - Fetch a random Bonequest line".to_string())
    }
}