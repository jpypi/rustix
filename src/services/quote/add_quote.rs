use chrono::offset::Local;
use chrono::DateTime;

use ::bot::{Bot, Node, RoomEvent};
use super::backend::Backend;

pub struct AddQuote {
    quote_db: Backend,
}

impl AddQuote {
    pub fn new() -> Self {
        Self {
            quote_db: Backend::new()
        }
    }
}

impl<'a> Node<'a> for AddQuote {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if revent.type_ == "m.room.message" &&
            revent.content["msgtype"] == "m.text" {
            let body = revent.content["body"].as_str().unwrap();

            if body.starts_with("addquote ") {
                let qid = self.quote_db.add_quote(&revent.sender, &body[9..]);
                let response = format!("Successfully added quote #{}!", qid);
                bot.reply(&event, &response);
            }

            if body.starts_with("getquote ") {
                let qid: i32 = (body[9..]).parse().unwrap();
                if let Some((quoter, quote)) = self.quote_db.get_quote(qid) {
                    let datetime: DateTime<Local> = quote.time.into();

                    let response = format!("\"{}\" set by {} {}",
                                           quote.value,
                                           quoter.user_id,
                                           datetime.format("on %Y-%m-%d at %T"));
                    bot.reply(&event, &response);
                } else {
                    bot.reply(&event, "No quote by that id was found");
                }
            }
        }
    }
}
