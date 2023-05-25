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


pub fn codeblock_format(message: &str) -> String {
    let sanitized = message.replace("<", "&lt;").replace(">", "&gt;");
    format!("<pre><code class=\"language-text\">{}</code></pre>", &sanitized)
}