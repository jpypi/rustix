use std::io::{BufReader, BufRead};
use std::fs::File;

use rand;
use rand::Rng;

use bot::{Bot, Node, RoomEvent};

const K: usize = 10;

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
        let revent = event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" &&
           revent.sender != "@rustix:cclub.cs.wmich.edu" {

            let body = revent.content["body"].as_str().unwrap();

            if body.starts_with("timecube") {
                let mut rng = rand::thread_rng();

                let f = match File::open("timecube.txt") {
                    Ok(d) => d,
                    Err(e) => {
                        bot.reply(&event, "Error: problem with timecube file!");
                        return;
                    }
                };

                let reader = BufReader::new(f);

                let mut reservoir: [String;K] = Default::default();

                for (i, line) in reader.lines().enumerate() {
                    let l = line.unwrap();

                    if i < K {
                        reservoir[i] = l;
                    } else {
                        let j = rng.gen_range(0, i);
                        if j < K {
                            reservoir[j] = l;
                        }
                    }
                }

                let n = rng.gen_range(0, K);

                bot.reply(&event, &reservoir[n]);
            }
        }
    }
}
