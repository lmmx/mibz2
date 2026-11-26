//! (research only, `supplied-codes`) The open kernel's emitter handed its word codes in output
//! order (arm B, `supplied`) against the kernel as shipped (arm A, `open`), in production's own
//! workers, for the bench-supplied-codes diagnostic
//! (docs/journal/2026-11-24-supplied-codes-registration.md):
//!
//!     supplied_codes FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv
//!
//! WORKERS is comma-separated. Every decode is production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, the window `SLOTS_PER_THREAD` a worker),
//! FILE read into memory once; the mode is switched between decodes in this one process
//! (`mibz2::supplied_codes::set_mode`).
//!
//! 1. **The oracle pass** (outside every clock): one decode in `record` mode at the largest
//!    worker count, each open block's marked-cycle codes recorded into the store by its key.
//!    A key recorded twice with different codes is a collision: the harness stops.
//! 2. **The rounds.** Round r decodes FILE once in each cell (workers, arm, cond), arm in
//!    {open, supplied}, cond 0 (the primary receipt: nothing conditioned) or 1 (the diagnostic
//!    receipt: both arms' consumer inputs read once before the consumer's clock), in this
//!    order:
//!    - the worker counts in WORKERS' order rotated by `(r / 4) mod len(WORKERS)`;
//!    - within each count, the pair cond 0 first when `(r / 2)` is even, cond 1 first when odd;
//!    - within each pair, `open` first when r is even, `supplied` first when odd.
//!
//!    Over 20 rounds each count runs each arm first 10 times, and each pair first 10 times.
//! 3. **The check** at the end of each round (outside every clock): one decode in `check` mode
//!    at the largest worker count, every open block's arm-B bytes compared with arm A's.
//!
//! SETTLE_S seconds of sleep precede each decode. The output is read and discarded; the decoder
//! checks every block's CRC and the stream's. OUT.tsv: one `meta` line (the store: `n` its
//! distinct keys, `m` the codes it holds, `scratch` their bytes, `mismatches` the oracle pass's
//! collisions, `bytes` the oracle decode's output), one `blk` line a block record, one `run`
//! line a timed decode (its wall time and bytes), one `check` line a round (`mismatches`, and
//! `supplied` the blocks checked).
use mibz2::supplied_codes::{self as sup, Mode};
use std::io::Read;
use std::time::{Duration, Instant};

const COLS: &str = "kind\tround\tworkers\tarm\tcond\torder\tseq\tn\tm\tcycle\tbidi\trefused\tsupplied\tconstruct_ns\tdeliver_ns\tcond_ns\tconsume_ns\tscratch\tid\twall_s\tbytes\tmismatches\n";

fn decode(data: &'static [u8], w: usize, mode: Mode, cond: bool, buf: &mut [u8]) -> (f64, u64) {
    let _ = sup::take();
    sup::set_mode(mode);
    sup::set_condition(cond);
    let k = mibz2::Kernels::detect();
    let s = Instant::now();
    let mut r =
        mibz2::BzReader::parallel_workers(data, w, mibz2::parallel::SLOTS_PER_THREAD * w, k, true);
    let mut bytes = 0u64;
    loop {
        match r.read(buf) {
            Ok(0) => break,
            Ok(x) => bytes += x as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    drop(r);
    let wall = s.elapsed().as_secs_f64();
    sup::set_mode(Mode::Open);
    sup::set_condition(false);
    (wall, bytes)
}

/// Round `round`'s cells (workers, cond, arm), in order
fn cells(round: usize, workers: &[usize]) -> Vec<(usize, bool, Mode)> {
    let rot = (round / 4) % workers.len();
    let conds = if (round / 2) % 2 == 0 {
        [false, true]
    } else {
        [true, false]
    };
    let arms = if round % 2 == 0 {
        [Mode::Open, Mode::Supplied]
    } else {
        [Mode::Supplied, Mode::Open]
    };
    let mut cells = Vec::new();
    for i in 0..workers.len() {
        let w = workers[(i + rot) % workers.len()];
        for c in conds {
            for m in arms {
                cells.push((w, c, m));
            }
        }
    }
    cells
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: supplied_codes FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv");
        std::process::exit(2);
    }
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let workers: Vec<usize> = a[2]
        .split(',')
        .map(|s| s.parse().expect("WORKERS"))
        .collect();
    let rounds: usize = a[3].parse().expect("ROUNDS");
    let settle = Duration::from_secs_f64(a[4].parse().expect("SETTLE_S"));
    let most = *workers.iter().max().expect("WORKERS");
    let mut buf = vec![0u8; 4 << 20];
    let mut out = String::from(COLS);

    // 1. the oracle pass
    sup::clear_store();
    let (wall, bytes) = decode(data, most, Mode::Record, false, &mut buf);
    let recs = sup::take();
    let (keys, codes, store_bytes) = sup::store_size();
    let collisions = sup::collisions();
    out += &format!(
        "meta\t\t{most}\trecord\t0\t\t\t{keys}\t{codes}\t\t\t\t\t\t\t\t\t{store_bytes}\t\t{wall:.6}\t{bytes}\t{collisions}\n"
    );
    eprintln!(
        "oracle pass: {wall:.3} s, {bytes} bytes, {} open blocks, {keys} distinct keys, {codes} codes ({store_bytes} bytes), {collisions} collisions",
        recs.iter().filter(|r| !r.refused).count()
    );
    if collisions > 0 || keys == 0 {
        std::fs::write(&a[5], out).expect("write OUT.tsv");
        eprintln!("supplied_codes: the store is not usable ({collisions} collisions, {keys} keys)");
        std::process::exit(1);
    }

    // 2. the rounds, and 3. each round's check
    for round in 0..rounds {
        let cells = cells(round, &workers);
        for (o, &(w, cond, mode)) in cells.iter().enumerate() {
            std::thread::sleep(settle);
            let (wall, bytes) = decode(data, w, mode, cond, &mut buf);
            let recs = sup::take();
            let name = mode.name();
            let c = u8::from(cond);
            for (i, x) in recs.iter().enumerate() {
                out += &format!(
                    "blk\t{round}\t{w}\t{}\t{}\t{o}\t{i}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:016x}\t\t\t\n",
                    x.mode.name(),
                    u8::from(x.cond),
                    x.n,
                    x.m,
                    x.cycle,
                    u8::from(x.bidi),
                    u8::from(x.refused),
                    u8::from(x.supplied),
                    x.construct_ns,
                    x.deliver_ns,
                    x.cond_ns,
                    x.consume_ns,
                    x.scratch,
                    x.id
                );
            }
            out += &format!(
                "run\t{round}\t{w}\t{name}\t{c}\t{o}\t\t\t\t\t\t\t\t\t\t\t\t\t\t{wall:.6}\t{bytes}\t\n"
            );
            let open = recs.iter().filter(|x| !x.refused);
            let (m, d, e) = open.fold((0u64, 0u64, 0u64), |(m, d, e), x| {
                (m + u64::from(x.m), d + x.deliver_ns, e + x.consume_ns)
            });
            eprintln!(
                "round {round} workers {w} {name} cond {c}: {wall:.3} s, {bytes} bytes, {} records, deliver {:.2} + consume {:.2} ns/row",
                recs.len(),
                d as f64 / m.max(1) as f64,
                e as f64 / m.max(1) as f64
            );
        }
        std::thread::sleep(settle);
        let _ = sup::mismatches();
        let (wall, bytes) = decode(data, most, Mode::Check, false, &mut buf);
        let recs = sup::take();
        let bad = sup::mismatches();
        let checked = recs.iter().filter(|r| !r.refused && r.supplied).count();
        out += &format!(
            "check\t{round}\t{most}\tcheck\t0\t\t\t\t\t\t\t\t{checked}\t\t\t\t\t\t\t{wall:.6}\t{bytes}\t{bad}\n"
        );
        eprintln!("round {round} check: {bytes} bytes, {checked} blocks checked, {bad} mismatches");
    }
    std::fs::write(&a[5], out).expect("write OUT.tsv");
}

#[test]
fn the_order_is_balanced() {
    use std::collections::HashMap;
    let w = [1, 10];
    // (count, cond) -> rounds with `open` first; count -> rounds with cond 0 first
    let mut open_first: HashMap<(usize, bool), usize> = HashMap::new();
    let mut plain_first: HashMap<usize, usize> = HashMap::new();
    for round in 0..20 {
        let c = cells(round, &w);
        assert_eq!(c.len(), 8);
        for &count in &w {
            let mine: Vec<_> = c.iter().filter(|x| x.0 == count).collect();
            assert_eq!(mine.len(), 4);
            *plain_first.entry(count).or_default() += usize::from(!mine[0].1);
            for cond in [false, true] {
                let pair: Vec<_> = mine.iter().filter(|x| x.1 == cond).collect();
                assert_eq!(pair.len(), 2);
                assert_ne!(pair[0].2, pair[1].2);
                *open_first.entry((count, cond)).or_default() +=
                    usize::from(pair[0].2 == Mode::Open);
            }
        }
    }
    assert!(open_first.values().all(|&v| v == 10), "{open_first:?}");
    assert!(plain_first.values().all(|&v| v == 10), "{plain_first:?}");
}
