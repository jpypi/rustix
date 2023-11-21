use std::{time::Duration, io::{prelude::*,BufReader}};
use std::fs::File;

use reqwest;
use rust_tokenizers::tokenizer::{TruncationStrategy, Gpt2Tokenizer, Tokenizer};
use sha3::Digest;
use toml::Value;
use serde::Deserialize;

use crate::{bot::{Bot, Node, RoomEvent}, utils, state};
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
            temperature: Some(0.8),
            top_p: None,
            presence_penalty: None,
            frequency_penalty: Some(0.4),
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

    fn build_context(&self, bot: &Bot, room_id: &str, username: &str, message: &str) -> (String, u32) {
        let mut to_complete = String::new();
        to_complete += &self.backstory;

        let max_context = 1000;
        let mut context_tokens = self.count_tokens(&to_complete);

        let mut messages = Vec::<String>::new();

        if let Ok(e) = bot.get_room_events(room_id, 100, None) {
            for event in e.chunk.iter().skip(1) {
                if !(event.type_ == "m.room.message" && event.content["msgtype"] == "m.text") {
                    continue;
                }

                let event_uname = trim_name(&event.sender);

                let event_body = event.content["body"].as_str().unwrap().trim_start_matches("!chat ");
                let context_line = format!("<{}> {}", event_uname, event_body);
                let context_line_size = self.count_tokens(&context_line);

                if context_tokens + context_line_size > max_context {
                    break;
                }

                messages.push(context_line);
                context_tokens += context_line_size;
            }
        }

        messages.reverse();
        to_complete += &messages.join("\n");

        let final_line = format!("\n<{}> {}\n<{}> ", username, message, bot.get_displayname());
        context_tokens += self.count_tokens(&final_line);

        to_complete += &final_line;

        (to_complete, context_tokens)
    }
}

fn trim_name(name: &str) -> &str{
    name.strip_prefix('@').unwrap().split(':').collect::<Vec<&str>>()[0]
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
                bot.indicate_typing(&event.room_id, Some(Duration::from_secs(60))).ok();

                let uname = trim_name(&revent.sender);
                let (context, _) = self.build_context(bot, event.room_id, uname, message);

                let count = self.count_tokens(&context);
                if  count as f64 > self.token_budget {
                    bot.indicate_typing(&event.room_id, None).ok();
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
                                println!("total tokens: {}", s.usage.total_tokens);
                                //println!("-----------\n{}{}\ntotal tokens: {}\n-----------", &context, txt, s.usage.total_tokens);
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

                bot.indicate_typing(&event.room_id, None).ok();
            }

            if let Some(_) = body.strip_prefix("budget") {
                bot.reply(&event, &format!("tokens used: {}\ntokens/second: {:.4}\ntoken budget: {:.0}",
                                           self.used_tokens, self.tokens_per_second, self.token_budget)).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("chat <message> - Talk to the bot. Historical room messages are sent so the response is more likely to be useful.".to_string())
    }

    fn on_load(&mut self, service_name: &str) {
        let saved_state = state::load_state(service_name);
        if let Some(state) = saved_state {
            let mut parsed = state.split(" ");

            if let Some(s) = parsed.next() {
                self.token_budget = s.parse().expect("Unable to parse token budget from save state");
            }
            if let Some(s) = parsed.next() {
                self.used_tokens = s.parse().expect("Unable to parse used tokens from save state");
            }
        }
    }

    fn on_exit(&self, service_name: &str) {
        state::save_state(service_name, &format!("{} {}", self.token_budget, self.used_tokens));
    }
}