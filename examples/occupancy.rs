//! Where the parallel decoder's time goes as its thread count grows (#39): the throughput
//! curve, and the decoder's own timeline beside it. `just bench-occupancy` runs it (with the
//! pinned Wikidata corpus); CI runs the same recipe.
//!
//! ```text
//! occupancy FILE.bz2 [--threads 1,2,4] [--rounds N] [--slots N] [--gap SECONDS] [--out PATH] [--meta K=V]...
//! occupancy FILE.bz2 --ledger [--threads 1,2,4,10] [--slots N]
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
//! **Queues and live structures** (the summary after the table, six lines a thread count,
//! from the same sweep; docs/ledger.md defines each figure):
//! - reader starved: the reader waiting for its next block (the decoder behind; not
//!   backpressure);
//! - scanner backpressured: waiting for a slot while the earliest untaken block is ready
//!   (the reader binds), apart from waiting while it is still decoding (the workers bind);
//! - workers decoding, idle with no job queued, idle with a job queued (the handoff);
//! - the T-vector phase (`decode_block`'s inverse BWT, its build and walk): blocks in it,
//!   mean and most, and their T-vectors' bytes, the hot model;
//! - the window's occupancy (issued and not taken, plus the block the scanner reads in its
//!   slot), the blocks ready and not taken and their bytes, split by what held them (head
//!   of line, the reader busy, the handoff), and how long each was held;
//! - the symbol buffers held, and how much of them dead (MTF done, not yet returned); the
//!   output buffers' capacity against their bytes; the scanner's window; each worker's
//!   buffers as it recorded them at its end, checked against the ledger's bounds.
//!
//! **The footprint ledger** (src/trace.rs, `trace::ledger`): what the decoder reserves,
//! holds live and (a model) reads at random, by category, from the file's level, the threads
//! and the window, with each thread count's totals beside its peak RSS. `--ledger` prints
//! the arithmetic alone, without a run. Reserved is `Vec` capacity, RSS is resident pages,
//! and neither is cache residency.
//!
//! A trace is used only if complete: every thread's start and end, every block's start
//! matched by its end (and its inverse BWT inside it), and the blocks decoded, published
//! and taken the same.
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
        // the decoded bytes the marks sum, against the bytes the reader read: an
        // independent anchor, its remainder shown rather than hidden
        let sum =
            kv.0.iter()
                .find(|(k, _)| k == "t.f.decoded_sum")
                .and_then(|(_, v)| v.parse::<f64>().ok());
        if let Some(sum) = sum {
            kv.f("t.f.decoded_unreconciled", h.n as f64 - sum);
        }
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
        let mut inverting = false;
        for m in &t.marks {
            match m.kind {
                Kind::DecodeStart if open.is_some() => {
                    problem = format!("{}: a block started twice", t.name)
                }
                Kind::DecodeStart => open = Some(m.place),
                Kind::DecodeEnd if open != Some(m.place) => {
                    problem = format!("{}: an end without its start", t.name)
                }
                Kind::DecodeEnd if inverting => {
                    problem = format!("{}: a block ended inside its inverse BWT", t.name)
                }
                Kind::DecodeEnd => {
                    open = None;
                    ended.push(m.place);
                }
                Kind::InvertStart if open.is_none() || inverting => {
                    problem = format!("{}: an inverse BWT outside a block, or twice", t.name)
                }
                Kind::InvertStart => inverting = true,
                Kind::InvertEnd if !inverting => {
                    problem = format!("{}: an inverse BWT's end without its start", t.name)
                }
                Kind::InvertEnd => inverting = false,
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
    // (stable: a thread's marks stay in its order, so each event's mark is its thread's next)
    ev.sort_by_key(|e| e.0);
    let marks_of: Vec<&[trace::Mark]> = traces.iter().map(|t| &t.marks[..]).collect();
    let mut at_of = vec![0usize; traces.len()];
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
    let mut q = Queues::default();
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
            q.credit(dt, reader_waiting, ss == S::Slot && ready.contains(&taken));
        }
        q.step(
            roles[i],
            widx.get(&i).copied(),
            kind,
            place,
            marks_of[i][at_of[i]],
        );
        at_of[i] += 1;
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
    kv.f(
        "t.w.idle.no_job",
        (w_acc[W::Lock as usize] + w_acc[W::Recv as usize] - id_queued) / wspan,
    );
    q.report(span, kv);
    footprint(traces, &roles, kv);

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

/// The sweep's queues and live structures (keys `t.q.*`, sizes `t.f.*`), each interval
/// credited to the state before the mark that ends it:
/// - the window: blocks issued and not taken, plus the one the scanner is reading in the
///   slot it got (`SlotGot` to `Issued`): at most `slots`, by the decoder's own count;
/// - the T-vector phase: workers between `InvertStart` and `InvertEnd` (the build and the
///   walk), and the rows they hold, at 4 bytes a row;
/// - ready, not taken: blocks published (`Ready`) and not yet taken, and their decoded
///   bytes, split three ways by what the reader was doing meanwhile: waiting while the next
///   block in order was still decoding (head of line: held for an earlier block), consuming
///   a block (the reader behind), or waiting with the next block ready (the handoff: its
///   wake-up);
/// - symbols held: from `Issued` until the worker returns the buffer (`Ready`), and of
///   them, dead: after MTF has read them (`InvertStart` on), still held;
/// - the handoff: the reader waiting while the next block in order is ready (its wake-up),
///   and of the scanner's backpressure, the part while the reader was so waiting.
#[derive(Default)]
struct Queues {
    issued: i64,
    taken: i64,
    scan_holds: bool,
    win_int: f64,
    win_hw: i64,
    inv_rows: Vec<u64>,
    tv_n: i64,
    tv_rows: u64,
    tv_int: f64,
    tv_rows_int: f64,
    tv_hw: i64,
    tv_rows_hw: u64,
    rows_max: u64,
    out_len: BTreeMap<u64, u64>,
    out_len_sum: f64,
    out_cap_sum: f64,
    out_cap_max: u64,
    decoded_max: u64,
    ready: BTreeMap<u64, u64>,
    taken_next: u64,
    ready_bytes: i64,
    ready_hw: usize,
    ready_bytes_hw: i64,
    rb_waiting_int: f64,
    rb_busy_int: f64,
    rb_handoff_int: f64,
    r_wait_next_ready: f64,
    bp_reader_waiting: f64,
    hold_sum: f64,
    hold_max: f64,
    held: f64,
    dec_start: Vec<u64>,
    dec_sum: f64,
    dec_n: f64,
    place_of: Vec<u64>,
    dead: Vec<bool>,
    syms_len: BTreeMap<u64, u64>,
    syms_cap_max: u64,
    syms_held: i64,
    syms_dead: i64,
    syms_held_int: f64,
    syms_dead_int: f64,
    window_live_max: u64,
    window_cap_max: u64,
    decoded_sum: u64,
    ready_after_taken: u64,
}

impl Queues {
    fn credit(&mut self, dt: f64, reader_waiting: bool, backpressured: bool) {
        let next_ready = self.ready.contains_key(&self.taken_next);
        if reader_waiting && next_ready {
            self.r_wait_next_ready += dt;
        }
        if backpressured && reader_waiting {
            self.bp_reader_waiting += dt;
        }
        self.win_int += dt * self.window() as f64;
        self.tv_int += dt * self.tv_n as f64;
        self.tv_rows_int += dt * self.tv_rows as f64;
        let rb = dt * self.ready_bytes as f64;
        if !reader_waiting {
            self.rb_busy_int += rb;
        } else if next_ready {
            self.rb_handoff_int += rb;
        } else {
            self.rb_waiting_int += rb;
        }
        self.syms_held_int += dt * self.syms_held as f64;
        self.syms_dead_int += dt * self.syms_dead as f64;
    }

    fn window(&self) -> i64 {
        self.issued - self.taken + self.scan_holds as i64
    }

    fn step(
        &mut self,
        role: Option<Role>,
        w: Option<usize>,
        kind: Kind,
        place: u64,
        m: trace::Mark,
    ) {
        if let Some(w) = w {
            if self.place_of.len() <= w {
                self.place_of.resize(w + 1, u64::MAX);
                self.dead.resize(w + 1, false);
                self.inv_rows.resize(w + 1, 0);
                self.dec_start.resize(w + 1, 0);
            }
        }
        match (role, kind) {
            (Some(Role::Scanner), Kind::SlotGot) => self.scan_holds = true,
            (Some(Role::Scanner), Kind::ThreadEnd) => self.scan_holds = false,
            (Some(Role::Scanner), Kind::Issued) => {
                self.scan_holds = false;
                self.issued += 1;
                if m.len != UNREAD {
                    self.syms_len.insert(place, m.len);
                    self.syms_held += m.len as i64;
                    self.syms_cap_max = self.syms_cap_max.max(m.cap);
                }
            }
            (Some(Role::Scanner), Kind::Window) => {
                self.window_live_max = self.window_live_max.max(m.len);
                self.window_cap_max = self.window_cap_max.max(m.cap);
            }
            (Some(Role::Worker), Kind::DecodeStart) => {
                let w = w.unwrap();
                self.place_of[w] = place;
                self.dec_start[w] = m.ns;
            }
            (Some(Role::Worker), Kind::InvertStart) => {
                let w = w.unwrap();
                self.inv_rows[w] = m.len;
                self.tv_n += 1;
                self.tv_rows += m.len;
                self.tv_hw = self.tv_hw.max(self.tv_n);
                self.tv_rows_hw = self.tv_rows_hw.max(self.tv_rows);
                self.rows_max = self.rows_max.max(m.len);
                self.dead[w] = true;
                self.syms_dead += *self.syms_len.get(&self.place_of[w]).unwrap_or(&0) as i64;
            }
            (Some(Role::Worker), Kind::InvertEnd) => {
                let w = w.unwrap();
                self.tv_n -= 1;
                self.tv_rows -= self.inv_rows[w];
                self.inv_rows[w] = 0;
            }
            (Some(Role::Worker), Kind::DecodeEnd) => {
                let w = w.unwrap();
                self.dec_sum += (m.ns - self.dec_start[w]) as f64;
                self.dec_n += 1.0;
                if m.len != UNREAD {
                    self.out_len.insert(place, m.len);
                    self.out_len_sum += m.len as f64;
                    self.out_cap_sum += m.cap as f64;
                    self.out_cap_max = self.out_cap_max.max(m.cap);
                    self.decoded_max = self.decoded_max.max(m.len);
                    self.decoded_sum += m.len;
                }
            }
            (Some(Role::Worker), Kind::Ready) => {
                let w = w.unwrap();
                let syms = *self.syms_len.get(&place).unwrap_or(&0) as i64;
                self.syms_held -= syms;
                if std::mem::take(&mut self.dead[w]) {
                    self.syms_dead -= syms;
                }
                // (published, then marked outside the lock: the reader may have taken it
                // between, and then it was never waiting)
                if place >= self.taken_next {
                    self.ready.insert(place, m.ns);
                    self.ready_bytes += *self.out_len.get(&place).unwrap_or(&0) as i64;
                    self.ready_hw = self.ready_hw.max(self.ready.len());
                    self.ready_bytes_hw = self.ready_bytes_hw.max(self.ready_bytes);
                } else {
                    self.ready_after_taken += 1;
                }
            }
            (Some(Role::Reader), Kind::Taken) => {
                self.taken += 1;
                self.taken_next = place + 1;
                let mut hold = 0.0;
                if let Some(at) = self.ready.remove(&place) {
                    self.ready_bytes -= *self.out_len.get(&place).unwrap_or(&0) as i64;
                    hold = m.ns.saturating_sub(at) as f64;
                }
                self.hold_sum += hold;
                self.hold_max = self.hold_max.max(hold);
                self.held += 1.0;
            }
            _ => {}
        }
        self.win_hw = self.win_hw.max(self.window());
    }

    fn report(&self, span: f64, kv: &mut Kv) {
        const MB: f64 = 1e6;
        let tt = trace::TT_ROW as f64;
        kv.f("t.r.waiting.next_ready", self.r_wait_next_ready / span);
        kv.f(
            "t.s.slot_earliest_ready.reader_waiting",
            self.bp_reader_waiting / span,
        );
        kv.f("t.q.window_mean", self.win_int / span);
        kv.f("t.q.window_hw", self.win_hw as f64);
        kv.f("t.q.tvec_mean", self.tv_int / span);
        kv.f("t.q.tvec_hw", self.tv_hw as f64);
        kv.f("t.q.tvec_mb_mean", tt * self.tv_rows_int / span / MB);
        kv.f("t.q.tvec_mb_hw", tt * self.tv_rows_hw as f64 / MB);
        kv.f("t.q.ready_hw", self.ready_hw as f64);
        kv.f("t.q.ready_mb_hw", self.ready_bytes_hw as f64 / MB);
        kv.f(
            "t.q.ready_mb_mean",
            (self.rb_waiting_int + self.rb_busy_int + self.rb_handoff_int) / span / MB,
        );
        kv.f(
            "t.q.ready_mb_mean.head_of_line",
            self.rb_waiting_int / span / MB,
        );
        kv.f(
            "t.q.ready_mb_mean.reader_busy",
            self.rb_busy_int / span / MB,
        );
        kv.f("t.q.ready_mb_mean.handoff", self.rb_handoff_int / span / MB);
        kv.f("t.q.hold_ms_mean", self.hold_sum / self.held / 1e6);
        kv.f("t.q.hold_ms_max", self.hold_max / 1e6);
        kv.f("t.q.decode_ms_mean", self.dec_sum / self.dec_n / 1e6);
        kv.f("t.q.syms_mb_mean", self.syms_held_int / span / MB);
        kv.f("t.q.syms_dead_mb_mean", self.syms_dead_int / span / MB);
        kv.f("t.f.rows_max", self.rows_max as f64);
        kv.f("t.f.decoded_max", self.decoded_max as f64);
        kv.f("t.f.out_cap_max_mb", self.out_cap_max as f64 / MB);
        kv.f("t.f.out_cap_over_len", self.out_cap_sum / self.out_len_sum);
        kv.f("t.f.syms_cap_max_mb", self.syms_cap_max as f64 / MB);
        kv.f("t.f.window_live_max_mb", self.window_live_max as f64 / MB);
        kv.f("t.f.window_cap_max_mb", self.window_cap_max as f64 / MB);
        // reconciliation: every figure above is summed from events, never sampled; what
        // the events leave over at the trace's end (each 0 when they balance), and the
        // decoded bytes they sum, for the child to hold against the bytes it read
        kv.f(
            "t.q.residual",
            (self.ready.len() as i64
                + self.ready_bytes.abs()
                + self.syms_held.abs()
                + self.syms_dead.abs()
                + self.tv_n.abs()
                + self.tv_rows as i64
                + self.window().abs()) as f64,
        );
        kv.f("t.q.ready_after_taken", self.ready_after_taken as f64);
        kv.f("t.f.decoded_sum", self.decoded_sum as f64);
    }
}

/// The workers' buffers as each recorded them at its end (their high-water capacities):
/// each buffer's mean over the workers, and a worker's total, mean and largest (`t.f.w.*`)
fn footprint(traces: &[ThreadTrace], roles: &[Option<Role>], kv: &mut Kv) {
    let mut by: BTreeMap<&str, f64> = BTreeMap::new();
    let (mut n, mut max) = (0f64, 0f64);
    for (t, r) in traces.iter().zip(roles) {
        if *r != Some(Role::Worker) || t.buffers.is_empty() {
            continue;
        }
        n += 1.0;
        let mut total = 0f64;
        for b in &t.buffers {
            *by.entry(b.name).or_default() += b.cap as f64;
            total += b.cap as f64;
        }
        max = max.max(total);
    }
    kv.f("t.f.w.recorded", n);
    if n == 0.0 {
        return;
    }
    for (name, cap) in &by {
        kv.f(&format!("t.f.w.{name}.cap_mb"), cap / n / 1e6);
    }
    kv.f("t.f.w.cap_mb", by.values().sum::<f64>() / n / 1e6);
    kv.f("t.f.w.cap_mb_max", max / 1e6);
}

// ---------------------------------------------------------------------------------------
// The sweep: runs, statistics, the machine, the self-test, the report

const MODES: [(&str, u8); 3] = [("off", 0), ("timeline", 1), ("full", 2)];

fn parent(args: &[String]) {
    let mut file = None;
    let (mut threads, mut rounds, mut slots, mut out) = (None, 6usize, None, None);
    let mut gap = 0.0f64;
    let mut placements: Option<Vec<String>> = None;
    let mut meta: Vec<(String, String)> = Vec::new();
    let mut ledger_only = false;
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
            "--gap" => {
                gap = it
                    .next()
                    .expect("--gap SECONDS")
                    .parse()
                    .expect("gap seconds")
            }
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
            "--ledger" => ledger_only = true,
            "-h" | "--help" => {
                println!("occupancy FILE.bz2 [--threads 1,2,4] [--rounds N] [--slots N] [--placements packed/spread] [--gap SECONDS] [--out PATH] [--meta K=V]... [--ledger]");
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
    let (level, n_blocks) = file_level(&file);
    if ledger_only {
        // the arithmetic alone, for full blocks of the file's level: no runs
        let ls: Vec<trace::Settings> = threads
            .iter()
            .map(|&t| trace::Settings::new(level, t, slots.unwrap_or(2 * t)))
            .collect();
        print!("{}", ledger_table(&ls, &[], n_blocks));
        return;
    }
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
                    if gap > 0.0 {
                        std::thread::sleep(Duration::from_secs_f64(gap));
                    }
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
    let mut summary = String::new();
    let mut ledgers: Vec<(trace::Settings, Option<f64>)> = Vec::new();
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
        let s = slots.unwrap_or(2 * t);
        let rows = g(&tl, "f.rows_max");
        let decoded = g(&tl, "f.decoded_max");
        let mut ls = trace::Settings::new(level, t, s);
        if rows.is_finite() && rows > 0.0 {
            (ls.rows, ls.decoded) = (rows as usize, decoded as usize);
        }
        let rss: Vec<f64> = runs.get(&(t, spec.clone(), "off")).map_or(vec![], |v| {
            v.iter().filter_map(|kv| num(kv, "rss_kb")).collect()
        });
        if spec == specs[0] {
            ledgers.push((ls, Some(median(&mut rss.clone()) * 1024.0)));
        }
        summary.push_str(&queue_summary(t, &spec, s, &tl, &trace::ledger(&ls)));
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
                ("gap_seconds".into(), J::N(gap)),
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
    let (ls, rss): (Vec<_>, Vec<_>) = ledgers.into_iter().unzip();
    print!("\n{}", ledger_table(&ls, &rss, n_blocks));
    println!("\nQueues and live structures (timeline medians; docs/ledger.md has the exact definitions):");
    print!("{summary}");
    println!("Written: {out}");
    if !parity {
        std::process::exit(1);
    }
}

/// The file's highest stream level, and its blocks (their symbols read, not decoded)
fn file_level(path: &str) -> (u32, usize) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let (mut level, mut n) = (0, 0);
    for span in mibz2::scan(&data) {
        match span {
            Ok(b) => {
                level = level.max(b.level);
                n += 1;
            }
            Err(e) => {
                eprintln!("occupancy: {path}: {e}");
                break;
            }
        }
    }
    (level.max(1), n)
}

fn mb(x: usize) -> String {
    format!("{:.2}", x as f64 / 1e6)
}

/// A size in B, KB or MB (decimal)
fn size(x: usize) -> String {
    match x {
        x if x >= 100_000 => format!("{:.2} MB", x as f64 / 1e6),
        x if x >= 1000 => format!("{:.1} KB", x as f64 / 1e3),
        x => format!("{x} B"),
    }
}

/// The footprint ledger (src/trace.rs) for each thread count: its entries once (at the
/// first settings), then each thread count's totals, beside its peak RSS where measured
fn ledger_table(ls: &[trace::Settings], rss: &[Option<f64>], blocks: usize) -> String {
    let mut o = String::new();
    let Some(first) = ls.first() else {
        return o;
    };
    let _ = writeln!(
        o,
        "Footprint ledger (src/trace.rs, docs/ledger.md): level {} ({} blocks; at most {} rows a block), longest block {} rows, largest decoded {} MB, input chunks of {} MiB. Each buffer:",
        first.level,
        blocks,
        trace::block_rows(first.level),
        first.rows,
        mb(first.decoded),
        first.chunk >> 20
    );
    let _ = writeln!(
        o,
        "  {:<8} {:<15} {:<16} {:>10} {:>10} {:>10} {:>10}  {:<10}",
        "owner", "buffer", "how many", "reserved", "≤ bound", "live ≤", "hot*", "allocated"
    );
    for e in trace::ledger(first) {
        let _ = writeln!(
            o,
            "  {:<8} {:<15} {:<16} {:>10} {:>10} {:>10} {:>10}  {:<10}",
            e.owner,
            e.name,
            e.per,
            size(e.alloc),
            e.alloc_max.map_or("ratchets".into(), size),
            size(e.live),
            if e.hot > 0 { size(e.hot) } else { "–".into() },
            e.life
        );
    }
    for (i, s) in ls.iter().enumerate() {
        let l = trace::ledger(s);
        let (alloc, max, live, hot) = trace::totals(&l);
        let bounded: Vec<trace::Entry> = l
            .iter()
            .filter(|e| e.alloc_max.is_some())
            .cloned()
            .collect();
        let _ = writeln!(
            o,
            "  {:>3} threads, window {:>3}: reserved ≥ {:>6} MB, ≤ {:>6} MB{} · live ≤ {:>6} MB (each buffer's maximum, summed) · hot* {:>5} MB{}",
            s.threads,
            s.slots,
            mb(alloc),
            mb(max.unwrap_or_else(|| trace::totals(&bounded).1.unwrap_or(0))),
            if max.is_none() { " + the segment buffers (they ratchet: measured below)" } else { "" },
            mb(live),
            mb(hot),
            rss.get(i)
                .copied()
                .flatten()
                .filter(|r| r.is_finite())
                .map_or(String::new(), |r| format!(" · peak RSS (off, median) {:.1} MB", r / 1e6))
        );
    }
    let _ = writeln!(o, "  * hot: a model of what the T-vector phase reads at random (each worker's T-vector), not a measured cache residency. Reserved is Vec capacity: pages never written are not resident, so RSS can be below it; RSS is resident pages, not cache residency.");
    o
}

/// One thread count's queues and live structures, from its timeline medians, in at most
/// six lines; and the ledger's bounds checked against what was measured
fn queue_summary(
    t: usize,
    spec: &str,
    slots: usize,
    tl: &[(String, J)],
    l: &[trace::Entry],
) -> String {
    let g = |k: &str| {
        tl.iter()
            .find(|(kk, _)| kk == k)
            .map_or(f64::NAN, |(_, v)| match v {
                J::N(x) => *x,
                _ => f64::NAN,
            })
    };
    let p = |k: &str| {
        let x = g(k);
        if x.is_finite() {
            format!("{:.1}%", 100.0 * x)
        } else {
            "–".into()
        }
    };
    let pd = |a: &str, b: &str| {
        let x = g(a) - g(b);
        if x.is_finite() {
            format!("{:.1}%", 100.0 * x)
        } else {
            "–".into()
        }
    };
    let f = |k: &str, d: usize| {
        let x = g(k);
        if x.is_finite() {
            format!("{x:.d$}")
        } else {
            "–".into()
        }
    };
    let mut o = String::new();
    let head = if spec.is_empty() {
        format!("{t} threads, window {slots}")
    } else {
        format!("{t} threads {spec}, window {slots}")
    };
    if !g("complete_rounds").is_finite() || g("complete_rounds") == 0.0 {
        let _ = writeln!(o, "{head}: no complete trace");
        return o;
    }
    let _ = writeln!(
        o,
        "{head}: reader waiting {} (starved, the next block not ready, {}; the handoff, ready and the reader not yet woken, {}) · scanner backpressured (slot wait, the earliest untaken block ready: the reader binds) {} ({} of it in the handoff) · slot wait, the earliest untaken still decoding (the workers bind) {} · input wait {}  [of the span]",
        p("r.waiting"),
        pd("r.waiting", "r.waiting.next_ready"),
        p("r.waiting.next_ready"),
        p("s.slot_earliest_ready"),
        p("s.slot_earliest_ready.reader_waiting"),
        p("s.slot_earliest_decoding"),
        p("s.input")
    );
    let _ = writeln!(
        o,
        "  workers: decoding {} · idle, no job queued {} · idle, a job queued (handoff) {}  [of worker time] · mean decoding {} of {t}",
        p("w.decoding"),
        p("w.idle.no_job"),
        p("w.idle.jobs_queued"),
        f("w.mean_decoding", 2)
    );
    let _ = writeln!(
        o,
        "  T-vector phase (build and walk): mean {} blocks, max {} · live T-vectors mean {} MB, max {} MB (the hot model)",
        f("q.tvec_mean", 2),
        f("q.tvec_hw", 0),
        f("q.tvec_mb_mean", 1),
        f("q.tvec_mb_hw", 1)
    );
    let _ = writeln!(
        o,
        "  window: mean {}, max {} of {slots} · ready, not taken: max {} blocks, {} MB; mean {} MB = {} head of line (an earlier block decoding) + {} reader busy + {} handoff · ready→taken mean {} ms, max {} ms (decode mean {} ms)",
        f("q.window_mean", 1),
        f("q.window_hw", 0),
        f("q.ready_hw", 0),
        f("q.ready_mb_hw", 1),
        f("q.ready_mb_mean", 2),
        f("q.ready_mb_mean.head_of_line", 2),
        f("q.ready_mb_mean.reader_busy", 2),
        f("q.ready_mb_mean.handoff", 2),
        f("q.hold_ms_mean", 2),
        f("q.hold_ms_max", 1),
        f("q.decode_ms_mean", 1)
    );
    let _ = writeln!(
        o,
        "  held: symbols mean {} MB, {} MB of it dead (read by MTF, not yet returned) · output buffers reserved/used {}×, largest {} MB · window live ≤ {} MB, reserved ≤ {} MB",
        f("q.syms_mb_mean", 1),
        f("q.syms_dead_mb_mean", 1),
        f("f.out_cap_over_len", 2),
        f("f.out_cap_max_mb", 2),
        f("f.window_live_max_mb", 1),
        f("f.window_cap_max_mb", 1)
    );
    // the worker's buffers (each worker's at its end), largest first
    let mut w: Vec<(String, f64)> = tl
        .iter()
        .filter_map(|(k, v)| {
            match (
                k.strip_prefix("f.w.")
                    .and_then(|k| k.strip_suffix(".cap_mb")),
                v,
            ) {
                (Some(n), J::N(x)) => Some((n.to_string(), *x)),
                _ => None,
            }
        })
        .collect();
    w.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let parts: Vec<String> = w
        .iter()
        .take(4)
        .map(|(n, x)| format!("{n} {x:.2}"))
        .collect();
    // the ledger's bounds against the measurements (medians over the rounds)
    let mut bad = Vec::new();
    let e = |n: &str| l.iter().find(|e| e.name == n);
    let mut check = |what: &str, x: f64, lo: f64, hi: Option<f64>| {
        if x.is_finite() && (x < lo * 0.999 || hi.is_some_and(|h| x > h * 1.001)) {
            bad.push(format!("{what} {x:.0} outside {lo:.0}..={hi:?}"));
        }
    };
    for n in [
        "ll",
        "tt",
        "rle",
        "seg.starts",
        "seg.next",
        "seg.bufs.outer",
    ] {
        if let Some(e) = e(n) {
            check(
                n,
                g(&format!("f.w.{n}.cap_mb")) * 1e6,
                e.alloc as f64,
                e.alloc_max.map(|m| m as f64),
            );
        }
    }
    if let Some(e) = e("symbols") {
        check(
            "symbols",
            g("f.syms_cap_max_mb") * 1e6,
            e.alloc as f64,
            e.alloc_max.map(|m| m as f64),
        );
    }
    if let Some(e) = e("out") {
        check(
            "out",
            g("f.out_cap_max_mb") * 1e6,
            0.0,
            e.alloc_max.map(|m| m as f64),
        );
    }
    if let Some(e) = e("window") {
        check(
            "window",
            g("f.window_cap_max_mb") * 1e6,
            0.0,
            e.alloc_max.map(|m| m as f64),
        );
    }
    check(
        "window occupancy",
        g("q.window_hw"),
        0.0,
        Some(slots as f64),
    );
    check("T-vector phase", g("q.tvec_hw"), 0.0, Some(t as f64));
    // the events' reconciliations: what they leave over at the end, the decoded bytes
    // they sum against the bytes read, and every worker's buffers recorded
    let mut rec = Vec::new();
    if g("q.residual") != 0.0 {
        rec.push(format!("end state not empty ({})", f("q.residual", 0)));
    }
    if g("f.decoded_unreconciled") != 0.0 {
        rec.push(format!(
            "decoded bytes off by {}",
            f("f.decoded_unreconciled", 0)
        ));
    }
    if g("f.w.recorded") != t as f64 {
        rec.push(format!(
            "{} of {t} workers' buffers recorded",
            f("f.w.recorded", 0)
        ));
    }
    let _ = writeln!(
        o,
        "  worker buffers (each at its end, mean MB): {} total = {} · ledger check: {} · events reconciled: {}{}",
        f("f.w.cap_mb", 2),
        parts.join(" + "),
        if bad.is_empty() { "ok".into() } else { format!("VIOLATED: {}", bad.join("; ")) },
        if rec.is_empty() { "ok (Σ decoded = bytes read, end state empty, every worker recorded)".into() } else { format!("NO: {}", rec.join("; ")) },
        if g("q.ready_after_taken") > 0.0 { format!(" · {} blocks marked ready after taken (held 0)", f("q.ready_after_taken", 0)) } else { String::new() }
    );
    o
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

#[cfg(test)]
mod tests {
    use super::*;
    use mibz2::trace::{Mark, NONE};

    fn mk(ns: u64, kind: Kind, place: u64, len: u64, cap: u64) -> Mark {
        Mark {
            ns,
            kind,
            place,
            cpu_ns: UNREAD,
            run_ns: UNREAD,
            wait_ns: UNREAD,
            slices: UNREAD,
            len,
            cap,
        }
    }

    /// A thread's marks, its times in ms and its sizes in MB (so that every figure below
    /// is well above the 6 decimals a key is written with)
    fn thread(name: &str, marks: &[(u64, Kind, u64, u64, u64)]) -> ThreadTrace {
        let mb = |x: u64| if x == UNREAD { x } else { x * 1_000_000 };
        ThreadTrace {
            name: name.into(),
            marks: marks
                .iter()
                .map(|&(ms, k, p, l, c)| mk(ms * 1_000_000, k, p, mb(l), mb(c)))
                .collect(),
            buffers: vec![],
        }
    }

    /// The queue summary's definitions on a hand-built trace: 2 workers, a window of 2,
    /// 3 blocks, over 90 ms (sizes in MB, rows in millions). Block 1 is ready at 35 while the reader still waits for block
    /// 0 (head of line); the reader consumes block 0 from 51 to 80, while the scanner waits
    /// for a slot with block 1 ready (backpressure: the reader binds).
    #[test]
    fn queue_definitions() {
        use Kind::*;
        const U: u64 = UNREAD;
        let n = NONE;
        let scan = thread(
            "mibz2-scan",
            &[
                (0, ThreadStart, n, U, U),
                (0, SlotWait, n, U, U),
                (0, SlotGot, n, U, U),
                (5, Window, n, 1000, 4096),
                (10, Issued, 0, 200, 1000),
                (10, Enqueued, 0, U, U),
                (10, SlotWait, n, U, U),
                (10, SlotGot, n, U, U),
                (20, Issued, 1, 100, 1000),
                (20, Enqueued, 1, U, U),
                (20, SlotWait, n, U, U),
                (52, SlotGot, n, U, U),
                (55, Issued, 2, 50, 1000),
                (55, Enqueued, 2, U, U),
                (55, SlotWait, n, U, U),
                (81, SlotGot, n, U, U),
                (82, ThreadEnd, n, U, U),
            ],
        );
        let w0 = thread(
            "mibz2-0",
            &[
                (0, ThreadStart, n, U, U),
                (0, JobLock, n, U, U),
                (0, JobRecv, n, U, U),
                (10, JobGot, 0, U, U),
                (10, DecodeStart, 0, U, U),
                (15, InvertStart, n, 1000, U),
                (45, InvertEnd, n, U, U),
                (50, DecodeEnd, 0, 900, 1064),
                (50, Ready, 0, U, U),
                (50, JobLock, n, U, U),
                (50, JobRecv, n, U, U),
                (57, JobGot, 2, U, U),
                (57, DecodeStart, 2, U, U),
                (58, InvertStart, n, 100, U),
                (60, InvertEnd, n, U, U),
                (61, DecodeEnd, 2, 90, 1064),
                (61, Ready, 2, U, U),
                (61, JobLock, n, U, U),
                (61, JobRecv, n, U, U),
                (90, ThreadEnd, n, U, U),
            ],
        );
        let w1 = thread(
            "mibz2-1",
            &[
                (0, ThreadStart, n, U, U),
                (0, JobLock, n, U, U),
                (0, JobRecv, n, U, U),
                (20, JobGot, 1, U, U),
                (20, DecodeStart, 1, U, U),
                (22, InvertStart, n, 500, U),
                (30, InvertEnd, n, U, U),
                (35, DecodeEnd, 1, 400, 2128),
                (35, Ready, 1, U, U),
                (35, JobLock, n, U, U),
                (35, JobRecv, n, U, U),
                (90, ThreadEnd, n, U, U),
            ],
        );
        let reader = thread(
            "main",
            &[
                (0, ThreadStart, n, U, U),
                (0, ReadWait, n, U, U),
                (51, Taken, 0, U, U),
                (80, ReadWait, n, U, U),
                (80, Taken, 1, U, U),
                (85, ReadWait, n, U, U),
                (85, Taken, 2, U, U),
                (88, ReadWait, n, U, U),
                (90, ThreadEnd, n, U, U),
            ],
        );
        let mut kv = Kv::default();
        summarise(&[scan, w0, w1, reader], 2, 1, None, &mut kv);
        let got: BTreeMap<String, String> = kv.0.into_iter().collect();
        let v = |k: &str| -> f64 {
            got.get(k)
                .unwrap_or_else(|| panic!("no {k}"))
                .parse()
                .unwrap_or_else(|_| panic!("{k} = {}", got[k]))
        };
        assert_eq!(got["t.complete"], "yes", "{}", got["t.problem"]);
        let close = |k: &str, want: f64| {
            let x = v(k);
            assert!(
                (x - want).abs() <= 1e-6 + 1e-6 * want.abs(),
                "{k}: {x}, want {want}"
            );
        };
        let (span, wspan) = (90.0, 180.0);
        // the reader waiting for its next block, 0-51 and 88-90: of it, 50-51 with block 0
        // ready (the handoff: its wake-up), the rest starved
        close("t.r.waiting", 53.0 / span);
        close("t.r.waiting.next_ready", 1.0 / span);
        // the scanner waiting for a slot: the earliest untaken block still decoding (20-50:
        // the workers bind), or ready (50-52, 55-81: the reader binds, backpressure)
        close("t.s.slot_earliest_decoding", 30.0 / span);
        close("t.s.slot_earliest_ready", 28.0 / span);
        // (of the backpressure, 50-51 while the reader was waking to block 0)
        close("t.s.slot_earliest_ready.reader_waiting", 1.0 / span);
        // workers: decoding 10-50, 57-61 and 20-35; idle 121 worker-ns, 4 of it (55-57,
        // both idle) with a job queued, so 117 with none
        close("t.w.decoding", 59.0 / wspan);
        close("t.w.idle", 121.0 / wspan);
        close("t.w.idle.jobs_queued", 4.0 / wspan);
        close("t.w.idle.no_job", 117.0 / wspan);
        // the T-vector phase: 15-45 (1000 rows), 22-30 (500), 58-60 (100)
        close("t.q.tvec_mean", 40.0 / span);
        close("t.q.tvec_hw", 2.0);
        close("t.q.tvec_mb_mean", 4.0 * 34_200.0 / span);
        close("t.q.tvec_mb_hw", 4.0 * 1500.0);
        // the window: issued and not taken, plus the block the scanner reads in its slot
        close("t.q.window_mean", 155.0 / span);
        close("t.q.window_hw", 2.0);
        // ready, not taken: block 1 35-50 head of line (block 0 decoding), 0 and 1 50-51
        // at the handoff, then 1 (51-80) and 2 (61-85) while the reader consumed
        close("t.q.ready_hw", 2.0);
        close("t.q.ready_mb_hw", 1300.0);
        close("t.q.ready_mb_mean.head_of_line", 6000.0 / span);
        close("t.q.ready_mb_mean.handoff", 1300.0 / span);
        close("t.q.ready_mb_mean.reader_busy", 13_760.0 / span);
        close("t.q.hold_ms_mean", (1.0 + 45.0 + 24.0) / 3.0);
        close("t.q.hold_ms_max", 45.0);
        // symbols held from their read to their return, dead from MTF's end
        close("t.q.syms_mb_mean", 9800.0 / span);
        close("t.q.syms_dead_mb_mean", 8450.0 / span);
        close("t.f.rows_max", 1000.0 * 1e6);
        close("t.f.out_cap_max_mb", 2128.0);
        close("t.f.out_cap_over_len", (1064.0 * 2.0 + 2128.0) / 1390.0);
        close("t.f.window_cap_max_mb", 4096.0);
        close("t.q.residual", 0.0);
        close("t.q.ready_after_taken", 0.0);
        close("t.f.decoded_sum", 1390.0 * 1e6);
        close("t.f.window_live_max_mb", 1000.0);
        close("t.q.decode_ms_mean", (40.0 + 15.0 + 4.0) / 3.0);
        close("t.f.syms_cap_max_mb", 1000.0);
    }

    /// An inverse BWT outside its block makes the trace incomplete
    #[test]
    fn unbalanced_inverse_is_incomplete() {
        use Kind::*;
        const U: u64 = UNREAD;
        let n = NONE;
        let t = vec![
            thread(
                "mibz2-scan",
                &[(0, ThreadStart, n, U, U), (1, ThreadEnd, n, U, U)],
            ),
            thread(
                "mibz2-0",
                &[
                    (0, ThreadStart, n, U, U),
                    (1, InvertStart, n, 10, U),
                    (2, ThreadEnd, n, U, U),
                ],
            ),
            thread(
                "main",
                &[(0, ThreadStart, n, U, U), (2, ThreadEnd, n, U, U)],
            ),
        ];
        let mut kv = Kv::default();
        summarise(&t, 1, 1, None, &mut kv);
        let got: BTreeMap<String, String> = kv.0.into_iter().collect();
        assert_eq!(got["t.complete"], "no");
    }
}
