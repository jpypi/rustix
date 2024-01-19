use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

pub fn save_state(service_name: &str, value: &str) {
    let mut path = PathBuf::from(".rustix");

    if let Err(e) = fs::create_dir(&path) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            println!("Unable to create .rustix save state directory");
            panic!("{:?}", e);
        }
    }

    path.push(service_name);
    let mut f = File::create(path).unwrap_or_else(|_| panic!("Unable to create save state file for {}.", service_name));
    f.write_all(value.as_bytes()).expect("Failed to write state to save file.");
}

pub fn load_state(service_name: &str) -> Option<String> {
    let mut path = PathBuf::from(".rustix");
    path.push(service_name);

    if let Ok(f) = File::open(path) {
        let mut s = String::new();
        if let Err(e) = BufReader::new(f).read_to_string(&mut s) {
            println!("Error reading state save file for {}: {:?}", service_name, e);
            None
        } else {
            Some(s)
        }
    } else {
        None
    }
}