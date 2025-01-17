#![allow(dead_code)]
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;

use serde_json::Value;


pub trait EventContainer {
    fn get_events(&self) -> &Vec<Event>;
}

#[derive(Serialize)]
pub struct LoginIdentifier<'a> {
    #[serde(rename="type")]
    pub type_: &'a str,
    pub user: &'a str,
}

#[derive(Serialize)]
pub struct Login<'a> {
    #[serde(rename="type")]
    pub type_: &'a str,
    pub identifier: LoginIdentifier<'a>,
    pub password: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Init {
    pub user_id: String,
    pub access_token: String,
    pub home_server: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatrixSync {
    /*
    account_data: Events,
    device_lists: HashMap,
    */
    pub next_batch: String,
    //presence: HashMap,
    pub rooms: Option<Rooms>,
    //to_device: HashMap,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rooms {
    pub join: Option<HashMap<String, Room>>,
    pub invite: Option<HashMap<String, InviteRoom>>,
    pub leave: Option<HashMap<String, LeaveRoom>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Room {
    pub account_data: Value,
    pub ephemeral: Value,
    pub state: StateEvents,
    pub timeline: Timeline,
    pub unread_notifications: Value
}

impl EventContainer for Room {
    fn get_events(&self) -> &Vec<Event> {
        &self.timeline.events
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteRoom {
    pub invite_state: StateEvents,
}

impl EventContainer for InviteRoom {
    fn get_events(&self) -> &Vec<Event> {
        &self.invite_state.events
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveRoom {
    pub state: StateEvents,
    pub timeline: Timeline,
}

impl EventContainer for LeaveRoom {
    fn get_events(&self) -> &Vec<Event> {
        &self.timeline.events
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateEvents {
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timeline {
    pub events: Vec<Event>,
    pub limited: Option<bool>,
    pub prev_batch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub content: Value,
    pub event_id: Option<String>,
    //membership: String,
    pub origin_server_ts: Option<u64>,
    pub sender: String,
    //state_key: String,
    #[serde(rename="type")]
    pub type_: String,
    pub unsigned: Option<Value>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicRooms {
    pub total_room_count_estimate: u32,
    pub next_batch: Option<String>,
    pub chunk: Vec<PublicRoom>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicRoom {
    pub canonical_alias: Option<String>,
    pub name: String,
    pub world_readable: bool,
    pub topic: Option<String>,
    pub num_joined_members: u32,
    pub avatar_url: Option<String>,
    pub room_id: String,
    pub guest_can_join: bool,
    pub aliases: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinedRooms {
    pub joined_rooms: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomAlias {
    pub room_id: String,
    pub servers: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct RoomName {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatedRoom {
    pub room_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserDirectory {
    pub limited: bool,
    pub results: Vec<DirectoryProfile>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectoryProfile {
    pub avatar_url: String,
    pub display_name: String,
    pub user_id: String,
}

#[derive(Deserialize, Debug)]
pub struct RoomChunks {
    pub chunk: Vec<Event>,
    pub start: String,
    pub end: Option<String>,
    pub state: Option<Vec<Event>>,
}

#[derive(Deserialize, Debug)]
pub struct RoomMembers {
    pub joined: HashMap<String, Value>,
}