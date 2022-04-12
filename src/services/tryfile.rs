use std::fs::File;
use std::fs;
use std::io::{BufReader, BufRead};

use rand::SeedableRng;
use rand::rngs::SmallRng;
use regex::Regex;
use toml::value::Value;

use crate::bot::{Bot, Node, RoomEvent};
use crate::services::utils::reservoir_sample;


pub struct TryFile {
    safe_re: Regex,
    rng: SmallRng,
    directory: String
}


impl TryFile {
    pub fn new(config: Option<&Value>) -> Self {
        Self {
            safe_re: Regex::new(r"[a-zA-Z]").unwrap(),
            rng: SmallRng::from_entropy(),
            directory: config.and_then(|c| c.get("directory")
                                            .and_then(|d| d.as_str())
                                            .map(|s| s.to_string()))
                             .unwrap_or("var".to_string()),
        }
    }
}


impl<'a> Node<'a> for TryFile {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = event.raw_event.content["body"].as_str().unwrap();

        if self.safe_re.is_match(body) {
            let fname = format!("{}/{}.txt", &self.directory, body);
            match File::open(fname) {
                Ok(d) => {
                    let reader = BufReader::new(d);
                    if let Ok(v) = reservoir_sample(reader.lines(), &mut self.rng) {
                        bot.reply(&event, &v).ok();
                    }
                },
                Err(_) => (),
            };
        }
    }

    fn description(&self) -> Option<String> {
        match fs::read_dir(format!("{}", &self.directory)) {
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
