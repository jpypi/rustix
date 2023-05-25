use reqwest;
use toml::Value;

use crate::bot::{Bot, Node, RoomEvent};


const BASE_URL: &str = "https://www.googleapis.com/customsearch/v1";


#[derive(Deserialize, Debug)]
struct SearchResults {
    items: Option<Vec<ResultItem>>,
    spelling: Option<Spelling>,
}

#[derive(Deserialize, Debug)]
struct ResultItem {
    title: String,
    link: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Spelling {
    correctedQuery: String,
}

#[derive(Deserialize)]
pub struct WebSearch {
    key: String,
    seid: String,
}

impl WebSearch {
    pub fn new(config: &Value) -> Self {
        config.clone().try_into().expect("")
    }

    fn search(&self, query: &str) -> Result<SearchResults, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let query = client.get(BASE_URL)
                          .query(&[("key", self.key.as_str()),
                                   ("cx", self.seid.as_str()),
                                   ("q", query)])
                          .header("Accept", "application/json");

        query.send().and_then(|o| o.json())
    }
}

impl<'a> Node<'a> for WebSearch {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if event.is_normal() {
            let body = &revent.content["body"].as_str().unwrap();

            if let Some(query) = body.strip_prefix("s ") {
                let mut res = self.search(query);

                let mut tries: u32 = 0;
                while res.is_ok() && tries < 2 {
                    if let Ok(sr) = res.as_ref() {
                        // Might need to re-search due to spelling issue
                        if sr.items.is_none() && sr.spelling.is_some() {
                            res = self.search(&sr.spelling.as_ref().unwrap().correctedQuery);
                            tries += 1;
                            continue;
                        }

                        if let Some(f) = sr.items.as_ref().and_then(|i| i.first()) {
                            bot.reply(&event, &format!("{} - {}", f.title, f.link)).ok();
                            return;
                        }
                    }
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("s <query> - Search the internet for something.".to_string())
    }
}