//! Time the previous published SUP1 encoder and inverse inside a full bzip2 decode.
#[allow(dead_code)]
mod contexts;

use mibz2::bwt::{Cells, InverseBwt};
use mibz2::extrema::Extrema;
use mibz2::Kernels;
use std::{env, fs, time::Instant};

#[derive(Default)]
struct Support;

impl InverseBwt for Support {
    fn invert(&mut self, last: &[u8], _: &[u32; 256], primary: u32, out: &mut Vec<u8>) {
        *out = contexts::through_support(last, primary as usize);
    }
}

fn main() {
    let compressed = fs::read(env::args().nth(1).unwrap()).unwrap();
    let rounds: usize = env::args().nth(2).unwrap().parse().unwrap();
    let kernels = Kernels::detect();
    let reference = mibz2::decode(&compressed).unwrap();
    let arms = ["support", "extrema", "cells"];
    println!("round,arm,seconds,output");
    for round in 0..=rounds {
        for j in 0..arms.len() {
            let arm = arms[(j + round) % arms.len()];
            let start = Instant::now();
            let decoded = match arm {
                "support" => mibz2::decode_with::<Support>(&compressed, kernels),
                "extrema" => mibz2::decode_with::<Extrema>(&compressed, kernels),
                _ => mibz2::decode_with::<Cells>(&compressed, kernels),
            }
            .unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            assert_eq!(decoded, reference);
            println!("{round},{arm},{elapsed:.6},{}", decoded.len());
        }
    }
}
