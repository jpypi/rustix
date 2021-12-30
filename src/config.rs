use std::io::Read;
use std::fs::File;


#[derive(Deserialize, Debug)]
pub struct Config {
    pub connection: Connection,
    pub bot: Bot,
}

#[derive(Deserialize, Debug)]
pub struct Connection {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct Bot {
    pub display_name: String,
    pub prefix: String,
    pub rooms: Vec<String>,
    pub admins: Vec<String>,
    pub ignore: Vec<String>,
}


pub fn load_config(filename: &str) -> Config {
    let mut f = File::open(filename).expect(&format!("Missing required file: {}", filename));
    let mut config_data = String::new();
    f.read_to_string(&mut config_data).expect(&format!("Could not read {}", filename));

    toml::from_str(&config_data).expect("Bad config file formatting")
}
