use std::error::Error;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use toml::value::Value;

use crate::bot::{Bot, Node, RoomEvent};
use crate::utils::{reservoir_sample, AliasStripPrefix};

#[derive(Deserialize, Debug, Default, Clone, Eq, PartialEq)]
struct OldQuote {
    pub id: i32,
    pub text: String,
    pub user: String,
    pub timestamp: String,
    pub channel: String,
}

pub struct ReadQuote {
    rng: SmallRng,
    filename: String,
}

impl ReadQuote {
    pub fn new(config: &Value) -> Self {
        let filename = config
            .get("file")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string())
            .unwrap_or("csv_quotes.csv".to_string());

        Self {
            rng: SmallRng::from_entropy(),
            filename,
        }
    }

    fn get_quote(&self, id: i32) -> Result<Option<OldQuote>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(&self.filename)?;
        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if record.id == id {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    fn rand_quote(&mut self) -> Result<Option<OldQuote>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(&self.filename)?;
        match reservoir_sample(reader.deserialize(), &mut self.rng) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn search_quote(&mut self, sub_str: &str) -> Result<Option<OldQuote>, Box<dyn Error>> {
        let query = sub_str.to_lowercase();
        let mut reader = csv::Reader::from_path(&self.filename)?;

        let mut quotes = Vec::new();

        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if record.text.to_lowercase().contains(&query) {
                quotes.push(record);
            }
        }

        if quotes.len() > 0 {
            let index = self.rng.gen_range(0..quotes.len());
            return Ok(Some(quotes[index].clone()));
        }

        Ok(None)
    }

    fn search_quotes(&mut self, sub_str: &str) -> Result<Vec<i32>, Box<dyn Error>> {
        let query = sub_str.to_lowercase();
        let mut reader = csv::Reader::from_path(&self.filename)?;

        let mut quotes = Vec::new();

        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if record.text.to_lowercase().contains(&query) {
                quotes.push(record.id);
            }
        }

        Ok(quotes)
    }
}

impl<'a> Node<'a> for ReadQuote {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        if let Some(ids) = body.alias_strip_prefix(&["oldgetquote ", "ogq "]) {
            for (i, (orig, id)) in ids.split(",").map(|s| (s, s.trim().parse())).enumerate() {
                // Limit the max number of quotes to get at a time to 5
                if i > 4 {
                    break;
                }
                bot.reply(&event, &match id {
                    Ok(qid) => match self.get_quote(qid) {
                        Ok(v) => match v {
                            Some(s) => render_quote(&s),
                            None => format!("No quote found with id {}", qid),
                        },
                        Err(e) => e.to_string(),
                    },
                    Err(e) => format!("Invalid quote id: '{orig}' - {e}"),
                }).ok();
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["oldsearchquote ", "osq "]) {
            query = query.trim();

            if !query.is_empty() {
                bot.reply(&event, &match self.search_quotes(query) {
                    Ok(quote_ids) => {
                        if !quote_ids.is_empty() {
                            let ids = quote_ids.into_iter()
                                               .map(|id| id.to_string())
                                               .collect::<Vec<String>>();
                            format!("Found {} quotes: {}", ids.len(), ids.join(", "))
                        } else {
                            format!("No quotes found matching \"{query}\"")
                        }
                    },
                    Err(_) => format!("Error while looking for quote matching \"{query}\""),
                }).ok();
            } else {
                bot.reply(&event, "oldsearchquote requires search terms").ok();
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["oldrandquote", "orq"]) {
            query = query.trim();

            if !query.is_empty() {
                bot.reply(&event, &match self.search_quote(query) {
                    Ok(v) => match v {
                        Some(s) => render_quote(&s),
                        None => format!("No quote found matching {query}"),
                    },
                    Err(e) => e.to_string(),
                }).ok();
            } else {
                bot.reply(&event, &match self.rand_quote() {
                    Ok(v) => match v {
                        Some(s) => render_quote(&s),
                        None => "No quote found".to_string(),
                    },
                    Err(e) => e.to_string(),
                }).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("oldgetquote (alt: ogq) <quote id 0>, <quote id 1> - Get up to 5 specific quotes by providing valid integer quote ids.\n\
              oldsearchquote (alt: osq) <search string> - Performs string search across quotes and returns all quote ids.\n\
              oldrandquote (alt: orq) <optional search string> - Returns a random quote, optionally from the set of quotes which match a given query.".to_string())
    }
}

fn render_quote(quote: &OldQuote) -> String {
    //let datetime: DateTime<Local> = quote.time.into();
    format!(
        "{}\n{} set by {} {}",
        quote.text, quote.id, quote.user, quote.timestamp
    )
}
