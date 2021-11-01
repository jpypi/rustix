use std::error::Error;

use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;

use crate::bot::{Bot, Node, RoomEvent};
use crate::services::utils::reservoir_sample;


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
}

impl ReadQuote {
    pub fn new() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }

    fn get_quote(&self, id: i32) -> Result<Option<OldQuote>, Box<dyn Error>>{
        let mut reader = csv::Reader::from_path("gb_quotes_all.csv")?;
        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if record.id == id {
                return Ok(Some(record));
            }
        }
        Ok(None)
    }

    fn rand_quote(&mut self) -> Result<Option<OldQuote>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path("gb_quotes_all.csv")?;
        match reservoir_sample(reader.deserialize(), &mut self.rng) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn search_quote(&mut self, sub_str: &str) -> Result<Option<OldQuote>, Box<dyn Error>>{
        let mut reader = csv::Reader::from_path("gb_quotes_all.csv")?;

        let mut quotes = Vec::new();

        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if let Some(_) = record.text.find(sub_str) {
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
        let mut reader = csv::Reader::from_path("gb_quotes_all.csv")?;

        let mut quotes = Vec::new();

        for result in reader.deserialize() {
            let record: OldQuote = result?;
            if let Some(_) = record.text.find(sub_str) {
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

        if body.starts_with("oldgetquote ") {
            resp = Some(match body[12..].parse() {
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
        } else if body.starts_with("oldsearchquote ") {
            let query = body[15..].trim();

            if query.len() > 0 {
                resp = Some(match self.search_quotes(&query) {
                    Ok(quote_ids) => {
                        if quote_ids.len() > 0 {
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
        } else if body.starts_with("oldrandquote") {
            let query = body[12..].trim();

            if query.len() > 0 {
                resp = Some(match self.search_quote(&query) {
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
}


fn render_quote(quote: &OldQuote) -> String {
    //let datetime: DateTime<Local> = quote.time.into();
    format!("{}\n{} set by {} {}",
            quote.text, quote.id, quote.user,
            quote.timestamp)
}
