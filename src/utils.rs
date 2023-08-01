use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use rand::Rng;

const K: usize = 5;

pub fn reservoir_sample<R: Rng, T: Clone + Default, E>(iterable: impl Iterator<Item=Result<T, E>>, rng: &mut R) -> Result<T, E> {
    let mut reservoir: [T;K] = Default::default();

    for (i, row) in iterable.enumerate() {
        if i < K {
            reservoir[i] = row?;
        } else {
            let j = rng.gen_range(0..i);
            if j < K {
                reservoir[j] = row?;
            }
        }
    }

    let n = rng.gen_range(0..K);

    Ok(reservoir[n].clone())
}


pub trait AliasStripPrefix {
    fn alias_strip_prefix<'a>(&'a self, aliases: &[&str]) -> Option<&'a str>;
}

impl AliasStripPrefix for str {
    fn alias_strip_prefix<'a>(&'a self, aliases: &[&str]) -> Option<&'a str> {
        for alias in aliases {
            if let Some(res) = self.strip_prefix(alias) {
                return Some(res);
            }
        }

        None
    }
}


pub trait TrimMatch {
    fn trim_match<'a>(&'a self, variants: &[&str]) -> Option<&'a str>;
}

impl TrimMatch for str {
    fn trim_match<'a>(&'a self, variants: &[&str]) -> Option<&'a str> {
        let trimmed = self.trim();
        for variant in variants {
            if trimmed == *variant {
                return Some(trimmed);
            }
        }

        None
    }
}


pub fn codeblock_format(message: &str) -> String {
    let sanitized = message.replace("<", "&lt;").replace(">", "&gt;");
    format!("<pre><code class=\"language-text\">{}</code></pre>", &sanitized)
}


pub fn save_state(service_name: &str, value: &str) {
    let mut path = PathBuf::from(".rustix");

    if let Err(e) = fs::create_dir(&path) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            println!("Unable to create .rustix save state directory");
            panic!("{:?}", e);
        }
    }

    path.push(service_name);
    let mut f = File::create(path).expect(&format!("Unable to create save state file for {}.", service_name));
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