use http::Method;
use regex::Regex;
use reqwest;

use crate::bot::{Bot, Node, RoomEvent};

pub struct CryptoCoin {}

impl CryptoCoin {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'a> Node<'a> for CryptoCoin {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if event.is_normal() {
            let body = &revent.content["body"].as_str().unwrap();

            let re = Regex::new(r"^p +([A-Za-z]{2,4}) *$").unwrap();

            if let Some(m) = re.captures(body) {
                let ticker = m.get(1).unwrap().as_str().to_uppercase();
                bot.reply(&event, &price_string(&ticker)).ok();
            }
        }
    }

    fn description(&self) -> Option<String> {
        Some("p <ticker> - Get the price of any cryptocurrency traded on bitfinex.".to_string())
    }
}

fn price_string(sym: &str) -> String {
    match get_ticker(&sym[..3]) {
        Some(values) => {
            format!(
                "{} - Last Price: ${}, Bid/Ask: ${}/{}, Volume: {}",
                sym,
                values[TickerValue::LastPrice.value()],
                values[TickerValue::Bid.value()],
                values[TickerValue::Ask.value()],
                values[TickerValue::Volume.value()]
            )
        }
        None => format!("Could not fetch market data for: {}", sym),
    }
}

fn get_ticker(sym: &str) -> Option<Vec<f32>> {
    let url = format!("https://api.bitfinex.com/v2/ticker/t{}USD", sym);

    let client = reqwest::blocking::Client::new();
    match client.request(Method::GET, &url).send() {
        Ok(resp) => match resp.json() {
            Ok(v) => Some(v),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

#[allow(dead_code)]
#[repr(usize)]
enum TickerValue {
    Bid,             // float  Price of last highest bid
    BidSize,         // float  Size of the last highest bid
    Ask,             // float  Price of last lowest ask
    AskSize,         // float  Size of the last lowest ask
    DailyChange,     // float  Amount that the last price has changed since yesterday
    DailyChangePerc, // float  Amount that the price has changed expressed in percentage terms
    LastPrice,       // float  Price of the last trade
    Volume,          // float  Daily volume
    High,            // float  Daily high
    Low,             // float  Daily low
}

impl TickerValue {
    fn value(self) -> usize {
        self as usize
    }
}
