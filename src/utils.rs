use rand::Rng;

const K: usize = 10;

pub fn reservoir_sample<R: Rng, T: Clone + Default, E>(iterable: impl Iterator<Item=Result<T, E>>, rng: &mut R) -> Result<T, E> {
    let mut reservoir: [T;K] = Default::default();

    let mut max = 0;
    for (i, row) in iterable.enumerate() {
        max = i;
        if i < K {
            reservoir[i] = row?;
        } else {
            let j = rng.gen_range(0..i);
            if j < K {
                reservoir[j] = row?;
            }
        }
    }

    let n = rng.gen_range(0..(max + 1).min(K));

    Ok(reservoir[n].clone())
}


pub trait AliasStripPrefix {
    fn alias_strip_prefix<'a>(&'a self, aliases: &[&str]) -> Option<&'a str>;
}

impl AliasStripPrefix for str {
    /// Check's a string for multiple `prefixes` and upon match returns a string
    /// slice with the prefix removed.
    ///
    /// If the string does not start with any of the `prefixes`, return `None`.
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
    /// Trim's the string and strict compares it to all variants for a match
    ///
    /// # Arguments
    ///
    /// * `variants` - Reference to a slice of string references to compare the
    ///                base string against
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
    let sanitized = message.replace('<', "&lt;").replace('>', "&gt;");
    format!("<pre><code class=\"language-text\">{}</code></pre>", &sanitized)
}