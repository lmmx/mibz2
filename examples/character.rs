//! (research only, `character`) One decode of bench-character, the decoder's character across
//! worker counts, stage 1 (docs/journal/2026-11-24-algorithm-level/briefs/character.md;
//! docs/journal/2026-11-24-character-stage1-registration.md), in a process of its own:
//!
//!     character --one FILE.bz2 K ON BLOCKS.tsv|- [PMU]
//!     character --api FILE.bz2 THREADS
//!     character --features FILE.bz2 OUT.tsv
//!
//! **`--one`**, the mechanism: production's lazy workers at exactly K
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, a window of 4K blocks, the ready bytes'
//! cap of 12 MiB with its floor of K + 2), the caller's thread reading with a 4 MiB buffer.
//! FILE is read into memory before the clock starts, and the decoder reads it from there. With
//! ON = 1, every thread's ledger (src/character.rs) is kept and read here: each thread's
//! timeline partitioned into its activities and waits, each block's demand at each activity,
//! the buffers' occupancy, and the accounting identities (the operational laws), checked.
//! With ON = 0 the ledgers are off (the instrumentation's cost check, and the counters' runs
//! under `perf stat`). Where BLOCKS.tsv is given, every block's row is appended to it.
//!
//! **PMU** (1 or 2; the ledgers off): the decode's hardware counters, opened in this process
//! before the decoder (inherited by its threads), enabled for the decode alone and read once
//! its threads are joined, each with its time enabled and running (its coverage, where the
//! kernel multiplexes them). Group 1: cycles, instructions, ref-cycles, branches and branch
//! misses. Group 2: last-level-cache load misses, L2 misses (`L2_RQSTS.MISS`), dTLB load
//! misses and memory-stall cycles (`CYCLE_ACTIVITY.STALLS_MEM_ANY`); the last two of these are
//! the i9's (Comet Lake's) raw codes, meaningless elsewhere. Group 3, software events (task
//! clock, context switches, migrations, page faults), checks the counters anywhere.
//!
//! **`--api`**, the public API: `BzReader::parallel(FILE, THREADS)` as shipped, and what it
//! made of the request (the decoder's threads, window and kernel).
//!
//! **`--features`**, untimed: FILE decoded at 1 worker with the ledgers on, and each block's
//! place, route, rows (n), run records (r), decoded bytes and the first-order entropy of its
//! decoded bytes (h₁, bits a byte), one row a block.
//!
//! Either way (`--one`, `--api`), the process's memory and scheduling over the decode: its resident memory at the
//! start (the input and runtime), its peak (`VmHWM`, reset first) and time-weighted mean
//! (`/proc/self/statm` every 5 ms), and from `getrusage` its user and system CPU, minor and
//! major faults and voluntary and involuntary context switches. One JSON line on stdout.
use mibz2::character::{self as ch, route, Ev, Ledger, ACTIVITIES, BUFFERS, OCCS, UNREAD};
use std::fmt::Write as _;
use std::io::{Read, Write as _};
use std::time::{Duration, Instant};

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

/// This process's resource use so far (RUSAGE_SELF): user s, system s, minor and major
/// faults, voluntary and involuntary context switches
fn usage() -> [f64; 6] {
    let mut r = Rusage::default();
    // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
    unsafe { getrusage(0, &mut r) };
    let t = |tv: [i64; 2]| tv[0] as f64 + tv[1] as f64 * 1e-6;
    [
        t(r.utime),
        t(r.stime),
        r.rest[4] as f64,
        r.rest[5] as f64,
        r.rest[12] as f64,
        r.rest[13] as f64,
    ]
}

fn reset_peak() -> bool {
    std::fs::write("/proc/self/clear_refs", "5").is_ok()
}

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

fn rss_kb() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: u64 = s.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4)
}

const SAMPLE: Duration = Duration::from_millis(5);

/// The hardware counters (`perf_event_open`), group by group
mod pmu {
    extern "C" {
        fn syscall(n: std::ffi::c_long, ...) -> std::ffi::c_long;
        fn ioctl(fd: i32, req: std::ffi::c_ulong, ...) -> i32;
        fn read(fd: i32, buf: *mut std::ffi::c_void, n: usize) -> isize;
        fn close(fd: i32) -> i32;
    }
    #[cfg(target_arch = "x86_64")]
    const NR: std::ffi::c_long = 298;
    #[cfg(target_arch = "aarch64")]
    const NR: std::ffi::c_long = 241;
    const HW: u64 = 0;
    const CACHE: u64 = 3;
    const RAW: u64 = 4;
    const SW: u64 = 1;

    /// The groups' events: (name, type, config)
    pub fn group(g: u8) -> Vec<(&'static str, u64, u64)> {
        match g {
            1 => vec![
                ("cycles", HW, 0),
                ("instructions", HW, 1),
                ("ref-cycles", HW, 9),
                ("branches", HW, 4),
                ("branch-misses", HW, 5),
            ],
            2 => vec![
                // (LL, read, miss), (dTLB, read, miss)
                ("LLC-load-misses", CACHE, 2 | 1 << 16),
                ("l2_rqsts.miss", RAW, 0x3F24),
                ("dTLB-load-misses", CACHE, 3 | 1 << 16),
                (
                    "cycle_activity.stalls_mem_any",
                    RAW,
                    0xA3 | 0x14 << 8 | 20 << 24,
                ),
            ],
            // (the software group: the counters' check where there is no hardware PMU)
            3 => vec![
                ("task-clock", SW, 1),
                ("context-switches", SW, 3),
                ("cpu-migrations", SW, 4),
                ("page-faults", SW, 2),
            ],
            _ => vec![],
        }
    }

    /// One counter, opened disabled and inherited by the threads started after; kernel mode
    /// too where allowed (`true`), else user only
    fn open(kind: u64, config: u64, user_only: bool) -> Option<i32> {
        // perf_event_attr's first 64 bytes (PERF_ATTR_SIZE_VER0), the rest zero: type and
        // size, config, read_format (the times enabled and running), the flags (disabled,
        // inherit, exclude_kernel where user only, exclude_hv)
        let mut attr = [0u64; 8];
        attr[0] = kind | (64 << 32);
        attr[1] = config;
        attr[4] = 1 | 2;
        attr[5] = 1 | 2 | if user_only { 1 << 5 } else { 0 } | 1 << 6;
        // SAFETY: a valid perf_event_attr of the size it declares; this process, any CPU, no
        // group, close-on-exec
        let fd = unsafe { syscall(NR, attr.as_ptr(), 0i64, -1i64, -1i64, 8u64) };
        (fd >= 0).then_some(fd as i32)
    }

    pub struct Counters {
        fds: Vec<(&'static str, Option<i32>)>,
        pub scope: &'static str,
    }

    impl Counters {
        pub fn open(g: u8) -> Counters {
            let ev = group(g);
            let both: Vec<_> = ev.iter().map(|&(n, t, c)| (n, open(t, c, false))).collect();
            if both.iter().all(|x| x.1.is_some()) {
                return Counters {
                    fds: both,
                    scope: "user+kernel",
                };
            }
            for (_, fd) in both.into_iter().filter_map(|x| x.1.map(|f| ((), f))) {
                // SAFETY: an fd this opened
                unsafe { close(fd) };
            }
            Counters {
                fds: ev.iter().map(|&(n, t, c)| (n, open(t, c, true))).collect(),
                scope: "user",
            }
        }
        fn each(&self, req: std::ffi::c_ulong) {
            for &(_, fd) in &self.fds {
                if let Some(fd) = fd {
                    // SAFETY: an fd this opened; ENABLE/DISABLE take no argument
                    unsafe { ioctl(fd, req, 0) };
                }
            }
        }
        pub fn enable(&self) {
            self.each(0x2400);
        }
        pub fn disable(&self) {
            self.each(0x2401);
        }
        /// Each counter's value, time enabled and time running (None where it did not open)
        pub fn read(&self) -> Vec<(&'static str, Option<[u64; 3]>)> {
            self.fds
                .iter()
                .map(|&(n, fd)| {
                    (
                        n,
                        fd.and_then(|fd| {
                            let mut v = [0u64; 3];
                            // SAFETY: 24 bytes into a buffer of 24
                            let got = unsafe { read(fd, v.as_mut_ptr().cast(), 24) };
                            (got == 24).then_some(v)
                        }),
                    )
                })
                .collect()
        }
    }

    impl Drop for Counters {
        fn drop(&mut self) {
            for &(_, fd) in &self.fds {
                if let Some(fd) = fd {
                    // SAFETY: an fd this opened
                    unsafe { close(fd) };
                }
            }
        }
    }
}

/// A decode's process figures, as JSON fields
struct Proc {
    wall: f64,
    bytes: u64,
    base_kb: u64,
    peak_kb: u64,
    mean_kb: f64,
    samples: u64,
    use_: [f64; 6],
}

impl Proc {
    fn json(&self) -> String {
        format!(
            "\"wall_s\":{:.6},\"bytes\":{},\"base_kb\":{},\"peak_kb\":{},\"mean_kb\":{:.1},\"rss_kb_s\":{:.1},\"samples\":{},\"user_s\":{:.6},\"sys_s\":{:.6},\"minflt\":{},\"majflt\":{},\"nvcsw\":{},\"nivcsw\":{}",
            self.wall,
            self.bytes,
            self.base_kb,
            self.peak_kb,
            self.mean_kb,
            self.mean_kb * self.wall,
            self.samples,
            self.use_[0],
            self.use_[1],
            self.use_[2],
            self.use_[3],
            self.use_[4],
            self.use_[5]
        )
    }
}

/// A decode with the process's figures over it: `make` builds the reader (the clock running)
fn measured<R: Read>(make: impl FnOnce() -> R, after: impl FnOnce(&R)) -> (Proc, R) {
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let st = stop.clone();
    let base_kb = rss_kb().unwrap_or(0);
    let reset = reset_peak();
    let u0 = usage();
    let sampler = std::thread::Builder::new()
        .name("sampler".into())
        .spawn(move || {
            let (mut sum, mut n) = (0u64, 0u64);
            while !st.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(kb) = rss_kb() {
                    sum += kb;
                    n += 1;
                }
                std::thread::sleep(SAMPLE);
            }
            (sum, n)
        })
        .expect("the sampler");
    let mut buf = vec![0u8; 4 << 20];
    let s = Instant::now();
    let mut r = make();
    after(&r);
    let mut bytes = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(x) => bytes += x as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    let wall = s.elapsed().as_secs_f64();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (sum, n) = sampler.join().expect("the sampler");
    let u1 = usage();
    let mut use_ = [0.0; 6];
    for i in 0..6 {
        use_[i] = u1[i] - u0[i];
    }
    (
        Proc {
            wall,
            bytes,
            base_kb,
            peak_kb: if reset { peak_kb() } else { 0 },
            mean_kb: if n > 0 { sum as f64 / n as f64 } else { 0.0 },
            samples: n,
            use_,
        },
        r,
    )
}

const NA: usize = ACTIVITIES.len();
/// The ledger's activity tags' index (`Tag as u8` for A0..ReadWait)
fn act(tag: u8) -> Option<usize> {
    (usize::from(tag) < NA).then_some(usize::from(tag))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    Feeder,
    Scanner,
    Worker,
    Reader,
}

fn role(name: &str) -> Role {
    match name {
        "mibz2-read" => Role::Feeder,
        "mibz2-scan" => Role::Scanner,
        n if n
            .strip_prefix("mibz2-")
            .is_some_and(|d| d.parse::<u32>().is_ok()) =>
        {
            Role::Worker
        }
        _ => Role::Reader,
    }
}

/// The decode's phases: start-up (to the first block taken by the reader), steady (to the
/// last block issued), drain (to the end)
const PHASES: [&str; 3] = ["startup", "steady", "drain"];

#[derive(Default, Clone)]
struct Block {
    route: u8,
    n: u64,
    r: u64,
    syms_bytes: u64,
    out_bytes: u64,
    worker: i64,
    /// Issued, Enqueued, Got, Ready, Taken (ns), `UNREAD` where missing
    t: [u64; 5],
    /// counts of each point (each must be 1)
    seen: [u32; 6],
    scan: [u64; NA + 1],
    work: [u64; NA + 1],
    read_wait: u64,
}

struct ThreadOut {
    name: String,
    role: Role,
    life: u64,
    /// per phase, per activity (NA: the residual)
    states: [[u64; NA + 1]; 3],
    events: usize,
    balanced: bool,
    monotone: bool,
    wait_wall: u64,
    wait_cpu: u64,
    cpu: u64,
    sched: [u64; 6],
    migrations: u64,
    /// begins of each activity
    visits: [u64; NA],
    /// per-block attribution left over: activity time on a worker outside any block, or the
    /// scanner's after its last block
    unattributed: [u64; NA + 1],
    /// the scanner's waits for admission by what held it: the window, the ready bytes' cap
    slot_bound: [u64; 2],
}

/// Split `[a, b)` across the phases' bounds, adding to `out[phase][i]`
fn add(out: &mut [[u64; NA + 1]; 3], i: usize, a: u64, b: u64, t1: u64, t2: u64) {
    let cuts = [0, t1, t2, u64::MAX];
    for p in 0..3 {
        let lo = a.max(cuts[p]);
        let hi = b.min(cuts[p + 1]);
        if hi > lo {
            out[p][i] += hi - lo;
        }
    }
}

fn walk(
    l: &Ledger,
    idx: i64,
    blocks: &mut [Block],
    t1: u64,
    t2: u64,
    pending_wait: &mut u64,
) -> ThreadOut {
    let ro = role(&l.name);
    let ev: &[Ev] = &l.events;
    let mut o = ThreadOut {
        name: l.name.clone(),
        role: ro,
        life: 0,
        states: [[0; NA + 1]; 3],
        events: ev.len(),
        balanced: true,
        monotone: true,
        wait_wall: 0,
        wait_cpu: 0,
        cpu: 0,
        sched: [
            l.sched0[0],
            l.sched0[1],
            l.sched0[2],
            l.sched1[0],
            l.sched1[1],
            l.sched1[2],
        ],
        migrations: if l.migrations0 == UNREAD || l.migrations1 == UNREAD {
            UNREAD
        } else {
            l.migrations1 - l.migrations0
        },
        visits: [0; NA],
        unattributed: [0; NA + 1],
        slot_bound: [0; 2],
    };
    let mut bound = 0usize;
    if ev.is_empty() {
        o.balanced = false;
        return o;
    }
    let start = ev[0];
    let end = *ev.last().unwrap();
    o.life = end.ns.saturating_sub(start.ns);
    if start.cpu != UNREAD && end.cpu != UNREAD {
        o.cpu = end.cpu - start.cpu;
    }
    if start.tag != ch::Tag::Start as u8 || end.tag != ch::Tag::End as u8 {
        o.balanced = false;
    }
    let mut stack: Vec<u8> = Vec::new();
    let mut last = start.ns;
    let mut wait_from: Option<(u64, u64)> = None;
    // the block this thread's time goes to: a worker's current, or the scanner's pending
    let mut cur: Option<usize> = None;
    let mut pend = [0u64; NA + 1];
    let mut last_issued: Option<usize> = None;
    let mut wait_acc = 0u64;
    for e in ev {
        if e.ns < last {
            o.monotone = false;
        }
        let now = e.ns.max(last);
        if now > last {
            let i = stack.last().map_or(NA, |&t| usize::from(t));
            add(&mut o.states, i, last, now, t1, t2);
            let d = now - last;
            match ro {
                Role::Scanner => pend[i] += d,
                Role::Worker => match cur {
                    Some(p) if i != ch::Tag::JobWait as usize => blocks[p].work[i] += d,
                    _ => o.unattributed[i] += d,
                },
                Role::Reader if i == ch::Tag::ReadWait as usize => wait_acc += d,
                _ => {}
            }
        }
        last = now;
        match e.edge {
            1 => {
                stack.push(e.tag);
                if let Some(a) = act(e.tag) {
                    o.visits[a] += 1;
                }
                if ch::is_wait(e.tag) {
                    wait_from = Some((e.ns, e.cpu));
                }
                if ro == Role::Worker && e.tag == ch::Tag::JobWait as u8 {
                    cur = None;
                }
            }
            2 => {
                if stack.pop() != Some(e.tag) {
                    o.balanced = false;
                }
                if ch::is_wait(e.tag) {
                    if let Some((w0, c0)) = wait_from.take() {
                        o.wait_wall += e.ns - w0;
                        if e.tag == ch::Tag::SlotWait as u8 {
                            o.slot_bound[bound] += e.ns - w0;
                            bound = 0;
                        }
                        if c0 != UNREAD && e.cpu != UNREAD {
                            o.wait_cpu += e.cpu.saturating_sub(c0);
                        }
                    } else {
                        o.balanced = false;
                    }
                }
                // the scanner's block: everything since its last block, charged at the end
                // of the issue
                if ro == Role::Scanner && e.tag == ch::Tag::Issue as u8 {
                    if let Some(p) = last_issued.take() {
                        for (a, b) in blocks[p].scan.iter_mut().zip(pend.iter()) {
                            *a += b;
                        }
                        pend = [0; NA + 1];
                    }
                }
            }
            _ => {
                let p = e.place as usize;
                let k = e.tag;
                let b = |blocks: &mut [Block], i: usize| -> Option<usize> {
                    if p < blocks.len() {
                        blocks[p].seen[i] += 1;
                        Some(p)
                    } else {
                        None
                    }
                };
                if k == ch::Tag::Issued as u8 {
                    if let Some(p) = b(blocks, 0) {
                        blocks[p].t[0] = e.ns;
                        blocks[p].syms_bytes = e.x;
                        last_issued = Some(p);
                    }
                } else if k == ch::Tag::Enqueued as u8 {
                    if let Some(p) = b(blocks, 1) {
                        blocks[p].t[1] = e.ns;
                    }
                } else if k == ch::Tag::Got as u8 {
                    if let Some(p) = b(blocks, 2) {
                        blocks[p].t[2] = e.ns;
                        blocks[p].worker = idx;
                        cur = Some(p);
                    }
                } else if k == ch::Tag::Ready as u8 {
                    if let Some(p) = b(blocks, 3) {
                        blocks[p].t[3] = e.ns;
                        blocks[p].out_bytes = e.x;
                    }
                } else if k == ch::Tag::Taken as u8 {
                    if let Some(p) = b(blocks, 4) {
                        blocks[p].t[4] = e.ns;
                        blocks[p].read_wait = wait_acc;
                        wait_acc = 0;
                    }
                } else if k == ch::Tag::Bound as u8 {
                    bound = usize::from(e.x == 1);
                } else if k == ch::Tag::Route as u8 {
                    if let Some(p) = b(blocks, 5) {
                        let (r, n, rr) = route::unpack(e.x);
                        (blocks[p].route, blocks[p].n, blocks[p].r) = (r, n, rr);
                    }
                }
            }
        }
    }
    if !stack.is_empty() {
        o.balanced = false;
    }
    if ro == Role::Scanner {
        o.unattributed = pend;
    }
    *pending_wait += wait_acc;
    o
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("--one") if a.len() == 6 || a.len() == 7 => one(
            &a[2],
            a[3].parse().expect("K"),
            a[4] == "1",
            &a[5],
            a.get(6).map_or(0, |g| g.parse().expect("PMU")),
        ),
        Some("--api") if a.len() == 4 => api(&a[2], a[3].parse().expect("THREADS")),
        Some("--features") if a.len() == 4 => features(&a[2], &a[3]),
        _ => {
            eprintln!(
                "usage: character --one FILE.bz2 K ON BLOCKS.tsv|- [PMU]\n       character --api FILE.bz2 THREADS"
            );
            std::process::exit(2);
        }
    }
}

fn features(file: &str, out: &str) {
    let data: &'static [u8] = std::fs::read(file).expect("read the input").leak();
    ch::set_on(true);
    let mut r = mibz2::BzReader::parallel_workers(
        data,
        1,
        mibz2::parallel::SLOTS_PER_THREAD,
        mibz2::Kernels::detect(),
        true,
    );
    let mut bytes = Vec::new();
    r.read_to_end(&mut bytes).expect("decode");
    drop(r);
    let deadline = Instant::now() + Duration::from_secs(10);
    while ch::finished() < 3 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    ch::set_on(false);
    let ledgers = ch::take();
    let mut rows: Vec<(u32, u64, u64)> = Vec::new(); // (place, route packing, bytes)
    let mut routes = std::collections::HashMap::new();
    for l in &ledgers {
        for e in &l.events {
            if e.edge == 0 && e.tag == ch::Tag::Ready as u8 {
                rows.push((e.place, 0, e.x));
            } else if e.edge == 0 && e.tag == ch::Tag::Route as u8 {
                routes.insert(e.place, e.x);
            }
        }
    }
    rows.sort();
    let mut s = String::from("place\troute\tn\tr\tbytes\th1\n");
    let mut at = 0usize;
    for (place, _, len) in rows {
        let blk = &bytes[at..at + len as usize];
        at += len as usize;
        let mut c = [0u64; 256];
        for &b in blk {
            c[b as usize] += 1;
        }
        let t = blk.len() as f64;
        let h1: f64 = c
            .iter()
            .filter(|&&x| x > 0)
            .map(|&x| {
                let q = x as f64 / t;
                -q * q.log2()
            })
            .sum();
        let (ro, n, r) = route::unpack(routes.get(&place).copied().unwrap_or(0));
        let _ = writeln!(s, "{place}\t{ro}\t{n}\t{r}\t{len}\t{h1:.6}");
    }
    assert_eq!(at, bytes.len(), "the blocks' bytes are the output");
    std::fs::write(out, s).expect("OUT.tsv");
}

fn api(file: &str, threads: usize) {
    let data: &'static [u8] = std::fs::read(file).expect("read the input").leak();
    let mut info = String::new();
    let (p, r) = measured(
        || mibz2::BzReader::parallel(data, threads),
        |r| {
            if let Some(d) = r.parallel_decoder() {
                info = format!(
                    "\"threads\":{},\"window\":{},\"lazy\":{},\"service\":{}",
                    d.threads(),
                    d.window(),
                    d.is_lazy(),
                    d.is_service()
                );
            } else {
                info = "\"threads\":0,\"window\":0,\"lazy\":false,\"service\":false".into();
            }
        },
    );
    drop(r);
    println!(
        "{{\"kind\":\"api\",\"requested\":{threads},{info},{}}}",
        p.json()
    );
}

fn one(file: &str, k: usize, on: bool, blocks_out: &str, group: u8) {
    let data: &'static [u8] = std::fs::read(file).expect("read the input").leak();
    let slots = mibz2::parallel::SLOTS_PER_THREAD * k;
    assert!(!(on && group > 0), "the counters run with the ledgers off");
    let counters = (group > 0).then(|| pmu::Counters::open(group));
    ch::set_on(on);
    let mut window = 0u64;
    let (p, r) = measured(
        || {
            if let Some(c) = &counters {
                c.enable();
            }
            mibz2::BzReader::parallel_workers(data, k, slots, mibz2::Kernels::detect(), true)
        },
        |r| {
            window = r.parallel_decoder().map_or(0, |d| d.window());
        },
    );
    let end_ns = ch::now_ns();
    // (the decoder's threads joined: their counts folded into the inherited counters)
    drop(r);
    let mut line = format!(
        "{{\"kind\":\"decode\",\"k\":{k},\"on\":{},\"window\":{window},{}",
        u8::from(on),
        p.json()
    );
    if let Some(c) = &counters {
        c.disable();
        let _ = write!(
            line,
            ",\"pmu_group\":{group},\"pmu_scope\":\"{}\",\"pmu\":{{",
            c.scope
        );
        for (i, (name, v)) in c.read().into_iter().enumerate() {
            let _ = write!(
                line,
                "{}\"{name}\":{}",
                if i > 0 { "," } else { "" },
                v.map_or("null".into(), |v| format!("[{},{},{}]", v[0], v[1], v[2]))
            );
        }
        line.push('}');
    }
    if !on {
        println!("{line}}}");
        return;
    }
    // every thread's ledger: the workers', the scanner's and the feeder's as they end
    let deadline = Instant::now() + Duration::from_secs(10);
    while ch::finished() < k + 2 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    let occ = ch::take_occ(end_ns);
    ch::set_on(false);
    let mut ledgers = ch::take();
    ledgers.sort_by_key(|l| (role(&l.name) as u8, l.name.clone()));
    // the blocks: as many as were issued
    let n_blocks = ledgers
        .iter()
        .flat_map(|l| l.events.iter())
        .filter(|e| e.edge == 0 && e.tag == ch::Tag::Issued as u8)
        .count();
    let mut blocks = vec![
        Block {
            worker: -1,
            t: [UNREAD; 5],
            ..Block::default()
        };
        n_blocks
    ];
    // the phases' bounds: the first block taken, the last issued
    let pts = |tag: ch::Tag| {
        ledgers
            .iter()
            .flat_map(|l| l.events.iter())
            .filter(move |e| e.edge == 0 && e.tag == tag as u8)
            .map(|e| e.ns)
    };
    let t1 = pts(ch::Tag::Taken).min().unwrap_or(0);
    let t2 = pts(ch::Tag::Issued).max().unwrap_or(0);
    let mut pending_wait = 0;
    let mut threads = Vec::new();
    let mut wi = 0i64;
    for l in &ledgers {
        let idx = if role(&l.name) == Role::Worker {
            wi += 1;
            wi - 1
        } else {
            -1
        };
        threads.push(walk(l, idx, &mut blocks, t1, t2, &mut pending_wait));
    }
    // ---- the identities (the operational laws) ----
    let mut checks: Vec<(String, bool, String)> = Vec::new();
    let count = |r: Role| threads.iter().filter(|t| t.role == r).count();
    checks.push((
        "threads".into(),
        count(Role::Worker) == k
            && count(Role::Scanner) == 1
            && count(Role::Feeder) == 1
            && count(Role::Reader) == 1,
        format!(
            "workers {} scanner {} feeder {} reader {}",
            count(Role::Worker),
            count(Role::Scanner),
            count(Role::Feeder),
            count(Role::Reader)
        ),
    ));
    let bad_tl: Vec<&str> = threads
        .iter()
        .filter(|t| !t.balanced || !t.monotone)
        .map(|t| t.name.as_str())
        .collect();
    checks.push((
        "timelines partitioned".into(),
        bad_tl.is_empty()
            && threads
                .iter()
                .all(|t| t.states.iter().flatten().sum::<u64>() == t.life),
        format!("unbalanced or out of order: {bad_tl:?}"),
    ));
    let flow_bad = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.seen != [1; 6])
        .count();
    checks.push((
        "flow: every block issued, enqueued, taken by a worker, routed, ready and read once".into(),
        flow_bad == 0 && n_blocks > 0,
        format!("{flow_bad} of {n_blocks} blocks not seen once at each point"),
    ));
    // forced flow: the visits each route makes
    let mut routes = [0u64; 6];
    for b in &blocks {
        routes[usize::from(b.route).min(5)] += 1;
    }
    let (open, dense, refused, skipped, column) =
        (routes[1], routes[2], routes[3], routes[4], routes[5]);
    let visits = |t: ch::Tag| -> u64 { threads.iter().map(|x| x.visits[t as usize]).sum() };
    let nb = n_blocks as u64;
    let ff = [
        ("A5", visits(ch::Tag::A5), nb),
        ("pool", visits(ch::Tag::Pool), nb),
        ("A6a", visits(ch::Tag::A6a), open + dense + refused),
        ("A6b", visits(ch::Tag::A6b), open + dense),
        ("A6c", visits(ch::Tag::A6c), open + dense),
        ("A7", visits(ch::Tag::A7), open + dense),
        ("A8", visits(ch::Tag::A8), refused + skipped + column),
        ("A9", visits(ch::Tag::A9), nb),
        ("publish", visits(ch::Tag::Publish), nb),
        ("issue", visits(ch::Tag::Issue), nb),
    ];
    let ff_bad: Vec<String> = ff
        .iter()
        .filter(|(_, v, w)| v != w)
        .map(|(n, v, w)| format!("{n} {v} visits, {w} expected"))
        .collect();
    checks.push((
        "forced flow: each activity's visits are its routes' blocks".into(),
        ff_bad.is_empty() && routes[0] == 0,
        format!("{ff_bad:?}; unrouted {}", routes[0]),
    ));
    // conservation: every buffer empty at the end, its arrivals the blocks
    let cons_bad: Vec<String> = (0..OCCS)
        .filter(|&i| occ[i].n != 0 || (i < 4 && occ[i].arrivals != nb))
        .map(|i| {
            format!(
                "{} ends {} after {} arrivals",
                BUFFERS[i], occ[i].n, occ[i].arrivals
            )
        })
        .collect();
    checks.push((
        "conservation: enqueues − dequeues = occupancy, each buffer empty at the end".into(),
        cons_bad.is_empty(),
        format!("{cons_bad:?}"),
    ));
    // issued − taken: within the window at every instant
    let mut ev: Vec<(u64, i64)> = blocks
        .iter()
        .flat_map(|b| [(b.t[0], 1i64), (b.t[4], -1i64)])
        .collect();
    ev.sort();
    let (mut out, mut hi, mut lo) = (0i64, 0i64, 0i64);
    for (_, d) in ev {
        out += d;
        hi = hi.max(out);
        lo = lo.min(out);
    }
    checks.push((
        "admission: 0 ≤ issued − taken ≤ the window".into(),
        lo >= 0 && hi as u64 <= window && occ[0].hwm as u64 <= window,
        format!(
            "ledger high-water {hi}, live {}, window {window}",
            occ[0].hwm
        ),
    ));
    // Little's law: each buffer's live occupancy integral against its blocks' residence
    let res = |i: usize, j: usize, w: bool| -> u128 {
        blocks
            .iter()
            .map(|b| {
                let d = u128::from(b.t[j].saturating_sub(b.t[i]));
                if w {
                    d * u128::from(b.out_bytes)
                } else {
                    d
                }
            })
            .sum()
    };
    let little = [
        res(0, 4, false),
        res(1, 2, false),
        res(2, 3, false),
        res(3, 4, false),
        res(3, 4, true),
    ];
    let mut little_bad = Vec::new();
    let mut little_txt = String::new();
    for i in 0..OCCS {
        // (each change and its block's point share one clock read, under the buffers' lock:
        // the integral and the residence are the same sum of integers, exactly, unless an
        // event is missing or doubled)
        let (live, led) = (occ[i].integral, little[i]);
        let _ = write!(little_txt, "{}: live {live} ledger {led}; ", BUFFERS[i]);
        if live != led {
            little_bad.push(BUFFERS[i]);
        }
    }
    checks.push((
        "Little's law: each buffer's occupancy integral = its blocks' residence".into(),
        little_bad.is_empty(),
        little_txt,
    ));
    // utilisation: every block's work charged (a worker's activities fall within its blocks)
    let wk_un: u64 = threads
        .iter()
        .filter(|t| t.role == Role::Worker)
        .map(|t| {
            (0..NA)
                .filter(|&i| !ch::is_wait(i as u8))
                .map(|i| t.unattributed[i])
                .sum::<u64>()
        })
        .sum();
    checks.push((
        "utilisation: every worker activity charged to a block".into(),
        wk_un == 0,
        format!("{wk_un} ns of worker activity outside any block"),
    ));
    let all_ok = checks.iter().all(|c| c.1);
    // ---- the line ----
    let _ = write!(
        line,
        ",\"blocks\":{n_blocks},\"routes\":{{\"open\":{open},\"open_dense\":{dense},\"refused\":{refused},\"skipped\":{skipped},\"column\":{column}}},\"t1_ns\":{t1},\"t2_ns\":{t2},\"end_ns\":{end_ns},\"phases\":[{}],\"identities_ok\":{all_ok},\"identities\":[",
        PHASES.map(js).join(",")
    );
    for (i, (name, ok, why)) in checks.iter().enumerate() {
        let _ = write!(
            line,
            "{}{{\"name\":{},\"ok\":{ok},\"detail\":{}}}",
            if i > 0 { "," } else { "" },
            js(name),
            js(why)
        );
    }
    line += "],\"occupancy\":{";
    for i in 0..OCCS {
        let _ = write!(
            line,
            "{}\"{}\":{{\"integral\":{},\"hwm\":{},\"arrivals\":{},\"ledger\":{}}}",
            if i > 0 { "," } else { "" },
            BUFFERS[i],
            occ[i].integral,
            occ[i].hwm,
            occ[i].arrivals,
            little[i]
        );
    }
    line += "},\"threads\":[";
    for (i, t) in threads.iter().enumerate() {
        let _ = write!(
            line,
            "{}{{\"name\":{},\"role\":\"{:?}\",\"life_ns\":{},\"cpu_ns\":{},\"wait_wall_ns\":{},\"wait_cpu_ns\":{},\"events\":{},\"sched\":[{}],\"migrations\":{},\"states\":[{}],\"visits\":[{}],\"unattributed\":[{}],\"slot_bound\":[{},{}]}}",
            if i > 0 { "," } else { "" },
            js(&t.name),
            t.role,
            t.life,
            t.cpu,
            t.wait_wall,
            t.wait_cpu,
            t.events,
            t.sched.map(|x| x.to_string()).join(","),
            if t.migrations == UNREAD { "null".to_string() } else { t.migrations.to_string() },
            t.states
                .iter()
                .map(|p| format!("[{}]", p.map(|x| x.to_string()).join(",")))
                .collect::<Vec<_>>()
                .join(","),
            t.visits.map(|x| x.to_string()).join(","),
            t.unattributed.map(|x| x.to_string()).join(","),
            t.slot_bound[0],
            t.slot_bound[1]
        );
    }
    line += "]}";
    println!("{line}");
    if blocks_out != "-" {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(blocks_out)
            .expect("BLOCKS.tsv");
        let mut s = String::new();
        for (p, b) in blocks.iter().enumerate() {
            let _ = write!(
                s,
                "{p}\t{}\t{}\t{}\t{}\t{}\t{}",
                b.route, b.n, b.r, b.syms_bytes, b.out_bytes, b.worker
            );
            for t in b.t {
                let _ = write!(s, "\t{}", if t == UNREAD { -1 } else { t as i64 });
            }
            for i in SCAN_COLS {
                let _ = write!(s, "\t{}", b.scan[i]);
            }
            for i in WORK_COLS {
                let _ = write!(s, "\t{}", b.work[i]);
            }
            let _ = writeln!(s, "\t{}", b.read_wait);
        }
        f.write_all(s.as_bytes()).expect("write BLOCKS.tsv");
    }
}

/// The scanner's columns of a block's row: in_wait, in_maint, A2, A3, slot_wait, issue,
/// residual
const SCAN_COLS: [usize; 7] = [2, 3, 4, 5, 6, 7, NA];
/// A worker's: A5, pool, A6a, A6b, A6c, A7, A8, A9, publish, residual
const WORK_COLS: [usize; 10] = [9, 10, 11, 12, 13, 14, 15, 16, 17, NA];

/// The block rows' header (`BLOCKS.tsv`, before the harness's own leading columns)
#[allow(dead_code)]
const BLOCK_COLS: &str = "place\troute\tn\tr\tsyms_bytes\tout_bytes\tworker\tt_issued\tt_enqueued\tt_got\tt_ready\tt_taken\ts_in_wait\ts_in_maint\ts_A2\ts_A3\ts_slot_wait\ts_issue\ts_resid\tw_A5\tw_pool\tw_A6a\tw_A6b\tw_A6c\tw_A7\tw_A8\tw_A9\tw_publish\tw_resid\tr_wait";

fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o += "\\\"",
            '\\' => o += "\\\\",
            '\n' => o += "\\n",
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}
