use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;

use signal_hook::consts::signal::{SIGINT, SIGTERM};

use rustix::{
    bot,
    config,
    client::MatrixClient,
    services::{
        echo::Echo,
        karma::{KarmaTracker, ShowKarma, RankKarma},
        quote::{Quotes, DelQuote},
        prefix::Prefix,
        choose::Choose,
        roulette::Roulette,
        crypto_coin::CryptoCoin,
        tryfile::TryFile,
        membership::{Join, Leave, EmptyCleanup, AcceptInvite},
        admin::Admin,
        get_joined::GetJoined,
        csv_quote::ReadQuote,
        help::Help,
        logging::Logger,
        websearch::WebSearch,
        openai::gpt::GPT,
        factoid::{Factoid, DelFactoid, ListAllFactoid},
        structure::Structure,
        nodectrl::Configure,
        bonequest::Bonequest,
        voteremove::Voteremove,
    },
    filters::{
        SelfFilter,
        UserFilter,
        MessageTypeFilter,
        ChannelFilter,
        ForwardFilter,
    }
};


fn main() {
    // Load config
    let config = config::load_config("config.toml");

    // Set up a matrix HTTP client
    let m = Arc::new(RwLock::new(MatrixClient::new(&config.connection.server)));

    m.write().unwrap()
     .login(&config.connection.username,
            &config.connection.password).expect("login failed!");

    // Create a new bot
    let mut b = bot::Bot::new(Arc::clone(&m));
    b.set_displayname(&config.bot.display_name).unwrap();

    // Register services with the bot
    let sf = b.register_service("self_filter", None,
                                Box::new(SelfFilter::new(
                                        &config.connection.username,
                                        &config.connection.server
                                    )));

    let uf = b.register_service("user_filter", sf,
                                Box::new(UserFilter::new(config.bot.ignore.clone(), false)));

    let ff = b.register_service("forward_filter", uf, Box::new(ForwardFilter::new()));

    b.register_service("accept_invite", ff, Box::new(AcceptInvite::new()));

    let mt = b.register_service("message_type_filter", ff,
                                Box::new(MessageTypeFilter::new()));

    let karma_config = config.services.as_ref().and_then(|s| s.get("karma"));
    b.register_service("karma_tracker", mt,
                       Box::new(KarmaTracker::new(config.bot.prefix.clone(), karma_config)));

    let pf = b.register_service("prefix", mt,
                                Box::new(Prefix::new(config.bot.prefix.clone())));

    b.register_service("logging", pf, Box::new(Logger::new()));

    b.register_service("show_karma",  pf, Box::new(ShowKarma::new()));
    b.register_service("rank_karma",  pf, Box::new(RankKarma::new()));
    b.register_service("echo",        pf, Box::new(Echo::new()));
    b.register_service("structure",   pf, Box::new(Structure::new()));
    b.register_service("read_quote",  pf, Box::new(Quotes::new()));
    b.register_service("choose",      pf, Box::new(Choose::new()));
    b.register_service("roulette",    pf, Box::new(Roulette::new(config::RemovalMode::Kick)));
    b.register_service("rroulette",   pf, Box::new(Roulette::new(config::RemovalMode::Ban)));
    b.register_service("crypto_coin", pf, Box::new(CryptoCoin::new()));
    b.register_service("votekick",    pf, Box::new(Voteremove::new(5, 5, config::RemovalMode::Kick)));

    if let Some(bq_profanity) = config.services.as_ref().and_then(|s| s.get("bonequest")) {
        let bq_cf = b.register_service("bq_channel_filter", pf, Box::new(ChannelFilter::new(vec![], false)));
        b.register_service("bonequest", bq_cf, Box::new(Bonequest::new(bq_profanity)));
    }

    // Optional configurable services
    if let Some(csv_quote_cfg) = config.services.as_ref().and_then(|s| s.get("csv_quote")) {
        b.register_service("csv_quotes", pf, Box::new(ReadQuote::new(csv_quote_cfg)));
    }
    if let Some(try_file_cfg) = config.services.as_ref().and_then(|s| s.get("try_file")) {
        b.register_service("try_file", pf, Box::new(TryFile::new(try_file_cfg)));
    }
    if let Some(ws_cfg) = config.services.as_ref().and_then(|s| s.get("web_search")) {
        b.register_service("web_search", pf, Box::new(WebSearch::new(ws_cfg)));
    }
    if let Some(oa_cfg) = config.services.as_ref().and_then(|s| s.get("openai")) {
        /*
        let users = oa_cfg.get("whitelist").unwrap()
                          .as_array().unwrap()
                          .iter().map(|x| x.as_str().unwrap().to_string())
                          .collect();
        let allowed = b.register_service("openai_priv", pf, Box::new(Admin::new(users)));
        */
        b.register_service("openai", pf, Box::new(GPT::new(oa_cfg)));
    }
    if let Some(f_cfg) = config.services.as_ref().and_then(|s| s.get("factoid")) {
        b.register_service("factoid", mt, Box::new(Factoid::new(f_cfg)));
        b.register_service("del_factoid", pf, Box::new(DelFactoid::new()));

        if let Some(lc_cfg) = f_cfg.get("list_all_channels") {
            let channels: Vec<String> = lc_cfg.clone().try_into().expect("Invalid factoids list_all_channels config");
            let cf = b.register_service("all_factoids_channel_filter", pf, Box::new(ChannelFilter::new(channels, true)));
            b.register_service("list_factoids", cf, Box::new(ListAllFactoid::new()));
        }
    }

    b.register_service("help", pf, Box::new(Help::new()));

    let adm = b.register_service("admin", pf, Box::new(Admin::new(config.bot.admins)));
    b.register_service("join",         adm, Box::new(Join::new()));
    b.register_service("leave",        adm, Box::new(Leave::new()));
    b.register_service("emptycleanup", adm, Box::new(EmptyCleanup::new()));
    b.register_service("del_quote",    adm, Box::new(DelQuote::new()));
    b.register_service("get_joined",   adm, Box::new(GetJoined::new()));
    b.register_service("nodectl",      adm, Box::new(Configure::new()));


    // Join bot to initial rooms
    for room in &config.bot.rooms {
        println!("Joining {}", &room);
        if b.join_public(room).is_err() {
            println!("Could not join room");
        }
    }

    // Set up SIGTERM signal handler
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGINT, Arc::clone(&term))
                       .expect("Failed to setup SIGINT handler.");
    signal_hook::flag::register(SIGTERM, Arc::clone(&term))
                       .expect("Failed to setup SIGTERM handler.");

    // Start bot main loop
    b.run(&term);
}
