use std::collections::HashMap;

use reqwest::{Response};

use errors::*;
use client::MatrixClient;
use matrix_types::Event;

pub struct Bot <'a> {
    client: &'a mut MatrixClient,
    root_services: Vec<&'a str>,
    all_services: HashMap<&'a str, Box<Node>>,

    rooms: Vec<&'a str>,
}

impl <'a> Bot<'a> {
    pub fn new(client: &'a mut MatrixClient) -> Self{
        Bot {
            client,
            root_services: Vec::new(),
            all_services: HashMap::new(),
            rooms: Vec::new(),
        }
    }

    pub fn join(&mut self, room_id: &'a str) {
        self.client.join(room_id);
        self.rooms.push(room_id);
    }

    pub fn say(&mut self, room_id: &str, message: &str) -> Result<Response, RustixError> {
        self.client.send_msg(room_id, message)
    }

    pub fn reply(&self, event: Event, message: &str) {
    }

    pub fn action(&self, room_id: &str, action: &str) {
    }

    pub fn register_service(&mut self, name: &'a str, service: Box<Node>) {
        match service.parent() {
            Some(p) => {
                self.all_services.get_mut(p).unwrap().register_child(name)
            },
            None => self.root_services.push(name),
        };

        self.all_services.insert(name, service);
    }

    pub fn get_service(&mut self, name: &str) -> &mut Box<Node> {
        self.all_services.get_mut(name).unwrap()
    }

    pub fn propagate_event(&self, event: &Event) {
        for service in &self.root_services {
            self.all_services.get(service).unwrap().handle(self, event);
        }
    }

    pub fn run(&mut self) {
        let mut monitor_room;
        match self.client.get_public_room_id("test") {
            Some(room_id) => {
                self.client.join(&room_id);
                println!("Room id: {}", &room_id);
                self.client.send_msg(&room_id, "Hello world from rust!");
                monitor_room = room_id;
            },
            None => panic!(),
        };

        if let Some(rid) = self.client.get_public_room_id("#geeks") {
            self.client.join(&rid);
        }

        let mut next_batch;
        match self.client.sync(None) {
            Ok(res) => next_batch = res.next_batch,
            Err(e) => panic!(e),
        }

        loop {
            if let Ok(res) = self.client.sync(Some(&next_batch)) {
                /*
                if let Ok(x) = serde_json::to_string_pretty(&res.rooms.join) {
                    println!("{}", x);
                }
                */
                match res.rooms.join.get(&monitor_room) {
                    Some(room) => {
                        for event in &room.timeline.events {
                            if event.type_ == "m.room.message" {
                                if event.content["msgtype"] == "m.text" {
                                    let sender = &event.sender;
                                    let body = &event.content["body"].as_str().unwrap();

                                    println!("<{}> | {}", sender, body);
                                }
                            }
                        }
                    },
                    None => (),
                }


                next_batch = res.next_batch;
            }
        }
    }
}

pub trait Node {
    fn parent(&self) -> Option<&str>;

    fn children(&self) -> Vec<&str>;

    fn register_child(&mut self, name: &str);

    fn propagate_event(&self, bot: &Bot, event: &Event) {
        for child in self.children() {
            //bot.get_service(child).handle(bot, event);
        }
    }

    fn handle(&self, bot: &Bot, event: &Event) {
        self.propagate_event(bot, event);
    }

    fn on_load(&mut self) { }

    fn on_exit(&self) { }
}
