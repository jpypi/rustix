use std::{
    io::BufWriter,
    thread
};

use bf::lang::run as run_bf;

use crate::{bot::{Bot, Node, RoomEvent}, utils::codeblock_format};


pub struct BFLang {
    cycle_limit: u32,
    debug: bool,
}

impl BFLang {
    pub fn new(cycle_limit: u32, debug: bool) -> Self {
        Self {
            cycle_limit,
            debug,
        }
    }
}

impl Default for BFLang {
    fn default() -> Self {
        Self {
            cycle_limit: 5000,
            debug: false,
        }
    }
}

impl<'a> Node<'a> for BFLang {
    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        let revent = &event.raw_event;
        if event.is_normal() {
            let body = revent.content["body"].as_str().unwrap();
            if let Some(code) = body.strip_prefix("bf ") {
                let t_code = code.to_owned();
                let t_timeout = self.cycle_limit;
                let t_debug = self.debug;
                let t_room_id = event.room_id.to_string();
                let t_client = bot.arc_client();
                thread::spawn(move || {
                    let mut out = BufWriter::new(Vec::new());
                    let r = run_bf(&t_code, String::new().as_bytes(), &mut out, t_debug, Some(t_timeout));
                    match r {
                        Err(e) => {
                            let mut client = t_client.write().unwrap();
                            let raw = e.trim_start_matches('\n');
                            let message = codeblock_format(raw);
                            client.send_msg_fmt(&t_room_id, &message, raw).ok();
                        },
                        Ok(_) => {
                            if let Ok(s) = String::from_utf8(out.buffer().to_vec()) {
                                let mut client = t_client.write().unwrap();
                                client.send_msg(&t_room_id, &s).ok();
                            }
                        },
                    };
                });
            }
        }
    }

    fn configure(&mut self, bot: &Bot, command: &str, event: RoomEvent) {
        if let Some(cycles) = command.strip_prefix("cycles ")  {
            match cycles.parse() {
                Ok(value) => self.cycle_limit = value,
                Err(msg) => { bot.reply(&event, &format!("Error: Could not set cycle limit - {}", msg)).ok(); }
            };
        } else if let Some(mode_args) = command.strip_prefix("debug ")  {
            match mode_args {
                "on" => self.debug = true,
                "off" => self.debug = false,
                x => { bot.reply(&event, &format!("Invalid mode passed to mode: {}", x)).ok(); },
            }
        } else if command.starts_with("status") {
            bot.reply(&event, &format!("cycle limit: {}", self.cycle_limit)).ok();
        }
    }

    fn configure_description(&self) -> Option<String> {
        Some("cycles <int>    - Set the cycle limit of the executor.\n\
              debug  <on|off> - Turn debug output on or off.\n\
              status          - View the current configuration of the node.".to_string())
    }

    fn description(&self) -> Option<String> {
        Some("bf <code> - Interpret brainfuck code.".to_string())
    }
}