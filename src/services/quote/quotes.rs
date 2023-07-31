use chrono::offset::Local;
use chrono::DateTime;

use crate::bot::{Bot, Node, RoomEvent};
use crate::utils::AliasStripPrefix;
use super::backend::Backend;
use super::models::Quote;
use super::super::db::user::User;


pub struct Quotes {
    quote_db: Backend,
}

impl Quotes {
    pub fn new() -> Self {
        Self {
            quote_db: Backend::new()
        }
    }
}

impl<'a> Node<'a> for Quotes {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        let mut resp: Option<String> = None;

        if let Some(quote) = body.alias_strip_prefix(&["addquote ", "aq ", "quote ", "q "]) {
            if let Ok(qid) = self.quote_db.add_quote(&revent.sender, quote) {
                resp = Some(format!("Successfully added quote #{}!", qid));
            } else {
                resp = Some("Failed to add quote.".to_string());
            }
        } else if let Some(id) = body.alias_strip_prefix(&["getquote ", "gq "]) {
            resp = Some(match id.parse() {
                Ok(qid) => {
                    match self.quote_db.get_quote(qid) {
                        Ok((quoter, quote)) => render_quote(&quote, &quoter),
                        Err(_) => format!("No quote found with id {}", qid),
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            });
        } else if let Some(mut query) = body.alias_strip_prefix(&["searchquote ", "sq "]) {
            query = query.trim();

            if query.len() > 0 {
                resp = Some(match self.quote_db.search_quotes(query) {
                    Ok(quotes) => {
                        let n_quotes = quotes.len();
                        if n_quotes > 0 {
                            let quot_ids = quotes.into_iter()
                                                 .map(|q| q.id.to_string())
                                                 .collect::<Vec<String>>()
                                                 .join(", ");
                            format!("Found {} quotes: {}", n_quotes, quot_ids)
                        } else {
                            format!("No quotes found matching \"{}\"", query)
                        }
                    },
                    Err(_) => format!("Error while looking for quote matching \"{}\"", query),
                });
            } else {
                resp = Some("searchquote requires search terms".to_string());
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["randquote", "rq"]) {
            query = query.trim();

            if !query.is_empty() {
                resp = Some(match self.quote_db.search_quote(query) {
                    Ok((quoter, quote)) => render_quote(&quote, &quoter),
                    Err(_) => format!("No quote found matching {}", query),
                });
            } else {
                resp = Some(match self.quote_db.random_quote() {
                    Ok(Some((quoter, quote))) => render_quote(&quote, &quoter),
                    Ok(None) | Err(_) => "No quote found.".to_string(),
                });
            }
        }

        resp.map(|s| bot.reply(&event, &s));
    }

    fn description(&self) -> Option<String> {
        Some("quotes:\n\
              \taddquote (alt: aq) <quote> - Add a quote to the database. (Please format as: <nick> phrase<newline><othernick> phrase)\n\
              \tgetquote (alt: gq) <quote id> - Get a specific quote by providing a valid integer quote id.\n\
              \tsearchquote (alt: sq) <search string> - Performs string search across quotes and returns all quote ids.\n\
              \trandquote (alt: rq) <optional search string> - Returns a random quote, optionally from the set of quotes which match a given query.".to_string())
    }
}


fn render_quote(quote: &Quote, quoter: &User) -> String {
    let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.value, quote.id, quoter.user_id,
            datetime.format("on %Y-%m-%d at %R"))
}
