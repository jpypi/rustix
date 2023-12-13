use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{result, thread};
use std::cell::{RefCell, RefMut};
use std::any::Any;

use reqwest::blocking::Response;

use crate::errors::Error;
use crate::client::MatrixClient;
use crate::matrix_types::*;


type Result<T> = result::Result<T, Error>;


#[derive(Clone, Debug)]
pub struct RoomEvent<'a> {
    pub room_id: &'a str,
    pub from: &'a str,
    pub raw_event: Event,
}

impl<'a> RoomEvent<'a> {
    pub fn is_normal(&self) -> bool {
        self.raw_event.type_ == "m.room.message" && self.raw_event.content["msgtype"] == "m.text"
    }
}


type NodeProcessFn<'a> = dyn Fn(&Bot, &mut dyn Node) -> Box<dyn Any> + 'a;
pub struct Bot<'a, 'c> {
    p_client: Arc<RwLock<MatrixClient>>,
    root_services: Vec<&'a str>,
    all_services: HashMap<&'a str, RefCell<Box<dyn Node<'a>>>>,
    delayed_queries: RefCell<HashMap<&'c str, (Option<String>, Box<NodeProcessFn<'c>>)>>,
    display_name: String,
}

impl<'a, 'c> Bot<'a, 'c> {
    pub fn new(client_ref: Arc<RwLock<MatrixClient>>) -> Self {
        Bot {
            p_client: client_ref,
            root_services: Vec::new(),
            all_services: HashMap::new(),
            delayed_queries: RefCell::new(HashMap::new()),
            display_name: "".to_string(),
        }
    }

    pub fn client(&self) -> RwLockWriteGuard<MatrixClient> {
        self.p_client.write().unwrap()
    }

    pub fn arc_client(&self) -> Arc<RwLock<MatrixClient>>{
        Arc::clone(&self.p_client)
    }

    pub fn join_public(&self, room_id: &str) -> Result<Response> {
        let pub_room = self.p_client.read().unwrap().get_public_room_id(room_id);

        match pub_room {
            Some(id) => self.p_client.read().unwrap().join(&id),
            None => Err("Could not join invalid room.".into()),
        }
    }

    pub fn leave_public(&self, room_name: &str) -> Result<Response> {
        let pub_room = self.p_client.read().unwrap().get_public_room_id(room_name);

        match pub_room {
            Some(id) => self.p_client.read().unwrap().leave(&id),
            None => Err(format!("Could not find room: {}", room_name).into()),
        }
    }

    pub fn reply(&self, event: &RoomEvent, message: &str) -> Result<Response> {
        self.p_client.write().unwrap().send_msg(event.room_id, message)
    }

    pub fn reply_fmt(&self, event: &RoomEvent, fmt_message: &str, message: &str) -> Result<Response> {
        self.p_client.write().unwrap().send_msg_fmt(event.room_id, fmt_message, message)
    }

    pub fn reply_action(&self, event: &RoomEvent, message: &str) -> Result<Response> {
        self.p_client.write().unwrap().send_action(event.room_id, message)
    }

    pub fn uid_from_displayname(&self, name_query: &str) -> Result<String> {
        let res = self.p_client.read().unwrap().get_directory(name_query, Some(10))?;
        match res.results.first() {
            Some(n) => Ok(n.user_id.clone()),
            None => Err(Error::Generic("Empty".to_owned())),
        }
    }

    pub fn set_displayname(&mut self, name: &str) -> Result<Response> {
        self.display_name = name.to_string();
        self.p_client.write().unwrap().set_displayname(name)
    }

    pub fn get_displayname(&self) -> &str {
        &self.display_name
    }

    pub fn register_service(&mut self,
                            name: &'a str,
                            parent: Option<&'a str>,
                            mut service: Box<dyn Node<'a>>) -> Option<&'a str> {
        match parent {
            Some(p) => self.all_services.get_mut(p).expect("Invalid parent node")
                           .borrow_mut().register_child(name),
            None => self.root_services.push(name),
        };

        service.on_load(name);

        self.all_services.insert(name, RefCell::new(service));

        Some(name)
    }

    // TODO: This should be a Result and use ? instead of .unwrap()
    pub fn get_service(&self, name: &str) -> Option<RefMut<Box<dyn Node<'a>>>> {
        Some(self.all_services.get(name)?.borrow_mut())
    }

    pub fn get_service_names(&self) -> Vec<&str> {
        let keys = self.all_services.keys();
        keys.map(|k| *k).collect()
    }

    pub fn get_root_services(&self) -> &Vec<&str> {
        &self.root_services
    }

    // Two stage query all method
    pub fn delay_service_query<T: Fn(&Bot, &mut dyn Node) -> Box<dyn Any> + 'c>(&self, node: &'c str, target: Option<String>, func: T) {
        self.delayed_queries.borrow_mut().insert(node, (target, Box::new(func)));
    }

    fn process_delayed_queries(&mut self) {
        for (query_service_name, (target, func)) in self.delayed_queries.borrow().iter() {
            let mut results: Vec<(&str, Box<dyn Any>)> = Vec::new();
            if let Some(t) = target {
                if let Some(mut service) = self.get_service(&t) {
                    results.push((t, func(&self, &mut **service)));
                }
            } else {
                for (service_name, service) in &self.all_services {
                    results.push((service_name, func(&self, &mut **service.borrow_mut())));
                }
            }

            // Get the service
            if let Some(mut service) = self.get_service(query_service_name) {
                service.recieve_all_node_post(self, results);
            }
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

    fn on_exit(&self) {
        for (name, service) in &self.all_services {
            service.borrow().on_exit(name);
        }
    }

    pub fn run(&mut self, exit_flag: &Arc<AtomicBool>) {
        let mut next_batch: String = self.p_client.read().unwrap().sync(None).unwrap().next_batch;

        let delay = Duration::from_millis(500);

        while !exit_flag.load(Ordering::Relaxed) {
            let sync = self.p_client.read().unwrap().sync(Some(&next_batch));

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
                                    from: &"join",
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    for (room_id, room) in sync_data.rooms.invite {
                        for raw_event in &room.invite_state.events {
                            self.propagate_event(
                                &RoomEvent{
                                    room_id: &room_id,
                                    from: &"invite",
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    for (room_id, room) in sync_data.rooms.leave {
                        for raw_event in &room.timeline.events {
                            self.propagate_event(
                                &RoomEvent{
                                    room_id: &room_id,
                                    from: &"leave",
                                    raw_event: raw_event.clone()
                                });
                        }
                    }

                    self.process_delayed_queries();

                    next_batch = sync_data.next_batch;
                },
                Err(Error::Reqwest(e)) => {
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

        println!("Allowing services to exit cleanly...");
        self.on_exit();
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
                if let Some(mut service) = bot.get_service(child) {
                    service.handle(bot, event.clone());
                }
            }
        }
    }

    fn handle(&mut self, bot: &Bot, event: RoomEvent) {
        self.propagate_event(bot, &event);
    }

    #[allow(unused_variables)]
    fn recieve_all_node_post(&mut self, bot: &Bot, result: Vec<(&str, Box<dyn Any>)>) {
    }

    #[allow(unused_variables)]
    fn on_load(&mut self, service_name: &str) -> result::Result<(), String> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_exit(&self, service_name: &str) { }

    #[allow(unused_variables)]
    fn configure(&mut self, bot: &Bot, command: &str, event: RoomEvent) { }

    fn configure_description(&self) -> Option<String> {
        None
    }
}
