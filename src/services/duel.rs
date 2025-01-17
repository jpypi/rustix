use std::collections::HashMap;

use rand::Rng;

use crate::{config::RemovalMode, bot::{Bot, Node, RoomEvent}};

pub struct Duel {
    duels: HashMap<String, String>,
    mode: RemovalMode,
}

impl Duel {
    pub fn new(mode: RemovalMode) -> Self {
        Self {
            duels: HashMap::new(),
            mode,
        }
    }
}

impl<'a> Node<'a> for Duel {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        if event.is_normal() {
            let revent = &event.raw_event;
            let body = revent.content["body"].as_str().unwrap();

            if (self.mode == RemovalMode::Ban && body.starts_with("dduel")) ||
               (self.mode == RemovalMode::Kick && body.starts_with("duel")) {

                match self.duels.get(event.room_id) {
                    Some(d) => {
                        let mut rng = rand::thread_rng();
                        let loser = match rng.gen_bool(0.55) {
                            true => d,
                            false => &revent.sender,
                        };

                        match &self.mode {
                            RemovalMode::Kick => bot.client().kick(event.room_id, loser, Some("Bang!")),
                            RemovalMode::Ban =>  bot.client().ban(event.room_id, loser, Some("Bang!")),
                        }.ok();

                        bot.reply(&event, "Bang!").ok();
                        println!("{} lost the duel", loser);
                        self.duels.remove(event.room_id);
                    },
                    None => {
                        self.duels.insert(event.room_id.to_string(), revent.sender.clone());
                        bot.reply(&event, "Quick! Someone duel 'em!").ok();
                    },
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        match &self.mode {
            RemovalMode::Kick => Some("duel - Challenge someone to a duel of boots".to_string()),
            RemovalMode::Ban => Some("dduel - Challenge someone to a duel to the banishment".to_string()),
        }
    }
}
