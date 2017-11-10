use rand;
use rand::Rng;
use bot::{Bot, Node, RoomEvent};

pub struct Choose {
}

impl Choose {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl<'a> Node<'a> for Choose {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if revent.type_ == "m.room.message" &&
           revent.content["msgtype"] == "m.text"{
            let body = revent.content["body"].as_str().unwrap();
            if body.starts_with("choose ") {

                let choices: Vec<&str> = body[7..].split_whitespace().collect();

                let mut rng = rand::thread_rng();
                let n = rng.gen_range(0, choices.len());
                bot.reply(&event, choices[n]);
            }
        }
    }
}
