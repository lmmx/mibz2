//! Exact comparison of two cyclic length-k spectra, independent of the BWT and tree count.
use std::{collections::HashMap, env, fs};

fn main() {
    let args: Vec<_> = env::args().collect();
    let a = fs::read(&args[1]).unwrap();
    let b = fs::read(&args[2]).unwrap();
    let k: usize = args[3].parse().unwrap();
    let n = a.len();
    assert!(n == b.len() && k > 0 && k < n);
    let mut ids: Vec<u32> = a.iter().chain(&b).map(|&c| u32::from(c)).collect();
    let mut span = 1;
    // Identical halves receive identical IDs across both words. HashMap verifies
    // key equality, so hash collisions do not identify different substrings.
    while span * 2 <= k {
        let mut pairs = HashMap::new();
        let mut next = Vec::with_capacity(ids.len());
        for offset in [0, n] {
            for i in 0..n {
                let key = (ids[offset + i], ids[offset + (i + span) % n]);
                let fresh = pairs.len() as u32;
                next.push(*pairs.entry(key).or_insert(fresh));
            }
        }
        ids = next;
        span *= 2;
    }
    for length in [k, k + 1] {
        // Two possibly overlapping spans cover the entire length, including the case length = 2*span.
        let mut balance = HashMap::<(u32, u32), i32>::new();
        for (offset, sign) in [(0, 1), (n, -1)] {
            for i in 0..n {
                let key = (ids[offset + i], ids[offset + (i + length - span) % n]);
                *balance.entry(key).or_default() += sign;
            }
        }
        let changed = balance.values().filter(|&&v| v != 0).count();
        let l1: usize = balance.values().map(|v| v.unsigned_abs() as usize).sum();
        println!("{length},{},{changed},{l1}", balance.len());
    }
}
