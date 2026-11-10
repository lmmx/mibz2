//! Scratch driver: callgrind's per-block decode (`measured`), and the selector's census.
//! Usage: drv cg FILE BLOCK REPS | drv census FILE
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Kernels, Scratch};

#[inline(never)]
fn measured(s: &mibz2::block::Symbols, k: Kernels, sc: &mut Scratch, out: &mut Vec<u8>) -> u32 {
    out.clear();
    decode_block(s, k, sc, out).unwrap()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[2]).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut i = 0;
    match a[1].as_str() {
        "cg" => {
            let want: usize = a[3].parse().unwrap();
            let reps: usize = a[4].parse().unwrap();
            let block = loop {
                match sc.next(syms).unwrap() {
                    Event::Block { symbols, .. } => {
                        if i == want { break symbols; }
                        i += 1;
                        syms = symbols.syms;
                    }
                    Event::Done => panic!("no block {want}"),
                }
            };
            let mut scratch = Scratch::default();
            let mut out = Vec::new();
            let mut crc = 0;
            for _ in 0..reps { crc ^= measured(&block, k, &mut scratch, &mut out); }
            println!("block {want}: {} bytes, crc {crc:08x}, syms {}, takes {}", out.len(), block.syms.len(),
                block.max_len < 1 << 24 && 2 * block.syms.len() <= block.max_len);
        }
        "census" => {
            let (mut blocks, mut taken, mut syms_all) = (0, 0, 0usize);
            let mut scratch = Scratch::default();
            let mut out = Vec::new();
            let (mut rows_all, mut rows_taken) = (0usize, 0usize);
            loop {
                match sc.next(syms).unwrap() {
                    Event::Block { symbols, .. } => {
                        let t = symbols.max_len < 1 << 24 && 2 * symbols.syms.len() <= symbols.max_len;
                        out.clear();
                        measured(&symbols, k, &mut scratch, &mut out);
                        blocks += 1; taken += t as usize; syms_all += symbols.syms.len();
                        rows_all += out.len(); if t { rows_taken += out.len(); }
                        syms = symbols.syms;
                    }
                    Event::Done => break,
                }
            }
            println!("{} blocks, {} take records ({:.1}% of output bytes), {:.4} symbols a byte", blocks, taken,
                100.0 * rows_taken as f64 / rows_all.max(1) as f64, syms_all as f64 / rows_all.max(1) as f64);
        }
        _ => panic!("mode"),
    }
}
