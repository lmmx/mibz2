//! (research only, `squared-trace`) The open kernel's construct, square and emit per block in
//! production's own workers, under each of MONODROMY record 93's arms, for the
//! bench-emit-cost-sq diagnostic (docs/journal/2026-11-23-emit-cost-sq-registration.md).
//!
//!     emit_cost_sq FILE.bz2 WORKERS ARMS ROUNDS SETTLE_S OUT.tsv
//!
//! WORKERS and ARMS are comma-separated (ARMS of `base`, `sq`, `sq-shadow`). The cells are
//! (workers, arm), workers-major in the order given; each round decodes FILE once in every
//! cell, the order rotated by one cell a round, with production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, the window `SLOTS_PER_THREAD` a worker),
//! the output read and discarded (every block's CRC and the stream's are checked by the
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
    let cells: Vec<(usize, Arm)> = workers
        .iter()
        .flat_map(|&w| arms.iter().map(move |&arm| (w, arm)))
        .collect();
    let mut out = String::from(COLS);
    let k = mibz2::Kernels::detect();
    let mut buf = vec![0u8; 4 << 20];
    for round in 0..rounds {
        for o in 0..cells.len() {
            let (w, arm) = cells[(o + round) % cells.len()];
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
