use crate::bot::{Bot, Node, RoomEvent};
use super::backend::Backend;


pub struct DelQuote {
    quote_db: Backend,
}

impl DelQuote {
    pub fn new() -> Self {
        Self {
            quote_db: Backend::new()
        }
    }
}

impl<'a> Node<'a> for DelQuote {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();

        let mut resp: Option<String> = None;

        if body.starts_with("delquote ") {
            resp = Some(match body[9..].parse() {
                Ok(qid) => {
                    if let Ok(_) = self.quote_db.del_quote(qid) {
                        format!("Successfully deleted quote {}", qid)
                    } else {
                        format!("Failed to delete quote {}", qid)
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            });
        }

        resp.map(|s| bot.reply(&event, &s));
    }
}
