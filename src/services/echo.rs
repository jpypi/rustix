use crate::bot::{Bot, Node, RoomEvent};

pub struct Echo<'a> {
    children: Vec<&'a str>,
}

impl<'a> Echo<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl<'a> Node<'a> for Echo<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        Some(&self.children)
    }

    fn register_child(&mut self, name: &'a str) {
        self.children.push(name);
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;

        if event.is_normal() {
            let body = &revent.content["body"].as_str().unwrap();
            if body.starts_with("echo ") {
                bot.reply(&event, &body[5..]).ok();
            }
        }

        self.propagate_event(bot, &event);
    }

    fn description(&self) -> Option<String> {
        Some("echo - Replys with the argument passed.".to_string())
    }
}
