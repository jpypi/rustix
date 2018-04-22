use std::fs::File;

use rand;
use regex::Regex;

use bot::{Bot, Node, RoomEvent};
use services::utils::reservoir_sample;

pub struct TryFile {
    safe_re: Regex,
}

impl TryFile {
    pub fn new() -> Self {
        Self {
            safe_re: Regex::new(r"[a-zA-Z]").unwrap(),
        }
    }
}


impl<'a> Node<'a> for TryFile {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = event.raw_event.content["body"].as_str().unwrap();

        if self.safe_re.is_match(body) {
            let rng = rand::thread_rng();

            let fname = format!("var/{}.txt", body);
            match File::open(fname) {
                Ok(d) => {
                    let line = reservoir_sample(d, rng);
                    bot.reply(&event, &line);
                },
                Err(e) => (),
                //{ println!("Tried to open: \"{}\" failed: {:?}", body, e); }
            };
        }
    }
}
