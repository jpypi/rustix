use std::fs::File;

use rand;

use bot::{Bot, Node, RoomEvent};
use services::utils::reservoir_sample;

pub struct Timecube {
}

impl Timecube {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl<'a> Node<'a> for Timecube {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" &&
           revent.sender != "@rustix:cclub.cs.wmich.edu" {

            let body = revent.content["body"].as_str().unwrap();

            if body.starts_with("timecube") {
                let rng = rand::thread_rng();

                match File::open("timecube.txt") {
                    Ok(d) => {
                        let line = reservoir_sample(d, rng);
                        bot.reply(&event, &line);
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        bot.reply(&event, "Error: problem with timecube file!");
                    }
                };
            }
        }
    }
}
