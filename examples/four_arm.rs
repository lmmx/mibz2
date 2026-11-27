//! (research only, `scan-oracle`) The four-arm whole-decoder oracle, scanner × emission, for the
//! bench-four-arm diagnostic (docs/journal/2026-11-24-four-arm-registration.md):
//!
//!     four_arm FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv
//!
//! WORKERS is comma-separated. Every decode is production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, the window `SLOTS_PER_THREAD` a worker),
//! FILE read into memory once; the arm is switched between decodes in this one process:
//!
//! | arm | scanner (`scan_oracle`) | Open's emission (`supplied_codes`) |
//! |---|---|---|
//! | A | `scan`: as shipped | `open`: as shipped |
//! | B | `scan` | `supplied`: the codes in output order (bench-supplied-codes' arm B) |
//! | C | `replay`: the recorded symbols through the job queue | `open` |
//! | D | `replay` | `supplied` |
//!
//! 1. **The oracle pass** (outside every clock): one decode at the largest worker count with both
//!    sides recording: each block's symbols by its index (`scan_oracle`), each open block's
//!    codes in output order by its key (`supplied_codes`). Its output's size and CRC-32 are the
//!    reference. A supplied-codes key recorded twice with different codes, or a block index
//!    recorded twice with different symbols, stops the harness.
//! 2. **The rounds.** Round r decodes FILE once in each cell (workers, arm): the worker counts in
//!    WORKERS' order rotated by `(r / 4) mod len(WORKERS)`; within each count, the four arms in
//!    row `r mod 4` of a Williams square (`ORDER`: each arm in each position, and each arm
//!    before each other arm once, every four rounds).
//! 3. **The check** at the end of each round (outside every clock): one decode at the largest
//!    worker count with the scanner replayed and the emission in supplied-codes' `check` mode
//!    (every open block's supplied bytes compared with the walk's), its output's size and CRC-32
//!    compared with the reference.
//!
//! SETTLE_S seconds of sleep precede each decode. The output is read and discarded (hashed in the
//! oracle pass and the checks alone); the decoder checks every block's CRC. Each timed decode's
//! peak resident memory: the process's high-water mark reset before it (`/proc/self/clear_refs`)
//! and read after (`VmHWM`), the store included; 0 where the kernel refuses the reset.
//! OUT.tsv: a `meta` line (the stores, the reference), a `run` line a timed decode, a `check`
//! line a round.
use mibz2::scan_oracle as scan;
use mibz2::supplied_codes::{self as sup, Mode};
use std::io::Read;
use std::time::{Duration, Instant};

const COLS: &str = "kind\tround\tworkers\tarm\torder\twall_s\tbytes\tcrc\tpeak_kb\treplayed\topen_blocks\tdeliver_ns\tconsume_ns\tmismatches\tdetail\n";

/// The arms: (name, scanner, emission)
const ARMS: [(&str, scan::Mode, Mode); 4] = [
    ("A", scan::Mode::Scan, Mode::Open),
    ("B", scan::Mode::Scan, Mode::Supplied),
    ("C", scan::Mode::Replay, Mode::Open),
    ("D", scan::Mode::Replay, Mode::Supplied),
];

/// A Williams square for four arms: row r's order (each arm once a position, each ordered pair
/// of arms adjacent once)
const ORDER: [[usize; 4]; 4] = [[0, 1, 3, 2], [1, 2, 0, 3], [2, 3, 1, 0], [3, 0, 2, 1]];

/// The process's peak resident memory reset (Linux: `5` to clear_refs), and whether it took
fn reset_peak() -> bool {
    std::fs::write("/proc/self/clear_refs", "5").is_ok()
}

/// The process's peak resident memory since the last reset, in kB (`VmHWM`), or 0
fn peak_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
        })
        .unwrap_or(0)
}

struct Decoded {
    wall: f64,
    bytes: u64,
    crc: u32,
    peak: u64,
}

fn decode(
    data: &'static [u8],
    w: usize,
    scanner: scan::Mode,
    emission: Mode,
    hash: bool,
    buf: &mut [u8],
) -> Decoded {
    let _ = sup::take();
    let _ = scan::take_replayed();
    sup::set_mode(emission);
    sup::set_condition(false);
    scan::set_mode(scanner);
    let k = mibz2::Kernels::detect();
    let reset = reset_peak();
    let s = Instant::now();
    let mut r =
        mibz2::BzReader::parallel_workers(data, w, mibz2::parallel::SLOTS_PER_THREAD * w, k, true);
    let mut bytes = 0u64;
    let mut crc = 0u32;
    loop {
        match r.read(buf) {
            Ok(0) => break,
            Ok(x) => {
                bytes += x as u64;
                if hash {
                    crc = mibz2::crc::update_with(crc, &buf[..x], k);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    drop(r);
    let wall = s.elapsed().as_secs_f64();
    let peak = if reset { peak_kb() } else { 0 };
    sup::set_mode(Mode::Open);
    scan::set_mode(scan::Mode::Scan);
    Decoded {
        wall,
        bytes,
        crc,
        peak,
    }
}

/// Round `round`'s cells (workers, arm index), in order
fn cells(round: usize, workers: &[usize]) -> Vec<(usize, usize)> {
    let rot = (round / 4) % workers.len();
    let mut cells = Vec::new();
    for i in 0..workers.len() {
        let w = workers[(i + rot) % workers.len()];
        for &a in &ORDER[round % 4] {
            cells.push((w, a));
        }
    }
    cells
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: four_arm FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.tsv");
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

    // 1. the oracle pass: both sides recording
    sup::clear_store();
    scan::clear_store();
    let r = decode(data, most, scan::Mode::Record, Mode::Record, true, &mut buf);
    let recs = sup::take();
    let (keys, codes, code_bytes) = sup::store_size();
    let collisions = sup::collisions();
    let (blocks, distinct, sym_bytes, conflicts, missing) = scan::store_size();
    let (ref_bytes, ref_crc) = (r.bytes, r.crc);
    out += &format!(
        "meta\t\t{most}\trecord\t\t{:.6}\t{ref_bytes}\t{ref_crc:08x}\t{}\t\t{}\t\t\t{}\tblocks={blocks} distinct={distinct} symbol_bytes={sym_bytes} conflicts={conflicts} missing={missing} code_keys={keys} codes={codes} code_bytes={code_bytes}\n",
        r.wall,
        r.peak,
        recs.iter().filter(|x| !x.refused).count(),
        collisions + conflicts + missing,
    );
    eprintln!(
        "oracle pass: {:.3} s, {ref_bytes} bytes (crc {ref_crc:08x}), {blocks} blocks ({distinct} distinct, {sym_bytes} symbol bytes), {keys} code keys ({code_bytes} bytes); {collisions} collisions, {conflicts} conflicts, {missing} missing",
        r.wall
    );
    if collisions > 0 || conflicts > 0 || missing > 0 || keys == 0 || blocks == 0 {
        std::fs::write(&a[5], out).expect("write OUT.tsv");
        eprintln!("four_arm: the stores are not usable");
        std::process::exit(1);
    }

    // 2. the rounds, and 3. each round's check
    for round in 0..rounds {
        for (o, &(w, arm)) in cells(round, &workers).iter().enumerate() {
            let (name, scanner, emission) = ARMS[arm];
            std::thread::sleep(settle);
            let d = decode(data, w, scanner, emission, false, &mut buf);
            let replayed = scan::take_replayed();
            let recs = sup::take();
            let open = recs.iter().filter(|x| !x.refused);
            let (n_open, deliver, consume) = open.fold((0u64, 0u64, 0u64), |(c, d, e), x| {
                (c + 1, d + x.deliver_ns, e + x.consume_ns)
            });
            out += &format!(
                "run\t{round}\t{w}\t{name}\t{o}\t{:.6}\t{}\t\t{}\t{replayed}\t{n_open}\t{deliver}\t{consume}\t\t\n",
                d.wall, d.bytes, d.peak
            );
            eprintln!(
                "round {round} workers {w} arm {name}: {:.3} s, {} bytes, peak {} kB, {replayed} blocks replayed, {n_open} open",
                d.wall, d.bytes, d.peak
            );
        }
        std::thread::sleep(settle);
        let _ = sup::mismatches();
        let d = decode(data, most, scan::Mode::Replay, Mode::Check, true, &mut buf);
        let replayed = scan::take_replayed();
        let recs = sup::take();
        let bad = sup::mismatches();
        let checked = recs.iter().filter(|r| !r.refused && r.supplied).count();
        let wrong = u64::from(d.bytes != ref_bytes || d.crc != ref_crc || replayed != blocks);
        out += &format!(
            "check\t{round}\t{most}\tcheck\t\t{:.6}\t{}\t{:08x}\t\t{replayed}\t{checked}\t\t\t{}\t\n",
            d.wall,
            d.bytes,
            d.crc,
            bad + wrong
        );
        eprintln!(
            "round {round} check: {} bytes (crc {:08x}), {replayed} blocks replayed, {checked} supplied blocks checked, {bad} mismatches{}",
            d.bytes,
            d.crc,
            if wrong > 0 { ", the output differs" } else { "" }
        );
    }
    std::fs::write(&a[5], out).expect("write OUT.tsv");
}

#[test]
fn the_order_is_balanced() {
    use std::collections::HashMap;
    let w = [1, 6, 10];
    // (count, arm, position) -> rounds; (count, first, second) -> adjacent pairs
    let mut pos: HashMap<(usize, usize, usize), usize> = HashMap::new();
    let mut pairs: HashMap<(usize, usize, usize), usize> = HashMap::new();
    for round in 0..20 {
        let c = cells(round, &w);
        assert_eq!(c.len(), 12);
        for &count in &w {
            let mine: Vec<usize> = c.iter().filter(|x| x.0 == count).map(|x| x.1).collect();
            assert_eq!(mine.len(), 4);
            let mut sorted = mine.clone();
            sorted.sort();
            assert_eq!(sorted, vec![0, 1, 2, 3]);
            for (p, &arm) in mine.iter().enumerate() {
                *pos.entry((count, arm, p)).or_default() += 1;
            }
            for x in mine.windows(2) {
                *pairs.entry((count, x[0], x[1])).or_default() += 1;
            }
        }
    }
    // every arm in every position 5 times in 20 rounds, at every count
    assert_eq!(pos.len(), 3 * 4 * 4);
    assert!(pos.values().all(|&v| v == 5), "{pos:?}");
    // every ordered pair adjacent 5 times (once a square)
    assert_eq!(pairs.len(), 3 * 12);
    assert!(pairs.values().all(|&v| v == 5), "{pairs:?}");
}
