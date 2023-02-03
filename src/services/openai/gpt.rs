use std::{time::Duration, io::{prelude::*,BufReader}};
use std::fs::File;

use reqwest;
use rust_tokenizers::tokenizer::{TruncationStrategy, Gpt2Tokenizer, Tokenizer};
use sha3::Digest;
use toml::Value;
use serde::{Deserialize};

use crate::bot::{Bot, Node, RoomEvent};
use super::types::*;

const BASE_URL: &str = "https://api.openai.com/v1/completions";


#[derive(Deserialize)]
struct Config {
    secret: String,
    backstory_file: String,
    monthly_budget: f64,
    starting_tokens: Option<u64>,
}

pub struct GPT {
    secret: String,
    tokenizer: Gpt2Tokenizer,
    current_model: ModelType,
    backstory: String,
    used_tokens: u64,
    token_budget: f64,
    tokens_per_second: f64,
    last_query: std::time::Instant,
}


impl GPT {
    pub fn new(config: &Value) -> Self {
        let cfg: Config = config.clone().try_into().expect("Bad openai config.");

        // Load backstory
        let backstory_file = File::open(cfg.backstory_file).expect("Unable to open backstory file.");
        let mut buf_reader = BufReader::new(backstory_file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).expect("Error reading backstory contents.");

        // Files retrieved from here:
        // https://huggingface.co/gpt2/tree/main
        let tokenizer = Gpt2Tokenizer::from_file("vocab.json", "merges.txt", false).unwrap();

        let daily = cfg.monthly_budget / 30.0;
        let model_cost = 0.02;
        let daily_tokens = daily / model_cost * 1000.0;
        let tokens_per_second = daily_tokens / 24.0 / 60.0/ 60.0;

        Self {
            secret: cfg.secret,
            tokenizer,
            current_model: ModelType::Davinci,
            backstory: contents,
            used_tokens: 0,
            token_budget: cfg.starting_tokens.unwrap_or(0) as f64,
            tokens_per_second,
            last_query: std::time::Instant::now(),
        }
    }

    fn count_tokens(&self, message: &str) -> u32 {
        let res = self.tokenizer.encode(message, None, 8000, &TruncationStrategy::DoNotTruncate, 0);
        res.token_ids.len() as u32
    }

    fn complete(&mut self, message: &str, userid: &str) -> Result<Response, reqwest::Error> {
        let msg_tokens = self.count_tokens(message);
        let hash = sha3::Sha3_256::new_with_prefix(userid).finalize();
        let hashed_userid = base16ct::lower::encode_string(&hash);

        let client = reqwest::blocking::Client::new();
        let req = Query {
            model: self.current_model,
            prompt: message.to_string(),
            max_tokens: Some(4096 - msg_tokens),
            temperature: Some(0.2),
            top_p: None,
            n: Some(1),
            user: Some(hashed_userid),
        };

        let res = client.post(BASE_URL).timeout(Duration::new(30, 0))
                        .bearer_auth(&self.secret)
                        .json(&req)
                        .send()?
                        .json::<Response>();

        self.used_tokens += msg_tokens as u64;

        res.and_then(|r| {
            match r {
                Response::Success(s) => {
                    self.used_tokens += self.count_tokens(&s.choices[0].text) as u64;
                    Ok(Response::Success(s))
                },
                Response::Error(e) => Ok(Response::Error(e))
            }
        })
    } 

    fn build_context(&self, bot_name: &str, username: &str, message: &str) -> String {
        let mut to_complete = String::new();
        to_complete += &self.backstory;
        to_complete += &format!("\n<{}> {}\n<{}> ", username, message, bot_name);

        to_complete
    }
}


impl<'a> Node<'a> for GPT {
   fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if event.is_normal() {
            let body = &revent.content["body"].as_str().unwrap();

            let dt = self.last_query.elapsed().as_secs_f64();
            self.token_budget += self.tokens_per_second * dt;
            self.last_query = std::time::Instant::now();

            if let Some(message) = body.strip_prefix("chat ") {
                let uname = revent.sender.strip_prefix('@').unwrap().split(':').collect::<Vec<&str>>()[0];
                let context = self.build_context(bot.get_displayname(), uname, message);

                let count = self.count_tokens(&context);
                if  count as f64 > self.token_budget {
                    bot.reply(&event, &format!("Sorry. Rate limited. :(\n{} tokens > token budget of {:.0}", count, self.token_budget)).ok();
                    return;
                }

                match self.complete(&context, &revent.sender) {
                    Ok(r) => {
                        match r {
                            Response::Error(e) => println!("{:?}", e),
                            Response::Success(s) => {
                                let txt = s.choices[0].text.trim();
                                self.token_budget -= s.usage.total_tokens as f64;
                                println!("-----------\n{}{}\ntotal tokens: {}\n-----------", &context, txt, s.usage.total_tokens);
                                bot.reply(&event, &txt).ok();
                            }
                        }
                    },
                    Err(e) => {
                        if e.is_timeout() {
                            bot.reply(&event, &"Chat response timed out.").ok();
                        } else {
                            println!("{:?}", e);
                        }
                    }
                }

            }

            if let Some(_) = body.strip_prefix("budget") {
                bot.reply(&event, &format!("tokens used: {}\ntokens/second: {:.4}\ntoken budget: {:.0}",
                                           self.used_tokens, self.tokens_per_second, self.token_budget)).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("chat <message> - talk to rustix".to_string())
    }
}