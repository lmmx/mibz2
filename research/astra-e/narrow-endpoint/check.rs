//! Compare the narrowed endpoint and its wide fallback with stable LF and historical Open.
#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
#[cfg(not(any(short_words, ahead)))]
#[path = "../../../docs/journal/2026-11-18-open-return/open_return.rs"]
mod old;
#[cfg(any(short_words, ahead))]
#[path = "../short-words/constructor.rs"]
mod old;
#[cfg(not(any(short_words, ahead)))]
mod candidate;
#[cfg(short_words)]
#[path = "../narrow-short/candidate.rs"]
mod candidate;
#[cfg(ahead)]
#[path = "../narrow-short/ahead.rs"]
mod candidate;

fn reference(column: &[u8], primary: usize) -> Vec<u8> {
    let mut starts = [0; 256];
    for &byte in column { starts[byte as usize] += 1; }
    let mut end = 0;
    for count in &mut starts {
        let width = *count;
        *count = end;
        end += width;
    }
    let mut inverse = vec![0; column.len()];
    for (row, &byte) in column.iter().enumerate() {
        let cursor = &mut starts[byte as usize];
        inverse[*cursor] = row;
        *cursor += 1;
    }
    let mut row = primary;
    let mut out = vec![0; column.len()];
    for byte in &mut out { row = inverse[row]; *byte = column[row]; }
    out
}

fn records(column: &[u8]) -> Vec<u16> {
    let mut result: Vec<u16> = Vec::new();
    for &byte in column {
        if let Some(last) = result.last_mut() {
            if *last >> 8 == u16::from(byte) && (*last & 255) < 127 { *last += 1; continue; }
        }
        result.push((u16::from(byte) << 8) | 1);
    }
    result
}

fn main() {
    let mut wide = old::Kernel::default();
    let mut narrow = candidate::kernel::Kernel::default();
    let mut cases = 0;
    let mut fallback = 0;
    let (mut control, mut got) = (Vec::new(), Vec::new());
    let mut verify = |column: &[u8], p: usize, limit: u32| {
        let recs = records(column);
        let baseline = wide.run::<true, true, false>(&recs, column.len(), p as u32, limit, &mut control);
        let actual = narrow.run::<true, true, false>(&recs, column.len(), p as u32, limit, &mut got);
        assert_eq!(got, reference(column, p));
        assert_eq!(got, control);
        assert_eq!(format!("{baseline:?}"), format!("{actual:?}"));
        cases += 1;
        fallback += usize::from(actual.thick > (u16::MAX as usize - 3) / 3);
    };
    for n in 1..=9 {
        for mask in 0..1 << n {
            let column: Vec<u8> = (0..n).map(|i| ((mask >> i) & 1) as u8).collect();
            for p in 0..n { for limit in [1, 2, 16] { verify(&column, p, limit); } }
        }
    }
    // The leading singleton keeps the primary without splitting any of the long runs.
    // Check the last fitting H and the very next H, then reuse the same kernel across both.
    for h in [21_844, 21_845, 100, 21_846] {
        let mut column = vec![2];
        for j in 0..h { column.extend(std::iter::repeat_n((j & 1) as u8, 17)); }
        verify(&column, 0, 16);
    }
    for p in [0, 126, 127, 500] { verify(&[71; 501], p, 16); }
    assert_eq!(fallback, 2);
    println!("{cases} marked comparisons passed with dense stable LF and canonical Open counters/output; {fallback} exercised the wide fallback.");
}
