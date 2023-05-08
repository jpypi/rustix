use std::{env, time::SystemTime};

use diesel::sql_types::Text;
use diesel::{prelude::*, sql_query};
use diesel::{Connection, PgConnection};
use dotenv::dotenv;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use regex::Regex;
use toml::Value;

use crate::bot::Node;

use super::super::db::schema::{factoids as fs, users as us};
use super::super::db::user::{self, User};
use super::models;

#[derive(Deserialize)]
struct Config {
    factoid_leader: String,
}

pub struct Factoid {
    connection: PgConnection,
    rng: SmallRng,
    leader: String,
    set_pattern: Regex,
}

impl Factoid {
    pub fn new(config: &Value) -> Self {
        let cfg: Config = config.clone().try_into().expect("Bad openai config.");

        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let connection = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let leader = cfg.factoid_leader;
        let set_pattern =
            Regex::new(&format!("{}\\s?(.+?) is (<reply>|<action>) (.+)", &leader)).unwrap();

        Self {
            connection,
            rng: SmallRng::from_entropy(),
            leader,
            set_pattern,
        }
    }

    fn del_quote(&self, qid: i32) -> QueryResult<models::Factoid> {
        diesel::delete(fs::dsl::factoids.filter(fs::id.eq(qid))).get_result(&self.connection)
    }
}

impl<'a> Node<'a> for Factoid {
    fn handle(&mut self, bot: &crate::bot::Bot, event: crate::bot::RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        let captures = self.set_pattern.captures(body);
        if let Some(groups) = captures {
            let factoid_key = groups.get(1).unwrap().as_str();
            let factoid_kind = groups.get(2).unwrap().as_str();
            let factoid_value = groups.get(3).unwrap().as_str();

            let user = user::fetch_or_create(&self.connection, &event.raw_event.sender).unwrap();

            let fact_kind = match factoid_kind {
                "<reply>" => models::FactoidKind::Reply,
                "<action>" => models::FactoidKind::Action,
                &_ => panic!("This should never occur"),
            };

            let f = models::NewFactoid {
                time: SystemTime::now(),
                user_id: user.id,
                pattern: factoid_key,
                kind: fact_kind,
                value: factoid_value,
            };

            diesel::insert_into(fs::table)
                .values(&f)
                .execute(&self.connection)
                .ok();
        } else if let Some(factoid_key) = body.strip_prefix("literal ") {
            let res: Vec<models::Factoid> = fs::dsl::factoids
                .filter(fs::pattern.eq(factoid_key))
                .load(&self.connection)
                .unwrap();
            let mut response = vec![format!(
                "{:>4} - {:^34} - {:^8}: {}",
                "id", "user", "kind", "factoid"
            )];
            for r in res {
                let user: User = us::dsl::users
                    .filter(us::id.eq(r.user_id))
                    .get_result(&self.connection)
                    .unwrap();
                response.push(format!(
                    "{:>4} - {:>34} - {:^8}: {}",
                    r.id, user.user_id, r.kind, r.value
                ));
            }

            if response.len() > 0 {
                let raw = response.join("\n");
                let sanitized = raw.replace("<", "&lt;").replace(">", "&gt;");
                let message = format!("<pre><code>{}</code></pre>", &sanitized);
                bot.reply_fmt(&event, &message, &raw).ok();
            }
        } else if let Some(qid) = body.strip_prefix("delfactoid ") {
            let response = match qid.parse() {
                Ok(qid) => {
                    if self.del_quote(qid).is_ok() {
                        format!("Successfully deleted factoid {}", qid)
                    } else {
                        format!("Failed to delete factoid {}", qid)
                    }
                }
                Err(_) => "Invalid factoid id".to_string(),
            };

            bot.reply(&event, &response).ok();
        } else {
            let res: Vec<models::Factoid> =
                sql_query("SELECT * FROM factoids WHERE $1 LIKE '%' || pattern || '%'")
                    .bind::<Text, _>(body)
                    .load(&self.connection)
                    .unwrap();
            if res.len() > 0 {
                let i = self.rng.gen_range(0..res.len());
                if let Some(f) = res.get(i) {
                    let msg = &f.value;
                    match f.kind {
                        models::FactoidKind::Reply => bot.reply(&event, msg).ok(),
                        models::FactoidKind::Action => bot.reply_action(&event, msg).ok(),
                    };
                }
            }
        }
    }

    fn description(&self) -> Option<String> {
        /*
        rustix, pizza is <reply> some link

        rustix, factoid key is <fact type> factoid value (the angle brackets are part of the syntax in this case).
        <reply> means respond directly with the factoid value when the factoid key is written in chat
        <action> respond with equivalent of /me

        - Allow multiple factoids with the same key.
        literal <factoid key> 	Show who set which factoids associated with the given key when
        */
        Some(format!(
            "factoids - e.g. {} something something is <reply> some response",
            self.leader
        ))
    }
}
