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
            level,
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
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap();

            if (self.level == RouletteLevel::Ban && body.starts_with("rroulette")) ||
               (self.level == RouletteLevel::Kick && body.starts_with("roulette")) {
                println!("Found roulette state: {}, rounds: {:?}", self.state, self.rounds);

                match self.fire() {
                    true => {
                        self.reset();
                        match &self.level {
                            RouletteLevel::Kick => bot.kick(event.room_id, &revent.sender, Some("Bang!")),
                            RouletteLevel::Ban => bot.ban(event.room_id, &revent.sender, Some("Bang!")),
                        }.ok();
                        bot.reply(&event, "Bang!").ok()
                    },
                    false => bot.reply(&event, "Click.").ok(),
                };
            }
        }
    }

    fn description(&self) -> Option<String> {
        match &self.level {
            RouletteLevel::Kick => Some("roulette - Six chambers; don't mostly die.".to_string()),
            RouletteLevel::Ban => Some("rroulette - Six chambers; don't die.".to_string()),
        }
    }
}
