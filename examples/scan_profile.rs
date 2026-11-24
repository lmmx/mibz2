//! (research only, `scan-profile`) The parallel decoder's serial scanner profiled, work against
//! waiting, by component, for the bench-scan-profile diagnostic
//! (docs/journal/2026-11-24-scan-profile-registration.md):
//!
//!     scan_profile FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.jsonl
//!
//! WORKERS is comma-separated. Every decode is production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, 4 blocks a worker), FILE read into memory
//! once, the output read and discarded (the decoder checks every block's CRC and the stream's).
//! Round r decodes FILE twice at each count, once with the timers on (`scan_profile::set_on`)
//! and once off, the counts in WORKERS' order rotated by `r mod len(WORKERS)`, the timed decode
//! first in even rounds. SETTLE_S seconds of sleep precede each decode.
//!
//! Every decode, timed or not, also records the process's memory and scheduling over it: its
//! peak resident memory (the high-water mark reset before it, `/proc/self/clear_refs`, and read
//! after, `VmHWM`), its time-averaged resident memory (`/proc/self/statm` read every 5 ms by a
//! sampler thread, the harness's, in both kinds of decode), and from `getrusage` (RUSAGE_SELF,
//! every thread's, exited ones included) its user and system CPU time, minor and major page
//! faults, and voluntary and involuntary context switches.
//!
//! OUT.jsonl, one line a decode: its round, count, whether timed, order, wall, bytes, and those
//! figures. Timed, also:
//! - `n` its blocks, and `indices_ok` whether they are one each of 0..n-1 (checked here);
//! - `sums`, each of `BLOCK_FIELDS` summed over its blocks (`scan_profile::BlockRec`);
//! - every worker's record (`WorkerRec`: jobs, idle, life);
//! - in the first `PER_BLOCK_ROUNDS` rounds only, every block's record as an array in
//!   `BLOCK_FIELDS`' order (the file stays small enough for the bundle at every count).
//!
//! OUT.tsv beside it (OUT.jsonl's name with `.tsv`): one row a decode (`TSV_COLS`), for the
//! journal's math-targets project.
use mibz2::scan_profile as sp;
use std::fmt::Write as _;
use std::io::Read;
use std::time::{Duration, Instant};

/// The rounds whose timed decodes keep every block's record
const PER_BLOCK_ROUNDS: usize = 2;

/// The per-decode table's columns (OUT.tsv, beside OUT.jsonl): one row a (workers, round,
/// timers on or off); the scanner's and workers' totals (seconds) in the timed rows alone
const TSV_COLS: &str = "workers\tround\ttimed\torder\twall_s\tbytes\trss_peak_kb\trss_mean_kb\trss_samples\tuser_s\tsys_s\tminflt\tmajflt\tnvcsw\tnivcsw\tblocks\tscanner_work_s\tscanner_slot_wait_s\tscanner_input_wait_s\tworkers_busy_s\tworkers_idle_s\n";

/// A block record's fields, in the arrays' order
const BLOCK_FIELDS: [&str; 16] = [
    "index",
    "bits",
    "syms",
    "selectors",
    "switches",
    "tables",
    "slot_wait_ns",
    "read_ns",
    "input_wait_ns",
    "header_ns",
    "bitmap_ns",
    "selectors_ns",
    "tables_ns",
    "prep_ns",
    "decode_ns",
    "send_ns",
];

#[repr(C)]
#[derive(Default)]
struct Rusage {
    utime: [i64; 2],
    stime: [i64; 2],
    /// ru_maxrss, ixrss, idrss, isrss, minflt, majflt, nswap, inblock, oublock, msgsnd,
    /// msgrcv, nsignals, nvcsw, nivcsw
    rest: [i64; 14],
}

extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}

/// This process's resource use so far, every thread's, live and exited (RUSAGE_SELF): (user
/// s, system s, minor faults, major faults, voluntary and involuntary context switches)
fn usage() -> (f64, f64, i64, i64, i64, i64) {
    let mut r = Rusage::default();
    // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
    unsafe { getrusage(0, &mut r) };
    let t = |tv: [i64; 2]| tv[0] as f64 + tv[1] as f64 * 1e-6;
    (
        t(r.utime),
        t(r.stime),
        r.rest[4],
        r.rest[5],
        r.rest[12],
        r.rest[13],
    )
}

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

/// The process's resident memory now, in kB (`/proc/self/statm`'s second field, in pages)
fn rss_kb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = s.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4)
}

/// The interval of the resident-memory sampler
const SAMPLE: Duration = Duration::from_millis(5);

/// A decode's figures: its wall, bytes, and the process's memory and scheduling over it
struct Decoded {
    wall: f64,
    bytes: u64,
    /// peak resident memory (kB; 0 where the reset is refused), and its time average (kB) over
    /// `samples` reads at `SAMPLE` apart
    peak_kb: u64,
    mean_kb: f64,
    samples: u64,
    user_s: f64,
    sys_s: f64,
    minflt: i64,
    majflt: i64,
    nvcsw: i64,
    nivcsw: i64,
}

fn decode(data: &'static [u8], w: usize, timed: bool, buf: &mut [u8]) -> Decoded {
    let _ = sp::take();
    sp::set_on(timed);
    let k = mibz2::Kernels::detect();
    // the sampler: the process's resident memory every SAMPLE until the decode ends
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let st = stop.clone();
    let reset = reset_peak();
    let u0 = usage();
    let sampler = std::thread::spawn(move || {
        let (mut sum, mut n) = (0u64, 0u64);
        while !st.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(kb) = rss_kb() {
                sum += kb;
                n += 1;
            }
            std::thread::sleep(SAMPLE);
        }
        (sum, n)
    });
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
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (sum, n) = sampler.join().expect("the sampler");
    let u1 = usage();
    sp::set_on(false);
    Decoded {
        wall,
        bytes,
        peak_kb: if reset { peak_kb() } else { 0 },
        mean_kb: if n > 0 { sum as f64 / n as f64 } else { 0.0 },
        samples: n,
        user_s: u1.0 - u0.0,
        sys_s: u1.1 - u0.1,
        minflt: u1.2 - u0.2,
        majflt: u1.3 - u0.3,
        nvcsw: u1.4 - u0.4,
        nivcsw: u1.5 - u0.5,
    }
}

/// Round `round`'s cells (workers, timed), in order
fn cells(round: usize, workers: &[usize]) -> Vec<(usize, bool)> {
    let rot = round % workers.len();
    let first = round % 2 == 0;
    let mut cells = Vec::new();
    for i in 0..workers.len() {
        let w = workers[(i + rot) % workers.len()];
        cells.push((w, first));
        cells.push((w, !first));
    }
    cells
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: scan_profile FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.jsonl");
        std::process::exit(2);
    }
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let workers: Vec<usize> = a[2]
        .split(',')
        .map(|s| s.parse().expect("WORKERS"))
        .collect();
    let rounds: usize = a[3].parse().expect("ROUNDS");
    let settle = Duration::from_secs_f64(a[4].parse().expect("SETTLE_S"));
    let mut buf = vec![0u8; 4 << 20];
    let mut out = String::new();
    let mut tsv = String::from(TSV_COLS);
    writeln!(
        out,
        "{{\"kind\":\"meta\",\"block_fields\":[{}],\"input_bytes\":{}}}",
        BLOCK_FIELDS.map(|f| format!("\"{f}\"")).join(","),
        data.len()
    )
    .unwrap();
    for round in 0..rounds {
        for (o, &(w, timed)) in cells(round, &workers).iter().enumerate() {
            std::thread::sleep(settle);
            let d = decode(data, w, timed, &mut buf);
            let (wall, bytes) = (d.wall, d.bytes);
            let (blocks, ws) = sp::take();
            let mut line = format!(
                "{{\"kind\":\"decode\",\"round\":{round},\"workers\":{w},\"timed\":{timed},\"order\":{o},\"wall_s\":{wall:.6},\"bytes\":{bytes},\"peak_kb\":{},\"mean_kb\":{:.1},\"samples\":{},\"user_s\":{:.6},\"sys_s\":{:.6},\"minflt\":{},\"majflt\":{},\"nvcsw\":{},\"nivcsw\":{}",
                d.peak_kb, d.mean_kb, d.samples, d.user_s, d.sys_s, d.minflt, d.majflt, d.nvcsw, d.nivcsw
            );
            // the per-decode table (OUT.tsv): its memory and scheduling, and timed, the scanner's
            // and the workers' totals
            let sum = |f: fn(&sp::BlockRec) -> u64| blocks.iter().map(f).sum::<u64>();
            let (work, slot, inw) = (
                sum(|b| b.read_ns - b.input_wait_ns + b.send_ns),
                sum(|b| b.slot_wait_ns),
                sum(|b| b.input_wait_ns),
            );
            let (busy, idle) = ws.iter().fold((0u64, 0u64), |(b, i), x| {
                (b + x.life_ns - x.idle_ns, i + x.idle_ns)
            });
            let opt = |v: u64| {
                if timed {
                    format!("{:.6}", v as f64 / 1e9)
                } else {
                    String::new()
                }
            };
            writeln!(
                tsv,
                "{w}\t{round}\t{}\t{o}\t{wall:.6}\t{bytes}\t{}\t{:.1}\t{}\t{:.6}\t{:.6}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                u8::from(timed),
                d.peak_kb,
                d.mean_kb,
                d.samples,
                d.user_s,
                d.sys_s,
                d.minflt,
                d.majflt,
                d.nvcsw,
                d.nivcsw,
                if timed { blocks.len().to_string() } else { String::new() },
                opt(work),
                opt(slot),
                opt(inw),
                opt(busy),
                opt(idle),
            )
            .unwrap();
            if timed {
                let n = blocks.len();
                let mut seen = vec![false; n];
                let mut indices_ok = true;
                for b in &blocks {
                    match seen.get_mut(b.index as usize) {
                        Some(s) if !*s => *s = true,
                        _ => indices_ok = false,
                    }
                }
                let rows: Vec<[u64; 16]> = blocks
                    .iter()
                    .map(|b| {
                        [
                            b.index,
                            b.bits,
                            b.syms,
                            b.selectors,
                            b.switches,
                            b.tables,
                            b.slot_wait_ns,
                            b.read_ns,
                            b.input_wait_ns,
                            b.phase_ns[0],
                            b.phase_ns[1],
                            b.phase_ns[2],
                            b.phase_ns[3],
                            b.phase_ns[4],
                            b.phase_ns[5],
                            b.send_ns,
                        ]
                    })
                    .collect();
                let mut sums = [0u64; 16];
                for v in &rows {
                    for (s, x) in sums.iter_mut().zip(v) {
                        *s += x;
                    }
                }
                write!(
                    line,
                    ",\"n\":{n},\"indices_ok\":{indices_ok},\"sums\":[{}]",
                    sums.map(|x| x.to_string()).join(",")
                )
                .unwrap();
                if round < PER_BLOCK_ROUNDS {
                    line += ",\"blocks\":[";
                    for (i, v) in rows.iter().enumerate() {
                        if i > 0 {
                            line.push(',');
                        }
                        write!(line, "[{}]", v.map(|x| x.to_string()).join(",")).unwrap();
                    }
                    line.push(']');
                }
                line += ",\"workers_rec\":[";
                for (i, x) in ws.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                    }
                    write!(line, "[{},{},{}]", x.jobs, x.idle_ns, x.life_ns).unwrap();
                }
                line.push(']');
            }
            line += "}\n";
            out += &line;
            let busy: u64 = blocks.iter().map(|b| b.read_ns + b.send_ns).sum();
            let slot: u64 = blocks.iter().map(|b| b.slot_wait_ns).sum();
            eprintln!(
                "round {round} workers {w} {}: {wall:.3} s, {bytes} bytes{}",
                if timed { "timed" } else { "untimed" },
                if timed {
                    format!(
                        ", {} blocks, scanner busy {:.3} s, slot wait {:.3} s, worker idle {:.3} s",
                        blocks.len(),
                        busy as f64 / 1e9,
                        slot as f64 / 1e9,
                        ws.iter().map(|x| x.idle_ns).sum::<u64>() as f64 / 1e9
                    )
                } else {
                    String::new()
                }
            );
        }
    }
    std::fs::write(&a[5], out).expect("write OUT.jsonl");
    let tsv_path = std::path::Path::new(&a[5]).with_extension("tsv");
    std::fs::write(&tsv_path, tsv).expect("write OUT.tsv");
}

#[test]
fn the_order_is_balanced() {
    let w = [1, 6, 10];
    let mut first = std::collections::HashMap::<(usize, bool), usize>::new();
    let mut pos = std::collections::HashMap::<(usize, usize), usize>::new();
    for round in 0..6 {
        let c = cells(round, &w);
        assert_eq!(c.len(), 6);
        for (i, &(count, timed)) in c.iter().enumerate() {
            if i % 2 == 0 {
                *first.entry((count, timed)).or_default() += 1;
                *pos.entry((count, i / 2)).or_default() += 1;
            }
        }
    }
    // each count timed first in half the rounds, and in each count position twice in six
    assert!(first.values().all(|&v| v == 3), "{first:?}");
    assert!(pos.values().all(|&v| v == 2), "{pos:?}");
}
