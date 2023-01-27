use std::collections::HashMap;
use std::{result, thread, time};
use std::cell::{RefCell, RefMut};
use std::any::Any;

use reqwest::blocking::{Response};

use crate::errors::Error;
use crate::client::MatrixClient;
use crate::matrix_types::*;


type Result<T> = result::Result<T, Error>;


#[derive(Clone)]
pub struct RoomEvent<'a> {
    pub room_id: &'a str,
    pub raw_event: Event,
}

impl<'a> RoomEvent<'a> {
    pub fn is_normal(&self) -> bool {
        self.raw_event.type_ == "m.room.message" && self.raw_event.content["msgtype"] == "m.text"
    }
}


pub struct Bot<'a, 'b, 'c> {
    client: RefCell<&'b mut MatrixClient>,
    root_services: Vec<&'a str>,
    all_services: HashMap<&'a str, RefCell<Box<dyn Node<'a>>>>,
    delayed_queries: RefCell<HashMap<&'c str, Box<dyn Fn(&mut dyn Node) -> Box<dyn Any> + 'c>>>,
}

impl<'a, 'b, 'c> Bot<'a, 'b, 'c> {
    pub fn new(client: &'b mut MatrixClient) -> Self {
        Bot {
            client: RefCell::new(client),
            root_services: Vec::new(),
            all_services: HashMap::new(),
            delayed_queries: RefCell::new(HashMap::new()),
        }
    }

    pub fn join(&self, room_id: &str) -> Result<Response> {
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
        self.client.borrow().leave(room_id)
    }

    pub fn leave_public(&self, room_name: &str) -> Result<Response> {
        let pub_room = self.client.borrow().get_public_room_id(room_name);

        match pub_room {
            Some(id) => self.leave(&id),
            None => Err(format!("Could not find room: {}", room_name).into()),
        }
    }

    pub fn get_joined(&self) -> Result<JoinedRooms> {
        self.client.borrow().get_joined()
    }

    pub fn say(&self, room_id: &str, message: &str) -> Result<Response> {
        self.client.borrow_mut().send_msg(room_id, message)
    }

    pub fn reply(&self, event: &RoomEvent, message: &str) -> Result<Response> {
        self.say(event.room_id, message)
    }

    pub fn kick(&self, room_id: &str, user_id: &str, reason: Option<&str>) -> Result<Response> {
        self.client.borrow().kick(room_id, user_id, reason)
    }

    pub fn ban(&self, room_id: &str, user_id: &str, reason: Option<&str>) -> Result<Response> {
        self.client.borrow().ban(room_id, user_id, reason)
    }

    pub fn uid_from_displayname(&self, name_query: &str) -> Result<String> {
        let res = self.client.borrow().get_directory(name_query, Some(10))?;
        match res.results.first() {
            Some(n) => Ok(n.user_id.clone()),
            None => Err(Error::Generic("Empty".to_owned())),
        }
    }

    /*
    pub fn action(&self, room_id: &str, action: &str) {
    }
    */

    pub fn set_displayname(&self, name: &str) -> Result<Response> {
        self.client.borrow_mut().set_displayname(name)
    }

    pub fn register_service(&mut self,
                            name: &'a str,
                            parent: Option<&'a str>,
                            mut service: Box<dyn Node<'a>>) -> Option<&'a str> {
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

    // TODO: This should be a Result and use ? instead of .unwrap()
    pub fn get_service(&self, name: &str) -> RefMut<Box<dyn Node<'a>>> {
        self.all_services.get(name).unwrap().borrow_mut()
    }

    pub fn get_service_names(&self) -> Vec<&str> {
        let keys = self.all_services.keys();
        keys.map(|k| *k).collect()
    }

    // Two stage query all method
    pub fn delay_service_query<T: Fn(&mut dyn Node) -> Box<dyn Any> + 'c>(&self, node: &dyn Node, func: T) {
        self.delayed_queries.borrow_mut().insert("help", Box::new(func));
    }

    fn process_delayed_queries(&mut self) {
        for (query_service_name, func) in self.delayed_queries.borrow().iter() {
            let mut results: Vec<(&str, Box<dyn Any>)> = Vec::new();
            for (service_name, service) in &self.all_services {
                results.push((service_name, func(&mut **service.borrow_mut())));
            }
            // Get the service
            self.get_service(query_service_name).recieve_all_node_post(self, results);
        }

        self.delayed_queries = RefCell::new(HashMap::new());
    }
    // end

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
                if let Err(_) = self.join(&rid) {
                    println!("Could not join room");
                }
            } else {
                println!("Failure to find public room {}", &room);
            }
        }

        let mut next_batch: String = self.client.borrow().sync(None).unwrap().next_batch;

        let delay = time::Duration::from_millis(500);

        loop {
            let sync = self.client.borrow().sync(Some(&next_batch));

            match sync {
                Ok(sync_data) => {
                    /*
                    if let Ok(x) = serde_json::to_string_pretty(&sync_data) {
                        println!("{}", x);
                    }
                    */

                    for (room_id, room) in sync_data.rooms.join {
                        for raw_event in &room.timeline.events {
                            self.propagate_event(
                                &RoomEvent{
                                    room_id: &room_id,
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    for (room_id, room) in sync_data.rooms.invite {
                        for raw_event in &room.invite_state.events {
                            self.propagate_event(
                                &RoomEvent{
                                    room_id: &room_id,
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    for (room_id, room) in sync_data.rooms.leave {
                        for raw_event in &room.timeline.events {
                            self.propagate_event(
                                &RoomEvent{
                                    room_id: &room_id,
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    self.process_delayed_queries();

                    next_batch = sync_data.next_batch;
                },
                Err(Error::ReqwestError(e)) => {
                    if e.is_timeout() {
                        if let Some(url) = e.url() {
                            println!("Request timed out for {}", url);
                        } else {
                            println!("Request timed out");
                        }
                    } else {
                        println!("ReqwestError: {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }

            thread::sleep(delay);
        }
    }
}


pub trait Node<'a> {
    fn description(&self) -> Option<String> {
        None
    }

    fn children(&self) -> Option<&Vec<&'a str>> {
        None
    }

    #[allow(unused_variables)]
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

    #[allow(unused_variables)]
    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn Any>)>) {
    }

    fn on_load(&mut self) { }

    fn on_exit(&self) { }
}
