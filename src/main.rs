#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate rustix;

use std::io::Read;
use std::fs::File;

use rustix::{
    bot,
    client::MatrixClient,
    services:: {
        echo::Echo,
        self_filter::SelfFilter,
        karma::*,
        quote::{read_quote, del_quote},
        prefix::Prefix,
        choose::Choose,
        roulette::Roulette,
        crypto_coin::CryptoCoin,
        tryfile::TryFile,
        membership::{Join, Leave},
        admin::Admin,
    },
};


#[derive(Deserialize, Debug)]
struct Config {
    connection: Connection,
    bot: Bot,
}

#[derive(Deserialize, Debug)]
struct Connection {
    server: String,
    username: String,
    password: String,
}

#[derive(Deserialize, Debug)]
struct Bot {
    display_name: String,
    prefix: String,
    rooms: Vec<String>,
    admins: Vec<String>,
}


fn main() {
    let mut f = File::open("config.toml").expect("auth file not found");
    let mut config_data = String::new();
    f.read_to_string(&mut config_data).expect("Couldn't read config.toml");

    let config: Config = toml::from_str(&config_data)
                         .expect("Bad config.toml");

    let mut m = MatrixClient::new(&config.connection.server);

    m.login(&config.connection.username,
            &config.connection.password).expect("login failed!");
    m.set_display_name(&config.bot.display_name).unwrap();

    let mut b = bot::Bot::new(&mut m);

    let sf = b.register_service("self_filter", None, Box::new(SelfFilter::new()));
    let pf = b.register_service("prefix", sf,
                                Box::new(Prefix::new(config.bot.prefix)));
    b.register_service("echo", pf, Box::new(Echo::new()));

    b.register_service("show_karma", pf, Box::new(show_karma::ShowKarma::new()));
    b.register_service("karma_tracker", sf, Box::new(KarmaTracker::new()));

    b.register_service("read_quote", pf, Box::new(read_quote::ReadQuote::new()));
    b.register_service("choose", pf, Box::new(Choose::new()));
    b.register_service("roulette", pf, Box::new(Roulette::new()));
    b.register_service("crypto_coin", pf, Box::new(CryptoCoin::new()));

    let adm = b.register_service("admin", pf,
                                 Box::new(Admin::new(config.bot.admins)));

    b.register_service("join", adm, Box::new(Join::new()));
    b.register_service("leave", adm, Box::new(Leave::new()));
    b.register_service("del_quote", adm, Box::new(del_quote::DelQuote::new()));

    b.register_service("try_file", pf, Box::new(TryFile::new()));

    b.run(&config.bot.rooms);
}
