use std::fs::{self, File};
use std::path::PathBuf;
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
    directory: PathBuf
}


impl TryFile {
    pub fn new(config: Option<&Value>) -> Self {
        let dir_string = config.and_then(|c| c.get("directory")
                                              .and_then(|d| d.as_str())
                                              .map(|s| s.to_string()))
                               .unwrap_or("var".to_string());
        let path = PathBuf::from(dir_string)
            .canonicalize()
            .expect("Error validating directory for tryfile");

        Self {
            safe_re: Regex::new(r"[a-zA-Z]").unwrap(),
            rng: SmallRng::from_entropy(),
            directory: path,
        }
    }
}


impl<'a> Node<'a> for TryFile {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = event.raw_event.content["body"].as_str().unwrap();

        if self.safe_re.is_match(body) {
            // Build path to file to try
            // canonicalize normalizes the path and validates that it exists
            let path = match (&self.directory).join(body).with_extension("txt").canonicalize() {
                Ok(p) => p,
                Err(_) => return,
            };

            // Make sure any accessed file is a child of the config directory
            if !path.starts_with(&self.directory) {
                return;
            }

            match File::open(path) {
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
        match fs::read_dir(&self.directory) {
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
