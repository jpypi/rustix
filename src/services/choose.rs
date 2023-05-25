use rand::Rng;

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
            let choices: Vec<&str> = raw_choices.split_whitespace().collect();

            let mut rng = rand::thread_rng();
            let n = rng.gen_range(0..choices.len());
            bot.reply(&event, choices[n]).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("choose <optional item 1> <optional item 2> <optional item N> - Randomly selects from a space separated list of items.".to_string())
    }
}
