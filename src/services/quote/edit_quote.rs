use regex::Regex;
use crate::bot::{Bot, Node, RoomEvent};
use super::backend::Backend;


pub struct EditQuote {
    quote_db: Backend,
    edit_re: Regex,
}

impl EditQuote {
    pub fn new() -> Self {
        Self {
            quote_db: Backend::new(),
            edit_re: Regex::new(r"^(?:editquote|eq) (\d+) (.+)$").unwrap(),
        }
    }
}

impl<'a> Node<'a> for EditQuote {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let body = &event.raw_event.content["body"].as_str().unwrap();

        if let Some(captures) = self.edit_re.captures(body) {
            let qid = captures.get(1).unwrap().as_str();
            let new_text = captures.get(2).unwrap().as_str();

            let resp = match qid.parse() {
                Ok(qid) => {
                    if self.quote_db.update_quote(qid, new_text).is_ok() {
                        format!("Successfully updated quote {}", qid)
                    } else {
                        format!("Failed to update quote {}", qid)
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            };

            bot.reply(&event, &resp).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("editquote (alt: eq) <quote id> <updated quote text> - Update text of uote with given id.".to_string())
    }
}
