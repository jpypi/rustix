use rustix::{
    bot,
    config,
    client::MatrixClient,
    services::{
        echo::Echo,
        karma::*,
        quote::{read_quote, del_quote},
        prefix::Prefix,
        choose::Choose,
        roulette::Roulette,
        crypto_coin::CryptoCoin,
        tryfile::TryFile,
        membership::{Join, Leave, AcceptInvite},
        admin::Admin,
        get_joined::GetJoined,
        csv_quote::csv_quote,
        help::Help,
    },
    filters::{
        SelfFilter,
        UserFilter,
        MessageTypeFilter,
    }
};


fn main() {
    // Load config
    let config = config::load_config("config.toml");

    // Set up a matrix HTTP client
    let mut m = MatrixClient::new(&config.connection.server);

    m.login(&config.connection.username,
            &config.connection.password).expect("login failed!");
    m.set_displayname(&config.bot.display_name).unwrap();

    // Create a new bot
    let mut b = bot::Bot::new(&mut m);

    // Register services with the bot
    let sf = b.register_service("self_filter", None,
                                Box::new(SelfFilter::new(
                                        &config.connection.username,
                                        &config.connection.server
                                    )));
    let uf = b.register_service("user_filter", sf,
                                Box::new(UserFilter::new(config.bot.ignore)));

    b.register_service("accept_invite", uf, Box::new(AcceptInvite::new()));

    let mt = b.register_service("message_type_filter", uf,
                                Box::new(MessageTypeFilter::new()));

    b.register_service("karma_tracker", mt, Box::new(KarmaTracker::new()));

    let pf = b.register_service("prefix", mt,
                                Box::new(Prefix::new(config.bot.prefix)));

    b.register_service("show_karma", pf, Box::new(show_karma::ShowKarma::new()));
    b.register_service("echo", pf, Box::new(Echo::new()));
    b.register_service("read_quote", pf, Box::new(read_quote::ReadQuote::new()));
    b.register_service("csv_quotes", pf, Box::new(csv_quote::ReadQuote::new()));
    b.register_service("choose", pf, Box::new(Choose::new()));
    b.register_service("roulette", pf, Box::new(Roulette::new()));
    b.register_service("crypto_coin", pf, Box::new(CryptoCoin::new()));
    b.register_service("try_file", pf, Box::new(TryFile::new()));
    b.register_service("help", pf, Box::new(Help::new()));

    let adm = b.register_service("admin", pf,
                                 Box::new(Admin::new(config.bot.admins)));
    b.register_service("join", adm, Box::new(Join::new()));
    b.register_service("leave", adm, Box::new(Leave::new()));
    b.register_service("del_quote", adm, Box::new(del_quote::DelQuote::new()));
    b.register_service("get_joined", adm, Box::new(GetJoined::new()));

    // Start bot main loop
    b.run(&config.bot.rooms);
}
