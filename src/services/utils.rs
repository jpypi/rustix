use std::io::{BufReader, BufRead};
use std::fs::File;
use rand::Rng;

const K: usize = 5;

pub fn reservoir_sample<R: Rng>(f: File, mut rng: R) -> String {
    let reader = BufReader::new(f);

    let mut reservoir: [String;K] = Default::default();

    for (i, line) in reader.lines().enumerate() {
        let l = line.unwrap();

        if i < K {
            reservoir[i] = l;
        } else {
            let j = rng.gen_range(0, i);
            if j < K {
                reservoir[j] = l;
            }
        }
    }

    let n = rng.gen_range(0, K);

    reservoir[n].clone()
}
