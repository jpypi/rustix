use chrono::offset::Local;
use chrono::DateTime;

use crate::bot::{Bot, Node, RoomEvent};
use super::backend::Backend;
use super::models::{Quote, User};


pub struct ReadQuote {
    quote_db: Backend,
}

impl ReadQuote {
    pub fn new() -> Self {
        Self {
            quote_db: Backend::new()
        }
    }
}

impl<'a> Node<'a> for ReadQuote {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        let mut resp: Option<String> = None;

        if body.starts_with("addquote ") {
            if let Ok(qid) = self.quote_db.add_quote(&revent.sender, &body[9..]) {
                resp = Some(format!("Successfully added quote #{}!", qid));
            } else {
                resp = Some("Failed to add quote.".to_string());
            }
        }

        else if body.starts_with("getquote ") {
            resp = Some(match body[9..].parse() {
                Ok(qid) => {
                    match self.quote_db.get_quote(qid) {
                        Ok((quoter, quote)) => render_quote(&quote, &quoter),
                        Err(_) => "No quote by that id was found".to_string()
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            });
        }

        else if body.starts_with("searchquote ") {
            resp = Some(match self.quote_db.search_quote(&body[12..]) {
                Ok((quoter, quote)) => render_quote(&quote, &quoter),
                Err(_) => format!("No quote found matching \"{}\"", &body[12..]),
            });
        }

        else if body.starts_with("randquote") {
            resp = Some(match self.quote_db.random_quote() {
                Ok((quoter, quote)) => render_quote(&quote, &quoter),
                Err(_) => "No quote found.".to_string(),
            });
        }

        resp.map(|s| bot.reply(&event, &s));
    }
}


fn render_quote(quote: &Quote, quoter: &User) -> String {
    let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.value, quote.id, quoter.user_id,
            datetime.format("on %Y-%m-%d at %R"))
}
