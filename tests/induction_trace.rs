//! Research only (`induction-trace`): the induction recorder (`mibz2::itrace`) on every block
//! of the test corpora and samples, at levels 9 and 1. Each record is returned only once the
//! replay's steps equal the kernel's log and its final word expands to the kernel's output
//! (`itrace::record`); here, in addition, the record's own arithmetic is checked, and its
//! `irec/1` bytes against its arrays. (A letter that closes before the marked one needs a
//! column of several cycles that close first, which no bzip2 block is: `itrace`'s unit tests
//! take that path, on arbitrary columns.)
#![cfg(feature = "induction-trace")]

mod common;
use common::*;
use mibz2::itrace::{self, Record, ARENA, BYTE, EQUAL, MARKED, NONE, RULE, TOP_WINS};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;

/// Every block of a .bz2, recorded
fn records(z: &[u8]) -> Vec<Record> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(z));
    let mut syms = Vec::new();
    let mut out = Vec::new();
    while let Event::Block { symbols, index, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let rec = itrace::record(&symbols, k).unwrap_or_else(|e| panic!("block {index}: {e}"));
        out.push(rec);
        syms = symbols.syms;
    }
    out
}

/// The record's arithmetic: its arrays' lengths, each cell's word and width from its two
/// letters, the uses, the deaths, the closures and the final letter
fn consistent(rec: &Record) {
    let (r, steps) = (rec.r as usize, rec.s_a.len());
    for a in [&rec.l_byte, &rec.l_lam, &rec.l_bottom] {
        assert_eq!(a.len(), r);
    }
    for a in [&rec.s_b, &rec.s_ra, &rec.s_rb, &rec.s_type] {
        assert_eq!(a.len(), steps);
    }
    for a in [
        &rec.w_len,
        &rec.w_kind,
        &rec.w_uses,
        &rec.w_birth,
        &rec.w_death,
    ] {
        assert_eq!(a.len(), r + steps);
    }
    // the letters tile the block, and the bottom order is the top order sorted by byte
    assert_eq!(
        rec.l_lam.iter().map(|&x| x as u64).sum::<u64>(),
        rec.n as u64
    );
    let mut sorted: Vec<u32> = (0..rec.r).collect();
    sorted.sort_by_key(|&i| rec.l_byte[i as usize]);
    assert_eq!(rec.l_bottom, sorted);
    assert_eq!(rec.l_lam[rec.row_letter as usize], 1);
    let mut uses = vec![0u32; r + steps];
    for i in 0..steps {
        let (a, b) = (rec.s_a[i] as usize, rec.s_b[i] as usize);
        assert!(a < r + i && b < r + i && a != b, "step {i}");
        let c = r + i;
        assert_eq!(rec.w_len[c], rec.w_len[a] + rec.w_len[b], "step {i}");
        assert_eq!(rec.w_birth[c], i as u32);
        let marked = (rec.w_kind[a] | rec.w_kind[b]) & MARKED;
        assert_eq!(rec.w_kind[c] & MARKED, marked, "step {i}");
        let kind = if marked == 0 && rec.w_len[c] <= 16 {
            ARENA
        } else {
            RULE
        };
        assert_eq!(rec.w_kind[c] & 3, kind, "step {i}");
        let (ra, rb) = (rec.s_ra[i], rec.s_rb[i]);
        let ty = rec.s_type[i];
        assert_eq!(ty == EQUAL, ra == rb);
        assert_eq!(ty == TOP_WINS, ra > rb);
        uses[a] += 1;
        uses[b] += 1;
    }
    assert_eq!(uses, rec.w_uses);
    // a letter leaves at each equal step (two end, one is made) and each closure, and none
    // at another step. When the cycle is the whole block, every letter has left by its
    // close, so the equal steps and the closures add up to the block's letters; when the
    // block is the cycle's copies, the letters of the other cycles are left
    let equal = rec.s_type.iter().filter(|&&t| t == EQUAL).count();
    if rec.period == rec.n {
        assert_eq!(equal + rec.c_step.len(), r);
    } else {
        assert!(equal + rec.c_step.len() < r);
    }
    for (i, k) in rec.w_kind[..r].iter().enumerate() {
        assert_eq!(k & 3, BYTE);
        assert_eq!(k & MARKED != 0, i as u32 == rec.row_letter);
        assert_eq!(rec.w_birth[i], NONE);
    }
    // closures: in order of steps, the last the final, marked letter, which never dies
    assert!(!rec.c_step.is_empty());
    assert!(rec.c_step.windows(2).all(|w| w[0] <= w[1]));
    assert_eq!(*rec.c_letter.last().unwrap(), rec.final_letter);
    assert_ne!(rec.w_kind[rec.final_letter as usize] & MARKED, 0);
    assert_eq!(rec.w_len[rec.final_letter as usize], rec.period);
    assert_eq!(rec.w_death[rec.final_letter as usize], NONE);
    assert_eq!(rec.n % rec.period, 0);
}

/// The `irec/1` bytes: one header line naming each array at its offset, then the arrays
fn bytes_match(rec: &Record) {
    let v = rec.to_bytes("corpus", 3, 7);
    let nl = v.iter().position(|&c| c == b'\n').unwrap();
    let head = std::str::from_utf8(&v[..nl]).unwrap();
    assert!(
        head.starts_with("{\"format\":\"irec/1\",\"corpus\":\"corpus\",\"block\":3,\"index\":7,")
    );
    let body = &v[nl + 1..];
    let mut at = 0;
    for (name, a) in rec.arrays() {
        assert!(
            head.contains(&format!("\"{name}\":[{at},{}]", a.len())),
            "{name}"
        );
        let got: Vec<u32> = body[at..at + 4 * a.len()]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(got, a, "{name}");
        at += 4 * a.len();
    }
    assert_eq!(at, body.len());
}

#[test]
fn every_block_records_and_checks() {
    let mut rng = Rng(0x1d0c_7ace);
    let mut inputs: Vec<(String, Vec<u8>)> = corpora(1)
        .into_iter()
        .map(|(n, d)| (n.to_string(), d))
        .collect();
    inputs.extend(samples(&mut rng).into_iter().filter(|(_, d)| !d.is_empty()));
    let mut blocks = 0;
    for (name, data) in &inputs {
        for level in [9, 1] {
            let z = compress(data, level);
            for rec in records(&z) {
                consistent(&rec);
                blocks += 1;
            }
        }
        // (one input's bytes checked, the first block of each at level 9)
        let z = compress(data, 9);
        bytes_match(&records(&z)[0]);
        let _ = name;
    }
    assert!(blocks > 50, "{blocks} blocks");
}
