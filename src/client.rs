#![allow(dead_code)]
use std::{result};
use std::io::Read;
use std::collections::HashMap;

use reqwest;
use reqwest::{Url, Response};
use reqwest::header::{ContentType};
use serde_json;
use reqwest::Method;

use errors::Error;
use matrix_types::*;


type Result<T> = result::Result<T, Error>;


pub struct MatrixClient {
    base_url: String,
    encoding: String,

    access_token: Option<String>,
    device_id: Option<String>,
    user_id: Option<String>,

    transaction_id: u64,
    client: reqwest::Client,
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
            client: reqwest::Client::new(),
        }
    }

    fn get_transaction_id(&mut self) -> u64 {
        self.transaction_id += 1;

        self.transaction_id
    }

    pub fn query(&self,
             method: Method,
             path: &str,
             params: Option<&HashMap<&str, &str>>,
             data: Option<&HashMap<&str, &str>>) -> Result<Response> {

        // Concat the path to the base url and constant string
        let mut uri = self.base_url.clone();
        uri += "/_matrix/client/r0";
        uri += path;

        let url = match params {
            // TODO: an unwrap ok here?
            Some(v) => Url::parse_with_params(&uri, v).unwrap().into_string(),
            None => uri
        };

        let nothing = HashMap::new();

        let mut builder = self.client.request(method.clone(), &url);

        let request = match method {
            Method::Post | Method::Put => {
                builder.header(ContentType::json())
                       .json(data.unwrap_or(&nothing))
            },
            _ => &mut builder,
        };

        Ok(request.send()?)

            /*
        match method {
            Method::Get => {
                client.get(&url)?.send()
            },
            POST => {
                client.post(&url)?
                      .header(ContentType::json())
                      .json(data.unwrap_or(&nothing)).unwrap().send()
            }
            PUT => {
                client.put(&url)?
                      .header(ContentType::json())
                      .json(data.unwrap_or(&nothing)).unwrap().send()
            }
        }
        */
    }

    pub fn auth_query(&self,
                  method: Method,
                  path: &str,
                  params: Option<HashMap<&str, &str>>,
                  data: Option<&HashMap<&str, &str>>) -> Result<Response> {

        let mut real_params = match params {
            Some(v) => v,
            None => HashMap::new(),
        };

        match self.access_token {
            Some(ref v) => {
                real_params.insert("access_token", v);
                self.query(method, path, Some(&real_params), data)
            },
            None => {
                Err(Error::Generic("User must be authenticated first.".to_string()))
            },
        }
    }

    pub fn get(&self, path: &str,
               params: Option<&HashMap<&str, &str>>) -> Result<Response> {
        self.query(Method::Get, path, params, None)
    }

    pub fn auth_get(&self, path: &str,
                    params: Option<HashMap<&str, &str>>) -> Result<Response> {
        self.auth_query(Method::Get, path, params, None)
    }

    pub fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let map = hashmap!{
            "user"     => username,
            "password" => password,
            "type"     => "m.login.password",
            "initial_device_display_name" => "rustix",
        };

        let mut content = String::new();

        self.query(Method::Post, "/login", Option::None, Some(&map))?
            .read_to_string(&mut content).unwrap();

        // Parse response into client state
        match serde_json::from_str::<Init>(&content) {
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

        let mut content = String::new();

        match self.auth_get("/sync", Some(params)) {
            Ok(mut resp) => {
                resp.read_to_string(&mut content).unwrap();
                match serde_json::from_str(&content) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        //println!("{}", content);
                        let err = format!("problem syncing: {:?}", e);
                        Err(err.into())
                    },
                }
            },
            Err(e) => Err(e.into())
        }
    }

    //TODO: Validate this
    pub fn get_public_rooms(&self) -> Result<PublicRooms> {
        let params = hashmap! {
            "from" => "",
            "dir"  => "f"
        };

        let mut content = String::new();
        match self.auth_get("/publicRooms", Some(params)) {
            Ok(mut resp) => {
                resp.read_to_string(&mut content).unwrap();
                match serde_json::from_str(&content) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        println!("{}", content);
                        panic!("{:?}", e);
                    }
                }
            },
            Err(v) => Err(v),
        }
    }

    pub fn get_public_room_id(&self, name: &str) -> Option<String> {
        if let Ok(v) = self.get_public_rooms() {
            for room in v.chunk {
                if room.name == name {
                    return Some(room.room_id.clone());
                }
            }
        }

        None
    }

    pub fn join(&self, room_id: &str) -> Result<Response> {
        self.auth_query(Method::Post,
                        &format!("/join/{}", room_id),
                        None, None)
    }

    pub fn leave(&self, room_id: &str) -> Result<Response> {
        self.auth_query(Method::Post,
                        &format!("/rooms/{}/leave", room_id),
                        None, None)
    }

    pub fn set_display_name(&self, name: &str) -> Result<Response> {
        let data = hashmap! {
            "displayname" => name,
        };

        let path = format!("/profile/{}/displayname",
                           self.user_id.as_ref().expect("Must be logged in"));
        self.auth_query(Method::Put, &path, None, Some(&data))
    }

    pub fn send(&mut self,
                room_id: &str,
                event_type: &str,
                data: Option<&HashMap<&str, &str>>) -> Result<Response> {
        let path = format!("/rooms/{}/send/{}/{}", room_id, event_type,
                           self.get_transaction_id());

        self.auth_query(Method::Put, &path, None, data)
    }

    pub fn send_msg(&mut self, room_id: &str, message: &str) -> Result<Response> {
        let data = hashmap! {
            "msgtype" => "m.text",
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

        self.auth_query(Method::Post, &path, None, Some(&data))
    }

    pub fn get_joined(&self) -> Result<JoinedRooms> {
        match self.auth_get("/joined_rooms", None) {
            Ok(resp) => Ok(serde_json::from_reader(resp)?),
            Err(e) => Err(e.into())
        }
    }

    /*
    pub fn indicate_typing(&self, room_id: &str, length: Option<u32>) -> Result<Response, RustixError> {
        let mut data = HashMap::new();
        data.insert("typing", serde_json::Value::Bool(true));
        let v = match length {
            Some(i) => i,
            None => 1000,
        };


        data.insert("timeout", serde_json::Value::from(v));

        let path = format!("/rooms/{}/typing/{}", room_id,
                           self.user_id.as_ref().expect("Must be logged in"));
        self.auth_query(HTTPVerb::PUT, &path, None, Some(&data))
    }
    */
}
