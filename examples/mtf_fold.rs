//! Research only: the MTF-into-construction fold kill-test harness (`bench-mtf-fold`).
//!
//! Frozen registration: `docs/journal/2026-11-24-algorithm-level/mtf-fold/registration.md` §3.
//!
//! Two modes:
//! - `--calibrate <iters> <repeats>`: runs a dependent multiply-add loop (pinned externally via
//!   `taskset -c <cpu>`) and prints JSON timing repeats for choosing the scanner's fastest core.
//! - `--one <corpus.bz2> --arm <none|F|S|FS|SM|SC> --workers <k> [--prespin-ms 100]
//!   [--scanner-cpu N] [--trace]`: runs one fresh-process measurement and prints one JSON line.

use std::io::Read;
use std::time::{Duration, Instant};

use mibz2::ideal::{self, construction as ic, mtf as im};
use mibz2::parallel::SLOTS_PER_THREAD;
use mibz2::runs::fold;
use mibz2::scan_oracle;
use mibz2::{BzReader, Kernels};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Arm {
    name: &'static str,
    ideal_s: bool,
    ideal_m: bool,
    ideal_c: bool,
    fold: bool,
}

impl Arm {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "none" => Ok(Self {
                name: "none",
                ideal_s: false,
                ideal_m: false,
                ideal_c: false,
                fold: false,
            }),
            "F" => Ok(Self {
                name: "F",
                ideal_s: false,
                ideal_m: false,
                ideal_c: false,
                fold: true,
            }),
            "S" => Ok(Self {
                name: "S",
                ideal_s: true,
                ideal_m: false,
                ideal_c: false,
                fold: false,
            }),
            "FS" => Ok(Self {
                name: "FS",
                ideal_s: true,
                ideal_m: false,
                ideal_c: false,
                fold: true,
            }),
            "SM" => Ok(Self {
                name: "SM",
                ideal_s: true,
                ideal_m: true,
                ideal_c: false,
                fold: false,
            }),
            "SC" => Ok(Self {
                name: "SC",
                ideal_s: true,
                ideal_m: false,
                ideal_c: true,
                fold: false,
            }),
            other => Err(format!(
                "unknown arm {other:?} (expected one of: none, F, S, FS, SM, SC)"
            )),
        }
    }

    fn apply(self) {
        scan_oracle::set_mode(if self.ideal_s {
            scan_oracle::Mode::Replay
        } else {
            scan_oracle::Mode::Scan
        });
        im::set_mode(if self.ideal_m {
            ideal::Mode::Replay
        } else {
            ideal::Mode::Real
        });
        ic::set_mode(if self.ideal_c {
            ideal::Mode::Replay
        } else {
            ideal::Mode::Real
        });
        fold::set_enabled(self.fold);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage_exit("missing arguments");
    }
    if args[0] == "--calibrate" {
        if args.len() != 3 {
            usage_exit("--calibrate expects <iters> <repeats>");
        }
        let iters: u64 = args[1]
            .parse::<u64>()
            .unwrap_or_else(|e| usage_exit(&e.to_string()));
        let repeats: usize = args[2]
            .parse::<usize>()
            .unwrap_or_else(|e| usage_exit(&e.to_string()));
        calibrate(iters, repeats);
        return;
    }

    let mut corpus: Option<String> = None;
    let mut arm_name: Option<String> = None;
    let mut workers: usize = 1;
    let mut prespin_ms: u64 = 100;
    let mut scanner_cpu: Option<usize> = None;
    let mut want_trace = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--one" => {
                i += 1;
                corpus = args.get(i).cloned();
            }
            "--arm" => {
                i += 1;
                arm_name = args.get(i).cloned();
            }
            "--workers" | "-k" => {
                i += 1;
                workers = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage_exit("invalid --workers"));
            }
            "--prespin-ms" => {
                i += 1;
                prespin_ms = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage_exit("invalid --prespin-ms"));
            }
            "--scanner-cpu" => {
                i += 1;
                let v = args
                    .get(i)
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| usage_exit("invalid --scanner-cpu"));
                scanner_cpu = Some(v);
            }
            "--trace" => {
                want_trace = true;
            }
            other => usage_exit(&format!("unknown argument {other:?}")),
        }
        i += 1;
    }

    let corpus = corpus.unwrap_or_else(|| usage_exit("missing --one <corpus.bz2>"));
    let arm_str = arm_name.unwrap_or_else(|| usage_exit("missing --arm <none|F|S|FS|SM|SC>"));
    let arm = Arm::parse(&arm_str).unwrap_or_else(|e| usage_exit(&e));
    if workers == 0 {
        usage_exit("--workers must be >= 1");
    }

    run_one(&corpus, arm, workers, prespin_ms, scanner_cpu, want_trace);
}

fn usage_exit(msg: &str) -> ! {
    eprintln!("mtf_fold: {msg}");
    eprintln!("usage:");
    eprintln!("  mtf_fold --calibrate <iters> <repeats>");
    eprintln!(
        "  mtf_fold --one <corpus.bz2> --arm <none|F|S|FS|SM|SC> --workers <k> \
         [--prespin-ms 100] [--scanner-cpu N] [--trace]"
    );
    std::process::exit(2);
}

/// Core calibration pacing loop (same multiply-add chain as `examples/character.rs`).
fn calibrate(iters: u64, repeats: usize) {
    #[inline(never)]
    fn spin(n: u64) {
        let mut x = std::hint::black_box(0x9E37_79B9_7F4A_7C15u64);
        for _ in 0..n {
            x = x
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
        }
        std::hint::black_box(x);
    }
    let mut rows = Vec::with_capacity(repeats);
    for r in 0..repeats {
        let t = Instant::now();
        spin(iters);
        rows.push((r, t.elapsed().as_nanos()));
    }
    let body: Vec<String> = rows.iter().map(|(r, ns)| format!("[{r},{ns}]")).collect();
    println!("{{\"iters\":{iters},\"repeats\":[{}]}}", body.join(","));
}

/// Standard warm-up: one thread pinned to each online CPU spinning until a shared deadline.
fn prespin(ms: u64) -> Option<(u64, usize, f64)> {
    if ms == 0 {
        return None;
    }
    let cpus: Vec<usize> = std::fs::read_to_string("/sys/devices/system/cpu/online")
        .ok()
        .map(|s| {
            s.trim()
                .split(',')
                .filter(|x| !x.is_empty())
                .flat_map(|r| match r.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap_or(0)..=b.parse().unwrap_or(0)).collect(),
                    None => vec![r.parse().unwrap_or(0)],
                })
                .collect()
        })
        .unwrap_or_else(|| {
            (0..std::thread::available_parallelism().map_or(1, |n| n.get())).collect()
        });
    let t0 = Instant::now();
    let until = t0 + Duration::from_millis(ms);
    let hs: Vec<_> = cpus
        .iter()
        .copied()
        .map(|c| {
            std::thread::spawn(move || {
                #[cfg(target_os = "linux")]
                {
                    extern "C" {
                        fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
                    }
                    let mut mask = [0u64; 16];
                    if c < mask.len() * 64 {
                        mask[c / 64] |= 1u64 << (c % 64);
                        // SAFETY: pid 0 targets this thread; `mask` is valid for its byte size.
                        unsafe {
                            sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr());
                        }
                    }
                }
                let _ = c;
                let mut x = 0u64;
                while Instant::now() < until {
                    for _ in 0..4096 {
                        x = std::hint::black_box(x.wrapping_add(1));
                    }
                }
            })
        })
        .collect();
    let n = hs.len();
    for h in hs {
        let _ = h.join();
    }
    Some((ms, n, t0.elapsed().as_secs_f64()))
}

#[derive(Clone, Copy, Default)]
struct Usage {
    user_s: f64,
    sys_s: f64,
    maxrss_kib: u64,
    minflt: u64,
    majflt: u64,
}

#[cfg(unix)]
fn rusage_self() -> Usage {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Timeval {
        tv_sec: i64,
        tv_usec: i64,
    }
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rusage {
        ru_utime: Timeval,
        ru_stime: Timeval,
        ru_maxrss: i64,
        ru_ixrss: i64,
        ru_idrss: i64,
        ru_isrss: i64,
        ru_minflt: i64,
        ru_majflt: i64,
        ru_nswap: i64,
        ru_inblock: i64,
        ru_oublock: i64,
        ru_msgsnd: i64,
        ru_msgrcv: i64,
        ru_nsignals: i64,
        ru_nvcsw: i64,
        ru_nivcsw: i64,
    }
    extern "C" {
        fn getrusage(who: i32, usage: *mut Rusage) -> i32;
    }
    let mut u = Rusage::default();
    // SAFETY: RUSAGE_SELF == 0 on Linux/macOS; `u` has the standard C layout.
    let rc = unsafe { getrusage(0, &mut u) };
    if rc != 0 {
        return Usage::default();
    }
    Usage {
        user_s: u.ru_utime.tv_sec as f64 + u.ru_utime.tv_usec as f64 * 1e-6,
        sys_s: u.ru_stime.tv_sec as f64 + u.ru_stime.tv_usec as f64 * 1e-6,
        maxrss_kib: u.ru_maxrss.max(0) as u64,
        minflt: u.ru_minflt.max(0) as u64,
        majflt: u.ru_majflt.max(0) as u64,
    }
}

#[cfg(not(unix))]
fn rusage_self() -> Usage {
    Usage::default()
}

/// Read `VmHWM` (peak RSS in KiB) from `/proc/self/status` on Linux.
fn vm_hwm_kib() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// Decode `data` through `BzReader::parallel_workers` and discard bytes into a 4 MiB buffer.
fn decode_timed(data: &'static [u8], k: usize) -> Result<u64, String> {
    let slots = SLOTS_PER_THREAD * k;
    let mut r = BzReader::parallel_workers(data, k, slots, Kernels::detect(), true);
    let mut buf = vec![0u8; 4 << 20];
    let mut total = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => total += n as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(format!("decode failed: {e}")),
        }
    }
    drop(r);
    Ok(total)
}

/// Decode `data` through `BzReader::parallel_workers`, computing exact byte count and SHA-256.
fn decode_hashed(data: &'static [u8], k: usize) -> Result<(u64, String), String> {
    let slots = SLOTS_PER_THREAD * k;
    let mut r = BzReader::parallel_workers(data, k, slots, Kernels::detect(), true);
    let mut buf = vec![0u8; 4 << 20];
    let mut total = 0u64;
    let mut hasher = Sha256::new();
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                hasher.update(&buf[..n]);
                total += n as u64;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(format!("decode failed: {e}")),
        }
    }
    drop(r);
    Ok((total, hasher.finish_hex()))
}

fn run_one(
    corpus: &str,
    arm: Arm,
    workers: usize,
    prespin_ms: u64,
    scanner_cpu: Option<usize>,
    want_trace: bool,
) {
    let data: &'static [u8] = std::fs::read(corpus)
        .unwrap_or_else(|e| {
            eprintln!("mtf_fold: failed to read {corpus}: {e}");
            std::process::exit(1);
        })
        .leak();

    // 1. Record pass (only when the arm uses an oracle: S, M, or C).
    //    For `none` and `F`, skip any pre-decode so `maxrss_kib` reflects a pure single decode!
    let mut ref_pass: Option<(u64, String)> = None;
    let mut syms_store_bytes = 0u64;
    let mut store_conflicts = 0u64;
    let mut store_missing = 0u64;
    let mut recorded_blocks = 0u64;

    scan_oracle::clear_store();
    im::clear_store();
    ic::clear_store();
    scan_oracle::set_scanner_cpu(None);
    #[cfg(feature = "trace")]
    mibz2::trace::set_level(0);

    if arm.ideal_s || arm.ideal_m || arm.ideal_c {
        fold::set_enabled(false);
        scan_oracle::set_mode(if arm.ideal_s {
            scan_oracle::Mode::Record
        } else {
            scan_oracle::Mode::Scan
        });
        im::set_mode(if arm.ideal_m {
            ideal::Mode::Record
        } else {
            ideal::Mode::Real
        });
        ic::set_mode(if arm.ideal_c {
            ideal::Mode::Record
        } else {
            ideal::Mode::Real
        });
        let rec_workers = std::thread::available_parallelism()
            .map_or(4, |n| n.get().min(8))
            .max(1);
        let (bytes, sha) = decode_hashed(data, rec_workers).unwrap_or_else(|e| {
            eprintln!("mtf_fold: record pass failed: {e}");
            std::process::exit(1);
        });
        ref_pass = Some((bytes, sha));
        if arm.ideal_s {
            let (idx, _, sbytes, conf, miss) = scan_oracle::store_size();
            recorded_blocks = idx;
            syms_store_bytes = sbytes;
            store_conflicts += conf;
            store_missing += miss;
        }
        if arm.ideal_m {
            let (_, _, _, conf, miss) = im::store_size();
            store_conflicts += conf;
            store_missing += miss;
        }
        if arm.ideal_c {
            let (_, _, _, conf, miss) = ic::store_size();
            store_conflicts += conf;
            store_missing += miss;
        }
        if store_conflicts > 0 || store_missing > 0 {
            eprintln!(
                "mtf_fold: oracle store defective: conflicts={store_conflicts} missing={store_missing}"
            );
            std::process::exit(1);
        }
    }

    // 2. Pre-spin across all online CPUs (outside the stopwatch).
    let spun = prespin(prespin_ms);

    // 3. Timed decode.
    scan_oracle::set_scanner_cpu(scanner_cpu);
    arm.apply();
    let _ = fold::take_counts();
    let _ = scan_oracle::take_scanned();
    let _ = scan_oracle::take_replayed();
    let _ = scan_oracle::take_lock_acquires();
    let _ = im::take_executed();
    let _ = im::take_replayed();
    let _ = im::take_lock_acquires();
    let _ = ic::take_constructed();
    let _ = ic::take_replayed();
    let _ = ic::take_refused_replayed();
    let _ = ic::take_lock_acquires();

    #[cfg(feature = "trace")]
    mibz2::trace::set_level(if want_trace { 2 } else { 0 });

    let u0 = rusage_self();
    let t0 = Instant::now();
    let timed_bytes = decode_timed(data, workers).unwrap_or_else(|e| {
        eprintln!("mtf_fold: timed decode failed: {e}");
        std::process::exit(1);
    });
    let wall_s = t0.elapsed().as_secs_f64();
    let u1 = rusage_self();
    let maxrss_kib = vm_hwm_kib().unwrap_or(u1.maxrss_kib).max(u1.maxrss_kib);

    let fold_counts = fold::take_counts();
    let scanned = scan_oracle::take_scanned();
    let replayed = scan_oracle::take_replayed();
    let scan_locks = scan_oracle::take_lock_acquires();
    let pin_ok = scan_oracle::scanner_pin_ok();
    let mtf_executed = im::take_executed();
    let mtf_replayed = im::take_replayed();
    let mtf_locks = im::take_lock_acquires();
    let constr_constructed = ic::take_constructed();
    let constr_replayed = ic::take_replayed();
    let constr_refused_replayed = ic::take_refused_replayed();
    let constr_locks = ic::take_lock_acquires();

    #[cfg(feature = "trace")]
    let trace_json = if want_trace {
        let traces = mibz2::trace::take();
        mibz2::trace::set_level(0);
        summarise_traces(&traces, workers, arm.fold)
    } else {
        "null".to_string()
    };
    #[cfg(not(feature = "trace"))]
    let trace_json = {
        let _ = want_trace;
        "null".to_string()
    };

    // 4. Check pass (outside the clock, after peak RSS is captured):
    //    Decode with the arm's modes and verify exact byte-identity against real `none`.
    scan_oracle::set_scanner_cpu(None);
    arm.apply();
    let check_workers = std::thread::available_parallelism()
        .map_or(4, |n| n.get().min(8))
        .max(1);
    let (arm_bytes, arm_sha) = decode_hashed(data, check_workers).unwrap_or_else(|e| {
        eprintln!("mtf_fold: arm check pass failed: {e}");
        std::process::exit(1);
    });
    let (ref_bytes, ref_sha) = match ref_pass {
        Some(pair) => pair,
        None => {
            Arm::parse("none").unwrap().apply();
            decode_hashed(data, check_workers).unwrap_or_else(|e| {
                eprintln!("mtf_fold: reference check pass failed: {e}");
                std::process::exit(1);
            })
        }
    };
    let identical = timed_bytes == ref_bytes && arm_bytes == ref_bytes && arm_sha == ref_sha;

    let user_s = (u1.user_s - u0.user_s).max(0.0);
    let sys_s = (u1.sys_s - u0.sys_s).max(0.0);
    let majflt = u1.majflt.saturating_sub(u0.majflt);
    let minflt = u1.minflt.saturating_sub(u0.minflt);
    let pin_ok_json = match pin_ok {
        Some(true) => "true",
        Some(false) => "false",
        None => "null",
    };
    let scanner_cpu_json = match scanner_cpu {
        Some(c) => c.to_string(),
        None => "null".to_string(),
    };
    let (prespin_cpus, prespin_s) = spun.map_or((0, 0.0), |(_, n, s)| (n, s));

    println!(
        "{{\"kind\":\"decode\",\"arm\":\"{}\",\"k\":{},\"trace\":{},\"wall_s\":{:.9},\
         \"user_s\":{:.6},\"sys_s\":{:.6},\"maxrss_kib\":{},\"majflt\":{},\"minflt\":{},\
         \"bytes\":{},\"scanner_cpu\":{},\"scanner_pin_ok\":{},\
         \"prespin_ms\":{},\"prespin_cpus\":{},\"prespin_s\":{:.4},\
         \"fold\":{{\"folded\":{},\"bailed\":{},\"not_open\":{}}},\
         \"oracles\":{{\"recorded_blocks\":{},\"syms_store_bytes\":{},\
         \"scanned\":{},\"replayed\":{},\"scan_locks\":{},\
         \"mtf_executed\":{},\"mtf_replayed\":{},\"mtf_locks\":{},\
         \"constr_constructed\":{},\"constr_replayed\":{},\"constr_refused_replayed\":{},\"constr_locks\":{}}},\
         \"check_pass\":{{\"identical\":{},\"bytes\":{},\"sha256\":\"{}\",\"ref_sha256\":\"{}\"}},\
         \"trace_summary\":{}}}",
        arm.name,
        workers,
        if want_trace { 1 } else { 0 },
        wall_s,
        user_s,
        sys_s,
        maxrss_kib,
        majflt,
        minflt,
        timed_bytes,
        scanner_cpu_json,
        pin_ok_json,
        prespin_ms,
        prespin_cpus,
        prespin_s,
        fold_counts.folded,
        fold_counts.bailed,
        fold_counts.not_open,
        recorded_blocks,
        syms_store_bytes,
        scanned,
        replayed,
        scan_locks,
        mtf_executed,
        mtf_replayed,
        mtf_locks,
        constr_constructed,
        constr_replayed,
        constr_refused_replayed,
        constr_locks,
        if identical { "true" } else { "false" },
        arm_bytes,
        arm_sha,
        ref_sha,
        trace_json,
    );
}

#[cfg(feature = "trace")]
fn summarise_traces(traces: &[mibz2::trace::ThreadTrace], k: usize, arm_fold: bool) -> String {
    use mibz2::trace::{Kind, UNREAD};
    use std::collections::HashMap;

    let mut scanner_busy_ns = 0u64;
    let mut scanner_cpu_ns = 0u64;
    let mut blocks_issued = 0u64;

    let mut workers_seen = 0usize;
    let mut worker_busy_ns = 0u64;
    let mut worker_cpu_ns = 0u64;
    let mut a5_ns = 0u64;
    let mut a6a_ns = 0u64;
    let mut fused_a5_a6a_ns = 0u64;
    let mut fold_fallback_ns = 0u64;
    let mut a6bc_a7_ns = 0u64;
    let mut a9_ns = 0u64;
    let mut blocks_decoded = 0u64;
    let mut blocks_open_pass1 = 0u64;

    let mut copy_out_ns = 0u64;
    let mut reader_wait_ns = 0u64;
    let mut reader_cpu_ns = 0u64;
    let mut ready_bytes_hwm = 0u64;

    let mut out_len_by_place: HashMap<u64, u64> = HashMap::new();
    let mut ready_events: Vec<(u64, i64)> = Vec::new();

    for t in traces {
        let mut t_start_cpu: Option<u64> = None;
        let mut t_end_cpu: Option<u64> = None;
        for m in &t.marks {
            if m.kind == Kind::ThreadStart && m.cpu_ns != UNREAD {
                t_start_cpu = Some(m.cpu_ns);
            } else if m.kind == Kind::ThreadEnd && m.cpu_ns != UNREAD {
                t_end_cpu = Some(m.cpu_ns);
            }
        }
        let thread_cpu = match (t_start_cpu, t_end_cpu) {
            (Some(a), Some(b)) => b.saturating_sub(a),
            _ => 0,
        };

        let is_scanner = t.marks.iter().any(|m| {
            matches!(
                m.kind,
                Kind::SlotWait | Kind::SlotGot | Kind::Issued | Kind::Enqueued
            )
        });
        let is_worker = t
            .marks
            .iter()
            .any(|m| matches!(m.kind, Kind::DecodeStart | Kind::DecodeEnd));
        let is_reader = t
            .marks
            .iter()
            .any(|m| matches!(m.kind, Kind::ReadWait | Kind::Taken));

        if is_scanner {
            scanner_cpu_ns += thread_cpu;
            let mut slot_got: Option<u64> = None;
            for m in &t.marks {
                match m.kind {
                    Kind::SlotGot => slot_got = Some(m.ns),
                    Kind::Issued => {
                        blocks_issued += 1;
                        if let Some(s) = slot_got.take() {
                            scanner_busy_ns += m.ns.saturating_sub(s);
                        }
                    }
                    _ => {}
                }
            }
        } else if is_worker {
            workers_seen += 1;
            worker_cpu_ns += thread_cpu;
            let mut dec_start: Option<u64> = None;
            let mut inverts: Vec<(u64, Option<u64>, u64)> = Vec::new();
            let mut cur_inv_start: Option<u64> = None;
            let mut cur_pass1_end: Option<u64> = None;

            for m in &t.marks {
                match m.kind {
                    Kind::DecodeStart => {
                        dec_start = Some(m.ns);
                        inverts.clear();
                        cur_inv_start = None;
                        cur_pass1_end = None;
                    }
                    Kind::InvertStart => {
                        cur_inv_start = Some(m.ns);
                        cur_pass1_end = None;
                    }
                    Kind::OpenPass1End => {
                        if cur_inv_start.is_some() {
                            cur_pass1_end = Some(m.ns);
                            blocks_open_pass1 += 1;
                        }
                    }
                    Kind::InvertEnd => {
                        if let Some(s) = cur_inv_start.take() {
                            inverts.push((s, cur_pass1_end.take(), m.ns));
                        }
                    }
                    Kind::DecodeEnd => {
                        blocks_decoded += 1;
                        if m.len != UNREAD {
                            out_len_by_place.insert(m.place, m.len);
                        }
                        if let Some(ds) = dec_start.take() {
                            worker_busy_ns += m.ns.saturating_sub(ds);
                            if inverts.len() == 1 {
                                let (is, p1, ie) = inverts[0];
                                a5_ns += is.saturating_sub(ds);
                                if let Some(p1e) = p1 {
                                    if arm_fold {
                                        fused_a5_a6a_ns += p1e.saturating_sub(is);
                                    } else {
                                        a6a_ns += p1e.saturating_sub(is);
                                    }
                                    a6bc_a7_ns += ie.saturating_sub(p1e);
                                } else {
                                    a6bc_a7_ns += ie.saturating_sub(is);
                                }
                                a9_ns += m.ns.saturating_sub(ie);
                            } else if inverts.len() >= 2 {
                                // Fold attempt fell back (`NotOpen`/`Bailed`) to shipped path.
                                let (is0, _p10, ie0) = inverts[0];
                                let (is1, p11, ie1) = *inverts.last().unwrap();
                                fold_fallback_ns += ie0.saturating_sub(is0);
                                a5_ns += is1.saturating_sub(ie0);
                                if let Some(p1e) = p11 {
                                    a6a_ns += p1e.saturating_sub(is1);
                                    a6bc_a7_ns += ie1.saturating_sub(p1e);
                                } else {
                                    a6bc_a7_ns += ie1.saturating_sub(is1);
                                }
                                a9_ns += m.ns.saturating_sub(ie1);
                            } else {
                                // No InvertStart/InvertEnd (e.g. trivial/error block).
                                a9_ns += m.ns.saturating_sub(ds);
                            }
                        }
                    }
                    Kind::Ready => {
                        if m.len != UNREAD {
                            ready_bytes_hwm = ready_bytes_hwm.max(m.len);
                        }
                        if let Some(&len) = out_len_by_place.get(&m.place) {
                            ready_events.push((m.ns, len as i64));
                        }
                    }
                    _ => {}
                }
            }
        } else if is_reader {
            reader_cpu_ns += thread_cpu;
            let mut wait_start: Option<u64> = None;
            let mut taken_at: Option<u64> = None;
            for m in &t.marks {
                match m.kind {
                    Kind::ReadWait => {
                        if let Some(ta) = taken_at.take() {
                            copy_out_ns += m.ns.saturating_sub(ta);
                        }
                        wait_start = Some(m.ns);
                    }
                    Kind::Taken => {
                        if let Some(ws) = wait_start.take() {
                            reader_wait_ns += m.ns.saturating_sub(ws);
                        }
                        taken_at = Some(m.ns);
                    }
                    Kind::ThreadEnd => {
                        if let Some(ta) = taken_at.take() {
                            copy_out_ns += m.ns.saturating_sub(ta);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Second pass over reader `Taken` marks now that `out_len_by_place` is populated.
    for t in traces {
        for m in &t.marks {
            if m.kind == Kind::Taken {
                if let Some(&len) = out_len_by_place.get(&m.place) {
                    ready_events.push((m.ns, -(len as i64)));
                }
            }
        }
    }
    ready_events.sort_by_key(|&(ns, delta)| (ns, delta));
    let mut cur_ready = 0i64;
    let mut ready_bytes_sweep_hwm = 0u64;
    for (_, d) in ready_events {
        cur_ready = (cur_ready + d).max(0);
        ready_bytes_sweep_hwm = ready_bytes_sweep_hwm.max(cur_ready as u64);
    }

    let denom = k.max(1) as f64;
    format!(
        "{{\"scanner_busy_ns\":{scanner_busy_ns},\"scanner_cpu_ns\":{scanner_cpu_ns},\
         \"workers_seen\":{workers_seen},\"worker_busy_ns_total\":{worker_busy_ns},\
         \"worker_busy_ns_per_w\":{:.1},\"worker_cpu_ns_total\":{worker_cpu_ns},\
         \"worker_cpu_ns_per_w\":{:.1},\
         \"a5_ns_total\":{a5_ns},\"a5_ns_per_w\":{:.1},\
         \"a6a_ns_total\":{a6a_ns},\"a6a_ns_per_w\":{:.1},\
         \"fused_a5_a6a_ns_total\":{fused_a5_a6a_ns},\"fused_a5_a6a_ns_per_w\":{:.1},\
         \"fold_fallback_ns_total\":{fold_fallback_ns},\
         \"a6bc_a7_ns_total\":{a6bc_a7_ns},\"a6bc_a7_ns_per_w\":{:.1},\
         \"a9_ns_total\":{a9_ns},\"a9_ns_per_w\":{:.1},\
         \"copy_out_ns\":{copy_out_ns},\"reader_wait_ns\":{reader_wait_ns},\
         \"reader_cpu_ns\":{reader_cpu_ns},\
         \"ready_bytes_hwm\":{ready_bytes_hwm},\"ready_bytes_sweep_hwm\":{ready_bytes_sweep_hwm},\
         \"blocks_issued\":{blocks_issued},\"blocks_decoded\":{blocks_decoded},\
         \"blocks_open_pass1\":{blocks_open_pass1}}}",
        worker_busy_ns as f64 / denom,
        worker_cpu_ns as f64 / denom,
        a5_ns as f64 / denom,
        a6a_ns as f64 / denom,
        fused_a5_a6a_ns as f64 / denom,
        a6bc_a7_ns as f64 / denom,
        a9_ns as f64 / denom,
    )
}

/// Minimal FIPS 180-4 SHA-256 (zero dependencies).
struct Sha256 {
    h: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total: u64,
}

impl Sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn new() -> Self {
        Self {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: [0; 64],
            buf_len: 0,
            total: 0,
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[4 * i],
                block[4 * i + 1],
                block[4 * i + 2],
                block[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.h;
        for (&ki, &wi) in Self::K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(ki)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, delta) in self.h.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(delta);
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total += data.len() as u64;
        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        while data.len() >= 64 {
            let block: &[u8; 64] = data[..64].try_into().unwrap();
            self.compress(block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn finish_hex(mut self) -> String {
        let bit_len = self.total * 8;
        self.update(&[0x80]);
        if self.buf_len > 56 {
            let pad = [0u8; 64];
            let rem = 64 - self.buf_len;
            self.update(&pad[..rem]);
        }
        let pad = [0u8; 64];
        let rem = 56 - self.buf_len;
        self.update(&pad[..rem]);
        self.update(&bit_len.to_be_bytes());
        let mut out = String::with_capacity(64);
        for word in self.h {
            use std::fmt::Write;
            let _ = write!(&mut out, "{word:08x}");
        }
        out
    }
}
