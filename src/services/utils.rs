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
