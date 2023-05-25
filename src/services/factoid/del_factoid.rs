use crate::bot::Node;

use super::backend::Backend;

pub struct DelFactoid {
    backend: Backend,
}

impl DelFactoid {
    pub fn new() -> Self {
        Self {
            backend: Backend::new(),
        }
    }
}

impl<'a> Node<'a> for DelFactoid {
    fn handle(&mut self, bot: &crate::bot::Bot, event: crate::bot::RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        if let Some(raw_fid) = body.strip_prefix("delfactoid ") {
            let response = match raw_fid.parse() {
                Ok(fid) => {
                    if self.backend.del_factoid(fid).is_ok() {
                        format!("Successfully deleted factoid {}", fid)
                    } else {
                        format!("Failed to delete factoid {}", fid)
                    }
                }
                Err(_) => "Invalid factoid id".to_string(),
            };

            bot.reply(&event, &response).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("delfactoid <id> - Delete a factoid by id.".to_string())
    }
}
