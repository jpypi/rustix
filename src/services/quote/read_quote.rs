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
        } else if body.starts_with("getquote ") {
            resp = Some(match body[9..].parse() {
                Ok(qid) => {
                    match self.quote_db.get_quote(qid) {
                        Ok((quoter, quote)) => render_quote(&quote, &quoter),
                        Err(_) => "No quote by that id was found".to_string()
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            });
        } else if body.starts_with("searchquote ") {
            let query = body[12..].trim();

            if query.len() > 0 {
                resp = Some(match self.quote_db.search_quotes(query) {
                    Ok(quotes) => {
                        if quotes.len() > 0 {
                            let quot_ids = quotes.into_iter()
                                                 .map(|q| q.id.to_string())
                                                 .collect::<Vec<String>>()
                                                 .join(", ");
                            format!("Matching quotes: {}", quot_ids)
                        } else {
                            format!("No quotes found matching \"{}\"", query)
                        }
                    },
                    Err(_) => format!("Error while looking for quote matching \"{}\"", query),
                });
            } else {
                resp = Some("searchquote requires search terms".to_string());
            }
        } else if body.starts_with("randquote") {
            let query = body[9..].trim();

            if query.len() > 0 {
                resp = Some(match self.quote_db.search_quote(&query) {
                    Ok((quoter, quote)) => render_quote(&quote, &quoter),
                    Err(_) => format!("No quote found matching {}", query),
                });
            } else {
                resp = Some(match self.quote_db.random_quote() {
                    Ok((quoter, quote)) => render_quote(&quote, &quoter),
                    Err(_) => "No quote found.".to_string(),
                });
            }

        }

        resp.map(|s| bot.reply(&event, &s));
    }

    fn description(&self) -> Option<String> {
        Some("addquote - Add a quote to the database. (Please format as: <nick> phrase)\n\
              getquote - Get a specific quote by id. Pass a valid integer quote id as the only argument.\n\
              searchquote - Performs string search using provided argument (may contain spaces) and returns all quote ids.\n\
              randquote - Returns a random quote. Random quotes can be filtered by a string search using an optional provided argument.".to_string())
    }
}


fn render_quote(quote: &Quote, quoter: &User) -> String {
    let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.value, quote.id, quoter.user_id,
            datetime.format("on %Y-%m-%d at %R"))
}
