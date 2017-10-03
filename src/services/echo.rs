use ::bot::{Bot, Node, RoomEvent};

pub struct Echo<'a> {
    children: Vec<&'a str>
}

impl<'a> Echo<'a> {
    pub fn new() -> Self {
        Self {
            children: Vec::new()
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
        let r = "!tEQUhDXnBDAeqCAgJk:cclub.cs.wmich.edu";
        let revent = event.raw_event;

        if revent.type_ == "m.room.message" && revent.content["msgtype"] == "m.text" {
            let body = &revent.content["body"].as_str().unwrap();
            let sender = &revent.sender;
            if body.starts_with("echo ") {
                bot.reply(&event, "HEY I'M MR. MESEEKS LOOK AT ME!");
            }

            println!("<{}> | {}", sender, body);
        }

        self.propagate_event(bot, event);
    }
}
