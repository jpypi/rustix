use crate::{bot::Node, utils::codeblock_format};

use super::backend::Backend;

pub struct ListAllFactoid {
    backend: Backend,
}

impl ListAllFactoid {
    pub fn new() -> Self {
        Self {
            backend: Backend::new(),
        }
    }
}

impl<'a> Node<'a> for ListAllFactoid {
    fn handle(&mut self, bot: &crate::bot::Bot, event: crate::bot::RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        if body.starts_with("allfactoids") {
            let factoids = self.backend.all_factoids();

            let mut response = vec![format!(
                "{:>4} - {:^34} - {:^8}: {}",
                "id", "user", "kind", "factoid"
            )];

            for f in factoids {
                let user = self.backend.get_user(f.user_id);
                response.push(format!(
                    "{:>4} - {:>34} - {:^8}: {}",
                    f.id, user.user_id, f.kind, f.value
                ));
            }

            // If the only row is the header row then the length will be 1
            if response.len() > 1 {
                let raw = response.join("\n");
                let message = codeblock_format(&raw);
                bot.reply_fmt(&event, &message, &raw).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("allfactoids - List all factoids.".to_string())
    }
}
