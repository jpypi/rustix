extern crate serde;
extern crate serde_json;

/*
#[macro_use]
extern crate serde_derive;
*/

use std::collections::HashMap;

use serde_json::Value;


#[derive(Serialize, Deserialize, Debug)]
//allow(dead_code)]
pub struct Init {
    pub access_token: String,
    pub home_server: String,
    pub user_id: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct MatrixSync {
    /*
    account_data: Events,
    device_lists: HashMap,
    */
    pub next_batch: String,
    //presence: HashMap,
    pub rooms: Rooms,
    //to_device: HashMap,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct Rooms {
    pub invite: HashMap<String, InviteRoom>,
    pub join: HashMap<String, Room>,
    //pub leave: HashMap<String, Room>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct Room {
    pub account_data: Value,
    pub ephemeral: Value,
    pub state: Events,
    pub timeline: Events,
    pub unread_notifications: Value
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct InviteRoom {
    invite_state: InviteEvents,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct InviteEvents {
    events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
pub struct Events {
    pub events: Vec<Event>,
    pub limited: Option<bool>,
    pub prev_batch: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct PublicRooms {
    pub total_room_count_estimate: u32,
    pub next_batch: String,
    pub chunk: Vec<PublicRoom>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(dead_code)]
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
