use rand;
use rand::Rng;

use bot::{Bot, Node, RoomEvent};

const SIZE:usize = 6;

pub struct Roulette {
    rounds: [u8; SIZE],
    state: u8,
}

impl Roulette {
    pub fn new() -> Self {
        let mut x = Self {
            rounds: [0; SIZE],
            state: 0,
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

        let i = rng.gen_range(0, SIZE);
        self.rounds[i] = 1;
    }
}

impl<'a> Node<'a> for Roulette {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text" {

            let body = revent.content["body"].as_str().unwrap();

            if body.starts_with("roulette") {
                println!("Found roulette state: {}, rounds: {:?}", self.state, self.rounds);

                match self.fire() {
                    true => {
                        self.reset();
                        bot.kick(event.room_id, &revent.sender, Some("Bang!"));
                        bot.reply(&event, "bang!")
                    },
                    false => bot.reply(&event, "click"),
                };
            }
        }
    }
}
