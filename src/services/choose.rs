use rand::seq::IteratorRandom;

use crate::bot::{Bot, Node, RoomEvent};

pub struct Choose {}

impl Choose {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Node<'a> for Choose {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();
        if let Some(raw_choices) = body.strip_prefix("choose ") {
            let mut rng = rand::thread_rng();
            if let Some(choice) = raw_choices.split(',').map(|c| c.trim()).choose(&mut rng) {
                bot.reply(&event, choice).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("choose <optional item 1>, <optional item 2>, <optional item N> - Randomly selects from a comma separated list of items.".to_string())
    }
}
