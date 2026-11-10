//! The cooperative waits' accounting, untimed (research; feature `wait-stats`): how often
//! each wait site in src/coop.rs spins, ends inside its spin budget, parks or yields, while
//! the pipelined teams decode FILE. Counts, not times: the one time-derived figure, the
//! spin loop's CPU, is an estimate (the spins times one spin's cost measured here, on an
//! idle thread) and labelled so.
//!
//! ```text
//! cargo run --release --features wait-stats --example wait_audit -- FILE.bz2 \
//!     [--teams 3] [--k 3] [--levers both|park|small|none] [--rounds 3] [--label TEXT]
//! ```
//!
//! Each round is one decode of FILE by `BzReader::parallel_pipelined_levers(File, teams, k,
//! SLOTS_PER_THREAD × teams, Kernels::detect(), levers)` (the default decoder's own
//! constructor at its plan, three teams of three with park and small), drained in 4 MiB
//! reads. The counts are reset before each round and read after it; every round must give
//! the same bytes. One JSON line a round on stdout. Run it beside a deliberate load (busy
//! processes of its own) to compare the counts on an idle and an oversubscribed machine.
//!
//! The process CPU (`/proc/self/stat`'s user and system ticks across the round) is printed
//! for context only: it is a time, local, and not evidence of performance.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use mibz2::wait_stats::{self, SITES};

fn arg<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> T {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(|v| v.parse().ok().expect("a valid value"))
        .unwrap_or(default)
}

/// The process's user + system CPU ticks so far (`/proc/self/stat` fields 14 and 15)
fn cpu_ticks() -> u64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let rest = s.rsplit_once(')').map(|(_, r)| r).unwrap_or("");
    let f: Vec<&str> = rest.split_whitespace().collect();
    // (after the comm: field 3 is f[0], so utime (14) is f[11] and stime (15) f[12])
    let g = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    g(11) + g(12)
}

/// An estimate of one spin's cost (an acquire load and `spin_loop`, as the waits spin), in
/// nanoseconds, on one thread of this machine with nothing to wait for
fn spin_cost_ns() -> f64 {
    let flag = AtomicUsize::new(0);
    let n = 5_000_000u64;
    let mut best = f64::MAX;
    for _ in 0..3 {
        let t = Instant::now();
        for _ in 0..n {
            if flag.load(Ordering::Acquire) != 0 {
                break;
            }
            std::hint::spin_loop();
        }
        best = best.min(t.elapsed().as_nanos() as f64 / n as f64);
    }
    best
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("FILE.bz2").clone();
    const { assert!(wait_stats::ENABLED, "build with --features wait-stats") };
    let teams: usize = arg(&args, "--teams", 3);
    let k: usize = arg(&args, "--k", 3);
    let rounds: usize = arg(&args, "--rounds", 3);
    let levers_name: String = arg(&args, "--levers", "both".to_string());
    let label: String = arg(&args, "--label", String::new());
    let levers = mibz2::parallel::Levers {
        park: levers_name == "park" || levers_name == "both",
        small: levers_name == "small" || levers_name == "both",
        spin2k: false,
    };
    let spin_ns = spin_cost_ns();
    let tick = 0.01; // (USER_HZ is 100 on Linux)
    let mut first: Option<(u64, u64)> = None;
    for round in 0..rounds {
        wait_stats::reset();
        let c0 = cpu_ticks();
        let file = std::fs::File::open(&path)?;
        let slots = mibz2::parallel::SLOTS_PER_THREAD * teams;
        let mut r = mibz2::BzReader::parallel_pipelined_levers(
            file,
            teams,
            k,
            slots,
            mibz2::Kernels::detect(),
            levers,
        );
        let mut buf = vec![0u8; 4 << 20];
        let (mut bytes, mut h) = (0u64, DefaultHasher::new());
        loop {
            let n = r.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.write(&buf[..n]);
            bytes += n as u64;
        }
        drop(r);
        let cpu_s = (cpu_ticks() - c0) as f64 * tick;
        let hash = h.finish();
        match first {
            None => first = Some((bytes, hash)),
            Some(f) => assert_eq!(f, (bytes, hash), "round {round} decoded other bytes"),
        }
        let mut sites = Vec::new();
        let mut all_spins = 0u64;
        for s in SITES {
            let c = wait_stats::get(s);
            all_spins += c.spins;
            sites.push(format!(
                "\"{}\":{{\"waits\":{},\"immediate\":{},\"ended_spinning\":{},\"parked\":{},\
                 \"yielded\":{},\"spins\":{},\"spins_wasted\":{},\"parks\":{},\"yields\":{}}}",
                s.name(),
                c.waits,
                c.immediate,
                c.ended_spinning,
                c.parked,
                c.yielded,
                c.spins,
                c.spins_wasted,
                c.parks,
                c.yields
            ));
        }
        println!(
            "{{\"label\":\"{label}\",\"round\":{round},\"teams\":{teams},\"k\":{k},\
             \"levers\":\"{levers_name}\",\"bytes\":{bytes},\"hash\":\"{hash:016x}\",\
             \"sites\":{{{}}},\"spins_all\":{all_spins},\"spin_ns_estimate\":{spin_ns:.2},\
             \"spin_cpu_s_estimate\":{:.4},\"process_cpu_s_indicative\":{cpu_s:.2}}}",
            sites.join(","),
            all_spins as f64 * spin_ns * 1e-9,
        );
    }
    Ok(())
}
