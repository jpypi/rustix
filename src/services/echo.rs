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
        if event.is_normal() {
            if let Some(content) = event.body().unwrap().strip_prefix("echo ") {
                bot.reply(&event, content).ok();
            }
        }

        self.propagate_event(bot, &event);
    }

    fn description(&self) -> Option<String> {
        Some("echo <any message> - Replys with the argument passed.".to_string())
    }
}
