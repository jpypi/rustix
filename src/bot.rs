use std::collections::HashMap;
use std::cell::{RefCell, RefMut};

use reqwest::{Response};

use errors::*;
use client::MatrixClient;
use matrix_types::Event;


pub struct BotEvent<'a> {
    user_bot_power: u32,
    room: &'a str,
    raw_event: Event,
}



pub struct Bot <'a, 'b> {
    client: RefCell<&'b mut MatrixClient>,
    root_services: Vec<&'a str>,
    all_services: HashMap<&'a str, RefCell<Box<Node<'a>>>>,

    //rooms: RefCell<Vec<&'a str>>,
}

impl<'a, 'b> Bot<'a, 'b> {
    pub fn new(client: &'b mut MatrixClient) -> Self{
        Bot {
            client: RefCell::new(client),
            root_services: Vec::new(),
            all_services: HashMap::new(),
            //rooms: RefCell::new(Vec::new()),
        }
    }

    pub fn join(&self, room_id: &str) -> Result<Response, RustixError>{
        //self.rooms.borrow_mut().push(room_id);
        self.client.borrow().join(room_id)
    }

    pub fn say(&self, room_id: &str, message: &str) -> Result<Response, RustixError> {
        self.client.borrow_mut().send_msg(room_id, message)
    }

    /*
    pub fn reply(&self, event: Event, message: &str) {
    }

    pub fn action(&self, room_id: &str, action: &str) {
    }

    pub fn set_name(&self, name: &str) {
    }
    */

    pub fn register_service(&mut self, name: &'a str, service: Box<Node<'a>>) {
        match service.parent() {
            Some(p) => {
                self.all_services.get_mut(p).unwrap().borrow_mut().register_child(name)
            },
            None => self.root_services.push(name),
        };

        self.all_services.insert(name, RefCell::new(service));
    }

    pub fn get_service(&self, name: &str) -> RefMut<Box<Node<'a>>> {
        self.all_services.get(name).unwrap().borrow_mut()
    }

    pub fn propagate_event(&self, event: &Event) {
        for service in &self.root_services {
            self.all_services.get(service).unwrap()
                .borrow_mut().handle(self, event.clone());
        }
    }

    pub fn run(&mut self) {
        let monitor_room;
        match self.client.borrow().get_public_room_id("test") {
            Some(room_id) => {
                self.join(&room_id);
                println!("Room id: {}", &room_id);
                monitor_room = room_id;
            },
            None => panic!(),
        };

        self.say(&monitor_room, "Hello world from rust!");


        if let Some(rid) = self.client.borrow().get_public_room_id("#geeks") {
            self.join(&rid);
        }

        let mut next_batch;
        match self.client.borrow().sync(None) {
            Ok(res) => next_batch = res.next_batch,
            Err(e) => panic!(e),
        }

        loop {
            let sync_data;
            if let Ok(res) = self.client.borrow().sync(Some(&next_batch)) {
                sync_data = res;
            } else {
                continue;
            }

                /*
                if let Ok(x) = serde_json::to_string_pretty(&res.rooms.join) {
                    println!("{}", x);
                }
                */
                if let Some(room) = sync_data.rooms.join.get(&monitor_room) {
                    for event in &room.timeline.events {
                        self.propagate_event(event);
                    }
                }


                next_batch = sync_data.next_batch;
        }
    }
}


pub trait Node<'a> {
    fn parent(&self) -> Option<&str>;

    fn children(&self) -> &Vec<&'a str>;

    fn register_child(&mut self, name: &'a str);

    fn propagate_event(&self, bot: &Bot, event: Event) {
        for child in self.children() {
            bot.get_service(child).handle(bot, event.clone());
        }
    }

    fn handle(&mut self, bot: &Bot, event: Event) {
        self.propagate_event(bot, event);
    }

    fn on_load(&mut self) { }

    fn on_exit(&self) { }
}
