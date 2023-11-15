use chrono::offset::Local;
use chrono::DateTime;
use itertools::Itertools;

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

        if let Some(quote) = body.alias_strip_prefix(&["addquote ", "aq ", "quote ", "q "]) {
            bot.reply(&event, &match self.quote_db.add_quote(&revent.sender, quote) {
                Ok(qid) => format!("Successfully added quote #{qid}!"),
                Err(_) => "Failed to add quote.".to_string(),
            }).ok();
        } else if let Some(ids) = body.alias_strip_prefix(&["getquote ", "gq "]) {
            for (i, (orig, id)) in ids.split(",").map(|s| (s, s.trim().parse())).enumerate() {
                // Limit the max number of quotes to get at a time to 5
                if i > 4 {
                    break;
                }
                bot.reply(&event, &match id {
                    Ok(qid) => {
                        match self.quote_db.get_quote(qid) {
                            Ok((quoter, quote)) => render_quote(&quote, &quoter),
                            Err(_) => format!("No quote found with id {qid}"),
                        }
                    },
                   Err(e) => format!("Invalid quote id: '{orig}' - {e}"),
                }).ok();
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["searchquote ", "sq "]) {
            query = query.trim();

            if !query.is_empty() {
                bot.reply(&event, &match self.quote_db.search_quotes(query) {
                    Ok(quotes) => {
                        let n_quotes = quotes.len();
                        if n_quotes > 0 {
                            let quote_ids = quotes.into_iter()
                                                  .map(|q| q.id.to_string())
                                                  .join(", ");
                            format!("Found {n_quotes} quotes: {quote_ids}")
                        } else {
                            format!("No quotes found matching \"{query}\"")
                        }
                    },
                    Err(_) => format!("Error while looking for quote matching \"{query}\""),
                }).ok();
            } else {
                bot.reply(&event, "searchquote requires search terms").ok();
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["randquote", "rq"]) {
            query = query.trim();

            bot.reply(&event, &match query.is_empty() {
                true => match self.quote_db.random_quote() {
                            Ok((quoter, quote)) => render_quote(&quote, &quoter),
                            Err(_) => "No quote found.".to_string(),
                        }
                false => match self.quote_db.search_quote(query) {
                             Ok((quoter, quote)) => render_quote(&quote, &quoter),
                             Err(_) => format!("No quote found matching {query}"),
                         },
            }).ok();
        } else if let Some(mut query) = body.alias_strip_prefix(&["quoteby", "qb"]) {
            query = query.trim();

            bot.reply(&event, &match query.is_empty() {
                true => "Please specify a user".to_string(),
                false => match bot.uid_from_displayname(query) {
                    Ok(uid) => {
                        if let Ok((quoter, quote)) = self.quote_db.quote_by(&uid) {
                            render_quote(&quote, &quoter)
                        } else {
                            "No quotes found.".to_string()
                        }
                    },
                    Err(_) => "Unable to identify user.".to_string(),
                },
            }).ok();
        }
    }

    fn description(&self) -> Option<String> {
        Some("quotes:\n\
              \taddquote (alt: quote, aq, q) <quote> - Add a quote to the database. (Please format as: <nick> phrase<newline><othernick> phrase)\n\
              \tgetquote (alt: gq) <quote id 0>, <quote id 1> - Get up to 5 specific quotes by providing valid integer quote ids.\n\
              \tsearchquote (alt: sq) <search string> - Performs string search across quotes and returns all quote ids.\n\
              \trandquote (alt: rq) <optional search string> - Returns a random quote, optionally from the set of quotes which match a given query.\n\
              \tquoteby (alt: qb) <user nickname> - Returns a random quote set by the given user.".to_string())
    }
}


fn render_quote(quote: &Quote, quoter: &User) -> String {
    let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.value, quote.id, quoter.user_id,
            datetime.format("on %Y-%m-%d at %R"))
}
