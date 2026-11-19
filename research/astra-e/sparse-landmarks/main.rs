use bwt::InverseBwt;
use mibz2::stream::{Event, Scanner, SliceSource};
pub use mibz2::{bwt, runs, Kernels};
mod sparse;
pub use sparse::Table;
use sparse::{oracle_write, paid, Plan};
#[allow(dead_code)]
mod original {
    include!(env!("EDGE_TABLE_SOURCE"));
}
include!("../edge-budget/gate.rs");
const IDS: [u64; 10] = [1, 41, 85, 134, 176, 225, 269, 313, 359, 397];
#[repr(C)]
struct Timespec {
    i: i64,
    n: i64,
}
unsafe extern "C" {
    fn clock_gettime(id: i32, t: *mut Timespec) -> i32;
}
fn cpu() -> u64 {
    let mut t = Timespec { i: 0, n: 0 };
    assert_eq!(unsafe { clock_gettime(3, &mut t) }, 0);
    t.i as u64 * 1_000_000_000 + t.n as u64
}
struct Block {
    id: u64,
    t: Table,
    gold: Vec<u8>,
    oracles: [Plan; 3],
}
const ARMS: [&str; 9] = [
    "stock",
    "oracle4s128",
    "paid4s128",
    "oracle8s128",
    "paid8s128",
    "oracle8s64",
    "paid8s64",
    "oracle8s256",
    "paid8s256",
];

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1]).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut original = original::Kernel::default();
    let mut lazy = bwt::LazyCells::default();
    let mut out = Vec::new();
    let mut r = runs::Runs::default();
    let mut blocks = Vec::new();
    let mut geo = std::fs::File::create(&args[2]).unwrap();
    use std::io::Write;
    writeln!(geo,"block,n,mark,retained,wide,shift,landmarks,primary_segments,period,used_descriptors,capacity_bytes").unwrap();
    while let Event::Block { index, symbols, .. } = scanner.next(Vec::new()).unwrap() {
        if !IDS.contains(&index) && args[3] != "census" {
            continue;
        }
        if !runs::takes_runs(&symbols) {
            continue;
        }
        let n = runs::front(&symbols, &mut r, k).unwrap();
        if r.records().len() * 20 >= 3 * n && (n < 65536 || !fits(r.records(), symbols.orig_ptr)) {
            continue;
        }
        let t = original.table(r.records(), n, symbols.orig_ptr);
        let mut gold = Vec::new();
        assert!(lazy.invert_runs(&r, n, symbols.orig_ptr, &mut gold, k));
        original::Kernel::stock(&t, &mut out);
        assert!(out == gold);
        let oracles = std::array::from_fn(|i| t.oracle(i as u32 + 6));
        for (i, oracle) in oracles.iter().enumerate() {
            let mut z = Plan::default();
            match i {
                0 => paid::<8, 6>(&mut z, &t, &mut out),
                1 => paid::<8, 7>(&mut z, &t, &mut out),
                2 => paid::<8, 8>(&mut z, &t, &mut out),
                _ => unreachable!(),
            }
            assert!(out == gold && z.jobs == oracle.jobs && z.period == oracle.period);
            writeln!(
                geo,
                "{index},{n},{},{},{},{},{},{},{},{},{}",
                t.mark,
                t.edges.len(),
                !t.extra.is_empty(),
                i + 6,
                z.landmarks(),
                z.jobs.len(),
                z.period,
                (z.landmarks() + z.jobs.len()) * 8,
                z.capacity_bytes()
            )
            .unwrap();
        }
        if IDS.contains(&index) {
            blocks.push(Block {
                id: index,
                t,
                gold,
                oracles,
            });
        }
    }
    assert_eq!(blocks.len(), 10);
    if args[3] == "census" {
        return;
    }
    let mut work: Vec<Plan> = (0..ARMS.len()).map(|_| Plan::default()).collect();
    println!("round,block,position,arm,cpu_ns");
    for round in 0..25 {
        for (ib, b) in blocks.iter().enumerate() {
            for pos in 0..ARMS.len() {
                let shift = (round / 2 + ib) % ARMS.len();
                let arm = if round % 2 == 0 {
                    (pos + shift) % ARMS.len()
                } else {
                    (shift + ARMS.len() - pos) % ARMS.len()
                };
                let now = cpu();
                match arm {
                    0 => original::Kernel::stock(&b.t, &mut out),
                    1 => oracle_write::<4>(&b.oracles[1], &b.t, &mut out),
                    2 => paid::<4, 7>(&mut work[arm], &b.t, &mut out),
                    3 => oracle_write::<8>(&b.oracles[1], &b.t, &mut out),
                    4 => paid::<8, 7>(&mut work[arm], &b.t, &mut out),
                    5 => oracle_write::<8>(&b.oracles[0], &b.t, &mut out),
                    6 => paid::<8, 6>(&mut work[arm], &b.t, &mut out),
                    7 => oracle_write::<8>(&b.oracles[2], &b.t, &mut out),
                    8 => paid::<8, 8>(&mut work[arm], &b.t, &mut out),
                    _ => unreachable!(),
                }
                let ns = cpu() - now;
                assert!(out == b.gold);
                println!("{round},{},{pos},{},{ns}", b.id, ARMS[arm]);
            }
        }
    }
}
