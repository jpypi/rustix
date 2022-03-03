use rand::Rng;

use crate::bot::{Bot, Node, RoomEvent};

const SIZE:usize = 6;

#[derive(PartialEq)]
pub enum RouletteLevel{
    Kick,
    Ban,
}

pub struct Roulette {
    rounds: [u8; SIZE],
    state: u8,
    level: RouletteLevel,
}

impl Roulette {
    // If no value is provided, default to false
    pub fn new(level: RouletteLevel) -> Self {
        let mut x = Self {
            rounds: [0; SIZE],
            state: 0,
            level: level,
        };

        Self::reset(&mut x);

        return x;
    }

    fn fire(&mut self) -> bool {
        self.state = (self.state + 1) % (SIZE as u8);
        self.rounds[self.state as usize] == 1
    }

    fn reset(&mut self) {
        let mut rng = rand::thread_rng();

        self.rounds = [0; SIZE];
        self.state = (SIZE as u8) - 1;

        let i = rng.gen_range(0..SIZE);
        self.rounds[i] = 1;
    }
}

impl<'a> Node<'a> for Roulette {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" {

            let body = revent.content["body"].as_str().unwrap();

            if (self.level == RouletteLevel::Ban && body.starts_with("rroulette")) ||
               body.starts_with("roulette") {
                println!("Found roulette state: {}, rounds: {:?}", self.state, self.rounds);

                match self.fire() {
                    true => {
                        self.reset();
                        match &self.level {
                            Kick => bot.kick(event.room_id, &revent.sender, Some("Bang!")),
                            Ban => bot.ban(event.room_id, &revent.sender, Some("Bang!")),
                        }
                        bot.reply(&event, "bang!")
                    },
                    false => bot.reply(&event, "click"),
                };
            }
        }
    }

    fn description(&self) -> Option<String> {
        match &self.level {
            Kick => Some("roulette - Six chambers; don't mostly die.".to_string()),
            Ban => Some("rroulette - Six chambers; don't die.".to_string()),
        }
    }
}
