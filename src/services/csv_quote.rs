use std::error::Error;

use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;
use toml::value::Value;

use crate::bot::{Bot, Node, RoomEvent};
use crate::services::utils::{reservoir_sample, AliasStripPrefix};


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
        let filename = config.get("file")
                             .and_then(|d| d.as_str())
                             .map(|s| s.to_string())
                             .unwrap_or("csv_quotes.csv".to_string());

        Self {
            rng: SmallRng::from_entropy(),
            filename,
        }
    }

    fn get_quote(&self, id: i32) -> Result<Option<OldQuote>, Box<dyn Error>>{
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

    fn search_quote(&mut self, sub_str: &str) -> Result<Option<OldQuote>, Box<dyn Error>>{
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

    fn search_quotes(&mut self, sub_str: &str) -> Result<Vec<i32>, Box<dyn Error>>{
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

        let mut resp: Option<String> = None;

        if let Some(id) = body.alias_strip_prefix(&["oldgetquote ", "ogq "]) {
            resp = Some(match id.parse() {
                Ok(qid) => {
                    match self.get_quote(qid) {
                        Ok(v) => match v {
                            Some(s) => render_quote(&s),
                            None => format!("No quote found with id {}", qid)
                        },
                        Err(e) => e.to_string(),
                    }
                },
                Err(_) => "Invalid quote id".to_string(),
            });
        } else if let Some(mut query) = body.alias_strip_prefix(&["oldsearchquote ", "osq "]) {
            query = query.trim();

            if !query.is_empty() {
                resp = Some(match self.search_quotes(query) {
                    Ok(quote_ids) => {
                        if !quote_ids.is_empty() {
                            let ids_str = quote_ids.into_iter()
                                                   .map(|id| id.to_string())
                                                   .collect::<Vec<String>>()
                                                   .join(", ");
                            format!("Matching quotes: {}", ids_str)
                        } else {
                            format!("No quotes found matching \"{}\"", query)
                        }
                    },
                    Err(e) => e.to_string(),
                });
            } else {
                resp = Some("oldsearchquote requires search terms".to_string());
            }
        } else if let Some(mut query) = body.alias_strip_prefix(&["oldrandquote", "orq"]) {
            query = query.trim();

            if !query.is_empty() {
                resp = Some(match self.search_quote(query) {
                    Ok(v) => match v {
                        Some(s) => render_quote(&s),
                        None => format!("No quote found matching {}", query)
                    },
                    Err(e) => e.to_string(),
                });
            } else {
                resp = Some(match self.rand_quote() {
                    Ok(v) => match v {
                        Some(s) => render_quote(&s),
                        None => "No quote found".to_string(),
                    },
                    Err(e) => e.to_string(),
                });
            }
        }

        resp.map(|s| bot.reply(&event, &s));
    }

    fn description(&self) -> Option<String> {
        Some("oldgetquote (ogq) - Get a specific quote by id. Pass a valid integer quote id as the only argument.\n\
              oldsearchquote (osq) - Performs string search using provided argument (may contain spaces) and returns all quote ids.\n\
              oldrandquote (orq) - Returns a random quote. Random quotes can be filtered by a string search using an optional provided argument.".to_string())
    }
}


fn render_quote(quote: &OldQuote) -> String {
    //let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.text, quote.id, quote.user,
            quote.timestamp)
}
