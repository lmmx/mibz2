//! (research only, `emit-cost`) The open kernel's emit cost per retained row in production's
//! own workers, for the bench-emit-cost diagnostic
//! (docs/journal/2026-11-22-emit-cost-registration.md).
//!
//!     emit_cost FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv
//!
//! Each round decodes FILE once at each worker count of WORKERS (comma-separated, the order
//! rotated round by round) with production's lazy workers (`BzReader::parallel_workers`, lazy:
//! `OpenCells`, the window `SLOTS_PER_THREAD` a worker), every worker decoding its own blocks
//! concurrently, the output read and discarded. The kernel records each block it takes
//! (`mibz2::emit_cost`); OUT.tsv gets one `blk` line per record and one `run` line per decode
//! (its wall time and bytes). SETTLE_S seconds of sleep precede each decode.
use std::io::Read;
use std::time::{Duration, Instant};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: emit_cost FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv");
        std::process::exit(2);
    }
    // (held for the whole run, as the decoder's feeder needs a `'static` source; read once,
    // so no decode reads the disk)
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let workers: Vec<usize> = a[2]
        .split(',')
        .map(|s| s.parse().expect("WORKERS"))
        .collect();
    let rounds: usize = a[3].parse().expect("ROUNDS");
    let settle = Duration::from_secs_f64(a[4].parse().expect("SETTLE_S"));
    let mut out = String::from(
        "kind\tround\tworkers\torder\tseq\tn\tm\tconstruct_ns\temit_ns\tbidi\trefused\twall_s\tbytes\n",
    );
    let k = mibz2::Kernels::detect();
    let mut buf = vec![0u8; 4 << 20];
    for round in 0..rounds {
        for o in 0..workers.len() {
            let w = workers[(o + round) % workers.len()];
            std::thread::sleep(settle);
            let _ = mibz2::emit_cost::take();
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
            let recs = mibz2::emit_cost::take();
            for (i, x) in recs.iter().enumerate() {
                out += &format!(
                    "blk\t{round}\t{w}\t{o}\t{i}\t{}\t{}\t{}\t{}\t{}\t{}\t\t\n",
                    x.n, x.m, x.construct_ns, x.emit_ns, x.bidi as u8, x.refused as u8
                );
            }
            out += &format!("run\t{round}\t{w}\t{o}\t\t\t\t\t\t\t\t{wall:.6}\t{bytes}\n");
            let open = recs.iter().filter(|x| !x.refused);
            let (m, e) = open.fold((0u64, 0u64), |(m, e), x| {
                (m + u64::from(x.m), e + x.emit_ns)
            });
            eprintln!(
                "round {round} workers {w}: {wall:.3} s, {bytes} bytes, {} records, emit {:.2} ns/row",
                recs.len(),
                if m > 0 { e as f64 / m as f64 } else { f64::NAN }
            );
        }
    }
    std::fs::write(&a[5], out).expect("write OUT.tsv");
}
