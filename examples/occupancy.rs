//! Where the parallel decoder's time goes as its thread count grows (#39): the throughput
//! curve, and the decoder's own timeline beside it. `just bench-occupancy` runs it (with the
//! pinned Wikidata corpus); CI runs the same recipe.
//!
//! ```text
//! occupancy FILE.bz2 [--threads 1,2,4] [--rounds N] [--slots N] [--out PATH] [--meta K=V]...
//! ```
//!
//! **The workload is the benchmark's.** Each run is a fresh process decoding FILE as
//! examples/ab.rs's `mibz2` arm does: `BzReader::parallel_with(File, t, slots,
//! Kernels::detect())` (`parallel`'s own call, its window `2 × t` unless `--slots`), its
//! feeder reading the file, the output drained in 4 MiB reads into a streaming hash. The
//! 1-thread runs are pinned to one CPU, as the benchmark's are. Every run must give the same
//! bytes.
//!
//! **Placements** (`--placements packed/spread`, or CPU lists): each an arm of its own in
//! every round, the whole process pinned to its CPUs (the scanner, feeder and reader with
//! the workers), so a difference is the decoder's placement, not the workers' alone.
//! `packed` takes each core's SMT siblings before the next core, `spread` one CPU a core
//! first, both from the machine's own sibling map. Each placement's throughput against
//! the first's, paired by round in `off`, is the comparison's verdict.
//!
//! **Three modes a thread count,** each round in a rotated order: `off` (no marks), the
//! `timeline` (src/trace.rs level 1: timestamps of the scanner's, workers' and reader's
//! states) and `full` (level 2: also each worker's CPU time and scheduler counters at
//! `decode_block`'s start and end). Throughput is read from `off`. Each traced mode's
//! cost is its paired ratio to `off`; a mode whose interval excludes no change is reported
//! as perturbing, and its trace is not to be interpreted.
//!
//! **What a trace reports**, from one sweep of every thread's marks in time order (shares
//! of the run's span, waits that overlap never added twice):
//! - workers: decoding, idle (on the jobs' lock or in `recv`), taking a buffer, publishing;
//!   idle time split by what the scanner was doing meanwhile (waiting for a slot, with the
//!   earliest block not taken still decoding or ready for the reader; waiting for input;
//!   reading a block; done), and apart, idle while jobs sat in the queue (admitted and sent,
//!   not yet received: the handoff); the mean number decoding, and the time all were;
//! - the scanner: waiting for input, waiting for a slot (split by whether the earliest
//!   block not yet taken was still decoding, or ready and waiting for the reader), reading
//!   blocks;
//! - the reader: waiting for its next block, or consuming one;
//! - in `full`, per block: elapsed, CPU time, elapsed less CPU (off a CPU), and on Linux the
//!   run-queue wait and timeslices, all over the same `decode_block` interval. CPU time
//!   counts a thread stalled on memory as running: CPU rising with little run-queue wait is
//!   an on-CPU cost, not by itself cache or memory contention.
//!
//! A trace is used only if complete: every thread's start and end, every block's start
//! matched by its end, and the blocks decoded, published and taken the same.
//!
//! **The run-queue counter is checked, not assumed:** once, after every timed run, threads
//! twice the CPUs allowed spin for 0.2 s; unless their run-queue wait rises with their time
//! off a CPU, the counter is reported unavailable (a zero there is never read as no wait).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Read;
use std::process::Command;
use std::time::{Duration, Instant};

use mibz2::trace::{self, Kind, ThreadTrace, UNREAD};
use mibz2::{BzReader, Kernels};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--child") {
        child(&args[1..]);
    } else {
        parent(&args);
    }
}

// ---------------------------------------------------------------------------------------
// One run, in a process of its own: a line of key=value pairs on stdout

fn child(a: &[String]) {
    let path = &a[0];
    let threads: usize = a[1].parse().unwrap();
    let slots: usize = a[2].parse().unwrap();
    let level: u8 = a[3].parse().unwrap();
    trace::set_level(level);
    let mut h = Hash::new();
    let t0 = Instant::now();
    let res = std::fs::File::open(path)
        .map_err(|e| e.to_string())
        .and_then(|f| {
            drain(
                BzReader::parallel_with(f, threads, slots, Kernels::detect()),
                &mut h,
            )
        });
    let wall = t0.elapsed().as_secs_f64();
    let mut kv = Kv::default();
    if level > 0 {
        // (the decoder's threads end once it is dropped; its last reference can be the
        // feeder's, so wait for their marks rather than assume them)
        let deadline = Instant::now() + Duration::from_secs(5);
        while trace::finished() < threads + 1 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        let got = trace::finished();
        let timeout = (got < threads + 1).then(|| {
            format!(
                "collection timeout: {got} of {} threads' marks at the deadline",
                threads + 1
            )
        });
        summarise(&trace::take(), threads, level, timeout, &mut kv);
    }
    trace::set_level(0);
    let (cpu, rss) = rusage();
    let err = res.err().unwrap_or_default();
    let mut line = format!(
        "wall={wall:.6} cpu={cpu:.6} rss_kb={rss} out={} hash={} err={}",
        h.n,
        h.finish(),
        word(&err)
    );
    for (k, v) in kv.0 {
        let _ = write!(line, " {k}={v}");
    }
    println!("{line}");
}

#[derive(Default)]
struct Kv(Vec<(String, String)>);

impl Kv {
    fn f(&mut self, k: &str, v: f64) {
        self.0.push((
            k.into(),
            if v.is_finite() {
                format!("{v:.6}")
            } else {
                "NA".into()
            },
        ));
    }
    fn s(&mut self, k: &str, v: &str) {
        self.0.push((k.into(), word(v)));
    }
}

fn word(s: &str) -> String {
    if s.is_empty() {
        "-".into()
    } else {
        s.replace([' ', '='], "_")
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Scanner,
    Worker,
    Reader,
}

#[derive(Clone, Copy, PartialEq)]
enum W {
    Outside,
    Lock,
    Recv,
    Buffer,
    Decoding,
    Publish,
    Other,
}

#[derive(Clone, Copy, PartialEq)]
enum S {
    Outside,
    Reading,
    Input,
    Slot,
}

/// The trace's checks and its sweep, into `kv` (keys `t.*`)
fn summarise(
    traces: &[ThreadTrace],
    threads: usize,
    level: u8,
    timeout: Option<String>,
    kv: &mut Kv,
) {
    let role = |name: &str| match name {
        "mibz2-scan" => Some(Role::Scanner),
        n if n.starts_with("mibz2-") && n[6..].parse::<usize>().is_ok() => Some(Role::Worker),
        "mibz2-read" => None,
        _ => Some(Role::Reader),
    };
    let roles: Vec<Option<Role>> = traces.iter().map(|t| role(&t.name)).collect();
    let n_workers = roles.iter().filter(|r| **r == Some(Role::Worker)).count();
    let n_scan = roles.iter().filter(|r| **r == Some(Role::Scanner)).count();
    let n_read = roles.iter().filter(|r| **r == Some(Role::Reader)).count();

    // completeness: every thread's start and end; every block started, ended, published
    // and taken once; the places 0..issued
    let mut problem = timeout.unwrap_or_default();
    if problem.is_empty() && n_workers != threads || n_scan != 1 || n_read != 1 {
        problem = format!(
            "threads traced: {n_workers} workers of {threads}, {n_scan} scanner, {n_read} reader"
        );
    }
    let (mut ended, mut published, mut taken_set) = (Vec::new(), Vec::new(), Vec::new());
    let mut issued = 0u64;
    for (t, r) in traces.iter().zip(&roles) {
        let first = t.marks.first().map(|m| m.kind);
        let last = t.marks.last().map(|m| m.kind);
        if first != Some(Kind::ThreadStart) || last != Some(Kind::ThreadEnd) {
            problem = format!("{}: no start or end mark (unbalanced)", t.name);
        }
        let mut open = None;
        for m in &t.marks {
            match m.kind {
                Kind::DecodeStart if open.is_some() => {
                    problem = format!("{}: a block started twice", t.name)
                }
                Kind::DecodeStart => open = Some(m.place),
                Kind::DecodeEnd if open != Some(m.place) => {
                    problem = format!("{}: an end without its start", t.name)
                }
                Kind::DecodeEnd => {
                    open = None;
                    ended.push(m.place);
                }
                Kind::Ready => published.push(m.place),
                Kind::Taken => taken_set.push(m.place),
                Kind::Issued => issued += 1,
                _ => {}
            }
        }
        if open.is_some() && *r == Some(Role::Worker) {
            problem = format!("{}: a block started and never ended", t.name);
        }
    }
    for v in [&mut ended, &mut published, &mut taken_set] {
        v.sort_unstable();
    }
    let all: Vec<u64> = (0..issued).collect();
    if problem.is_empty() && (ended != all || published != all || taken_set != all) {
        problem = format!(
            "blocks: {issued} issued, {} decoded, {} published, {} taken",
            ended.len(),
            published.len(),
            taken_set.len()
        );
    }
    kv.s("t.complete", if problem.is_empty() { "yes" } else { "no" });
    kv.s("t.problem", &problem);
    kv.f("t.blocks", issued as f64);
    if !problem.is_empty() {
        return;
    }

    // the sweep: every mark in time order, each interval credited to the states then
    let mut ev: Vec<(u64, usize, Kind, u64)> = Vec::new();
    for (i, t) in traces.iter().enumerate() {
        if roles[i].is_some() {
            ev.extend(t.marks.iter().map(|m| (m.ns, i, m.kind, m.place)));
        }
    }
    ev.sort_by_key(|e| e.0);
    let span = ev.last().map_or(0, |e| e.0) as f64;
    let widx: BTreeMap<usize, usize> = roles
        .iter()
        .enumerate()
        .filter(|(_, r)| **r == Some(Role::Worker))
        .enumerate()
        .map(|(w, (i, _))| (i, w))
        .collect();
    let mut ws = vec![W::Outside; n_workers];
    let mut ss = S::Outside;
    let (mut reader_waiting, mut reader_started, mut reader_ended) = (false, false, false);
    let mut taken = 0u64;
    let mut ready: BTreeSet<u64> = BTreeSet::new();
    let mut queued = 0i64;
    let mut w_acc = [0f64; 7];
    // worker-ns idle (on the jobs' lock or in recv), each credited to one concurrent state
    let (mut id_slot_dec, mut id_slot_ready, mut id_input, mut id_reading, mut id_outside) =
        (0f64, 0f64, 0f64, 0f64, 0f64);
    let mut id_queued = 0f64;
    let (mut s_input, mut s_input_busy, mut s_reading) = (0f64, 0f64, 0f64);
    let (mut s_slot_decoding, mut s_slot_ready) = (0f64, 0f64);
    let (mut r_wait, mut r_consume) = (0f64, 0f64);
    let (mut decoding_int, mut all_decoding) = (0f64, 0f64);
    let mut prev = 0u64;
    for &(ns, i, kind, place) in &ev {
        let dt = (ns - prev) as f64;
        prev = ns;
        if dt > 0.0 {
            let mut decoding = 0;
            let mut recv = 0;
            for &w in &ws {
                w_acc[w as usize] += dt;
                decoding += (w == W::Decoding) as usize;
                recv += matches!(w, W::Recv | W::Lock) as usize;
            }
            decoding_int += dt * decoding as f64;
            if decoding == n_workers {
                all_decoding += dt;
            }
            let idle = dt * recv as f64;
            if queued > 0 {
                id_queued += idle;
            }
            match ss {
                S::Slot if ready.contains(&taken) => id_slot_ready += idle,
                S::Slot => id_slot_dec += idle,
                S::Input => id_input += idle,
                S::Reading => id_reading += idle,
                S::Outside => id_outside += idle,
            }
            match ss {
                S::Input if recv == 0 => s_input_busy += dt,
                S::Input => s_input += dt,
                S::Reading => s_reading += dt,
                S::Slot if ready.contains(&taken) => s_slot_ready += dt,
                S::Slot => s_slot_decoding += dt,
                S::Outside => {}
            }
            if reader_waiting {
                r_wait += dt;
            } else if reader_started && !reader_ended {
                r_consume += dt;
            }
        }
        match roles[i] {
            Some(Role::Worker) => {
                let w = &mut ws[widx[&i]];
                *w = match kind {
                    Kind::JobLock => W::Lock,
                    Kind::JobRecv => W::Recv,
                    Kind::JobGot => {
                        queued -= 1;
                        W::Buffer
                    }
                    Kind::DecodeStart => W::Decoding,
                    Kind::DecodeEnd => W::Publish,
                    Kind::Ready => {
                        ready.insert(place);
                        W::Other
                    }
                    Kind::ThreadEnd => W::Outside,
                    _ => *w,
                };
            }
            Some(Role::Scanner) => {
                ss = match kind {
                    Kind::ThreadStart | Kind::InputGot | Kind::InputEnd | Kind::SlotGot => {
                        S::Reading
                    }
                    Kind::InputWait => S::Input,
                    Kind::SlotWait => S::Slot,
                    Kind::ThreadEnd => S::Outside,
                    Kind::Enqueued => {
                        queued += 1;
                        ss
                    }
                    _ => ss,
                };
            }
            Some(Role::Reader) => match kind {
                Kind::ReadWait => {
                    reader_waiting = true;
                    reader_started = true;
                }
                Kind::Taken => {
                    reader_waiting = false;
                    ready.remove(&place);
                    taken = place + 1;
                }
                Kind::ThreadEnd => {
                    reader_waiting = false;
                    reader_ended = true;
                }
                _ => {}
            },
            None => {}
        }
    }
    let wspan = span * n_workers as f64;
    kv.f("t.span_s", span / 1e9);
    for (k, s) in [
        ("decoding", W::Decoding),
        ("jobwait", W::Recv),
        ("lock", W::Lock),
        ("buffer", W::Buffer),
        ("publish", W::Publish),
        ("other", W::Other),
        ("outside", W::Outside),
    ] {
        kv.f(&format!("t.w.{k}"), w_acc[s as usize] / wspan);
    }
    kv.f(
        "t.w.idle",
        (w_acc[W::Lock as usize] + w_acc[W::Recv as usize]) / wspan,
    );
    kv.f("t.w.idle.slot_earliest_decoding", id_slot_dec / wspan);
    kv.f("t.w.idle.slot_earliest_ready", id_slot_ready / wspan);
    kv.f("t.w.idle.input", id_input / wspan);
    kv.f("t.w.idle.reading", id_reading / wspan);
    kv.f("t.w.idle.outside", id_outside / wspan);
    kv.f("t.w.idle.jobs_queued", id_queued / wspan);
    // (an upper bound on what a wider window could recover, not a prediction)
    kv.f(
        "t.w.idle_during_slot_wait_s",
        (id_slot_dec + id_slot_ready) / 1e9,
    );
    kv.f("t.w.mean_decoding", decoding_int / span);
    kv.f("t.w.all_decoding", all_decoding / span);
    kv.f("t.s.input", s_input / span);
    kv.f("t.s.input_workers_busy", s_input_busy / span);
    kv.f("t.s.slot_earliest_decoding", s_slot_decoding / span);
    kv.f("t.s.slot_earliest_ready", s_slot_ready / span);
    kv.f("t.s.reading", s_reading / span);
    kv.f("t.r.waiting", r_wait / span);
    kv.f("t.r.consuming", r_consume / span);

    if level < 2 {
        return;
    }
    // per block, over the same decode_block interval: elapsed, CPU, run-queue wait, slices
    let (mut n, mut el, mut cpu, mut wait, mut slices) = (0f64, 0f64, 0f64, 0f64, 0f64);
    let (mut cpu_ok, mut sched_ok) = (true, true);
    let (mut life_cpu, mut life_wait, mut life_slices) = (0f64, 0f64, 0f64);
    for (t, r) in traces.iter().zip(&roles) {
        if *r != Some(Role::Worker) {
            continue;
        }
        let d = |a: u64, b: u64, ok: &mut bool| {
            if a == UNREAD || b == UNREAD {
                *ok = false;
                0.0
            } else {
                b.saturating_sub(a) as f64
            }
        };
        let mut start = None;
        for m in &t.marks {
            match m.kind {
                Kind::DecodeStart => start = Some(*m),
                Kind::DecodeEnd => {
                    let s = start.take().unwrap();
                    n += 1.0;
                    el += (m.ns - s.ns) as f64;
                    cpu += d(s.cpu_ns, m.cpu_ns, &mut cpu_ok);
                    wait += d(s.wait_ns, m.wait_ns, &mut sched_ok);
                    slices += d(s.slices, m.slices, &mut sched_ok);
                }
                _ => {}
            }
        }
        let (a, b) = (t.marks[0], *t.marks.last().unwrap());
        life_cpu += d(a.cpu_ns, b.cpu_ns, &mut cpu_ok);
        life_wait += d(a.wait_ns, b.wait_ns, &mut sched_ok);
        life_slices += d(a.slices, b.slices, &mut sched_ok);
    }
    let na = f64::NAN;
    kv.f("t.b.elapsed_ms", el / n / 1e6);
    kv.f("t.b.cpu_ms", if cpu_ok { cpu / n / 1e6 } else { na });
    kv.f(
        "t.b.offcpu_ms",
        if cpu_ok { (el - cpu) / n / 1e6 } else { na },
    );
    kv.f("t.b.runq_ms", if sched_ok { wait / n / 1e6 } else { na });
    kv.f("t.b.slices", if sched_ok { slices / n } else { na });
    kv.f("t.wk.cpu_s", if cpu_ok { life_cpu / 1e9 } else { na });
    kv.f("t.wk.runq_s", if sched_ok { life_wait / 1e9 } else { na });
    kv.f("t.wk.slices", if sched_ok { life_slices } else { na });
}

// ---------------------------------------------------------------------------------------
// The sweep: runs, statistics, the machine, the self-test, the report

const MODES: [(&str, u8); 3] = [("off", 0), ("timeline", 1), ("full", 2)];

fn parent(args: &[String]) {
    let mut file = None;
    let (mut threads, mut rounds, mut slots, mut out) = (None, 6usize, None, None);
    let mut placements: Option<Vec<String>> = None;
    let mut meta: Vec<(String, String)> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--threads" => {
                threads = Some(
                    it.next()
                        .expect("--threads LIST")
                        .split(',')
                        .map(|x| x.trim().parse::<usize>().expect("a thread count"))
                        .collect::<Vec<_>>(),
                )
            }
            "--rounds" => rounds = it.next().expect("--rounds N").parse().expect("rounds"),
            "--slots" => {
                slots = Some(
                    it.next()
                        .expect("--slots N")
                        .parse::<usize>()
                        .expect("slots"),
                )
            }
            "--out" => out = Some(it.next().expect("--out PATH").clone()),
            "--placements" => {
                placements = Some(
                    it.next()
                        .expect("--placements A/B")
                        .split('/')
                        .map(String::from)
                        .collect(),
                )
            }
            "--meta" => {
                let kv = it.next().expect("--meta K=V");
                let (k, v) = kv.split_once('=').expect("--meta K=V");
                meta.push((k.into(), v.into()));
            }
            "-h" | "--help" => {
                println!("occupancy FILE.bz2 [--threads 1,2,4] [--rounds N] [--slots N] [--placements packed/spread] [--out PATH] [--meta K=V]...");
                return;
            }
            f => file = Some(f.to_string()),
        }
    }
    let file = file.expect("occupancy FILE.bz2 [...]");
    let m = machine();
    let threads = threads.unwrap_or_else(|| {
        let mut v: Vec<usize> = std::iter::successors(Some(1usize), |t| Some(t * 2))
            .take_while(|t| *t <= m.allowed)
            .collect();
        if *v.last().unwrap() != m.allowed {
            v.push(m.allowed);
        }
        v
    });
    let pin = m.allowed_list.first().copied();
    let taskset = Command::new("taskset").arg("-V").output().is_ok();
    let exe = std::env::current_exe().unwrap();
    // each placement an arm of its own in every round: the whole process (workers, scanner,
    // feeder and reader) pinned to its CPUs; "" is none (1-thread runs pinned, as the
    // benchmark's)
    let specs: Vec<String> = placements.clone().unwrap_or_else(|| vec![String::new()]);
    if placements.is_some() && !taskset {
        eprintln!("occupancy: --placements needs taskset on PATH");
        std::process::exit(2);
    }
    let mut cpus_of: BTreeMap<(usize, String), String> = BTreeMap::new();
    for &t in &threads {
        for spec in specs.iter().filter(|s| !s.is_empty()) {
            match placement(spec, t, &m) {
                Ok(c) => {
                    cpus_of.insert((t, spec.clone()), c);
                }
                Err(e) => {
                    eprintln!("occupancy: placement {spec:?} at {t} threads: {e}");
                    std::process::exit(2);
                }
            }
        }
    }
    eprintln!(
        "occupancy: {} · {} CPUs allowed · threads {threads:?} · {rounds} rounds × {} modes",
        m.cpu,
        m.allowed,
        MODES.len()
    );

    // runs[(threads, placement, mode)] = one key=value map a round
    type Runs<'a> = BTreeMap<(usize, String, &'a str), Vec<BTreeMap<String, String>>>;
    let mut runs: Runs = BTreeMap::new();
    let mut raw = Vec::new();
    for r in 0..rounds {
        for &t in &threads {
            let s = slots.unwrap_or(2 * t);
            let mut places = specs.clone();
            let n = places.len();
            places.rotate_left(r % n);
            for spec in places {
                let mut order: Vec<(&str, u8)> = MODES.to_vec();
                order.rotate_left(r % MODES.len());
                for (mode, level) in order {
                    let cpus = cpus_of
                        .get(&(t, spec.clone()))
                        .cloned()
                        .or_else(|| pin.filter(|_| t == 1 && taskset).map(|c| c.to_string()));
                    let mut cmd = match cpus {
                        Some(cpus) => {
                            let mut c = Command::new("taskset");
                            c.arg("-c").arg(cpus).arg(&exe);
                            c
                        }
                        None => Command::new(&exe),
                    };
                    cmd.args([
                        "--child",
                        &file,
                        &t.to_string(),
                        &s.to_string(),
                        &level.to_string(),
                    ]);
                    let o = cmd.output().expect("run a child");
                    let line = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    let kv: BTreeMap<String, String> = line
                        .split_whitespace()
                        .filter_map(|p| {
                            p.split_once('=')
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                        })
                        .collect();
                    if !o.status.success() || kv.get("err").map_or(true, |e| e != "-") {
                        eprintln!(
                            "r{r} t={t} {mode}: failed: {line} {}",
                            String::from_utf8_lossy(&o.stderr)
                        );
                    }
                    eprintln!(
                        "r{r} t={t:<3} {spec:<7} {mode:<8} wall {:>7} s{}",
                        kv.get("wall").map_or("?", String::as_str),
                        kv.get("t.complete")
                            .map_or(String::new(), |c| format!("  trace complete: {c}"))
                    );
                    raw.push((r, t, s, spec.clone(), mode, line));
                    runs.entry((t, spec.clone(), mode)).or_default().push(kv);
                    std::thread::sleep(Duration::from_millis(150));
                }
            }
        }
    }
    let selftest = sched_selftest(m.allowed);

    // parity: one output for every run
    let outs: BTreeSet<(String, String)> = runs
        .values()
        .flatten()
        .map(|kv| {
            (
                kv.get("out").cloned().unwrap_or_default(),
                kv.get("hash").cloned().unwrap_or_default(),
            )
        })
        .collect();
    let parity = outs.len() == 1
        && runs
            .values()
            .flatten()
            .all(|kv| kv.get("err").map(String::as_str) == Some("-"));
    let out_bytes: f64 = outs
        .iter()
        .next()
        .and_then(|o| o.0.parse().ok())
        .unwrap_or(f64::NAN);

    let num =
        |kv: &BTreeMap<String, String>, k: &str| kv.get(k).and_then(|v| v.parse::<f64>().ok());
    let mbs = |kv: &BTreeMap<String, String>| out_bytes / 1e6 / num(kv, "wall").unwrap_or(f64::NAN);
    let off_of = |t: usize, spec: &str| -> Vec<f64> {
        runs.get(&(t, spec.to_string(), "off"))
            .map_or(vec![], |v| v.iter().map(mbs).collect())
    };
    let mut comparisons = Vec::new();

    let mut results = Vec::new();
    let mut table = String::new();
    let _ = writeln!(
        table,
        "\n{:>4} {:>9} {:>17} {:>16} {:>16} | {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} | {:>7} {:>7} {:>7} {:>6}",
        "thr", "MB/s", "speedup (95%)", "timeline cost", "full cost", "dec", "idle", "i:sl-d", "i:sl-r", "i:in", "i:rd", "i:q", "s:in", "r:wait", "blk ms", "cpu ms", "runq ms", "slices"
    );
    for (t, spec) in threads
        .iter()
        .flat_map(|&t| specs.iter().map(move |s| (t, s.clone())))
    {
        let base1 = off_of(threads[0], &spec);
        let off = off_of(t, &spec);
        let speed = ratio_ci(
            &off.iter()
                .zip(&base1)
                .map(|(a, b)| a / b)
                .collect::<Vec<_>>(),
        );
        let mut cost = Vec::new();
        for mode in ["timeline", "full"] {
            let v: Vec<f64> = runs
                .get(&(t, spec.clone(), mode))
                .map_or(vec![], |v| v.iter().map(mbs).collect());
            // a mode's throughput over off's, round by round (< 1: tracing slows it)
            let c = ratio_ci(&v.iter().zip(&off).map(|(a, b)| a / b).collect::<Vec<_>>());
            cost.push((mode, c));
        }
        let traced = |mode: &str| -> Vec<(String, J)> {
            let rs: Vec<&BTreeMap<String, String>> =
                runs.get(&(t, spec.clone(), mode)).map_or(vec![], |v| {
                    v.iter()
                        .filter(|kv| kv.get("t.complete").map(String::as_str) == Some("yes"))
                        .collect()
                });
            let keys: BTreeSet<&String> = rs
                .iter()
                .flat_map(|kv| kv.keys())
                .filter(|k| k.starts_with("t.") && *k != "t.complete" && *k != "t.problem")
                .collect();
            let mut o = vec![("complete_rounds".to_string(), J::N(rs.len() as f64))];
            for k in keys {
                let mut v: Vec<f64> = rs.iter().filter_map(|kv| num(kv, k)).collect();
                o.push((k[2..].to_string(), J::N(median(&mut v))));
            }
            o
        };
        let tl = traced("timeline");
        let full = traced("full");
        let g = |o: &[(String, J)], k: &str| {
            o.iter().find(|(kk, _)| kk == k).map_or(f64::NAN, |(_, v)| {
                if let J::N(x) = v {
                    *x
                } else {
                    f64::NAN
                }
            })
        };
        let pct = |x: f64| {
            if x.is_finite() {
                format!("{:.0}%", 100.0 * x)
            } else {
                "–".into()
            }
        };
        let ci = |c: (f64, f64, f64)| {
            if c.1.is_finite() {
                format!(
                    "{:+.1}% ({:+.1},{:+.1})",
                    100.0 * (c.0 - 1.0),
                    100.0 * (c.1 - 1.0),
                    100.0 * (c.2 - 1.0)
                )
            } else {
                format!("{:+.1}%", 100.0 * (c.0 - 1.0))
            }
        };
        let _ = writeln!(
            table,
            "{:>4} {:>9.1} {:>17} {:>16} {:>16} | {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} | {:>6} {:>6} | {:>7.2} {:>7.2} {:>7.3} {:>6.1}",
            format!(
                "{t}{}{}",
                if m.cores.is_some_and(|c| t > c) { "*" } else { "" },
                cpus_of.get(&(t, spec.clone())).map_or(String::new(), |c| format!(" {spec} [{c}]"))
            ),
            median(&mut off.clone()),
            if t == threads[0] { "1".into() } else { format!("{:.2}× ({:.2},{:.2})", speed.0, speed.1, speed.2) },
            ci(cost[0].1),
            ci(cost[1].1),
            pct(g(&tl, "w.decoding")),
            pct(g(&tl, "w.idle")),
            pct(g(&tl, "w.idle.slot_earliest_decoding")),
            pct(g(&tl, "w.idle.slot_earliest_ready")),
            pct(g(&tl, "w.idle.input")),
            pct(g(&tl, "w.idle.reading")),
            pct(g(&tl, "w.idle.jobs_queued")),
            pct(g(&tl, "s.input")),
            pct(g(&tl, "r.waiting")),
            g(&full, "b.elapsed_ms"),
            g(&full, "b.cpu_ms"),
            g(&full, "b.runq_ms"),
            g(&full, "b.slices"),
        );
        let verdict = |c: (f64, f64, f64)| {
            J::S(if !c.1.is_finite() {
                "too few rounds".into()
            } else if c.1 <= 1.0 && c.2 >= 1.0 {
                "no perturbation shown (interval includes no change)".into()
            } else {
                "perturbs: its trace is not interpreted at this thread count".into()
            })
        };
        let cij = |c: (f64, f64, f64)| {
            J::O(vec![
                ("ratio".into(), J::N(c.0)),
                ("lo".into(), J::N(c.1)),
                ("hi".into(), J::N(c.2)),
            ])
        };
        results.push(J::O(vec![
            ("threads".into(), J::N(t as f64)),
            (
                "placement".into(),
                if spec.is_empty() {
                    J::Null
                } else {
                    J::S(spec.clone())
                },
            ),
            (
                "cpus".into(),
                cpus_of
                    .get(&(t, spec.clone()))
                    .map_or(J::Null, |c| J::S(c.clone())),
            ),
            ("slots".into(), J::N(slots.unwrap_or(2 * t) as f64)),
            (
                "above_physical_cores".into(),
                m.cores.map_or(J::Null, |c| J::B(t > c)),
            ),
            (
                "pinned_to_one_cpu".into(),
                J::B(t == 1 && taskset && pin.is_some() && spec.is_empty()),
            ),
            (
                "throughput_mb_s_off".into(),
                J::A(off.iter().map(|x| J::N(*x)).collect()),
            ),
            ("speedup_vs_first".into(), cij(speed)),
            (
                "perturbation".into(),
                J::O(
                    cost.iter()
                        .map(|(mode, c)| {
                            (
                                mode.to_string(),
                                J::O(vec![
                                    ("throughput_ratio".into(), cij(*c)),
                                    ("verdict".into(), verdict(*c)),
                                ]),
                            )
                        })
                        .collect(),
                ),
            ),
            ("timeline".into(), J::O(tl)),
            ("full".into(), J::O(full)),
        ]));
    }
    // each placement's throughput against the first's, paired by round (off mode): the
    // verdict; the per-block clocks above explain it
    for &t in &threads {
        for spec in specs.iter().skip(1) {
            let (a, b) = (off_of(t, spec), off_of(t, &specs[0]));
            let c = ratio_ci(&a.iter().zip(&b).map(|(x, y)| x / y).collect::<Vec<_>>());
            let _ = writeln!(
                table,
                "{t} threads: {spec} [{}] over {} [{}]: throughput {:+.1}% ({:+.1}, {:+.1}), paired by round",
                cpus_of.get(&(t, spec.clone())).map_or("", String::as_str),
                specs[0],
                cpus_of.get(&(t, specs[0].clone())).map_or("", String::as_str),
                100.0 * (c.0 - 1.0),
                100.0 * (c.1 - 1.0),
                100.0 * (c.2 - 1.0)
            );
            comparisons.push(J::O(vec![
                ("threads".into(), J::N(t as f64)),
                ("placement".into(), J::S(spec.clone())),
                ("against".into(), J::S(specs[0].clone())),
                (
                    "throughput_ratio".into(),
                    J::O(vec![
                        ("ratio".into(), J::N(c.0)),
                        ("lo".into(), J::N(c.1)),
                        ("hi".into(), J::N(c.2)),
                    ]),
                ),
            ]));
        }
    }

    let report = J::O(vec![
        ("tool".into(), J::S("examples/occupancy.rs (#39)".into())),
        ("format".into(), J::N(1.0)),
        ("meta".into(), J::O(meta.into_iter().map(|(k, v)| (k, J::S(v))).collect())),
        ("machine".into(), m.json()),
        (
            "settings".into(),
            J::O(vec![
                ("file".into(), J::S(file.clone())),
                ("threads".into(), J::A(threads.iter().map(|t| J::N(*t as f64)).collect())),
                ("rounds".into(), J::N(rounds as f64)),
                ("slots".into(), slots.map_or(J::S("2 x threads (BzReader::parallel's)".into()), |s| J::N(s as f64))),
                (
                    "workload".into(),
                    J::S("BzReader::parallel_with(File, t, slots, Kernels::detect()), drained in 4 MiB reads into a streaming hash: examples/ab.rs's mibz2 arm".into()),
                ),
                ("modes".into(), J::S("off, timeline (src/trace.rs level 1), full (level 2), each round in a rotated order".into())),
            ]),
        ),
        ("parity".into(), J::B(parity)),
        ("output_bytes".into(), J::N(out_bytes)),
        ("schedstat".into(), selftest.clone()),
        ("results".into(), J::A(results)),
        ("placement_comparisons".into(), J::A(comparisons)),
        (
            "runs".into(),
            J::A(raw.iter().map(|(r, t, s, spec, mode, line)| J::O(vec![("round".into(), J::N(*r as f64)), ("threads".into(), J::N(*t as f64)), ("slots".into(), J::N(*s as f64)), ("placement".into(), J::S(spec.clone())), ("mode".into(), J::S(mode.to_string())), ("line".into(), J::S(line.clone()))])).collect()),
        ),
    ]);
    let out = out.unwrap_or_else(|| "occupancy.json".into());
    std::fs::write(&out, format!("{report}\n")).unwrap();

    println!(
        "occupancy: {} · {} CPUs allowed ({}) · physical cores {} · SMT siblings {} · parity {}",
        m.cpu,
        m.allowed,
        m.allowed_str,
        m.cores.map_or("unknown".into(), |c| c.to_string()),
        m.siblings
            .as_ref()
            .map_or("unknown".into(), |v| v.join(" | ")),
        if parity { "ok" } else { "FAILED" }
    );
    println!("schedstat run-queue counter: {selftest}");
    print!("{table}");
    println!(
        "\nMB/s and speedup: the off mode (median; speedup paired by round against {} thread{}). Costs: each traced mode's throughput against off's, paired; an interval excluding 0 means that mode perturbs and its trace is not interpreted.",
        threads[0],
        if threads[0] == 1 { "" } else { "s" }
    );
    println!("Timeline (medians of complete traces; shares of the workers' time): dec = decoding; idle = on the jobs' lock or waiting in recv; i:sl-d, i:sl-r = idle while the scanner waited for a slot and the earliest block not taken was still decoding / ready for the reader (reader backpressure); i:in, i:rd = idle while the scanner waited for input / read a block; i:q = idle while jobs sat in the queue (handoff). s:in = the scanner waiting for input, a share of the span; r:wait = the reader waiting for its next block. Idle during slot waits, in worker-seconds, is in the file: an upper bound on what a wider window could recover.");
    println!("Full, per block over decode_block: elapsed, CPU, run-queue wait and timeslices (Linux). CPU counts memory stalls as running: CPU up with little run-queue wait is an on-CPU cost, not by itself contention. * = more threads than physical cores.");
    println!("Written: {out}");
    if !parity {
        std::process::exit(1);
    }
}

/// The geometric mean of paired ratios and its 95% interval (t distribution)
fn ratio_ci(r: &[f64]) -> (f64, f64, f64) {
    let l: Vec<f64> = r
        .iter()
        .filter(|x| x.is_finite() && **x > 0.0)
        .map(|x| x.ln())
        .collect();
    let n = l.len();
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let m = l.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m.exp(), f64::NAN, f64::NAN);
    }
    let sd = (l.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt();
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    let h = T.get(n - 2).copied().unwrap_or(1.96) * sd / (n as f64).sqrt();
    (m.exp(), (m - h).exp(), (m + h).exp())
}

/// A placement's CPUs for `t` threads, from the machine's own SMT sibling map: `packed`
/// fills each core's siblings before the next core, `spread` takes one CPU a core before
/// any core's second; anything else is a CPU list as taskset takes it
fn placement(spec: &str, t: usize, m: &Machine) -> Result<String, String> {
    if spec != "packed" && spec != "spread" {
        return Ok(spec.to_string());
    }
    let sets: Vec<Vec<usize>> = m
        .siblings
        .as_ref()
        .ok_or("no SMT sibling map on this machine")?
        .iter()
        .map(|s| cpu_list(s))
        .collect();
    let mut cpus = Vec::new();
    if spec == "packed" {
        cpus.extend(sets.iter().flatten().copied());
    } else {
        let depth = sets.iter().map(Vec::len).max().unwrap_or(0);
        for i in 0..depth {
            cpus.extend(sets.iter().filter_map(|s| s.get(i)).copied());
        }
    }
    if cpus.len() < t {
        return Err(format!("{} CPUs, fewer than the threads", cpus.len()));
    }
    let cpus = &cpus[..t];
    if sets.iter().all(|s| s.len() < 2) {
        return Err("no SMT here: packed and spread are the same CPUs".into());
    }
    Ok(cpus
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(","))
}

fn median(v: &mut [f64]) -> f64 {
    let mut v: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// The run-queue counter, checked once, after the timed runs: threads twice the CPUs
/// allowed spin 0.2 s each; the counter is usable if their run-queue wait accounts for at
/// least half their time off a CPU
fn sched_selftest(allowed: usize) -> J {
    #[cfg(target_os = "linux")]
    {
        let n = 2 * allowed.max(1);
        let hs: Vec<_> = (0..n)
            .map(|_| {
                std::thread::spawn(|| {
                    let f = std::fs::File::open("/proc/thread-self/schedstat").ok();
                    let a = f.as_ref().map(trace::schedstat);
                    let t0 = Instant::now();
                    while t0.elapsed() < Duration::from_millis(200) {
                        std::hint::spin_loop();
                    }
                    let wall = t0.elapsed().as_nanos() as f64;
                    let b = f.as_ref().map(trace::schedstat);
                    (wall, a, b)
                })
            })
            .collect();
        let (mut off, mut wait, mut readable) = (0f64, 0f64, true);
        for h in hs {
            let (wall, a, b) = h.join().unwrap();
            match (a, b) {
                (Some(a), Some(b))
                    if a.0 != UNREAD && b.0 != UNREAD && a.1 != UNREAD && b.1 != UNREAD =>
                {
                    off += wall - (b.0 - a.0) as f64;
                    wait += (b.1 - a.1) as f64;
                }
                _ => readable = false,
            }
        }
        let (off, wait) = (off / n as f64 / 1e6, wait / n as f64 / 1e6);
        let status = if !readable {
            "unavailable: /proc/thread-self/schedstat not readable"
        } else if off < 20.0 {
            "unchecked: the self-test's threads were not delayed (no oversubscription seen)"
        } else if wait >= 0.5 * off {
            "validated"
        } else {
            "unavailable: run-queue wait did not rise with time off a CPU (a zero is not no wait)"
        };
        J::O(vec![
            ("status".into(), J::S(status.into())),
            ("selftest_threads".into(), J::N(n as f64)),
            ("selftest_off_cpu_ms_mean".into(), J::N(off)),
            ("selftest_runq_wait_ms_mean".into(), J::N(wait)),
        ])
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = allowed;
        J::O(vec![(
            "status".into(),
            J::S("unavailable: not Linux".into()),
        )])
    }
}

/// The machine as this process sees it; what the OS doesn't expose is unknown, never
/// inferred
struct Machine {
    cpu: String,
    allowed: usize,
    allowed_list: Vec<usize>,
    allowed_str: String,
    cores: Option<usize>,
    siblings: Option<Vec<String>>,
    l3: Option<Vec<String>>,
    numa: Option<Vec<String>>,
    kernel: String,
}

impl Machine {
    fn json(&self) -> J {
        let list = |v: &Option<Vec<String>>| {
            v.as_ref().map_or(J::Null, |v| {
                J::A(v.iter().map(|s| J::S(s.clone())).collect())
            })
        };
        J::O(vec![
            ("cpu".into(), J::S(self.cpu.clone())),
            ("allowed_cpus".into(), J::N(self.allowed as f64)),
            ("allowed_list".into(), J::S(self.allowed_str.clone())),
            (
                "physical_cores_allowed".into(),
                self.cores.map_or(J::Null, |c| J::N(c as f64)),
            ),
            (
                "smt".into(),
                self.cores.map_or(J::Null, |c| J::B(self.allowed > c)),
            ),
            ("smt_sibling_lists".into(), list(&self.siblings)),
            ("l3_shared_cpu_lists".into(), list(&self.l3)),
            ("numa_node_cpu_lists".into(), list(&self.numa)),
            ("kernel".into(), J::S(self.kernel.clone())),
        ])
    }
}

fn read(p: &str) -> Option<String> {
    std::fs::read_to_string(p)
        .ok()
        .map(|s| s.trim().to_string())
}

fn cpu_list(s: &str) -> Vec<usize> {
    let mut v = Vec::new();
    for part in s.split(',').filter(|p| !p.is_empty()) {
        match part.split_once('-') {
            Some((a, b)) => v.extend(
                a.trim().parse::<usize>().unwrap_or(0)..=b.trim().parse::<usize>().unwrap_or(0),
            ),
            None => v.extend(part.trim().parse::<usize>().ok()),
        }
    }
    v
}

fn machine() -> Machine {
    let allowed = std::thread::available_parallelism().map_or(1, |n| n.get());
    let cpu = read("/proc/cpuinfo")
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .or_else(|| {
            Command::new("sysctl")
                .args(["-n", "machdep.cpu.brand_string"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    let allowed_str = read("/proc/self/status")
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Cpus_allowed_list:"))
                .map(|l| l[18..].trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into());
    let allowed_list = if allowed_str == "unknown" {
        (0..allowed).collect()
    } else {
        cpu_list(&allowed_str)
    };
    let topo = |c: usize, f: &str| read(&format!("/sys/devices/system/cpu/cpu{c}/topology/{f}"));
    let cores = allowed_list
        .iter()
        .map(|&c| Some((topo(c, "physical_package_id")?, topo(c, "core_id")?)))
        .collect::<Option<BTreeSet<_>>>()
        .map(|s| s.len());
    let siblings = allowed_list
        .iter()
        .map(|&c| topo(c, "thread_siblings_list"))
        .collect::<Option<BTreeSet<String>>>()
        .map(|s| s.into_iter().collect());
    let l3 = allowed_list
        .iter()
        .map(|&c| {
            (0..8).find_map(|i| {
                let d = format!("/sys/devices/system/cpu/cpu{c}/cache/index{i}");
                (read(&format!("{d}/level"))? == "3")
                    .then(|| read(&format!("{d}/shared_cpu_list")))?
            })
        })
        .collect::<Option<BTreeSet<String>>>()
        .map(|s| s.into_iter().collect());
    let numa = std::fs::read_dir("/sys/devices/system/node").ok().map(|d| {
        let mut v: Vec<String> = d
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("node"))
            .filter_map(|e| {
                read(&format!("{}/cpulist", e.path().display()))
                    .map(|l| format!("{}: {l}", e.file_name().to_string_lossy()))
            })
            .collect();
        v.sort();
        v
    });
    let kernel = Command::new("uname")
        .arg("-sr")
        .output()
        .ok()
        .map_or("unknown".into(), |o| {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        });
    Machine {
        cpu,
        allowed,
        allowed_list,
        allowed_str,
        cores,
        siblings,
        l3,
        numa,
        kernel,
    }
}

// ---------------------------------------------------------------------------------------
// As examples/ab.rs: peak RSS and CPU, the output's hash, the drain

#[repr(C)]
#[derive(Default)]
struct Rusage {
    utime: [i64; 2],
    stime: [i64; 2],
    rest: [i64; 14], // ru_maxrss (KiB on Linux, bytes on macOS) first
}

extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}

/// (CPU seconds, peak RSS) of this process
fn rusage() -> (f64, i64) {
    let mut r = Rusage::default();
    // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
    unsafe { getrusage(0, &mut r) };
    let t = |tv: [i64; 2]| tv[0] as f64 + tv[1] as f64 * 1e-6;
    (t(r.utime) + t(r.stime), r.rest[0])
}

/// A streaming 64-bit hash of the output (examples/ab.rs's), the same however the bytes are
/// cut into reads
struct Hash {
    h: [u64; 4],
    tail: [u8; 32],
    nt: usize,
    n: u64,
}

impl Hash {
    fn new() -> Self {
        Hash {
            h: [1, 2, 3, 4],
            tail: [0; 32],
            nt: 0,
            n: 0,
        }
    }
    #[inline]
    fn block(&mut self, b: &[u8]) {
        for (i, w) in b.chunks_exact(8).enumerate() {
            let w = u64::from_le_bytes(w.try_into().unwrap());
            self.h[i] = (self.h[i] ^ w)
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .rotate_left(31);
        }
    }
    fn update(&mut self, mut b: &[u8]) {
        self.n += b.len() as u64;
        if self.nt > 0 {
            let k = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + k].copy_from_slice(&b[..k]);
            self.nt += k;
            b = &b[k..];
            if self.nt < 32 {
                return;
            }
            let t = self.tail;
            self.block(&t);
            self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for x in &mut c {
            self.block(x);
        }
        let r = c.remainder();
        self.tail[..r.len()].copy_from_slice(r);
        self.nt = r.len();
    }
    fn finish(&self) -> String {
        let mut x = self.n;
        for (i, h) in self.h.iter().enumerate() {
            x = (x ^ h.rotate_left(i as u32 * 16)).wrapping_mul(0xff51_afd7_ed55_8ccd);
        }
        for &b in &self.tail[..self.nt] {
            x = (x ^ b as u64).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        }
        format!("{x:016x}")
    }
}

fn drain(mut r: impl Read, h: &mut Hash) -> Result<(), String> {
    let mut buf = vec![0u8; 4 << 20];
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => h.update(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------------------
// A small JSON writer (no dependencies)

#[derive(Clone)]
enum J {
    N(f64),
    S(String),
    B(bool),
    Null,
    A(Vec<J>),
    O(Vec<(String, J)>),
}

impl std::fmt::Display for J {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            J::N(x) if x.is_finite() => write!(f, "{}", (x * 1e6).round() / 1e6),
            J::N(_) | J::Null => write!(f, "null"),
            J::S(s) => write!(f, "\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            J::B(b) => write!(f, "{b}"),
            J::A(v) => {
                write!(f, "[")?;
                for (i, x) in v.iter().enumerate() {
                    write!(f, "{}{x}", if i > 0 { "," } else { "" })?;
                }
                write!(f, "]")
            }
            J::O(v) => {
                write!(f, "{{")?;
                for (i, (k, x)) in v.iter().enumerate() {
                    write!(f, "{}\"{k}\":{x}", if i > 0 { "," } else { "" })?;
                }
                write!(f, "}}")
            }
        }
    }
}
