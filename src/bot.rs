use std::collections::HashMap;
use std::{result, thread, time};
use std::cell::{RefCell, RefMut};

use reqwest::{Response};

use errors::Error;
use client::MatrixClient;
use matrix_types::Event;


type Result<T> = result::Result<T, Error>;


#[derive(Clone)]
pub struct RoomEvent<'a> {
    pub room_id: &'a str,
    pub raw_event: Event,
}


pub struct Bot <'a, 'b> {
    client: RefCell<&'b mut MatrixClient>,
    root_services: Vec<&'a str>,
    all_services: HashMap<&'a str, RefCell<Box<Node<'a>>>>,

    rooms: RefCell<Vec<String>>,
}

impl<'a, 'b> Bot<'a, 'b> {
    pub fn new(client: &'b mut MatrixClient) -> Self {
        Bot {
            client: RefCell::new(client),
            root_services: Vec::new(),
            all_services: HashMap::new(),
            rooms: RefCell::new(Vec::new()),
        }
    }

    pub fn join(&self, room_id: &str) -> Result<Response> {
        // Make sure that the bot isn't already in the room to be joined
        for room in self.rooms.borrow().iter() {
            if room_id == room {
                return Err("Already in room".into());
            }
        }

        self.rooms.borrow_mut().push(room_id.clone().to_string());

        self.client.borrow().join(room_id)
    }

    pub fn join_public(&self, room_id: &str) -> Result<Response> {
        let pub_room = self.client.borrow().get_public_room_id(room_id);

        match pub_room {
            Some(id) => self.join(&id),
            None => Err("Could not join invalid room.".into()),
        }
    }

    pub fn leave(&self, room_id: &str) -> Result<Response> {
        // Make sure that the bot is in the room already
        let mut remove_i = 0;
        let mut res = Err("Not in room".into());
        for (i, room) in self.rooms.borrow().iter().enumerate() {
            if room_id == room {
                res = self.client.borrow().leave(room_id);
                remove_i = i;
                break;
            }
        }

        match res {
            Ok(r) => {
                self.rooms.borrow_mut().swap_remove(remove_i);
                Ok(r)
            },
            Err(e) => Err(e),
        }
    }

    pub fn leave_public(&self, room_id: &str) -> Result<Response> {
        let pub_room = self.client.borrow().get_public_room_id(room_id);

        match pub_room {
            Some(id) => self.leave(&id),
            None => Err("Could not leave invalid room".into()),
        }
    }

    pub fn say(&self, room_id: &str, message: &str) -> Result<Response> {
        self.client.borrow_mut().send_msg(room_id, message)
    }

    pub fn reply(&self, event: &RoomEvent, message: &str) -> Result<Response> {
        self.say(event.room_id, message)
    }

    pub fn kick(&self, room_id: &str, user_id: &str, reason: Option<&str>) {
        self.client.borrow().kick(room_id, user_id, reason);
    }

    /*
    pub fn action(&self, room_id: &str, action: &str) {
    }

    pub fn set_name(&self, name: &str) {
    }
    */

    pub fn register_service(&mut self,
                            name: &'a str,
                            parent: Option<&'a str>,
                            mut service: Box<Node<'a>>) -> Option<&'a str> {
        match parent {
            Some(p) => {
                self.all_services.get_mut(p).expect("Invalid parent node")
                    .borrow_mut().register_child(name)
            },
            None => self.root_services.push(name),
        };

        service.on_load();

        self.all_services.insert(name, RefCell::new(service));

        Some(name)
    }

    pub fn get_service(&self, name: &str) -> RefMut<Box<Node<'a>>> {
        self.all_services.get(name).unwrap().borrow_mut()
    }

    pub fn propagate_event(&self, event: &RoomEvent) {
        for service in &self.root_services {
            self.all_services.get(service).unwrap()
                .borrow_mut().handle(self, event.clone());
        }
    }

    pub fn run(&mut self, initial_rooms: &Vec<String>) {
        for room in initial_rooms {
            if let Some(rid) = self.client.borrow().get_public_room_id(room) {
                println!("Joining {} id: {}", &room, &rid);
                self.join(&rid);
            } else {
                println!("Could not join room {}", &room);
            }
        }

        let mut next_batch: String = self.client.borrow().sync(None).unwrap().next_batch;

        let delay = time::Duration::from_millis(500);

        loop {
            // TODO: NLL May let this extra temp variable die
            let sync = self.client.borrow().sync(Some(&next_batch));

            match sync {
                Ok(sync_data) => {
                    let rooms = self.rooms.borrow().clone();
                    for room_id in rooms.iter() {
                        if let Some(room) = sync_data.rooms.join.get(room_id) {
                            for raw_event in &room.timeline.events {
                                self.propagate_event(
                                    &RoomEvent{room_id, raw_event: raw_event.clone()}
                                );
                            }
                        }
                    }

                    next_batch = sync_data.next_batch;
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }

            thread::sleep(delay);
        }
    }
}


pub trait Node<'a> {
    fn children(&self) -> Option<&Vec<&'a str>> {
        None
    }

    fn register_child(&mut self, name: &'a str) {
    }

    fn propagate_event(&self, bot: &Bot, event: &RoomEvent) {
        if let Some(children) = self.children() {
            for child in children {
                bot.get_service(child).handle(bot, event.clone());
            }
        }
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        self.propagate_event(bot, &event);
    }

    fn on_load(&mut self) { }

    fn on_exit(&self) { }
}


/*
if let Ok(x) = serde_json::to_string_pretty(&res.rooms.join) {
    println!("{}", x);
} */
