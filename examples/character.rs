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
use std::sync::atomic::Ordering;
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
            // (the achieved frequency, every decode: two fixed counters, no general one taken)
            4 => vec![("cycles", HW, 0), ("ref-cycles", HW, 9)],
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
        pub fn reset(&self) {
            self.each(0x2403);
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
    /// each logical CPU's mean achieved frequency over the decode (kHz; NaN where unread)
    freq_khz: Vec<f64>,
    /// the package's temperature, mean and maximum (milli-°C)
    temp_mc: Option<(f64, u64)>,
    /// the thermal throttle events over the decode: the cores' summed, the package's
    throttle: (Option<u64>, Option<u64>),
    /// the process's minor faults and system time at the start, at each sample (ns from the
    /// ledgers' epoch) and at the end, for the phases' split
    series: Vec<(u64, f64, f64)>,
    start_use: [f64; 6],
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
        ) + &format!(
            ",\"freq_khz\":[{}],\"temp_mc\":{},\"throttle_core\":{},\"throttle_pkg\":{}",
            self.freq_khz
                .iter()
                .map(|f| if f.is_nan() { "null".into() } else { format!("{f:.0}") })
                .collect::<Vec<_>>()
                .join(","),
            self.temp_mc.map_or("null".into(), |(m, x)| format!("[{m:.0},{x}]")),
            self.throttle.0.map_or("null".into(), |x| x.to_string()),
            self.throttle.1.map_or("null".into(), |x| x.to_string())
        )
    }
}

/// The machine's frequency and heat, read around and during a decode: each logical CPU's
/// `scaling_cur_freq` (the kernel's estimate of its achieved frequency), the package's
/// temperature (the `x86_pkg_temp` thermal zone), and the thermal throttle counts
struct Thermal {
    freq: Vec<String>,
    temp: Option<String>,
    core_throttle: Vec<String>,
    pkg_throttle: Option<String>,
}

fn read_u64(p: &str) -> Option<u64> {
    std::fs::read_to_string(p).ok()?.trim().parse().ok()
}

impl Thermal {
    fn new() -> Thermal {
        let n = std::thread::available_parallelism().map_or(1, |n| n.get());
        let cpus: Vec<usize> = (0..n.max(64))
            .filter(|c| std::path::Path::new(&format!("/sys/devices/system/cpu/cpu{c}")).exists())
            .collect();
        let temp = (0..64)
            .map(|z| format!("/sys/class/thermal/thermal_zone{z}"))
            .find(|z| {
                std::fs::read_to_string(format!("{z}/type"))
                    .is_ok_and(|t| t.trim() == "x86_pkg_temp")
            })
            .map(|z| format!("{z}/temp"));
        Thermal {
            freq: cpus
                .iter()
                .map(|c| format!("/sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq"))
                .collect(),
            temp,
            core_throttle: cpus
                .iter()
                .map(|c| {
                    format!("/sys/devices/system/cpu/cpu{c}/thermal_throttle/core_throttle_count")
                })
                .collect(),
            pkg_throttle: Some(
                "/sys/devices/system/cpu/cpu0/thermal_throttle/package_throttle_count".into(),
            )
            .filter(|p| std::path::Path::new(p).exists()),
        }
    }
    /// (the cores' throttle count summed, the package's)
    fn throttles(&self) -> (Option<u64>, Option<u64>) {
        let core: Option<u64> = self.core_throttle.iter().map(|p| read_u64(p)).sum();
        (core, self.pkg_throttle.as_deref().and_then(read_u64))
    }
}

/// The sampler's interval for the frequencies and the temperature (a multiple of `SAMPLE`)
const THERMAL_EVERY: u64 = 10;

/// A decode with the process's figures over it: `make` builds the reader (the clock running:
/// counters are enabled there), and `stop` runs the moment the clock stops (counters disabled
/// there, so they count the stopwatch's region alone)
fn measured<R: Read>(
    make: impl FnOnce() -> R,
    after: impl FnOnce(&R),
    stop_hook: impl FnOnce(),
) -> (Proc, R) {
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let st = stop.clone();
    let th = std::sync::Arc::new(Thermal::new());
    let th2 = th.clone();
    let base_kb = rss_kb().unwrap_or(0);
    let throttle0 = th.throttles();
    let reset = reset_peak();
    let u0 = usage();
    let sampler = std::thread::Builder::new()
        .name("sampler".into())
        .spawn(move || {
            let (mut sum, mut n) = (0u64, 0u64);
            let mut freq = vec![(0u64, 0u64); th2.freq.len()];
            let (mut tsum, mut tmax, mut tn) = (0u64, 0u64, 0u64);
            let mut series = Vec::new();
            let mut i = 0u64;
            while !st.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(kb) = rss_kb() {
                    sum += kb;
                    n += 1;
                }
                let u = usage();
                series.push((ch::now_ns(), u[2], u[1]));
                if i % THERMAL_EVERY == 0 {
                    for (f, p) in freq.iter_mut().zip(&th2.freq) {
                        if let Some(v) = read_u64(p) {
                            f.0 += v;
                            f.1 += 1;
                        }
                    }
                    if let Some(t) = th2.temp.as_deref().and_then(read_u64) {
                        tsum += t;
                        tmax = tmax.max(t);
                        tn += 1;
                    }
                }
                i += 1;
                std::thread::sleep(SAMPLE);
            }
            (sum, n, freq, tsum, tmax, tn, series)
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
    stop_hook();
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (sum, n, freq, tsum, tmax, tn, series) = sampler.join().expect("the sampler");
    let u1 = usage();
    let throttle1 = th.throttles();
    let mut use_ = [0.0; 6];
    for i in 0..6 {
        use_[i] = u1[i] - u0[i];
    }
    let delta = |a: Option<u64>, b: Option<u64>| a.zip(b).map(|(a, b)| b.saturating_sub(a));
    (
        Proc {
            wall,
            bytes,
            base_kb,
            peak_kb: if reset { peak_kb() } else { 0 },
            mean_kb: if n > 0 { sum as f64 / n as f64 } else { 0.0 },
            samples: n,
            use_,
            freq_khz: freq
                .iter()
                .map(|&(s, c)| if c > 0 { s as f64 / c as f64 } else { f64::NAN })
                .collect(),
            temp_mc: (tn > 0).then(|| (tsum as f64 / tn as f64, tmax)),
            throttle: (
                delta(throttle0.0, throttle1.0),
                delta(throttle0.1, throttle1.1),
            ),
            series,
            start_use: u0,
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
    /// Issued, Enqueued, Got, Ready, Taken, ScanStart, Consumed (ns), `UNREAD` where missing
    t: [u64; 7],
    /// the same points' places in the buffers' linear order (`seq`)
    seq: [u64; 7],
    /// counts of each point: Issued, Enqueued, Got, Ready, Taken, Route, ScanStart, Consumed
    /// (each must be 1)
    seen: [u32; 8],
    /// the logical CPU the scanner issued it on, and the one its worker took it on
    cpu_scan: u16,
    cpu_work: u16,
    scan: [u64; NA + 1],
    work: [u64; NA + 1],
    read_wait: u64,
    /// visits: the scanner's to each activity for this block, and the reader's waits for it
    scan_v: [u32; NA + 1],
    read_wait_v: u32,
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
    /// the scanner's waits for admission by what held it: the window alone, the ready bytes'
    /// cap alone, both
    slot_bound: [u64; 3],
    /// the scanner's last read, which gave no block: from its admission to its end
    abandon: (Option<u64>, Option<u64>),
    /// its time on each logical CPU (ns), each stretch charged to the CPU it began on
    cpus: std::collections::BTreeMap<u16, u64>,
    /// its ledger's events past their preallocation
    overflows: u64,
    /// the scanner's waits for admission: when, the head-of-line block (`taken`), the
    /// predicates' packing
    bounds: Vec<(u64, u32, u64)>,
    /// its waits' intervals: the scanner's for admission, a worker's for a job
    gaps: Vec<(u64, u64)>,
    /// the scanner's waits for input
    in_gaps: Vec<(u64, u64)>,
    /// each activity's visits' service times (wall, begin to end, nested activities within):
    /// their count, sum (ns) and sum of squares (ns²), for the means and C²
    moments: [(u64, f64, f64); NA],
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
        slot_bound: [0; 3],
        abandon: (None, None),
        cpus: std::collections::BTreeMap::new(),
        overflows: l.overflows,
        bounds: Vec::new(),
        moments: [(0, 0.0, 0.0); NA],
        gaps: Vec::new(),
        in_gaps: Vec::new(),
    };
    let mut bound = 0usize;
    let mut last_cpu = ev.first().map_or(u16::MAX, |e| e.on_cpu);
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
    let mut opened: Vec<u64> = Vec::new();
    let mut pend_v = [0u32; NA + 1];
    let mut wait_v = 0u32;
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
            *o.cpus.entry(last_cpu).or_default() += now - last;
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
        last_cpu = e.on_cpu;
        match e.edge {
            1 => {
                stack.push(e.tag);
                opened.push(e.ns);
                if let Some(a) = act(e.tag) {
                    o.visits[a] += 1;
                    match ro {
                        Role::Scanner => pend_v[a] += 1,
                        Role::Reader if a == ch::Tag::ReadWait as usize => wait_v += 1,
                        _ => {}
                    }
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
                if let (Some(t0), Some(a)) = (opened.pop(), act(e.tag)) {
                    let d = e.ns.saturating_sub(t0) as f64;
                    let m = &mut o.moments[a];
                    m.0 += 1;
                    m.1 += d;
                    m.2 += d * d;
                }
                if ch::is_wait(e.tag) {
                    if let Some((w0, c0)) = wait_from.take() {
                        o.wait_wall += e.ns - w0;
                        if e.tag == ch::Tag::SlotWait as u8 {
                            o.slot_bound[bound] += e.ns - w0;
                            bound = 0;
                        }
                        if (ro == Role::Scanner && e.tag == ch::Tag::SlotWait as u8)
                            || (ro == Role::Worker && e.tag == ch::Tag::JobWait as u8)
                        {
                            o.gaps.push((w0, e.ns));
                        }
                        if ro == Role::Scanner && e.tag == ch::Tag::InWait as u8 {
                            o.in_gaps.push((w0, e.ns));
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
                        for (a, b) in blocks[p].scan_v.iter_mut().zip(pend_v.iter()) {
                            *a += b;
                        }
                        pend = [0; NA + 1];
                        pend_v = [0; NA + 1];
                    }
                }
            }
            _ => {
                let p = e.place as usize;
                let k = e.tag;
                // (a point's block, counted; and its time and linear order, where it has them)
                let b = |blocks: &mut [Block], i: usize, ti: Option<usize>| -> Option<usize> {
                    if p < blocks.len() {
                        blocks[p].seen[i] += 1;
                        if let Some(ti) = ti {
                            blocks[p].t[ti] = e.ns;
                            blocks[p].seq[ti] = e.seq;
                        }
                        Some(p)
                    } else {
                        None
                    }
                };
                if k == ch::Tag::Issued as u8 {
                    if let Some(p) = b(blocks, 0, Some(0)) {
                        blocks[p].syms_bytes = e.x;
                        blocks[p].cpu_scan = e.on_cpu;
                        last_issued = Some(p);
                    }
                } else if k == ch::Tag::Enqueued as u8 {
                    b(blocks, 1, Some(1));
                } else if k == ch::Tag::Got as u8 {
                    if let Some(p) = b(blocks, 2, Some(2)) {
                        blocks[p].worker = idx;
                        blocks[p].cpu_work = e.on_cpu;
                        cur = Some(p);
                    }
                } else if k == ch::Tag::Ready as u8 {
                    if let Some(p) = b(blocks, 3, Some(3)) {
                        blocks[p].out_bytes = e.x;
                    }
                } else if k == ch::Tag::ScanStart as u8 {
                    if b(blocks, 6, Some(5)).is_none() {
                        o.abandon.0 = Some(e.ns);
                    }
                } else if k == ch::Tag::ScanAbandon as u8 {
                    o.abandon.1 = Some(e.ns);
                } else if k == ch::Tag::Consumed as u8 {
                    b(blocks, 7, Some(6));
                } else if k == ch::Tag::Taken as u8 {
                    if let Some(p) = b(blocks, 4, Some(4)) {
                        blocks[p].read_wait = wait_acc;
                        blocks[p].read_wait_v = wait_v;
                        wait_acc = 0;
                        wait_v = 0;
                    }
                } else if k == ch::Tag::Bound as u8 {
                    o.bounds.push((e.ns, e.place, e.x));
                    bound = match ch::bound::unpack(e.x) {
                        (true, true, ..) => 2,
                        (false, true, ..) => 1,
                        _ => 0,
                    };
                } else if k == ch::Tag::Route as u8 {
                    if let Some(p) = b(blocks, 5, None) {
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
        Some("--one") if (6..=8).contains(&a.len()) => {
            treatments(a.get(7).map_or("none", String::as_str));
            one(
                &a[2],
                a[3].parse().expect("K"),
                a[4] == "1",
                &a[5],
                a.get(6).map_or(0, |g| g.parse().expect("PMU")),
            );
            // (the ideal-parts matrix: the oracle's check pass, outside every clock and after
            // the ledgers are taken, its figures added to the record)
            let line = LINE.lock().unwrap().take().expect("a record");
            println!("{}", oracle::finish(line));
        }
        Some("--api") if a.len() == 4 => api(&a[2], a[3].parse().expect("THREADS")),
        Some("--features") if a.len() == 4 => features(&a[2], &a[3]),
        Some("--calibrate") if a.len() == 4 => calibrate(
            a[2].parse().expect("ITERATIONS"),
            a[3].parse().expect("REPEATS"),
        ),
        _ => {
            eprintln!(
                "usage: character --one FILE.bz2 K ON BLOCKS.tsv|- [PMU [TREAT]]\n       character --api FILE.bz2 THREADS"
            );
            std::process::exit(2);
        }
    }
}

/// The treatments switched on (`src/character.rs`' `treat`), from TREAT: `none`, or a file of
/// lines `pace ACTIVITY ITERATIONS`, `dose ACTIVITY BYTES stream|chase`, `ready_cap BYTES`,
/// `scanner_cpu CPU` (stage 2's; stage 1 runs with none, the same binary)
/// The pacing loop's calibration (`--calibrate ITERATIONS REPEATS`): the same dependent
/// multiply-add chain as `character::treat`'s pacing (src/character.rs `spin`, its code
/// copied: the library stays stage 1's), run ITERATIONS times, REPEATS times, each counted with
/// cycles and reference cycles (where the PMU counts) and timed; one JSON line, each repeat's
/// cycles, reference cycles and ns
fn calibrate(n: u64, repeats: usize) {
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
    let c = pmu::Counters::open(4);
    let mut rows = Vec::new();
    for _ in 0..repeats {
        c.reset();
        c.enable();
        let t = Instant::now();
        spin(n);
        let ns = t.elapsed().as_nanos();
        c.disable();
        let v = c.read();
        let get = |i: usize| {
            v.get(i)
                .and_then(|x| x.1)
                .map_or("null".into(), |x| x[0].to_string())
        };
        rows.push(format!("[{},{},{ns}]", get(0), get(1)));
    }
    println!(
        "{{\"kind\":\"calibration\",\"iterations\":{n},\"scope\":\"{}\",\"repeats\":[{}]}}",
        c.scope,
        rows.join(",")
    );
}

/// `one()`'s record line, printed by main once the oracle's check pass has added to it
static LINE: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn emit(line: String) {
    *LINE.lock().unwrap() = Some(line);
}

/// The pre-spin's length (`prespin MS`; 0: none)
static PRESPIN_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The pre-spin (journal §81's standard warm-up; bench-clock-mechanism: no decode at the low
/// clock after it): one thread pinned to each online CPU, all spinning until a shared deadline
/// `ms` away, joined before it returns. (ms, CPUs spun, seconds), or None for none
fn prespin(ms: u64) -> Option<(u64, usize, f64)> {
    if ms == 0 {
        return None;
    }
    unsafe extern "C" {
        fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
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
        .filter(|&&c| c < 1024)
        .map(|&c| {
            std::thread::spawn(move || {
                let mut mask = [0u64; 16];
                mask[c / 64] |= 1 << (c % 64);
                // SAFETY: a cpu_set_t of 1024 bits, read by the kernel; pid 0 is this thread
                let _ =
                    unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) };
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

/// The ideal-parts matrix's oracles, docs/journal/2026-11-24-algorithm-level/ideal-parts/: S, the
/// scan replayed (`scan-oracle`); E, emission on the supplied codes (`supplied-codes`); C,
/// construction replayed (`ideal-construction`); M, MTF/RLE2 replayed (`ideal-mtf`); R, RLE1,
/// the CRC and the output replayed (`ideal-rle1`); D, delivery discarded (`ideal-delivery`).
/// Every arm runs the same passes, so the arms differ only in the modes of their timed decode:
/// - **the record pass:** every part as shipped, each oracle's store recorded (all six in every
///   arm, so every arm holds the same stores), the reference output's size and digest;
/// - **the timed decode,** the arm's parts ideal, the rest real;
/// - **the check pass:** the arm's modes again, but emission in its check mode (every supplied
///   block's bytes compared with the walk's) and delivery as shipped (D's discard has no output
///   to check; the timed decode counts its bytes); the output's size and digest against the
///   reference.
///
/// Both passes run at `oracle::PASS_WORKERS` workers, outside every clock, with the ledgers off.
/// Their digest is SHA-256 over the output's 4 MiB chunks' SHA-256s in order and its length, the
/// chunks hashed on threads of their own, off the reader (the serial hash of delivery's check
/// mode was most of a process's time in the first full run).
/// A timed decode whose ideal parts were not wholly replayed fails (its arm was not measured).
mod oracle {
    use std::sync::atomic::{AtomicU8, Ordering};

    /// The matrix's factors in parts.md's order, each an arm's letter and a bit: S, the
    /// Huffman scan; E, emission; C, construction; M, MTF/RLE2; R, RLE1, CRC and output; D,
    /// delivery. An arm is `none` or its ideal parts' letters (`SE`, `SECMRD`, ...)
    const FACTORS: [char; 6] = ['S', 'E', 'C', 'M', 'R', 'D'];

    /// 0x80: in play (an `oracle` line); bit i: factor FACTORS[i] ideal
    static ARM: AtomicU8 = AtomicU8::new(0);
    #[cfg_attr(not(feature = "ideal-delivery"), allow(dead_code))]
    pub const PASS_WORKERS: usize = 10;

    pub fn set(x: &str) {
        let mut bits = 0u8;
        if x != "none" {
            for c in x.chars() {
                let i = FACTORS
                    .iter()
                    .position(|&f| f == c)
                    .unwrap_or_else(|| panic!("TREAT: oracle {x}: none, or letters of SECMRD"));
                assert!(bits & (1 << i) == 0, "TREAT: oracle {x}: {c} twice");
                bits |= 1 << i;
            }
        }
        ARM.store(0x80 | bits, Ordering::SeqCst);
    }

    /// Whether factor `c` is ideal in this decode's arm
    #[cfg_attr(not(feature = "ideal-delivery"), allow(dead_code))]
    fn ideal(c: char) -> bool {
        let i = FACTORS.iter().position(|&f| f == c).expect("a factor");
        ARM.load(Ordering::SeqCst) & (1 << i) != 0
    }

    /// The arm's name: `none`, or its ideal factors' letters in FACTORS' order
    #[cfg_attr(not(feature = "ideal-delivery"), allow(dead_code))]
    pub fn name() -> String {
        let n: String = FACTORS.iter().filter(|&&c| ideal(c)).collect();
        if n.is_empty() {
            "none".into()
        } else {
            n
        }
    }

    fn in_play() -> bool {
        ARM.load(Ordering::SeqCst) & 0x80 != 0
    }

    /// The timed decode's replays, for the trace's forced flow: the open blocks emitted from
    /// supplied codes (their walk removed), the open blocks whose construction was replayed
    /// built and refused (A6a to A6c removed)
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Replays {
        pub supplied: u64,
        pub c_built: u64,
        pub c_refused: u64,
        pub m: u64,
        pub r: u64,
    }

    // the matrix's binary: every oracle's feature (the bench builds them all)
    #[cfg(not(all(
        feature = "scan-oracle",
        feature = "ideal-construction",
        feature = "ideal-mtf",
        feature = "ideal-rle1",
        feature = "ideal-delivery"
    )))]
    mod live {
        pub fn record(_: &'static [u8]) {
            assert!(
                !super::in_play(),
                "TREAT: oracle needs the features scan-oracle, ideal-construction, ideal-mtf, ideal-rle1 and ideal-delivery"
            );
        }
        pub fn finish(line: String) -> String {
            line
        }
        pub fn take_handover() -> super::Replays {
            super::Replays::default()
        }
    }

    #[cfg(all(
        feature = "scan-oracle",
        feature = "ideal-construction",
        feature = "ideal-mtf",
        feature = "ideal-rle1",
        feature = "ideal-delivery"
    ))]
    mod live {
        use super::{ideal, in_play, name, Replays};
        use mibz2::ideal::{construction as ic, delivery as id, mtf as im, rle1 as ir, Mode};
        use mibz2::scan_oracle as scan;
        use mibz2::supplied_codes as sup;
        use std::io::Read;
        use std::sync::Mutex;

        /// The record pass: the reference (bytes, SHA-256, wall), the stores' JSON, the blocks
        /// (M's store, every block indexed) and C's open blocks
        struct Ref {
            bytes: u64,
            sha: String,
            wall: f64,
            store: String,
            blocks: u64,
            c_open: u64,
            data: &'static [u8],
        }
        static REF: Mutex<Option<Ref>> = Mutex::new(None);
        /// the timed decode's counts, as JSON
        static TIMED: Mutex<Option<String>> = Mutex::new(None);

        fn hex(h: [u8; 32]) -> String {
            h.iter().map(|b| format!("{b:02x}")).collect()
        }

        /// The modes of every oracle: (S, E, C, M, R, D)
        type Modes = (scan::Mode, sup::Mode, Mode, Mode, Mode, id::Mode);

        fn set_modes(m: Modes) {
            scan::set_mode(m.0);
            sup::set_mode(m.1);
            sup::set_condition(false);
            ic::set_mode(m.2);
            im::set_mode(m.3);
            ir::set_mode(m.4);
            id::set_mode(m.5);
        }

        /// The arm's modes: its ideal parts replayed (D discarded), the rest real; `check`, E
        /// in its check mode where ideal and delivery as shipped
        fn arm_modes(check: bool) -> Modes {
            let rep = |c: char| if ideal(c) { Mode::Replay } else { Mode::Real };
            (
                if ideal('S') {
                    scan::Mode::Replay
                } else {
                    scan::Mode::Scan
                },
                match (ideal('E'), check) {
                    (true, true) => sup::Mode::Check,
                    (true, false) => sup::Mode::Supplied,
                    (false, _) => sup::Mode::Open,
                },
                rep('C'),
                rep('M'),
                rep('R'),
                if ideal('D') && !check {
                    id::Mode::Discard
                } else {
                    id::Mode::Real
                },
            )
        }

        /// Every counter zeroed
        fn reset_counts() {
            let _ = scan::take_replayed();
            let _ = sup::take();
            let _ = sup::mismatches();
            let _ = ic::take_constructed();
            let _ = ic::take_replayed();
            let _ = ic::take_refused_replayed();
            let _ = im::take_executed();
            let _ = im::take_replayed();
            let _ = ir::take_executed();
            let _ = ir::take_replayed();
            let _ = id::take_stats();
        }

        /// SHA-256 (FIPS 180-4), std only: the passes' hash of the output's chunks
        fn sha256(data: &[u8]) -> [u8; 32] {
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
            let mut h: [u32; 8] = [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ];
            let mut block = |c: &[u8]| {
                let mut w = [0u32; 64];
                for (i, x) in c.chunks_exact(4).enumerate() {
                    w[i] = u32::from_be_bytes([x[0], x[1], x[2], x[3]]);
                }
                for i in 16..64 {
                    let s0 =
                        w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                    let s1 =
                        w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                    w[i] = w[i - 16]
                        .wrapping_add(s0)
                        .wrapping_add(w[i - 7])
                        .wrapping_add(s1);
                }
                let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
                for i in 0..64 {
                    let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                    let ch = (e & f) ^ (!e & g);
                    let t1 = hh
                        .wrapping_add(s1)
                        .wrapping_add(ch)
                        .wrapping_add(K[i])
                        .wrapping_add(w[i]);
                    let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                    let maj = (a & b) ^ (a & c) ^ (b & c);
                    let t2 = s0.wrapping_add(maj);
                    hh = g;
                    g = f;
                    f = e;
                    e = d.wrapping_add(t1);
                    d = c;
                    c = b;
                    b = a;
                    a = t1.wrapping_add(t2);
                }
                for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
                    *x = x.wrapping_add(y);
                }
            };
            let full = data.len() / 64 * 64;
            for c in data[..full].chunks_exact(64) {
                block(c);
            }
            let mut tail = data[full..].to_vec();
            tail.push(0x80);
            while tail.len() % 64 != 56 {
                tail.push(0);
            }
            tail.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
            for c in tail.chunks_exact(64) {
                block(c);
            }
            let mut out = [0u8; 32];
            for (o, x) in out.chunks_exact_mut(4).zip(h) {
                o.copy_from_slice(&x.to_be_bytes());
            }
            out
        }

        /// The passes' chunk: the output hashed in pieces of this many bytes
        const CHUNK: usize = 4 << 20;

        /// A pass outside the clock, the output read by the caller as shipped (delivery real)
        /// and hashed off the reader: each CHUNK bytes' SHA-256 on `PASS_WORKERS` threads, the
        /// digest the SHA-256 of their digests in order and the length. (bytes, digest, wall)
        fn pass(data: &'static [u8], m: Modes) -> (u64, String, f64) {
            use std::sync::mpsc;
            assert_eq!(m.5, id::Mode::Real, "a pass reads its output as shipped");
            reset_counts();
            set_modes(m);
            let k = mibz2::Kernels::detect();
            let w = super::PASS_WORKERS;
            let t = std::time::Instant::now();
            let (tx, rx) = mpsc::sync_channel::<(usize, Vec<u8>)>(2 * w);
            let rx = Mutex::new(rx);
            let digests: Mutex<Vec<(usize, [u8; 32])>> = Mutex::new(Vec::new());
            let bytes = std::thread::scope(|sc| {
                for _ in 0..w {
                    sc.spawn(|| loop {
                        let got = rx.lock().unwrap().recv();
                        let Ok((i, chunk)) = got else { break };
                        let d = sha256(&chunk);
                        digests.lock().unwrap().push((i, d));
                    });
                }
                let mut r = mibz2::BzReader::parallel_workers(
                    data,
                    w,
                    mibz2::parallel::SLOTS_PER_THREAD * w,
                    k,
                    true,
                );
                let (mut bytes, mut i) = (0u64, 0usize);
                let mut chunk = vec![0u8; CHUNK];
                let mut at = 0;
                loop {
                    match r.read(&mut chunk[at..]) {
                        Ok(0) => break,
                        Ok(x) => {
                            at += x;
                            bytes += x as u64;
                            if at == CHUNK {
                                tx.send((i, std::mem::replace(&mut chunk, vec![0u8; CHUNK])))
                                    .expect("the hashers");
                                i += 1;
                                at = 0;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(e) => panic!("an oracle pass failed: {e}"),
                    }
                }
                if at > 0 {
                    chunk.truncate(at);
                    tx.send((i, chunk)).expect("the hashers");
                }
                drop(r);
                drop(tx);
                bytes
            });
            let wall = t.elapsed().as_secs_f64();
            let st = id::take_stats();
            assert_eq!(
                (st.copied_bytes, st.discarded_bytes),
                (bytes, 0),
                "a pass's delivery"
            );
            let mut ds = digests.into_inner().unwrap();
            ds.sort_by_key(|x| x.0);
            assert!(ds.iter().enumerate().all(|(j, x)| x.0 == j), "a chunk lost");
            let mut cat: Vec<u8> = ds.iter().flat_map(|x| x.1).collect();
            cat.extend_from_slice(&bytes.to_le_bytes());
            (bytes, hex(sha256(&cat)), wall)
        }

        pub fn record(data: &'static [u8]) {
            if !in_play() {
                return;
            }
            // (the hash itself, against FIPS 180-4's vectors: one block, and two)
            assert_eq!(
                hex(sha256(b"abc")),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
            assert_eq!(
                hex(sha256(
                    b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
                )),
                "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
            );
            sup::clear_store();
            scan::clear_store();
            ic::clear_store();
            im::clear_store();
            ir::clear_store();
            let (bytes, sha, wall) = pass(
                data,
                (
                    scan::Mode::Record,
                    sup::Mode::Record,
                    Mode::Record,
                    Mode::Record,
                    Mode::Record,
                    id::Mode::Real,
                ),
            );
            let c_open = ic::take_constructed();
            let _ = sup::take();
            let (keys, codes, code_bytes) = sup::store_size();
            let collisions = sup::collisions();
            let (s_blocks, s_distinct, s_bytes, s_conflicts, s_missing) = scan::store_size();
            let c = ic::store_size();
            let m = im::store_size();
            let r = ir::store_size();
            assert!(
                collisions + s_conflicts + s_missing == 0 && keys > 0 && s_blocks > 0,
                "S's or E's store is not usable: {collisions} collisions, {s_conflicts} conflicts, {s_missing} missing"
            );
            assert!(
                c.3 == 0 && c.0 - c.4 == c_open && c_open > 0,
                "C's store is not usable: {c:?}, {c_open} open blocks constructed"
            );
            assert!(
                m.3 == 0 && m.4 == 0 && m.0 == s_blocks,
                "M's store is not usable: {m:?}, {s_blocks} blocks"
            );
            assert!(
                r.3 == 0 && r.4 == 0 && r.0 == s_blocks,
                "R's store is not usable: {r:?}, {s_blocks} blocks"
            );
            let total = s_bytes + code_bytes as u64 + c.2 + m.2 + r.2;
            let store = format!(
                "{{\"blocks\":{s_blocks},\"distinct\":{s_distinct},\"symbol_bytes\":{s_bytes},\"code_keys\":{keys},\"codes\":{codes},\"code_bytes\":{code_bytes},\
                 \"C\":{{\"blocks\":{},\"distinct\":{},\"bytes\":{},\"open\":{c_open}}},\
                 \"M\":{{\"blocks\":{},\"distinct\":{},\"bytes\":{}}},\
                 \"R\":{{\"blocks\":{},\"distinct\":{},\"bytes\":{}}},\"total_bytes\":{total}}}",
                c.0, c.1, c.2, m.0, m.1, m.2, r.0, r.1, r.2
            );
            *REF.lock().unwrap() = Some(Ref {
                bytes,
                sha,
                wall,
                store,
                blocks: s_blocks,
                c_open,
                data,
            });
            // the timed decode's modes, its counters from zero
            reset_counts();
            set_modes(arm_modes(false));
        }

        /// The timed decode's counts, taken as soon as its decoder is joined; it fails where an
        /// ideal part was not wholly replayed, or a real one was
        pub fn take_handover() -> Replays {
            if !in_play() {
                return Replays::default();
            }
            let g = REF.lock().unwrap();
            let rf = g.as_ref().expect("the record pass");
            let s_rep = scan::take_replayed();
            let recs = sup::take();
            let open = recs.iter().filter(|x| !x.refused);
            // (the delivery and consumption of the supplied blocks alone: an open block emitted
            // by its walk times the walk there, which is emission's real work, never a handover)
            let (n_open, n_sup, deliver, consume) =
                open.fold((0u64, 0u64, 0u64, 0u64), |(c, s, d, e), x| {
                    let sup = u64::from(x.supplied);
                    (
                        c + 1,
                        s + sup,
                        d + sup * x.deliver_ns,
                        e + sup * x.consume_ns,
                    )
                });
            let (c_con, c_built, c_ref) = (
                ic::take_constructed(),
                ic::take_replayed(),
                ic::take_refused_replayed(),
            );
            let (m_exec, m_rep) = (im::take_executed(), im::take_replayed());
            let (r_exec, r_rep) = (ir::take_executed(), ir::take_replayed());
            let d = id::take_stats();
            let nb = rf.blocks;
            let arm = name();
            let whole = |part: char, replayed: bool, real: bool| {
                assert!(
                    if ideal(part) { replayed } else { real },
                    "arm {arm}: {part} {} in the timed decode (S {s_rep}; C {c_con} constructed, {c_built} + {c_ref} replayed of {}; M {m_exec} + {m_rep}; R {r_exec} + {r_rep}; D {d:?}; {nb} blocks)",
                    if ideal(part) { "not wholly replayed" } else { "replayed" },
                    rf.c_open
                )
            };
            whole('S', s_rep == nb, s_rep == 0);
            whole(
                'C',
                c_con == 0 && c_built + c_ref == rf.c_open,
                c_built + c_ref == 0,
            );
            whole('M', m_exec == 0 && m_rep == nb, m_rep == 0);
            whole('R', r_exec == 0 && r_rep == nb, r_rep == 0);
            whole(
                'D',
                d.copied_bytes == 0 && d.discarded_bytes == rf.bytes,
                d.discarded_bytes == 0 && d.copied_bytes == rf.bytes,
            );
            *TIMED.lock().unwrap() = Some(format!(
                "\"replayed\":{s_rep},\"open_blocks\":{n_open},\"supplied_blocks\":{n_sup},\"deliver_ns\":{deliver},\"consume_ns\":{consume},\
                 \"replays\":{{\"S\":{s_rep},\"C_constructed\":{c_con},\"C_built\":{c_built},\"C_refused\":{c_ref},\"M_executed\":{m_exec},\"M_replayed\":{m_rep},\
                 \"R_executed\":{r_exec},\"R_replayed\":{r_rep},\"D_discarded_blocks\":{},\"D_discarded_bytes\":{},\"D_copied_bytes\":{}}}",
                d.discarded_blocks, d.discarded_bytes, d.copied_bytes
            ));
            Replays {
                supplied: n_sup,
                c_built,
                c_refused: c_ref,
                m: m_rep,
                r: r_rep,
            }
        }

        pub fn finish(line: String) -> String {
            if !in_play() {
                return line;
            }
            let timed = TIMED
                .lock()
                .unwrap()
                .take()
                .expect("the timed decode's counts");
            let (rbytes, rsha, rwall, store, data) = {
                let g = REF.lock().unwrap();
                let rf = g.as_ref().expect("the record pass");
                (rf.bytes, rf.sha.clone(), rf.wall, rf.store.clone(), rf.data)
            };
            let (cbytes, csha, cwall) = pass(data, arm_modes(true));
            let creplayed = scan::take_replayed();
            let crecs = sup::take();
            let checked = crecs.iter().filter(|r| !r.refused && r.supplied).count();
            let mismatches = sup::mismatches();
            reset_counts();
            set_modes((
                scan::Mode::Scan,
                sup::Mode::Open,
                Mode::Real,
                Mode::Real,
                Mode::Real,
                id::Mode::Real,
            ));
            let identical = cbytes == rbytes && csha == rsha && mismatches == 0;
            let tail = format!(
                ",\"oracle\":\"{}\",\"oracle_store\":{store},{timed},\"record_pass\":{{\"bytes\":{rbytes},\"sha256\":\"{rsha}\",\"wall_s\":{rwall:.4}}},\"check_pass\":{{\"bytes\":{cbytes},\"sha256\":\"{csha}\",\"wall_s\":{cwall:.4},\"replayed\":{creplayed},\"supplied_checked\":{checked},\"mismatches\":{mismatches},\"identical\":{identical}}}}}",
                name()
            );
            // (the record's closing brace replaced by the oracle's fields and its own)
            let body = line.strip_suffix('}').expect("a JSON object");
            format!("{body}{tail}")
        }
    }

    pub use live::{finish, record, take_handover};
}

/// The window a `--one` decode is built with (0: 4 a worker, as shipped)
static WINDOW: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn treatments(spec: &str) {
    if spec == "none" {
        return;
    }
    let text = std::fs::read_to_string(spec).expect("TREAT");
    let act = |n: &str| {
        ACTIVITIES
            .iter()
            .position(|a| *a == n)
            .unwrap_or_else(|| panic!("TREAT: no activity {n}"))
    };
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            ["pace", a, n] => ch::treat::set_pace(act(a), n.parse().expect("pace")),
            ["dose", a, b, k] => {
                ch::treat::set_dose(Some(act(a)), b.parse().expect("dose"), *k == "chase")
            }
            ["ready_cap", b] => ch::treat::set_ready_cap(Some(b.parse().expect("ready_cap"))),
            ["scanner_cpu", c] => ch::treat::set_scanner_cpu(Some(c.parse().expect("scanner_cpu"))),
            // (stage 2's gate contrast: the window, the harness's own; the library unchanged)
            ["window", w] => WINDOW.store(w.parse().expect("window"), Ordering::SeqCst),
            // (stage 2's pulses: the treatments on these places alone, the ledgers on)
            ["range", lo, hi] => ch::treat::set_range(Some((
                lo.parse().expect("range"),
                hi.parse().expect("range"),
            ))),
            // (the ideal-parts matrix: the arm's ideal parts, S the scan, E emission, and the
            // record pass that supplies them, in every arm; the pre-spin before the decode)
            ["oracle", x] => oracle::set(x),
            ["prespin", ms] => PRESPIN_MS.store(ms.parse().expect("prespin"), Ordering::SeqCst),
            _ => panic!("TREAT: {line}"),
        }
    }
}

fn features(file: &str, out: &str) {
    let data: &'static [u8] = std::fs::read(file).expect("read the input").leak();
    ch::set_on(true);
    ch::set_features(true);
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
    let feats: std::collections::HashMap<u32, ([u64; 5], f64)> = ch::take_features()
        .into_iter()
        .map(|(p, b, h)| (p, (b, h)))
        .collect();
    ch::set_features(false);
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
    let mut s =
        String::from("place\troute\tn\tr\tbytes\th1\th1_L\tw1\tw2_3\tw4_16\tw17_127\tw128\n");
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
        // (the run records' widths and the BWT column's entropy, where the block took the run
        // route; empty on the column route)
        let (w, hl) = feats
            .get(&place)
            .map_or((String::from("\t\t\t\t"), String::new()), |(b, h)| {
                (b.map(|x| x.to_string()).join("\t"), format!("{h:.6}"))
            });
        let _ = writeln!(s, "{place}\t{ro}\t{n}\t{r}\t{len}\t{h1:.6}\t{hl}\t{w}");
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
        || {},
    );
    drop(r);
    println!(
        "{{\"kind\":\"api\",\"requested\":{threads},{info},{}}}",
        p.json()
    );
}

fn one(file: &str, k: usize, on: bool, blocks_out: &str, group: u8) {
    let data: &'static [u8] = std::fs::read(file).expect("read the input").leak();
    let slots = match WINDOW.load(Ordering::SeqCst) {
        0 => mibz2::parallel::SLOTS_PER_THREAD * k,
        w => w,
    };
    assert!(!(on && group > 0), "the counters run with the ledgers off");
    // (the ideal-parts matrix: the record pass, outside every clock, the ledgers off; then the
    // arm's modes)
    oracle::record(data);
    // (every decode counts cycles and reference cycles, for its achieved frequency: group 4,
    // two fixed counters; a counter run's group in their place)
    let counters = Some(pmu::Counters::open(if group > 0 { group } else { 4 }));
    // the ledgers preallocated for the work ahead (no reallocation while decoding; an overflow
    // is counted): at most a block per 8 KiB of input, about 16 events a block on the scanner
    // and the reader, 20 on a worker
    let blocks = data.len() / 8192 + 64;
    ch::set_capacity(20 * (2 * blocks / k + 64), 16 * blocks);
    ch::set_on(on);
    let mut window = 0u64;
    let end = std::cell::Cell::new(0u64);
    // (the standard warm-up, journal §81: every CPU spun until the decode starts)
    let spun = prespin(PRESPIN_MS.load(Ordering::SeqCst));
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
        || {
            // (the stopwatch's region alone: the counters, on every thread of the decoder,
            // inherited ones included, disabled at its stop)
            if let Some(c) = &counters {
                c.disable();
            }
            end.set(ch::now_ns());
        },
    );
    let end_ns = end.get();
    // (the decoder's threads joined: their counts folded into the inherited counters)
    drop(r);
    // (the oracle's handover over the timed decode, taken before anything else decodes)
    let replays = oracle::take_handover();
    let mut line = format!(
        "{{\"kind\":\"decode\",\"k\":{k},\"on\":{},\"window\":{window},\"treat\":{},{}{}",
        u8::from(on),
        js(&ch::treat::describe()),
        p.json(),
        spun.map_or(String::new(), |(ms, cpus, s)| format!(
            ",\"prespin_ms\":{ms},\"prespin_cpus\":{cpus},\"prespin_s\":{s:.4}"
        ))
    );
    if let Some(c) = &counters {
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
        emit(format!("{line}}}"));
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
            t: [UNREAD; 7],
            cpu_scan: u16::MAX,
            cpu_work: u16::MAX,
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
    // (where every block is issued before the first is taken, as on a short input, the steady
    // phase is empty: its bounds kept in order, so the phases partition each timeline)
    let t2 = pts(ch::Tag::Issued).max().unwrap_or(0).max(t1);
    // the phases' minor faults and system time (the sampler's, every 5 ms: each bound's last
    // reading at or before it)
    let at = |t: u64| {
        p.series
            .iter()
            .rev()
            .find(|x| x.0 <= t)
            .map_or((p.start_use[2], p.start_use[1]), |x| (x.1, x.2))
    };
    let (f0, s0) = (p.start_use[2], p.start_use[1]);
    let ((f1, s1), (f2, s2)) = (at(t1), at(t2));
    let (f3, s3) = (f0 + p.use_[2], s0 + p.use_[1]);
    let phase_use = format!(
        "[[{:.0},{:.6}],[{:.0},{:.6}],[{:.0},{:.6}]]",
        f1 - f0,
        s1 - s0,
        f2 - f1,
        s2 - s1,
        f3 - f2,
        s3 - s2
    );
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
        .filter(|(_, b)| b.seen != [1; 8])
        .count();
    checks.push((
        "flow: every block admitted to the scanner, issued, enqueued, taken by a worker, routed, ready, taken by the reader and read to its end, once each".into(),
        flow_bad == 0 && n_blocks > 0,
        format!("{flow_bad} of {n_blocks} blocks not seen once at each point"),
    ));
    // the queue transitions' linear order (taken under the buffers' one lock): every point a
    // number of its own, 1 to their count, and each block's in its causal order (admitted,
    // issued, enqueued, taken by a worker, ready, taken by the reader, read to its end)
    let mut seqs: Vec<u64> = blocks.iter().flat_map(|b| b.seq).collect();
    let abandoned = threads.iter().any(|t| t.abandon.0.is_some());
    seqs.sort_unstable();
    let contiguous = seqs.iter().enumerate().all(|(i, &q)| q == i as u64 + 1);
    let order = [5usize, 0, 1, 2, 3, 4, 6];
    let lin_bad = blocks
        .iter()
        .filter(|b| order.windows(2).any(|w| b.seq[w[0]] >= b.seq[w[1]]))
        .count();
    checks.push((
        "linear order: every transition numbered once, each block's in causal order".into(),
        lin_bad == 0 && (contiguous || abandoned && seqs.windows(2).all(|w| w[1] > w[0])),
        format!(
            "{lin_bad} blocks out of order; numbers contiguous {contiguous} (an abandoned read takes one)"
        ),
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
        // (the ideal-parts matrix's oracles: a replayed construction removes A6a to A6c, a
        // supplied block's walk A7; each handover its own entry, one a replayed block)
        (
            "A6a",
            visits(ch::Tag::A6a),
            open + dense + refused - replays.c_built - replays.c_refused,
        ),
        ("A6b", visits(ch::Tag::A6b), open + dense - replays.c_built),
        ("A6c", visits(ch::Tag::A6c), open + dense - replays.c_built),
        ("A7", visits(ch::Tag::A7), open + dense - replays.supplied),
        (
            "hand_C",
            visits(ch::Tag::HandC),
            replays.c_built + replays.c_refused,
        ),
        ("hand_M", visits(ch::Tag::HandM), replays.m),
        ("hand_R", visits(ch::Tag::HandR), replays.r),
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
    // conservation: every buffer empty at the end, its arrivals the blocks (the scanner's,
    // one more where its last read gave no block)
    let arrivals = |i: usize| -> Option<u64> {
        match i {
            0..=3 | 6 => Some(nb),
            5 => Some(nb + u64::from(abandoned)),
            _ => None,
        }
    };
    let cons_bad: Vec<String> = (0..OCCS)
        .filter(|&i| occ[i].n != 0 || arrivals(i).is_some_and(|a| occ[i].arrivals != a))
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
    let abandon: u128 = threads
        .iter()
        .filter_map(|t| match t.abandon {
            (Some(a), Some(b)) => Some(u128::from(b - a)),
            _ => None,
        })
        .sum();
    let little = [
        res(0, 4, false),
        res(1, 2, false),
        res(2, 3, false),
        res(3, 4, false),
        res(3, 4, true),
        res(5, 0, false) + abandon,
        res(4, 6, false),
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
                .filter(|&i| !ch::is_wait(i as u8) && !ch::is_lock(i as u8))
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
    // ---- each wait for admission: what held it, and the head-of-line block (the review's §1:
    // its state then, queued, executing or ready, and its age since issued) ----
    let mut adm = [0u64; 3]; // window alone, cap alone, both
    let mut hol_state = [0u64; 4]; // queued, executing, ready (the reader's own pace), unknown
    let (mut hol_age, mut hol_n) = (0u128, 0u64);
    for t in threads.iter().filter(|t| t.role == Role::Scanner) {
        for &(at, taken, x) in &t.bounds {
            let (w, c, _, _) = ch::bound::unpack(x);
            adm[match (w, c) {
                (true, true) => 2,
                (false, true) => 1,
                _ => 0,
            }] += 1;
            match blocks.get(taken as usize) {
                Some(b) => {
                    let st = if b.t[3] <= at {
                        2
                    } else if b.t[2] <= at {
                        1
                    } else {
                        0
                    };
                    hol_state[st] += 1;
                    hol_age += u128::from(at.saturating_sub(b.t[0]));
                    hol_n += 1;
                }
                None => hol_state[3] += 1,
            }
        }
    }
    // ---- the gap to the bound, from the timestamps (the queueing-discipline lead) ----
    // the workers' idle time while the scanner waits for admission; in the start-up and in the
    // drain; and the head-of-line blocking: a block ready while an earlier one is not, so the
    // reader cannot take it (byte-weighted too: what holds the ready bytes' cap)
    let scan_waits: Vec<(u64, u64)> = threads
        .iter()
        .filter(|t| t.role == Role::Scanner)
        .flat_map(|t| t.gaps.iter().copied())
        .collect();
    let overlap = |a: &[(u64, u64)], b: &[(u64, u64)]| -> u64 {
        // (both sorted, each non-overlapping within itself)
        let (mut i, mut j, mut s) = (0, 0, 0u64);
        while i < a.len() && j < b.len() {
            let lo = a[i].0.max(b[j].0);
            let hi = a[i].1.min(b[j].1);
            if hi > lo {
                s += hi - lo;
            }
            if a[i].1 < b[j].1 {
                i += 1;
            } else {
                j += 1;
            }
        }
        s
    };
    let scan_in: Vec<(u64, u64)> = threads
        .iter()
        .filter(|t| t.role == Role::Scanner)
        .flat_map(|t| t.in_gaps.iter().copied())
        .collect();
    // (the scanner active: its life less its waits for input and for admission)
    let scan_life: Vec<(u64, u64)> = threads
        .iter()
        .filter(|t| t.role == Role::Scanner)
        .filter_map(|t| {
            let l = ledgers.iter().find(|l| l.name == t.name)?;
            Some((l.events.first()?.ns, l.events.last()?.ns))
        })
        .collect();
    let (mut idle_adm, mut idle_start, mut idle_drain) = (0u64, 0u64, 0u64);
    let (mut idle_input, mut idle_active, mut idle_all) = (0u64, 0u64, 0u64);
    for t in threads.iter().filter(|t| t.role == Role::Worker) {
        let a = overlap(&t.gaps, &scan_waits);
        let i = overlap(&t.gaps, &scan_in);
        idle_adm += a;
        idle_input += i;
        idle_active += overlap(&t.gaps, &scan_life).saturating_sub(a + i);
        idle_all += t.gaps.iter().map(|g| g.1 - g.0).sum::<u64>();
        idle_start += overlap(&t.gaps, &[(0, t1)]);
        idle_drain += overlap(&t.gaps, &[(t2, end_ns)]);
    }
    let (mut hol, mut hol_bytes, mut ready_rest) = (0u128, 0u128, 0u128);
    let mut before = 0u64;
    for b in &blocks {
        let (ready, taken) = (b.t[3], b.t[4]);
        let held = before.min(taken).saturating_sub(ready);
        hol += u128::from(held);
        hol_bytes += u128::from(held) * u128::from(b.out_bytes);
        ready_rest += u128::from(taken.saturating_sub(ready) - held);
        before = before.max(ready);
    }
    // ---- the line ----
    let _ = write!(
        line,
        ",\"blocks\":{n_blocks},\"routes\":{{\"open\":{open},\"open_dense\":{dense},\"refused\":{refused},\"skipped\":{skipped},\"column\":{column}}},\"t1_ns\":{t1},\"t2_ns\":{t2},\"end_ns\":{end_ns},\"phases\":[{}],\"phase_use\":{phase_use},\"admission\":{{\"waits\":[{},{},{}],\"hol_state\":[{},{},{},{}],\"hol_age_mean_ns\":{}}},\"gap\":{{\"idle_ns\":{idle_all},\"idle_scanner_active_ns\":{idle_active},\"idle_scanner_input_ns\":{idle_input},\"idle_admission_ns\":{idle_adm},\"idle_startup_ns\":{idle_start},\"idle_drain_ns\":{idle_drain},\"hol_ns\":{hol},\"hol_byte_ns\":{hol_bytes},\"ready_rest_ns\":{ready_rest}}},\"identities_ok\":{all_ok},\"identities\":[",
        PHASES.map(js).join(","),
        adm[0],
        adm[1],
        adm[2],
        hol_state[0],
        hol_state[1],
        hol_state[2],
        hol_state[3],
        if hol_n > 0 { hol_age / u128::from(hol_n) } else { 0 }
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
            "{}{{\"name\":{},\"role\":\"{:?}\",\"life_ns\":{},\"cpu_ns\":{},\"wait_wall_ns\":{},\"wait_cpu_ns\":{},\"events\":{},\"sched\":[{}],\"migrations\":{},\"states\":[{}],\"visits\":[{}],\"unattributed\":[{}],\"slot_bound\":[{},{},{}],\"moments\":[{}],\"cpus\":{{{}}},\"overflows\":{}}}",
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
            t.slot_bound[1],
            t.slot_bound[2],
            t.moments
                .iter()
                .map(|m| format!("[{},{:.0},{:.6e}]", m.0, m.1, m.2))
                .collect::<Vec<_>>()
                .join(","),
            t.cpus
                .iter()
                .map(|(c, ns)| format!("\"{c}\":{ns}"))
                .collect::<Vec<_>>()
                .join(","),
            t.overflows
        );
    }
    line += "]}";
    emit(line);
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
                "{p}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                b.route, b.n, b.r, b.syms_bytes, b.out_bytes, b.worker, b.cpu_scan, b.cpu_work
            );
            for t in b.t {
                let _ = write!(s, "\t{}", if t == UNREAD { -1 } else { t as i64 });
            }
            for q in b.seq {
                let _ = write!(s, "\t{q}");
            }
            for i in SCAN_COLS {
                let _ = write!(s, "\t{}", b.scan[i]);
            }
            for i in WORK_COLS {
                let _ = write!(s, "\t{}", b.work[i]);
            }
            let _ = write!(s, "\t{}", b.read_wait);
            for i in SCAN_VISITS {
                let _ = write!(s, "\t{}", b.scan_v[i]);
            }
            let _ = writeln!(s, "\t{}", b.read_wait_v);
        }
        f.write_all(s.as_bytes()).expect("write BLOCKS.tsv");
    }
}

/// The scanner's columns of a block's row: in_wait, in_maint, A2, A3, slot_wait, issue,
/// residual
const SCAN_COLS: [usize; 7] = [2, 3, 4, 5, 6, 7, NA];
/// The scanner's visits in a block's row: in_wait, in_maint, A2, A3, slot_wait (a worker's
/// visits are its route's, one each: forced flow, checked)
const SCAN_VISITS: [usize; 5] = [2, 3, 4, 5, 6];
/// A worker's: A5, pool, A6a, A6b, A6c, A7, A8, A9, publish, residual
const WORK_COLS: [usize; 10] = [9, 10, 11, 12, 13, 14, 15, 16, 17, NA];

/// The block rows' header (`BLOCKS.tsv`, before the harness's own leading columns)
#[allow(dead_code)]
const BLOCK_COLS: &str = "place\troute\tn\tr\tsyms_bytes\tout_bytes\tworker\tcpu_scan\tcpu_work\tt_issued\tt_enqueued\tt_got\tt_ready\tt_taken\tt_scan_start\tt_consumed\tq_issued\tq_enqueued\tq_got\tq_ready\tq_taken\tq_scan_start\tq_consumed\ts_in_wait\ts_in_maint\ts_A2\ts_A3\ts_slot_wait\ts_issue\ts_resid\tw_A5\tw_pool\tw_A6a\tw_A6b\tw_A6c\tw_A7\tw_A8\tw_A9\tw_publish\tw_resid\tr_wait\tv_in_wait\tv_in_maint\tv_A2\tv_A3\tv_slot_wait\tv_r_wait";

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
