use std::error::Error;

use rand::{SeedableRng, Rng};
use rand::rngs::SmallRng;

use crate::bot::{Bot, Node, RoomEvent};
use crate::services::utils;


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
        match utils::reservoir_sample(reader.deserialize(), &mut self.rng) {
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
        }

        else if body.starts_with("oldsearchquote ") {
            let query = &body[15..];

            resp = Some(match self.search_quote(query) {
                Ok(v) => match v {
                    Some(s) => render_quote(&s),
                    None => format!("No quote found matching {}", query)
                },
                Err(e) => e.to_string(),
            });
        }

        else if body.starts_with("oldrandquote") {
            resp = Some(match self.rand_quote() {
                Ok(v) => match v {
                    Some(s) => render_quote(&s),
                    None => "No random quote found".to_string(),
                },
                Err(e) => e.to_string(),
            });
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
