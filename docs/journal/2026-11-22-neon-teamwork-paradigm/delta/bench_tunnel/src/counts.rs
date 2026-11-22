#![forbid(unsafe_code)]

mod open_local;

use mibz2::block::Symbols;
use mibz2::runs::{front_scalar, Runs};
use open_local::{EmitMode, Kernel as LocalKernel};
use std::fs;
use std::path::Path;

struct BlockMeta { file: &'static str, n: usize, p: u32 }

const BLOCKS: &[BlockMeta] = &[
    BlockMeta { file: "block-0.L",   n: 899931, p: 264045 },
    BlockMeta { file: "block-2.L",   n: 900000, p: 320807 },
    BlockMeta { file: "block-85.L",  n: 899935, p: 325088 },
    BlockMeta { file: "block-143.L", n: 899869, p: 802868 },
    BlockMeta { file: "block-207.L", n: 899998, p: 205180 },
    BlockMeta { file: "block-210.L", n: 899959, p: 791429 },
];

fn encode_symbols(l: &[u8], p: u32) -> Symbols {
    let mut present = [false; 256];
    for &b in l { present[b as usize] = true; }
    let mut seq_to_unseq = [0u8; 256];
    let mut n_in_use = 0usize;
    for b in 0..256 { if present[b] { seq_to_unseq[n_in_use] = b as u8; n_in_use += 1; } }
    let mut mtf = [0u8; 256];
    mtf[..n_in_use].copy_from_slice(&seq_to_unseq[..n_in_use]);
    let mut syms = Vec::with_capacity(l.len() / 2);
    let mut zr = 0usize;
    let flush = |zr: &mut usize, syms: &mut Vec<u16>| {
        let mut r = *zr;
        while r > 0 {
            if r & 1 == 1 { syms.push(0); r = (r - 1) >> 1; }
            else { syms.push(1); r = (r - 2) >> 1; }
        }
        *zr = 0;
    };
    for &b in l {
        let pos = mtf[..n_in_use].iter().position(|&x| x == b).unwrap();
        if pos == 0 { zr += 1; }
        else {
            flush(&mut zr, &mut syms);
            syms.push((pos + 1) as u16);
            for k in (1..=pos).rev() { mtf[k] = mtf[k - 1]; }
            mtf[0] = b;
        }
    }
    flush(&mut zr, &mut syms);
    let eob = (n_in_use + 1) as u16;
    syms.push(eob);
    Symbols { crc: 0, orig_ptr: p, seq_to_unseq, syms, eob, max_len: 900_000 }
}

fn main() {
    let dir = Path::new("/tmp/mibz2-blocks");
    let mut k = LocalKernel::default();
    let mut out = Vec::new();
    println!("=== OPEN LIMIT SWEEP EXACT COUNTS (6-BLOCK SUMS & RANGES) ===");
    for &lim in &[16u32, 8, 4, 2, 1] {
        let (mut s_m, mut s_thick, mut s_steps, mut s_rules, mut s_words, mut s_bands, mut s_tbl) = (0,0,0,0,0,0,0);
        for b in BLOCKS {
            let l = fs::read(dir.join(b.file)).unwrap();
            let syms = encode_symbols(&l, b.p);
            let mut runs = Runs::default();
            front_scalar(&syms, &mut runs).unwrap();
            let (c, _, _) = k.run_timed(runs.records(), b.n, b.p, lim, EmitMode::Plain, &mut out);
            s_m += c.retained;
            s_thick += c.thick;
            s_steps += c.steps;
            s_rules += c.rules;
            s_words += c.words;
            s_bands += c.bands;
            s_tbl += c.table_bytes;
        }
        println!("  LIMIT={:<2} | m={:<7} | thick={:<6} | rv_steps={:<7} | rules={:<7} | words={:<6} | bands={:<7} | edges_KB={:.1}",
            lim, s_m, s_thick, s_steps, s_rules, s_words, s_bands, s_tbl as f64 / 1024.0);
    }
}
