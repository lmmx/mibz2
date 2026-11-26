#![allow(dead_code)]
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
use std::{io::Write, time::Instant};
#[path = "../../../../docs/journal/2026-11-18-open-return/open_return.rs"]
mod old;
#[path = "section.rs"]
mod section;
#[path = "open.rs"]
mod sparse;

#[repr(C)]
struct Ts {
    sec: i64,
    ns: i64,
}
unsafe extern "C" {
    fn clock_gettime(clock: i32, a: *mut Ts) -> i32;
}
fn cpu() -> u64 {
    let mut t = Ts { sec: 0, ns: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut t) }, 0);
    t.sec as u64 * 1_000_000_000 + t.ns as u64
}

fn main() {
    let mut hazard = section::Hazards::new();
    let mut sparse = sparse::Kernel::default();
    let mut old = old::Kernel::default();
    let mut raw = Vec::new();
    let mut expected = Vec::new();
    let mut out = Vec::new();
    let k = Kernels::detect();
    if std::env::args().nth(1).as_deref() == Some("small") {
        let mut cases = 0;
        let mut invalid = 0;
        for n in 1..=10 {
            for mask in 0..1 << n {
                let l: Vec<_> = (0..n)
                    .map(|i| if mask & (1 << i) == 0 { 0 } else { 255 })
                    .collect();
                let caps: Vec<_> = l.iter().map(|&v| ((v as u16) << 8) | 1).collect();
                let mut psi: Vec<_> = (0..n).collect();
                psi.sort_by_key(|&i| l[i]);
                assert!(hazard.build(&caps, n));
                for p in 0..n {
                    let mut x = p;
                    let raw: Vec<_> = (0..n)
                        .map(|_| {
                            x = psi[x];
                            l[x]
                        })
                        .collect();
                    expected.clear();
                    let want = mibz2::rle::unrle(&raw, &mut expected, k);
                    out.clear();
                    let got = sparse.expanded(&caps, n, p as u32, &hazard, &mut out);
                    assert_eq!(got, want);
                    if want.is_ok() {
                        assert_eq!(out, expected);
                    } else {
                        invalid += 1;
                    }
                    cases += 1;
                }
            }
        }
        println!(
            "{cases} forced final-output interval constructions ({invalid} reject unfinished RLE1)"
        );
        return;
    }
    let input = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut sc = Scanner::new(SliceSource(&input));
    let mut runs = runs::Runs::default();
    let mut phase =
        std::io::BufWriter::new(std::fs::File::create(std::env::args().nth(2).unwrap()).unwrap());
    writeln!(
        phase,
        "stream,round,arm,wall_ns,cpu_ns,hazards_ns,construction_walk_ns,unrle_ns,crc_ns"
    )
    .unwrap();
    println!("stream,rows,events,danger,seal,index_capacity,decoded,took");
    let mut count = 0;
    let mut verified = 0;
    while let Ok(Event::Block { index, symbols, .. }) = sc.next(Vec::new()) {
        let n = runs::front(&symbols, &mut runs, k).unwrap();
        if n < 65536 {
            continue;
        }
        let selected = count % 24 == 0;
        count += 1;
        let caps = runs.records();
        if caps.len() * 20 >= 3 * n {
            continue;
        }
        let p = symbols.orig_ptr;
        if !hazard.build(caps, 1024) {
            println!("{index},{n},,,,,,false");
            continue;
        }
        old.run::<true, true, false>(caps, n, p, 16, &mut raw);
        expected.clear();
        mibz2::rle::unrle(&raw, &mut expected, k).unwrap();
        out.clear();
        sparse.expanded(caps, n, p, &hazard, &mut out).unwrap();
        assert_eq!(out, expected);
        assert_eq!(mibz2::crc::crc32(&out, k), symbols.crc);
        verified += n;
        println!(
            "{index},{n},{},{},{},{},{},true",
            hazard.events.len(),
            hazard.danger.len(),
            hazard.seal.len(),
            hazard.index_bytes(),
            out.len()
        );
        if !selected {
            continue;
        }
        for round in 0..=20 {
            for offset in 0..2 {
                let arm = (round + offset) % 2;
                out.clear();
                let w = Instant::now();
                let t0 = cpu();
                if arm == 0 {
                    assert!(hazard.build(caps, 1024));
                }
                let t1 = cpu();
                if arm == 0 {
                    sparse.expanded(caps, n, p, &hazard, &mut out).unwrap();
                } else {
                    old.run::<true, true, false>(caps, n, p, 16, &mut raw);
                }
                let t2 = cpu();
                if arm != 0 {
                    mibz2::rle::unrle(&raw, &mut out, k).unwrap();
                }
                let t3 = cpu();
                let checksum = mibz2::crc::crc32(&out, k);
                let t4 = cpu();
                let wall = w.elapsed().as_nanos();
                assert_eq!(checksum, symbols.crc);
                assert_eq!(out, expected);
                if round != 0 {
                    writeln!(
                        phase,
                        "{index},{round},{},{wall},{},{},{},{},{}",
                        ["sparse", "opend"][arm],
                        t4 - t0,
                        t1 - t0,
                        t2 - t1,
                        t3 - t2,
                        t4 - t3
                    )
                    .unwrap();
                }
            }
        }
    }
    eprintln!("large {count}, checked {verified} inverse bytes");
}
