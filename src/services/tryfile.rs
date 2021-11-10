use std::fs::File;
use std::fs;
use std::io::{BufReader, BufRead};

use rand::SeedableRng;
use rand::rngs::SmallRng;
use regex::Regex;

use crate::bot::{Bot, Node, RoomEvent};
use crate::services::utils::reservoir_sample;

const DIR: &str = "var";

pub struct TryFile {
    safe_re: Regex,
    rng: SmallRng,
}

impl TryFile {
    pub fn new() -> Self {
        Self {
            safe_re: Regex::new(r"[a-zA-Z]").unwrap(),
            rng: SmallRng::from_entropy(),
        }
    }
}


impl<'a> Node<'a> for TryFile {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = event.raw_event.content["body"].as_str().unwrap();

        if self.safe_re.is_match(body) {
            let fname = format!("{}/{}.txt", DIR, body);
            match File::open(fname) {
                Ok(d) => {
                    let reader = BufReader::new(d);
                    if let Ok(v) = reservoir_sample(reader.lines(), &mut self.rng) {
                        bot.reply(&event, &v);
                    }
                },
                Err(_) => (),
                //{ println!("Tried to open: \"{}\" failed: {:?}", body, e); }
            };
        }
    }

    fn description(&self) -> Option<String> {
        match fs::read_dir(format!("{}", DIR)) {
            Ok(paths) => {
                Some(paths.map(|p| {
                                    let mut path = p.unwrap().path();
                                    path.set_extension("");
                                    path.into_iter()
                                        .next_back()
                                        .unwrap()
                                        .to_str() // consider to_string_lossy
                                        .unwrap()
                                        .to_string()
                               })
                          .collect::<Vec<String>>()
                          .join("\n"))
            }
            Err(_) => {
                None
            }
        }
    }
}
