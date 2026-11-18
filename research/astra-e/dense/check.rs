//! Force the straight tracer above and below its density gate, including widths
//! too small for the production setting. Arguments can be "primary:column-file".
pub use mibz2::{bwt, runs, Kernels};
mod candidate;
use std::env;
use std::fs;

fn check(kernel: &mut candidate::Candidate, column: &[u8], p: u32, all_widths: bool) -> usize {
    let mut psi: Vec<usize> = (0..column.len()).collect();
    psi.sort_by_key(|&i| column[i]);
    let mut row = p as usize;
    let reference: Vec<u8> = (0..column.len())
        .map(|_| {
            row = psi[row];
            column[row]
        })
        .collect();
    let mut runs = Vec::<u16>::new();
    for &c in column {
        if let Some(prev) = runs.last_mut() {
            if *prev >> 8 == u16::from(c) && *prev & 255 < 127 {
                *prev += 1;
                continue;
            }
        }
        runs.push((u16::from(c) << 8) | 1);
    }
    let n = column.len() as u32;
    let mut out = Vec::new();
    if all_widths {
        kernel.trace::<0>(&runs, n, p, &mut out);
        assert_eq!(out, reference);
        kernel.trace::<1>(&runs, n, p, &mut out);
        assert_eq!(out, reference);
        kernel.trace::<3>(&runs, n, p, &mut out);
        assert_eq!(out, reference);
    }
    kernel.trace::<16>(&runs, n, p, &mut out);
    assert_eq!(out, reference);
    if all_widths {
        4
    } else {
        1
    }
}

fn main() {
    let mut kernel = candidate::Candidate::default();
    let mut small = 0;
    for (alphabet, max) in [(2u32, 9), (3u32, 6)] {
        for n in 1..=max {
            for mut id in 0..alphabet.pow(n) {
                let column: Vec<u8> = (0..n)
                    .map(|_| {
                        let c = (id % alphabet) as u8;
                        id /= alphabet;
                        c
                    })
                    .collect();
                for p in 0..n {
                    small += check(&mut kernel, &column, p, true);
                }
            }
        }
    }
    println!("exhaustive forced traces: {small}");
    let mut total = 0;
    for arg in env::args().skip(1) {
        let (primary, file) = arg.split_once(':').unwrap();
        let column = fs::read(file).unwrap();
        check(&mut kernel, &column, primary.parse().unwrap(), false);
        println!("passed {file} rows={} primary={primary}", column.len());
        total += column.len();
    }
    println!("archive and adversarial rows: {total}");
}
