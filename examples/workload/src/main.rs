use std::fmt::Write;

use rand::{Rng, RngCore};
use rand::distributions::Distribution;
use sha1::{Sha1, Digest};
use zipf::ZipfDistribution;

const SIZE: usize = 10_000;
const READ_PCT: f64 = 0.75;

const ZIPF_N: usize = 10_000;
const ZIPF_COEF: f64 = 2.0;

pub fn hash(val: u64) -> String {
    let mut hasher = Sha1::new();
    hasher.update(val.to_le_bytes());
    let bytes = hasher.finalize();
    let mut out = String::with_capacity(2 * bytes.len());
    for byte in bytes {
        write!(out, "{:02x?}", byte).unwrap();
    }
    return out;
}

pub fn main() {
    let mut rng = rand::thread_rng();
    let zipf = ZipfDistribution::new(ZIPF_N, ZIPF_COEF).unwrap();
    for _ in 0..SIZE {
        let key = hash(zipf.sample(&mut rng) as u64);
        if rng.gen_bool(READ_PCT) {
            println!("Read({:?})", key);
        }
        else {
            println!("Write({:?}, {:?})", key, hash(rng.next_u64()));
        }
    }
}