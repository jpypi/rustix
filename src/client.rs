#![allow(dead_code)]
use std::time::Duration;
use std::result;
use std::collections::HashMap;

use reqwest;
use reqwest::Url;
use reqwest::blocking::Response;
use reqwest::header;
use http::Method;
use serde::Serialize;

use crate::errors::Error;
use crate::matrix_types::*;


type Result<T> = result::Result<T, Error>;


pub struct MatrixClient {
    base_url: String,
    encoding: String,

    access_token: Option<String>,
    device_id: Option<String>,
    user_id: Option<String>,

    transaction_id: u64,
    client: reqwest::blocking::Client,
}


impl MatrixClient {
    pub fn new(base_url: &str) -> Self {
        MatrixClient {
            base_url: String::from(base_url),
            encoding: String::from("utf-8"),
            access_token: None,
            device_id: None,
            user_id: None,
            transaction_id: 0,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get_transaction_id(&mut self) -> u64 {
        self.transaction_id += 1;

        self.transaction_id
    }

    pub fn query<T: Serialize + ?Sized>(&self,
             method: Method,
             path: &str,
             params: Option<&HashMap<&str, &str>>,
             data: Option<&T>,
             version: Option<&str>) -> Result<Response> {

        // Concat the path to the base url and constant string
        let mut uri = self.base_url.clone();
        uri += "/_matrix/client/";
        uri += version.unwrap_or("r0");
        uri += path;

        let url = match params {
            // TODO: an unwrap ok here?
            Some(v) => Url::parse_with_params(&uri, v).unwrap().into(),
            None => uri
        };

        let nothing = HashMap::<String, String>::new();

        let builder = self.client.request(method.clone(), url);

        let request = match method {
            Method::POST | Method::PUT => {
                let partial = builder.header(header::CONTENT_TYPE, "text/json");
                match data {
                    Some(d) => partial.json(d),
                    None    => partial.json(&nothing),
                }
            },
            _ => builder,
        };

        Ok(request.send()?)
    }

    pub fn auth_query<T: Serialize + ?Sized>(&self,
                  method: Method,
                  path: &str,
                  params: Option<HashMap<&str, &str>>,
                  data: Option<&T>,
                  version: Option<&str>) -> Result<Response> {

        let mut real_params = params.unwrap_or_default();

        match self.access_token {
            Some(ref v) => {
                real_params.insert("access_token", v);
                self.query(method, path, Some(&real_params), data, version)
            },
            None => Err("User must be authenticated first.".into()),
        }
    }

    pub fn get(&self, path: &str,
               params: Option<&HashMap<&str, &str>>,
               version: Option<&str>) -> Result<Response> {
        self.query::<()>(Method::GET, path, params, None, version)
    }

    pub fn auth_get(&self, path: &str,
                    params: Option<HashMap<&str, &str>>,
                    version: Option<&str>) -> Result<Response> {
        self.auth_query::<()>(Method::GET, path, params, None, version)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let params = hashmap!{
            "user"     => username,
            "password" => password,
            "type"     => "m.login.password",
            "initial_device_display_name" => "rustix",
        };

        // Parse response into client state
        match self.query(Method::POST, "/login", Option::None, Some(&params), None)?.json::<Init>() {
            Ok(v) => {
                self.user_id = Some(v.user_id);
                self.access_token = Some(v.access_token);
                self.device_id = Some(v.device_id);

                Ok(())
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn sync(&self, since: Option<&str>) -> Result<MatrixSync>{
        let mut params = HashMap::new();
        if let Some(v) = since {
            params.insert("since", v);
        }

        self.auth_get("/sync", Some(params), None)
            .and_then(|r| r.json().map_err(|e| format!("Problem syncing: {:?}", e).into()) )
    }

    //TODO: Validate this
    pub fn get_public_rooms(&self) -> Result<PublicRooms> {
        let params = hashmap! {
            "from" => "",
            "dir"  => "f"
        };

        self.auth_get("/publicRooms", Some(params), None)
            .and_then(|r| r.json().map_err(|e| e.into()))
    }

    pub fn get_public_room_id(&self, name: &str) -> Option<String> {
        if let Ok(v) = self.get_public_rooms() {
            for room in v.chunk {
                if room.name == name {
                    return Some(room.room_id);
                }
            }
        }

        None
    }

    pub fn join(&self, room_id: &str) -> Result<Response> {
        self.auth_query::<()>(Method::POST,
                              &format!("/join/{}", room_id),
                              None, None, None)
    }

    pub fn leave(&self, room_id: &str) -> Result<Response> {
        self.auth_query::<()>(Method::POST,
                              &format!("/rooms/{}/leave", room_id),
                              None, None, None)
    }

    pub fn set_displayname(&self, name: &str) -> Result<Response> {
        let data = hashmap! {
            "displayname" => name,
        };

        let path = format!("/profile/{}/displayname",
                           self.user_id.as_ref().expect("Must be logged in"));
        self.auth_query(Method::PUT, &path, None, Some(&data), None)
    }

    pub fn send(&mut self,
                room_id: &str,
                event_type: &str,
                data: Option<&HashMap<&str, &str>>) -> Result<Response> {
        let path = format!("/rooms/{}/send/{}/{}", room_id, event_type,
                           self.get_transaction_id());

        self.auth_query(Method::PUT, &path, None, data, None)
    }

    pub fn send_msg(&mut self, room_id: &str, message: &str) -> Result<Response> {
        let data = hashmap! {
            "msgtype" => "m.text",
            "body"    => message,
        };

        self.send(room_id, "m.room.message", Some(&data))
    }

    pub fn send_msg_fmt(&mut self, room_id: &str, fmt_message: &str, message: &str) -> Result<Response> {
        let data = hashmap! {
            "format"  => "org.matrix.custom.html",
            "msgtype" => "m.text",
            "formatted_body" => fmt_message,
            "body"    => message,
        };

        self.send(room_id, "m.room.message", Some(&data))
    }

    pub fn send_action(&mut self, room_id: &str, message: &str) -> Result<Response> {
        let data = hashmap! {
            "msgtype" => "m.emote",
            "body"    => message,
        };

        self.send(room_id, "m.room.message", Some(&data))
    }

    pub fn kick(&self, room_id: &str, user_id: &str, reason: Option<&str>) -> Result<Response> {
        let path = format!("/rooms/{}/kick", room_id);

        let mut data = hashmap! {
            "user_id" => user_id,
        };

        if let Some(r) = reason {
            data.insert("reason", r);
        }

        self.auth_query(Method::POST, &path, None, Some(&data), None)
    }

    pub fn ban(&self, room_id: &str, user_id: &str, reason: Option<&str>) -> Result<Response> {
        let path = format!("/rooms/{}/ban", room_id);

        let mut data = hashmap! {
            "user_id" => user_id,
        };

        if let Some(r) = reason {
            data.insert("reason", r);
        }

        self.auth_query(Method::POST, &path, None, Some(&data), None)
    }

    pub fn get_joined(&self) -> Result<JoinedRooms> {
        self.auth_get("/joined_rooms", None, None)
            .and_then(|o| o.json().map_err(|e| e.into()))
    }

    pub fn get_room_name(&self, room_id: &str) -> Result<String> {
        let path = format!("/rooms/{}/state/m.room.name/", room_id);
        let res: Result<RoomName> = self.auth_get(&path, None, None)
                                        .and_then(|o| o.json().map_err(|e| e.into()));
        res.map(|v| v.name)
    }

    pub fn get_directory(&self, search_term: &str, limit: Option<u32>) -> Result<UserDirectory> {
        #[derive(Serialize)]
        struct Query<'a> {
            limit: u32,
            search_term: &'a str,
        }

        let data = Query {
            limit: limit.unwrap_or(u32::MAX),
            search_term,
        };

        self.auth_query(Method::POST, "/user_directory/search", None, Some(&data), None)
            .and_then(|o| o.json().map_err(|e| e.into()))
    }

    pub fn get_members(&self, room_id: &str) -> Result<Vec<String>> {
        self.auth_get(&format!("/rooms/{}/joined_members", room_id), None, None).and_then(|o| {
            o.json::<RoomMembers>()
             .map(|obj| obj.joined.into_keys().collect())
             .map_err(|e| e.into())
        })
    }

    pub fn get_room_events(&self, room_id: &str, n: u32, from: Option<&str>) -> Result<RoomChunks> {
        let limit_str = n.to_string();
        let mut params = hashmap! {
            "dir"   => "b",
            "limit" => &limit_str,
        };

        if let Some(f) = from {
            params.insert("from", f);
        }

        self.auth_get(&format!("/rooms/{}/messages", room_id), Some(params), None)
            .and_then(|o| o.json().map_err(|e| e.into()))
    }

    pub fn indicate_typing(&self, room_id: &str, length: Option<Duration>) -> Result<Response> {
        #[derive(Serialize)]
        struct Data {
            typing: bool,
            timeout: u128,
        }

        let timeout = length.unwrap_or(Duration::new(1, 0)).as_millis();

        let data = Data {
            typing: length.is_some(),
            timeout,
        };

        let path = format!("/rooms/{}/typing/{}", room_id,
                           self.user_id.as_ref().expect("Must be logged in"));
        self.auth_query(Method::PUT, &path, None, Some(&data), None)
    }
}