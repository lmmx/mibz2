//! The walk's mechanisms taken apart on a file's blocks, on one thread: the interleaved
//! chains (the lane count), the software prefetch (none, into L1, into L2, non-temporal),
//! and the scheduling (the room walk's passes against the flagged walk's per-step
//! bookkeeping), each setting's output checked against the plain walk's. A diagnostic, for
//! choosing settings to try: the CI benchmark (the whole decoder, A/B) decides what is
//! faster.
//!
//! ```text
//! sweep FILE.bz2 [ROUNDS] [GHZ]
//! ```
//!
//! Each block's T-vector is built once, then each round walks it at every setting, the
//! settings' order rotated block by block so no setting always runs first after the build.
//! A setting's round is its time summed over the blocks. Against the default (the
//! settings `Segmented` uses), each round's ratio is paired, and the geometric mean of the
//! rounds' ratios is given with its 95% interval (a t-interval on the logs).
//!
//! Where the kernel lets a process count its own events (Linux on x86_64, a PMU exposed to
//! the machine, `perf_event_paranoid` at 2 or below), one more round, untimed, counts each
//! setting's instructions, cycles, last-level cache misses and back-end stall cycles, as
//! figures a byte of output: what a setting costs in work against what it waits for. Where
//! it doesn't, the table says so and has no counter columns.

use std::time::Instant;

use mibz2::bwt::{
    build, walk, walk_segments_flagged_with, walk_segments_room_with, SegScratch, TVector, PF_L1,
    PF_L2, PF_NTA, PF_OFF,
};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};

type Walk = fn(&mut TVector, u32, &mut Vec<u8>, &mut SegScratch, usize);

struct Setting {
    walk: &'static str,
    lanes: usize,
    pf: u8,
    segments: usize,
    f: Walk,
}

macro_rules! settings {
    ($v:ident, $segments:expr; $($l:literal),*) => {
        $(
            for (pf, room, flagged) in [
                (PF_OFF, walk_segments_room_with::<$l, PF_OFF> as Walk, walk_segments_flagged_with::<$l, PF_OFF> as Walk),
                (PF_L1, walk_segments_room_with::<$l, PF_L1>, walk_segments_flagged_with::<$l, PF_L1>),
                (PF_L2, walk_segments_room_with::<$l, PF_L2>, walk_segments_flagged_with::<$l, PF_L2>),
                (PF_NTA, walk_segments_room_with::<$l, PF_NTA>, walk_segments_flagged_with::<$l, PF_NTA>),
            ] {
                $v.push(Setting { walk: "room", lanes: $l, pf, segments: $segments, f: room });
                $v.push(Setting { walk: "flagged", lanes: $l, pf, segments: $segments, f: flagged });
            }
        )*
    };
}

// The settings `Segmented` uses (bwt.rs: walk_segments_room, FLAGGED_LANES, PREFETCH,
// FLAGGED_SEGMENTS)
const DEFAULT: (&str, usize, u8, usize) = ("room", 32, PF_L1, 512);

// Two-sided 95% t quantiles for 1 to 30 degrees of freedom
const T95: [f64; 30] = [
    12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
    2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
    2.052, 2.048, 2.045, 2.042,
];

fn pf_name(pf: u8) -> &'static str {
    match pf {
        PF_OFF => "none",
        PF_L1 => "L1",
        PF_L2 => "L2",
        _ => "NTA",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("usage: sweep FILE.bz2 [ROUNDS] [GHZ]");
    let rounds: usize = args.get(2).map_or(5, |s| s.parse().unwrap());
    let ghz: f64 = args.get(3).map_or(2.8, |s| s.parse().unwrap());
    assert!((2..=31).contains(&rounds), "ROUNDS: 2 to 31");
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();

    // The factorial at 512 segments, then the default's segment count varied alone
    let mut settings = Vec::new();
    settings!(settings, 512; 8, 12, 16, 24, 32, 48, 64);
    for segments in [256, 1024, 2048] {
        settings.push(Setting {
            walk: "room",
            lanes: 32,
            pf: PF_L1,
            segments,
            f: walk_segments_room_with::<32, PF_L1>,
        });
    }
    let default = settings
        .iter()
        .position(|s| (s.walk, s.lanes, s.pf, s.segments) == DEFAULT)
        .unwrap();
    let mut counters = pmu::Counters::open();

    let mut blocks = Vec::new();
    let mut sc = Scanner::new(SliceSource(&data));
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }

    // times[round][setting]: seconds summed over the blocks; counts[setting]: the
    // counters' totals over the untimed round
    let mut times = vec![vec![0f64; settings.len()]; rounds];
    let mut counts = vec![[0u64; pmu::EVENTS]; settings.len()];
    // each setting's group time enabled and running, summed
    let mut cover = vec![(0u64, 0u64); settings.len()];
    let (mut ll, mut freq) = (Vec::new(), [0u32; 256]);
    let (mut tt, mut seg) = (TVector::new(), SegScratch::default());
    let (mut plain, mut out) = (Vec::new(), Vec::new());
    let mut n = 0usize;
    for (bi, b) in blocks.iter().enumerate() {
        mtf::mtf(b, &mut ll, &mut freq, k).unwrap();
        build(&ll, &freq, &mut tt);
        walk(&tt, b.orig_ptr, &mut plain);
        n += plain.len();
        for s in &settings {
            (s.f)(&mut tt, b.orig_ptr, &mut out, &mut seg, s.segments);
            assert!(
                out == plain,
                "block {bi}: the {} walk, {} lanes, prefetch {}, {} segments",
                s.walk,
                s.lanes,
                pf_name(s.pf),
                s.segments
            );
        }
        for (r, round) in times.iter_mut().enumerate() {
            for j in 0..settings.len() {
                let si = (j + bi + r) % settings.len();
                let s = &settings[si];
                let t = Instant::now();
                (s.f)(&mut tt, b.orig_ptr, &mut out, &mut seg, s.segments);
                round[si] += t.elapsed().as_secs_f64();
            }
        }
        if let Some(c) = counters.as_mut() {
            for (si, s) in settings.iter().enumerate() {
                c.start();
                (s.f)(&mut tt, b.orig_ptr, &mut out, &mut seg, s.segments);
                let (v, enabled, running) = c.stop();
                for (total, v) in counts[si].iter_mut().zip(v) {
                    *total += v;
                }
                cover[si].0 += enabled;
                cover[si].1 += running;
            }
        }
    }

    let cb = |secs: f64| secs * ghz * 1e9 / n as f64;
    let median = |si: usize| {
        let mut v: Vec<f64> = times.iter().map(|round| round[si]).collect();
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let paired = |si: usize| {
        let logs: Vec<f64> = times
            .iter()
            .map(|round| (round[si] / round[default]).ln())
            .collect();
        let m = logs.iter().sum::<f64>() / rounds as f64;
        let var = logs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (rounds - 1) as f64;
        let h = T95[rounds - 2] * (var / rounds as f64).sqrt();
        (m.exp() - 1.0, (m - h).exp() - 1.0, (m + h).exp() - 1.0)
    };
    println!(
        "{path}: {} blocks, {:.1} MB of output; the walk alone at {ghz} GHz, {rounds} rounds, \
         against the {} walk, {} lanes, prefetch {}, {} segments",
        blocks.len(),
        n as f64 / 1e6,
        DEFAULT.0,
        DEFAULT.1,
        pf_name(DEFAULT.2),
        DEFAULT.3
    );
    match &counters {
        Some(c) => println!(
            "counters (one untimed round, one group): {}; a byte of output, misses a kB, \
             unscaled; \"counted\" is the share of the walks' time the group was on the PMU: \
             below 100% it was multiplexed out, and that row's counts are exploratory",
            c.names().join(", ")
        ),
        None => println!("counters: unavailable here ({})", pmu::why()),
    }
    print!("walk     lanes  prefetch  segments    c/B  change (95% interval)        ");
    if counters.is_some() {
        print!("  ins/B  cyc/B  LLC miss/kB  back-end stall  counted");
    }
    println!();
    let mut order: Vec<usize> = (0..settings.len()).collect();
    order.sort_by(|&a, &b| median(a).total_cmp(&median(b)));
    for si in order {
        let s = &settings[si];
        let (c, lo, hi) = paired(si);
        let mark = if si == default {
            "the default"
        } else if hi < 0.0 {
            "faster"
        } else if lo > 0.0 {
            "slower"
        } else {
            ""
        };
        print!(
            "{:7}  {:5}  {:8}  {:8}  {:5.2}  {:+6.1}% ({:+.1}%, {:+.1}%) {mark:11}",
            s.walk,
            s.lanes,
            pf_name(s.pf),
            s.segments,
            cb(median(si)),
            100.0 * c,
            100.0 * lo,
            100.0 * hi
        );
        if let Some(c) = &counters {
            // unscaled counts over the time the group ran, rated over the bytes that time
            // covered (all of them, where it was never multiplexed out)
            let (enabled, running) = cover[si];
            if running == 0 {
                print!("  {:>52}", "not counted (the group never ran)");
            } else {
                let counted = running as f64 / enabled as f64;
                let per = |i: usize| counts[si][i] as f64 / (n as f64 * counted);
                let or_na = |i: usize, v: String| if c.has(i) { v } else { "n/a".to_string() };
                print!(
                    "  {:5.2}  {:5.2}  {:>11}  {:>14}  {:6.1}%",
                    per(0),
                    per(1),
                    or_na(2, format!("{:.2}", 1e3 * per(2))),
                    or_na(3, format!("{:5.1}%", 100.0 * per(3) / per(1))),
                    100.0 * counted
                );
            }
        }
        println!();
    }
}

/// The process's own hardware counters, by perf_event_open(2), as one group (scheduled onto
/// the PMU together or not at all): instructions, cycles, last-level cache misses and
/// back-end stall cycles (that one optional: not every PMU counts it), user space only,
/// unscaled, with the group's times enabled and running
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod pmu {
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::sync::atomic::{AtomicIsize, Ordering};

    pub const EVENTS: usize = 4;
    // PERF_COUNT_HW_INSTRUCTIONS, _CPU_CYCLES, _CACHE_MISSES, _STALLED_CYCLES_BACKEND
    const CONFIG: [(u64, &str); EVENTS] = [
        (1, "instructions"),
        (0, "cycles"),
        (3, "LLC misses"),
        (8, "back-end stall cycles"),
    ];
    const SYS_PERF_EVENT_OPEN: usize = 298;
    const SYS_IOCTL: usize = 16;
    const IOC_ENABLE: usize = 0x2400;
    const IOC_DISABLE: usize = 0x2401;
    const IOC_RESET: usize = 0x2403;

    // The errno of the first counter that would not open
    static WHY: AtomicIsize = AtomicIsize::new(0);

    /// perf_event_attr as of its first version (PERF_ATTR_SIZE_VER0, 64 bytes)
    #[repr(C)]
    struct Attr {
        kind: u32,
        size: u32,
        config: u64,
        sample_period: u64,
        sample_type: u64,
        read_format: u64,
        flags: u64,
        wakeup_events: u32,
        bp_type: u32,
        config1: u64,
    }

    /// # Safety
    /// A raw system call: the arguments must be valid for call `n`
    unsafe fn syscall(n: usize, a: [usize; 5]) -> isize {
        let ret: isize;
        // SAFETY: the caller's; the syscall instruction clobbers rcx and r11 only
        unsafe {
            core::arch::asm!("syscall", inlateout("rax") n as isize => ret,
                in("rdi") a[0], in("rsi") a[1], in("rdx") a[2], in("r10") a[3], in("r8") a[4],
                lateout("rcx") _, lateout("r11") _, options(nostack));
        }
        ret
    }

    /// One event of the group led by `leader` (or the leader itself, disabled until the
    /// group is enabled); the leader reads the group's values with its times enabled and
    /// running, so every ratio comes from the same intervals
    fn open(config: u64, leader: Option<&File>) -> Result<File, isize> {
        let attr = Attr {
            kind: 0, // PERF_TYPE_HARDWARE
            size: 64,
            config,
            sample_period: 0,
            sample_type: 0,
            read_format: 1 | 2 | 8, // TOTAL_TIME_ENABLED | TOTAL_TIME_RUNNING | GROUP
            // disabled (the leader only), exclude_kernel, exclude_hv
            flags: u64::from(leader.is_none()) | (1 << 5) | (1 << 6),
            wakeup_events: 0,
            bp_type: 0,
            config1: 0,
        };
        let group = leader.map_or(usize::MAX, |f| f.as_raw_fd() as usize);
        let args = [&attr as *const Attr as usize, 0, usize::MAX, group, 0];
        // SAFETY: attr outlives the call; pid 0 (this process), cpu -1 (any)
        let fd = unsafe { syscall(SYS_PERF_EVENT_OPEN, args) };
        if fd < 0 {
            return Err(-fd);
        }
        // SAFETY: a new file descriptor this process owns
        Ok(unsafe { File::from_raw_fd(fd as i32) })
    }

    fn ioctl(f: &File, req: usize) {
        // SAFETY: a perf event's descriptor; the argument is PERF_IOC_FLAG_GROUP
        unsafe { syscall(SYS_IOCTL, [f.as_raw_fd() as usize, req, 1, 0, 0]) };
    }

    pub struct Counters {
        // the leader (instructions) first, then the events that joined its group
        files: Vec<(usize, File)>,
    }

    impl Counters {
        /// The largest of instructions + cycles + LLC misses + back-end stalls, the same
        /// without the stalls, and instructions + cycles alone that opens and is scheduled
        /// onto the PMU in a short trial (a group can open and still never be scheduled,
        /// its events together needing more counters than the PMU has); else none
        pub fn open() -> Option<Counters> {
            for events in [&[0usize, 1, 2, 3][..], &[0, 1, 2], &[0, 1]] {
                let Some(mut c) = Self::open_group(events) else {
                    continue;
                };
                c.start();
                let mut x = 0u64;
                for i in 0..1_000_000u64 {
                    x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(i));
                }
                let (_, enabled, running) = c.stop();
                if enabled > 0 && running > 0 {
                    return Some(c);
                }
                WHY.store(-1, Ordering::Relaxed);
            }
            None
        }
        fn open_group(events: &[usize]) -> Option<Counters> {
            let leader = match open(CONFIG[events[0]].0, None) {
                Ok(f) => f,
                Err(e) => {
                    WHY.store(e, Ordering::Relaxed);
                    return None;
                }
            };
            let mut files = Vec::new();
            for &i in &events[1..] {
                match open(CONFIG[i].0, Some(&leader)) {
                    Ok(f) => files.push((i, f)),
                    Err(e) => {
                        WHY.store(e, Ordering::Relaxed);
                        return None;
                    }
                }
            }
            files.insert(0, (events[0], leader));
            Some(Counters { files })
        }
        pub fn has(&self, i: usize) -> bool {
            self.files.iter().any(|(j, _)| *j == i)
        }
        pub fn names(&self) -> Vec<&'static str> {
            self.files.iter().map(|(i, _)| CONFIG[*i].1).collect()
        }
        pub fn start(&mut self) {
            ioctl(&self.files[0].1, IOC_RESET);
            ioctl(&self.files[0].1, IOC_ENABLE);
        }
        /// The group's counts since `start`, unscaled, and its times enabled and running
        /// (running below enabled: the group was multiplexed out for part of it)
        pub fn stop(&mut self) -> ([u64; EVENTS], u64, u64) {
            ioctl(&self.files[0].1, IOC_DISABLE);
            let mut v = [0u64; EVENTS];
            let mut buf = vec![0u8; 8 * (3 + self.files.len())];
            if self.files[0].1.read_exact(&mut buf).is_err() {
                return (v, 0, 0);
            }
            let w = |k: usize| u64::from_ne_bytes(buf[8 * k..8 * k + 8].try_into().unwrap());
            for (j, (i, _)) in self.files.iter().enumerate() {
                v[*i] = w(3 + j);
            }
            (v, w(1), w(2))
        }
    }

    pub fn why() -> String {
        match WHY.load(Ordering::Relaxed) {
            1 | 13 => "not permitted: perf_event_paranoid".into(),
            2 | 19 | 95 => "no PMU exposed to this machine".into(),
            -1 => "the counters opened but were never scheduled onto the PMU".into(),
            e => format!("perf_event_open: errno {e}"),
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
mod pmu {
    pub const EVENTS: usize = 4;
    pub struct Counters;
    impl Counters {
        pub fn open() -> Option<Counters> {
            None
        }
        pub fn names(&self) -> Vec<&'static str> {
            Vec::new()
        }
        pub fn has(&self, _: usize) -> bool {
            false
        }
        pub fn start(&mut self) {}
        pub fn stop(&mut self) -> ([u64; EVENTS], u64, u64) {
            ([0; EVENTS], 0, 0)
        }
    }
    pub fn why() -> String {
        "counters are read on Linux x86_64 only".into()
    }
}
