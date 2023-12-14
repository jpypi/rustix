use dotenv::dotenv;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use regex::Regex;
use toml::Value;

use crate::bot::Node;
use crate::utils::codeblock_format;

use super::models;
use super::backend::Backend;

#[derive(Deserialize)]
struct Config {
    factoid_leader: String,
}

pub struct Factoid {
    backend: Backend,
    rng: SmallRng,
    leader: String,
    set_pattern: Regex,
}

impl Factoid {
    pub fn new(config: &Value) -> Self {
        let cfg: Config = config.clone().try_into().expect("Bad factoid config.");

        dotenv().ok();

        let leader = cfg.factoid_leader;
        let set_pattern =
            Regex::new(&format!("^{}\\s?(.+?) is (<reply>|<action>) (.+)", &leader)).unwrap();

        Self {
            backend: Backend::new(),
            rng: SmallRng::from_entropy(),
            leader,
            set_pattern,
        }
    }

}

impl<'a> Node<'a> for Factoid {
    fn handle(&mut self, bot: &crate::bot::Bot, event: crate::bot::RoomEvent) {
        let revent = &event.raw_event;
        let body = revent.content["body"].as_str().unwrap();

        let captures = self.set_pattern.captures(body);
        if let Some(factoid_key) = body.strip_prefix("literal ") {
            let res = self.backend.match_factoids(factoid_key);
            let mut response = vec![format!(
                "{:>4} - {:^34} - {:^8}: {}",
                "id", "user", "kind", "factoid"
            )];
            for r in res {
                let user = self.backend.get_user(r.user_id);
                response.push(format!(
                    "{:>4} - {:>34} - {:^8}: {}",
                    r.id, user.user_id, r.kind, r.value
                ));
            }

            // If the only row is the header row then the length will be 1
            if response.len() > 1 {
                let raw = response.join("\n");
                let message = codeblock_format(&raw);
                bot.reply_fmt(&event, &message, &raw).ok();
            }
        } else if let Some(groups) = captures {
            let factoid_key = groups.get(1).unwrap().as_str();
            let factoid_kind = groups.get(2).unwrap().as_str();
            let factoid_value = groups.get(3).unwrap().as_str();

            let fact_kind = match factoid_kind {
                "<reply>" => models::FactoidKind::Reply,
                "<action>" => models::FactoidKind::Action,
                &_ => panic!("This should never occur"),
            };

            self.backend.add_factoid(&event.raw_event.sender, fact_kind, factoid_key, factoid_value);
        } else {
            let res = self.backend.match_factoids(body);
            if !res.is_empty() {
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
        Some(format!(
            "factoids:\n\
            \t{} something something is <reply> some response\n\
            \t{} another thing is <action> does a thing\n\
            \tliteral <string> - display any factoids that would trigger on <string>",
            self.leader, self.leader
        ))
    }
}
