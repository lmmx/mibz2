//! (research only, `squared-trace`) The open kernel's construct, square and emit per block in
//! production's own workers, under each of MONODROMY record 93's arms, for the
//! bench-emit-cost-sq diagnostic (docs/journal/2026-11-23-emit-cost-sq-registration.md).
//!
//!     emit_cost_sq FILE.bz2 WORKERS ARMS ROUNDS SETTLE_S OUT.tsv
//!
//! WORKERS and ARMS are comma-separated (ARMS of `base`, `sq`, `sq-shadow`). Each round decodes
//! FILE once in every (workers, arm) cell, with production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, the window `SLOTS_PER_THREAD` a worker),
//! in this order (MONODROMY record 94 at 1b188ada, #145 REVIEW 7097894105):
//!
//! - **the worker counts** in WORKERS' order rotated by `round mod len(WORKERS)`: for 1,6,10,
//!   round 0 runs 1, 6, 10; round 1 runs 6, 10, 1; round 2 runs 10, 1, 6;
//! - **within each count, the arms** in permutation `round mod len(ARMS)!` of ARMS' orders,
//!   listed lexicographically by position in ARMS. For base,sq,sq-shadow:
//!
//!   | round mod 6 | order                  |
//!   |------------:|------------------------|
//!   | 0           | base, sq, sq-shadow    |
//!   | 1           | base, sq-shadow, sq    |
//!   | 2           | sq, base, sq-shadow    |
//!   | 3           | sq, sq-shadow, base    |
//!   | 4           | sq-shadow, base, sq    |
//!   | 5           | sq-shadow, sq, base    |
//!
//!   so each ordered pair of adjacent arms occurs twice in six rounds and no arm has a fixed
//!   predecessor. 20 rounds are not a multiple of 6: permutations 0 and 1 run 4 times, the
//!   other four 3 times.
//!
//! The output is read and discarded (every block's CRC and the stream's are checked by the
//! decoder). The arm is switched between decodes in this one process
//! (`mibz2::emit_cost::set_arm`). OUT.tsv gets one `blk` line per block record and one `run`
//! line per decode (its wall time and bytes). SETTLE_S seconds of sleep precede each decode.
use mibz2::emit_cost::{self, Arm};
use std::io::Read;
use std::time::{Duration, Instant};

const COLS: &str = "kind\tround\tworkers\tarm\torder\tseq\tn\tm\tconstruct_ns\tsquare_ns\temit_ns\tbidi\trefused\tsq_rounds\tscratch\twall_s\tbytes\n";

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 7 {
        eprintln!("usage: emit_cost_sq FILE.bz2 WORKERS ARMS ROUNDS SETTLE_S OUT.tsv");
        std::process::exit(2);
    }
    // (held for the whole run, as the decoder's feeder needs a `'static` source; read once,
    // so no decode reads the disk)
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let workers: Vec<usize> = a[2]
        .split(',')
        .map(|s| s.parse().expect("WORKERS"))
        .collect();
    let arms: Vec<Arm> = a[3]
        .split(',')
        .map(|s| Arm::parse(s).expect("ARMS: base, sq or sq-shadow"))
        .collect();
    let rounds: usize = a[4].parse().expect("ROUNDS");
    let settle = Duration::from_secs_f64(a[5].parse().expect("SETTLE_S"));
    let perms = permutations(arms.len());
    let mut out = String::from(COLS);
    let k = mibz2::Kernels::detect();
    let mut buf = vec![0u8; 4 << 20];
    for round in 0..rounds {
        let perm = &perms[round % perms.len()];
        let cells: Vec<(usize, Arm)> = (0..workers.len())
            .map(|i| workers[(i + round) % workers.len()])
            .flat_map(|w| perm.iter().map(move |&a| (w, a)))
            .map(|(w, a)| (w, arms[a]))
            .collect();
        for (o, &(w, arm)) in cells.iter().enumerate() {
            std::thread::sleep(settle);
            let _ = emit_cost::take();
            emit_cost::set_arm(arm);
            let s = Instant::now();
            let mut r = mibz2::BzReader::parallel_workers(
                data,
                w,
                mibz2::parallel::SLOTS_PER_THREAD * w,
                k,
                true,
            );
            let mut bytes = 0u64;
            loop {
                match r.read(&mut buf) {
                    Ok(0) => break,
                    Ok(x) => bytes += x as u64,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(e) => panic!("decode failed: {e}"),
                }
            }
            drop(r);
            let wall = s.elapsed().as_secs_f64();
            emit_cost::set_arm(Arm::Base);
            let recs = emit_cost::take();
            let name = arm.name();
            for (i, x) in recs.iter().enumerate() {
                out += &format!(
                    "blk\t{round}\t{w}\t{}\t{o}\t{i}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\t\n",
                    x.arm.name(),
                    x.n,
                    x.m,
                    x.construct_ns,
                    x.square_ns,
                    x.emit_ns,
                    x.bidi as u8,
                    x.refused as u8,
                    x.sq_rounds as u8,
                    x.scratch
                );
            }
            out += &format!(
                "run\t{round}\t{w}\t{name}\t{o}\t\t\t\t\t\t\t\t\t\t\t{wall:.6}\t{bytes}\n"
            );
            let open = recs.iter().filter(|x| !x.refused);
            let (m, e) = open.fold((0u64, 0u64), |(m, e), x| {
                (m + u64::from(x.m), e + x.square_ns + x.emit_ns)
            });
            eprintln!(
                "round {round} workers {w} {name}: {wall:.3} s, {bytes} bytes, {} records, square + emit {:.2} ns/row",
                recs.len(),
                if m > 0 { e as f64 / m as f64 } else { f64::NAN }
            );
        }
    }
    std::fs::write(&a[6], out).expect("write OUT.tsv");
}

/// Every order of `0..k`, lexicographically (for k = 3: 012, 021, 102, 120, 201, 210)
fn permutations(k: usize) -> Vec<Vec<usize>> {
    if k == 0 {
        return vec![Vec::new()];
    }
    let mut all = Vec::new();
    for first in 0..k {
        for rest in permutations(k - 1) {
            let mut p = vec![first];
            p.extend(rest.into_iter().map(|x| if x >= first { x + 1 } else { x }));
            all.push(p);
        }
    }
    all
}

#[test]
fn the_six_orders() {
    assert_eq!(
        permutations(3),
        [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0]
        ]
    );
}
