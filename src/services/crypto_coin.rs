use std::io::Read;

use serde_json;
use reqwest;
use reqwest::Method;
use regex::Regex;

use ::bot::{Bot, Node, RoomEvent};


pub struct CryptoCoin {
}

impl CryptoCoin {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl<'a> Node<'a> for CryptoCoin {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if revent.type_ == "m.room.message" && revent.content["msgtype"] == "m.text" {
            let body = &revent.content["body"].as_str().unwrap();

            let re = Regex::new(r"^p +([A-Za-z]{2,4}) *$").unwrap();

            if let Some(m) = re.captures(body) {
                let ticker = m.get(1).unwrap().as_str().to_uppercase();
                bot.reply(&event, &price_string(&ticker));
            }
        }
    }
}


fn price_string(sym: &str) -> String {
    let spot_price = price(sym, PriceType::Spot);
    if spot_price < 0.0 {
        format!("Could not fetch market data for: {}", sym)
    } else {
        let bid_price = price(sym, PriceType::Bid);
        let ask_price = price(sym, PriceType::Ask);

        format!("{} - Spot: ${}, Bid/Ask: ${}/{}",
                sym, spot_price, bid_price, ask_price)
    }
}


fn price(sym: &str, price_type: PriceType) -> f32 {
    let url = format!("https://api.coinbase.com/v2/prices/{}-USD/{}",
                      sym, price_type.value());

    let client = reqwest::Client::new().unwrap();
    match client.request(Method::Get, &url).unwrap().send() {
        Ok(mut resp) => {
            let mut content = String::new();
            resp.read_to_string(&mut content).unwrap();
            match serde_json::from_str::<CBResponse>(&content) {
                Ok(v) => {
                    v.data.amount.parse().unwrap()
                },
                Err(_) => -1.0
            }
        },
        Err(_) => -1.0
    }
}


#[allow(dead_code)]
enum PriceType {
    Spot,
    Bid,
    Ask,
}

impl PriceType {
    fn value(&self) -> &str {
        match *self {
            PriceType::Spot => "spot",
            PriceType::Bid  => "sell",
            PriceType::Ask  => "buy",
        }
    }
}


// These types are used for deserializing the Coin Base API

#[derive(Serialize, Deserialize, Debug)]
struct CBResponse {
    data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    amount: String,
    currency: String,
}
