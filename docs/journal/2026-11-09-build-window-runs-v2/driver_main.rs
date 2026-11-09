//! Scratch driver (not in the repo): decode blocks of a .bz2 with the crate's default
//! kernel (`Scratch::default()`), each decode in `measured` for callgrind's toggle, or
//! timed. Usage: c30cg cg FILE BLOCK REPEATS | c30cg time FILE THREADS ROUNDS
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Kernels, Scratch};
use std::io::Read;

#[inline(never)]
fn measured(s: &mibz2::block::Symbols, k: Kernels, sc: &mut Scratch, out: &mut Vec<u8>) -> u32 {
    out.clear();
    decode_block(s, k, sc, out).unwrap()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[2]).unwrap();
    let k = Kernels::detect();
    match a[1].as_str() {
        "cg" => {
            let want: usize = a[3].parse().unwrap();
            let reps: usize = a[4].parse().unwrap();
            let mut sc = Scanner::new(SliceSource(&data));
            let mut syms = Vec::new();
            let mut i = 0;
            let block = loop {
                match sc.next(syms).unwrap() {
                    Event::Block { symbols, .. } => {
                        if i == want {
                            break symbols;
                        }
                        i += 1;
                        syms = symbols.syms;
                    }
                    Event::Done => panic!("no block {want}"),
                }
            };
            let mut scratch = Scratch::default();
            let mut out = Vec::new();
            let mut crc = 0;
            for _ in 0..reps {
                crc ^= measured(&block, k, &mut scratch, &mut out);
            }
            println!("block {want}: {} bytes, crc {crc:08x}", out.len());
        }
        "time" => {
            let threads: usize = a[3].parse().unwrap();
            let rounds: usize = a[4].parse().unwrap();
            for _ in 0..rounds {
                let owned = data.clone();
                let mut out = Vec::with_capacity(256 << 20);
                let t = std::time::Instant::now();
                if threads == 1 {
                    mibz2::BzReader::new(&data[..]).read_to_end(&mut out).unwrap();
                } else {
                    mibz2::BzReader::parallel(std::io::Cursor::new(owned), threads)
                        .read_to_end(&mut out)
                        .unwrap();
                }
                println!("{:.4}", t.elapsed().as_secs_f64());
            }
        }
        _ => panic!("mode"),
    }
}
