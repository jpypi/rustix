use rand::Rng;

use crate::bot::{Bot, Node, RoomEvent};

pub struct Roll {}

impl Roll {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Node<'a> for Roll {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if let Some(raw_n) = event.body().unwrap().strip_prefix("roll ") {
            let n: u32 = match raw_n.parse() {
                Ok(v) => v,
                Err(e) => {
                    bot.reply(&event, &format!("{}", e)).ok();
                    return
                },
            };
            let mut rng = rand::thread_rng();
            let value = rng.gen_range(1..=n);
            bot.reply(&event, &format!("Roll {}: {}", n, value)).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("roll <integer> - Rolls a dice from 1 to specified integer.".to_string())
    }
}
