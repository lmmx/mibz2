#![allow(dead_code)]
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::{io::Write, time::Instant};
mod ahead;
mod bundle;
mod combo;
mod old;
fn heap<T>(v: &Vec<T>) -> usize {
    v.capacity() * std::mem::size_of::<T>()
}
#[repr(C)]
struct Ts {
    sec: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, out: *mut Ts) -> i32;
}
fn cpu() -> u64 {
    let mut t = Ts { sec: 0, ns: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut t) }, 0);
    t.sec as u64 * 1_000_000_000 + t.ns as u64
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let mut time = std::io::BufWriter::new(std::fs::File::create(&args[2]).unwrap());
    let mut geom = std::io::BufWriter::new(std::fs::File::create(&args[3]).unwrap());
    let names = ["opend", "ahead", "bundle", "combo"];
    writeln!(time, "stream,round,position,arm,wall_ns,cpu_ns").unwrap();
    writeln!(
        geom,
        "stream,rows,arm,kernel_bytes,coded_capacity,output_capacity,retained_bytes,crc"
    )
    .unwrap();
    let (mut a, mut b, mut c, mut d) = (
        old::Kernel::default(),
        ahead::Kernel::default(),
        bundle::Kernel::default(),
        combo::Kernel::default(),
    );
    let mut raw: [Vec<u8>; 2] = std::array::from_fn(|_| Vec::new());
    let mut outs: [Vec<u8>; 4] = std::array::from_fn(|_| Vec::new());
    let mut sc = Scanner::new(SliceSource(&data));
    let mut r = runs::Runs::default();
    let k = Kernels::detect();
    let mut ordinal = 0;
    let mut bytes = 0;
    while let Ok(Event::Block { index, symbols, .. }) = sc.next(Vec::new()) {
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = ordinal % 24 == 0;
        ordinal += 1;
        let caps = r.records();
        assert!(runs::takes_runs(&symbols));
        if caps.len() * 20 >= 3 * n {
            continue;
        }
        let p = symbols.orig_ptr;
        for round in 0..=if selected { 32 } else { 0 } {
            for pos in 0..4 {
                let arm = if round == 0 {
                    pos
                } else {
                    let rot = (round - 1) % 4;
                    let off = if (round - 1) / 4 % 2 == 0 {
                        pos
                    } else {
                        4 - pos
                    };
                    (rot + off) % 4
                };
                let out = &mut outs[arm];
                out.clear();
                let w = Instant::now();
                let t = cpu();
                match arm {
                    0 => {
                        a.run::<true, true, false>(caps, n, p, 16, &mut raw[0]);
                        mibz2::rle::unrle(&raw[0], out, k).unwrap();
                    }
                    1 => {
                        b.run::<true, true, false>(caps, n, p, 16, &mut raw[1]);
                        mibz2::rle::unrle(&raw[1], out, k).unwrap();
                    }
                    2 => c.expanded(caps, n, p, out, k).unwrap(),
                    _ => d.expanded(caps, n, p, out, k).unwrap(),
                }
                let crc = mibz2::crc::crc32(out, k);
                let elapsed_cpu = cpu() - t;
                let wall = w.elapsed().as_nanos();
                assert_eq!(crc, symbols.crc);
                if arm != 0 {
                    assert_eq!(outs[0], outs[arm]);
                }
                if round > 0 {
                    writeln!(
                        time,
                        "{index},{round},{pos},{},{wall},{elapsed_cpu}",
                        names[arm]
                    )
                    .unwrap();
                } else {
                    let kernel = match arm {
                        0 => a.bytes(),
                        1 => b.bytes(),
                        2 => c.bytes(),
                        _ => d.bytes(),
                    };
                    let scratch = if arm < 2 { heap(&raw[arm]) } else { 0 };
                    let output = heap(&outs[arm]);
                    let total = kernel + scratch + output;
                    writeln!(
                        geom,
                        "{index},{n},{},{kernel},{scratch},{output},{total},{crc}",
                        names[arm]
                    )
                    .unwrap();
                }
            }
        }
        bytes += n;
    }
    println!("{ordinal} large blocks, {bytes} admitted inverse bytes; every arm matched decoded bytes and block CRC on every iteration");
    assert_eq!(bytes, 205197794);
}
