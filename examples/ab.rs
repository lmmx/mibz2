//! An interleaved A/B harness for whole decoders: each run is one process decoding one
//! file from disk, end to end (headers, Huffman, MTF, the T-vector, the walk and its
//! joining, RLE1, CRCs), its output hashed so every arm's bytes are compared.
//!
//! ```text
//! ab gen DIR [SCALE]                 the crate's corpora (tests/common), libbzip2 -9, into DIR
//! ab run OUT.jsonl [options] FILE..  time the arms on each file; one JSON line a run
//!     --arms a,b,c     arms (default mibz2,pbz2,lbzip2); ARM@EXE runs ARM in another
//!                      build of this harness (a baseline commit's, or bench/alternatives'),
//!                      so builds compare in the same process shape
//!     --threads 1,4    thread counts (1-thread runs are pinned to one CPU, so a
//!                      decoder's helper threads share its core)
//!     --rounds N       rounds (default 5); each round runs every arm once per file and
//!                      thread count, the arms' order rotated a step each round and
//!                      reversed every other cycle of rounds (`arm_order`): over 2 × arms
//!                      rounds, each arm runs in each position equally often
//!     --cap ARM=N,..   run ARM (as given in --arms) in the first N rounds only
//!     --pin CPU        the CPU 1-thread runs are pinned to (default 1)
//!     --spread         pin a run of T threads, where the machine has at least T physical
//!                      cores, to one CPU on each of T cores (its first SMT sibling), so 2
//!                      threads on CI's 2-core runners are two cores' worth, never one
//!                      core's two siblings; more threads than cores run unpinned
//!     --max-busy C     before each run, wait until the other processes keep at most C
//!                      cores busy (default 1.5; sampled over 2 s while nothing of ours runs)
//!     --settle S       that sample's length in seconds (default 2; CI's runners, which
//!                      run nothing else, take a shorter one)
//! ab summary OUT.jsonl [--ref ARM]   best and median wall and CPU, peak RSS, and paired
//!                                    ratios against ARM (geometric mean, 95% CI over rounds)
//! ab child ARM THREADS FILE          one run (what `run` spawns)
//! ab env                             the machine as the runs see it, one JSON line: the
//!                                    CPUs allowed, their physical cores, the cgroup's CPU
//!                                    quota, and whether the run-queue counter counts
//! ```
//!
//! Arms, block-parallel unless said:
//!
//! - `mibz2` (this build: `BzReader::new`, or `BzReader::parallel`, the cooperative
//!   policy's plan since it became the default), `mibz2-scalar` (the portable kernels),
//!   `mibz2-workers` (THREADS workers of a block each, `Plan::Workers`: the decoder before
//!   the policy);
//! - `mibz2-coop2`, `-coop3`, `-coop4` (the cooperative decoder, `bwt::Cooperative`:
//!   THREADS cores as THREADS / K teams of K, `BzReader::parallel_planned`) and
//!   `mibz2-auto` (the cooperative policy's plan for THREADS, `Plan::for_threads`: today's
//!   workers below 6 physical cores, else teams of 2 capped by the L3);
//! - `pbz2` (pbz2 0.2.0, a dev-dependency only: `DecoderReader`, or
//!   `ParallelDecoder::with_threads`);
//! - `libbzip2` (libbzip2 1.0.8, C, the bzip2 crate's bundled source: `MultiBzDecoder`),
//!   single-threaded;
//! - `lbzip2` (`lbzip2 -dc -nT`) and `indexed_bzip2` (bench/ib.py: indexed_bzip2's
//!   `IndexedBzip2File(parallelization=T)`), programs whose output is read through a pipe
//!   and hashed here, their CPU and peak RSS their own (an interpreter's start left out);
//! - in bench/alternatives' build of this harness only (`cfg(mibz2_alternatives)`: crates
//!   with dependencies of their own, kept out of this crate's dev-dependencies):
//!   `libbz2-rs` (libbz2-rs-sys, the Rust port of libbzip2, through the bzip2 crate 0.6:
//!   `MultiBzDecoder`), single-threaded; `lbzip2-rs` (the `lbzip2` crate:
//!   `StreamingBz2Read`, on the CPUs it may run on); `fbz` (`fbz::Reader`, `threads: T`);
//!   `bz2ngrs` (Softleif's Rust port of bz2-ng: `DecompressReader`), single-threaded;
//!   `bz2ngrs-oneshot` (`Decoder::decompress_all_to_vec` on the file read whole), beside
//!   `mibz2-oneshot` (`mibz2::decode` likewise), so the readers' cost shows apart;
//! - `bz2ng` (bz2-ng's C `bzip2 -dc`, the program AB_BZ2NG names: bench/compete.sh builds
//!   it), single-threaded.
//!
//! A single-threaded arm runs once a round on each file, at the fewest threads, and the
//! summaries pair that run with the reference's at every thread count.
//!
//! Recorded a run: wall (the decode, from opening the file to the last byte read), CPU
//! (user + sys, of every thread of the process: `getrusage`'s RUSAGE_SELF, and
//! RUSAGE_CHILDREN for a program's arm), peak RSS, output bytes and hash; each busy
//! thread's CPU and the threads' summed run-queue wait (time runnable but not running,
//! /proc's schedstat); and around it, the CPUs the run was given, the 1-minute load, the
//! cores the other processes kept busy during the run (the machine's busy time less the
//! run's own CPU), the steal, and the cgroup's CPU throttling and the CPU pressure stall
//! time (PSI) over the run, where the kernel has them (null where not).
//!
//! **Each thread's CPU is sampled from /proc every 10 ms** (and once more after the last
//! byte is read, before an in-process decoder is dropped), so a thread that ends between
//! samples loses at most about 10 ms, against the clock ticks' own 10 ms. Before bench
//! 3.0.0 the sampling was every 50 ms: mibz2's workers end when the jobs run out, before
//! the reader's last block, and lost up to 50 ms each, 0.1-0.35 s a run uncounted in the
//! per-thread split (never in the CPU total, which is the kernel's). The split's shortfall
//! against the total stays in the record, as the report's unattributed CPU. The sampler
//! is the harness's, not the decode's: it moves itself off the CPUs a run is pinned to
//! (onto the machine's others, where there are any: `sampler_cpus`), so it never preempts
//! a pinned decode, and its own CPU (RUSAGE_THREAD, exact: `sampler_s`) is left out of
//! the run's CPU. `AB_SAMPLE_MS` sets its interval (0: none, to measure its cost). A
//! 1-thread run of an in-process arm has no sampler: its split is its own thread's CPU
//! (RUSAGE_THREAD), and anything else its process spent shows as unattributed.

#[path = "../tests/common/mod.rs"]
mod common;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[repr(C)]
#[derive(Default)]
struct Rusage {
    utime: [i64; 2],
    stime: [i64; 2],
    rest: [i64; 14], // ru_maxrss (KiB) first
}

extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
    fn sched_setaffinity(pid: i32, size: usize, mask: *const u64) -> i32;
}

/// (CPU seconds, peak RSS in KiB) of this process (who 0), its waited-for children (-1)
/// or the calling thread (1, RUSAGE_THREAD)
fn rusage(who: i32) -> (f64, i64) {
    let mut r = Rusage::default();
    // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
    unsafe { getrusage(who, &mut r) };
    let t = |tv: [i64; 2]| tv[0] as f64 + tv[1] as f64 * 1e-6;
    (t(r.utime) + t(r.stime), r.rest[0])
}

/// A streaming 64-bit hash of the output (4 lanes of a multiply-rotate over 8-byte
/// words; the same however the bytes are cut into reads)
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

/// Move the calling thread (the sampler) off the CPUs its process is pinned to, onto the
/// machine's other online CPUs, so it never preempts a pinned decode: the CPUs it moved
/// to, or None where there are none (a run on every CPU) or the kernel refuses
fn off_the_pin() -> Option<String> {
    let status = std::fs::read_to_string("/proc/thread-self/status").ok()?;
    let pinned = cpu_list(
        status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))?,
    );
    let online = cpu_list(&std::fs::read_to_string("/sys/devices/system/cpu/online").ok()?);
    let others: Vec<usize> = online.into_iter().filter(|c| !pinned.contains(c)).collect();
    if others.is_empty() || others.iter().any(|&c| c >= 1024) {
        return None;
    }
    let mut mask = [0u64; 16];
    for &c in &others {
        mask[c / 64] |= 1 << (c % 64);
    }
    // SAFETY: a cpu_set_t of 1024 bits, read by the kernel; pid 0 is the calling thread
    let ok = unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) } == 0;
    ok.then(|| {
        others
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    })
}

/// Each thread, by its id: its name, its CPU (ticks) and its run-queue wait (ns: time
/// runnable, waiting for a CPU). Keyed by the id alone, the name read with the CPU from
/// the same `stat`: a name read apart can come back empty as a thread ends, and before
/// bench 3.0.0 that made a second entry for the same thread, counted twice
type Seen = BTreeMap<u32, (String, u64, u64)>;

/// A run's threads, as `Threads::finish` gives them: how many were busy, each one's CPU
/// ("name=seconds ..."), their summed run-queue wait, the sampler's own CPU (seconds,
/// exact) and the CPUs it ran on ("pinned": the run's own)
type Busy = (usize, String, f64, f64, String);

/// Each thread's CPU and run-queue wait of process `pid`, sampled every 10 ms until `stop`,
/// the last sample of each thread kept (both only grow)
struct Threads {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    seen: std::sync::Arc<std::sync::Mutex<Seen>>,
    /// the sampler: its own CPU seconds (RUSAGE_THREAD) and the CPUs it ran on, when it
    /// could leave the run's pinned CPUs
    handle: Option<std::thread::JoinHandle<(f64, Option<String>)>>,
    pid: u32,
}

/// A schedstat file's run-queue wait (its second field, ns), 0 where it can't be read
fn run_delay(path: &std::path::Path) -> u64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse().ok())
        .unwrap_or(0)
}

fn sample_tasks(pid: u32, seen: &std::sync::Mutex<Seen>) {
    let Ok(dir) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return;
    };
    let mut m = seen.lock().unwrap();
    for e in dir.flatten() {
        let Ok(st) = std::fs::read_to_string(e.path().join("stat")) else {
            continue;
        };
        // "tid (name) state ...": fields after the ")" of the name, state 3rd, utime 14th,
        // stime 15th
        let (Some((head, rest)), Ok(tid)) = (
            st.rsplit_once(')'),
            e.file_name().to_string_lossy().parse::<u32>(),
        ) else {
            continue;
        };
        let name = head.split_once('(').map_or("", |x| x.1);
        let f: Vec<&str> = rest.split_whitespace().collect();
        let ticks = f.get(11).and_then(|u| u.parse::<u64>().ok()).unwrap_or(0)
            + f.get(12).and_then(|u| u.parse::<u64>().ok()).unwrap_or(0);
        let wait = run_delay(&e.path().join("schedstat"));
        // (the latest name: a new thread has its parent's until it names itself)
        let v = m.entry(tid).or_insert_with(|| (String::new(), 0, 0));
        if !name.is_empty() {
            v.0 = name.to_string();
        }
        v.1 = v.1.max(ticks);
        v.2 = v.2.max(wait);
    }
}

impl Threads {
    /// The sampler, every AB_SAMPLE_MS milliseconds (default 10; 0: no sampler at all, so
    /// its cost to the decode can itself be measured: no split, no run-queue wait)
    fn watch(pid: u32) -> Threads {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let seen = std::sync::Arc::new(std::sync::Mutex::new(BTreeMap::new()));
        let (s, m) = (stop.clone(), seen.clone());
        let every: u64 = std::env::var("AB_SAMPLE_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        // (the decode's clock starts only once the sampler has left the pinned CPUs: until
        // it moves, it shares the decode's CPU, which cost a 12 ms run about 1 ms)
        let (moved, ready) = std::sync::mpsc::sync_channel::<()>(1);
        let handle = (every > 0).then(|| {
            std::thread::Builder::new()
                .name("sampler".into())
                .spawn(move || {
                    let away = off_the_pin();
                    let _ = moved.send(());
                    while !s.load(std::sync::atomic::Ordering::Relaxed) {
                        sample_tasks(pid, &m);
                        std::thread::sleep(Duration::from_millis(every));
                    }
                    (rusage(1).0, away)
                })
                .unwrap()
        });
        if handle.is_some() {
            let _ = ready.recv();
        }
        Threads {
            stop,
            seen,
            handle,
            pid,
        }
    }
    /// A sample now (an in-process decoder's, once its last byte is read and before it is
    /// dropped, while its threads are still there to read)
    fn sample(&self) {
        if self.handle.is_some() {
            sample_tasks(self.pid, &self.seen);
        }
    }
    /// The threads that ran (more than 2% of the busiest's CPU), and each one's CPU
    /// seconds, busiest first (the sampler left out; "main" is the thread reading the
    /// decoder and hashing its output); every thread's run-queue wait, summed, in
    /// seconds; and the sampler's own CPU seconds
    fn finish(mut self, own: bool) -> Busy {
        let Some(handle) = self.handle.take() else {
            return (0, String::new(), 0.0, 0.0, "off".into());
        };
        if own {
            sample_tasks(self.pid, &self.seen);
        }
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let (sampler, away) = handle.join().unwrap();
        let m = self.seen.lock().unwrap();
        let runq = m
            .values()
            .filter(|x| x.0 != "sampler")
            .map(|x| x.2 as f64 * 1e-9)
            .sum();
        // (in this process, the thread whose id is the process's is the reading thread)
        let mut v: Vec<(u64, String)> = m
            .iter()
            .filter(|(_, x)| x.0 != "sampler")
            .map(|(&tid, x)| {
                let main = own && tid == self.pid;
                (
                    x.1,
                    if main {
                        "main".to_string()
                    } else {
                        x.0.clone()
                    },
                )
            })
            .collect();
        v.sort_by(|a, b| b.cmp(a));
        let top = v.first().map_or(0, |x| x.0);
        let busy: Vec<String> = v
            .iter()
            .filter(|x| x.0 * 50 > top)
            .map(|(t, k)| format!("{k}={:.2}", *t as f64 / HZ))
            .collect();
        let away = away.unwrap_or_else(|| "pinned".into());
        (busy.len(), busy.join(" "), runq, sampler, away)
    }
}

/// Every byte of `r` through `h`; `last` runs after the last byte, while `r` (and a
/// decoder's threads) still lives
fn drain(mut r: impl Read, h: &mut Hash, last: &dyn Fn()) -> Result<(), String> {
    let mut buf = vec![0u8; 4 << 20];
    loop {
        match r.read(&mut buf) {
            Ok(0) => {
                last();
                return Ok(());
            }
            Ok(n) => h.update(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// Whether an arm decodes a stream's blocks in parallel. A single-threaded arm (libbzip2
/// and its Rust port, which have no thread count) runs once a round on each file, at the
/// smallest thread count (pinned to one CPU, as every 1-thread run), and the summaries
/// pair that run with the reference's at every thread count
fn parallel(arm: &str) -> bool {
    !matches!(
        arm,
        "libbzip2" | "libbz2-rs" | "bz2ngrs" | "bz2ngrs-oneshot" | "mibz2-oneshot" | "bz2ng"
    )
}

/// An arm that is another program writing the decoded bytes to stdout, read through a
/// pipe and hashed here: lbzip2's CLI, and indexed_bzip2 through Python (bench/ib.py, run
/// from the repository's root; AB_PYTHON the interpreter that has it, default python3).
/// The flag: the program starts an interpreter first, and says when it is ready to
/// decode, and the CPU it took to get there, in a first line ("ready SECONDS")
fn program(arm: &str, threads: usize, path: &str) -> Option<(Command, bool)> {
    let t = threads.to_string();
    match arm {
        "lbzip2" => {
            let mut c = Command::new("lbzip2");
            c.args(["-dc", "-n", &t, path]);
            Some((c, false))
        }
        "bz2ng" => {
            let mut c = Command::new(std::env::var("AB_BZ2NG").unwrap_or_else(|_| "bz2ng".into()));
            c.args(["-dc", path]);
            Some((c, false))
        }
        "indexed_bzip2" => {
            let mut c =
                Command::new(std::env::var("AB_PYTHON").unwrap_or_else(|_| "python3".into()));
            c.args(["bench/ib.py", &t, path]);
            Some((c, true))
        }
        _ => None,
    }
}

/// An in-process arm's decode of the file at `path`, every byte through `h`, `last` run
/// after its last byte (`drain`)
fn decode(
    arm: &str,
    threads: usize,
    path: &str,
    h: &mut Hash,
    last: &dyn Fn(),
) -> Result<(), String> {
    let open = || std::fs::File::open(path).map_err(|e| e.to_string());
    match arm {
        "mibz2" if threads == 1 => drain(mibz2::BzReader::new(open()?), h, last),
        "mibz2" => drain(mibz2::BzReader::parallel(open()?, threads), h, last),
        "mibz2-scalar" if threads == 1 => drain(
            mibz2::BzReader::with_kernels(open()?, mibz2::Kernels::scalar()),
            h,
            last,
        ),
        "mibz2-scalar" => drain(
            mibz2::BzReader::parallel_with(open()?, threads, 2 * threads, mibz2::Kernels::scalar()),
            h,
            last,
        ),
        // the cooperative decoder (bwt::Cooperative): THREADS cores as THREADS / K teams of
        // K, or as the policy plans THREADS (`Plan::for_threads`)
        "mibz2-coop2" | "mibz2-coop3" | "mibz2-coop4" | "mibz2-auto" | "mibz2-workers" => {
            use mibz2::parallel::Plan;
            let plan = match arm {
                "mibz2-auto" => Plan::for_threads(threads),
                "mibz2-workers" => Plan::Workers(threads),
                _ => {
                    let k = (arm.as_bytes()[arm.len() - 1] - b'0') as usize;
                    Plan::Teams {
                        teams: (threads / k).max(1),
                        k,
                    }
                }
            };
            let slots = 2 * plan.workers();
            drain(
                mibz2::BzReader::parallel_planned(open()?, plan, slots, mibz2::Kernels::detect()),
                h,
                last,
            )
        }
        "pbz2" if threads == 1 => drain(pbz2::DecoderReader::new(open()?), h, last),
        "pbz2" => drain(
            pbz2::ParallelDecoder::with_threads(open()?, threads),
            h,
            last,
        ),
        // libbzip2 1.0.8 (C: the bzip2 crate's bundled source, the tests' oracle), one
        // stream after another, as `bzip2 -dc` and Python's bz2 decode a file
        "libbzip2" => drain(bzip2::read::MultiBzDecoder::new(open()?), h, last),
        // (bench/alternatives' build of this harness only)
        #[cfg(mibz2_alternatives)]
        "libbz2-rs" => drain(bzip2_rs::read::MultiBzDecoder::new(open()?), h, last),
        // (no thread count: it takes the CPUs this process may run on, so a 1-thread
        // run's pin decides)
        #[cfg(mibz2_alternatives)]
        "lbzip2-rs" => drain(
            lbzip2::reader::StreamingBz2Read::new(open()?).map_err(|e| format!("{e:?}"))?,
            h,
            last,
        ),
        // (one stream after another, as `MultiBzDecoder`)
        #[cfg(mibz2_alternatives)]
        "bz2ngrs" => drain(bz2ngrs::DecompressReader::new(open()?), h, last),
        // (the file read whole, then decoded into one growing Vec: no reader between)
        #[cfg(mibz2_alternatives)]
        "bz2ngrs-oneshot" => {
            let data = std::fs::read(path).map_err(|e| e.to_string())?;
            let out = bz2ngrs::Decoder::new()
                .decompress_all_to_vec(&data, usize::MAX)
                .map_err(|e| e.to_string())?;
            h.update(&out);
            last();
            Ok(())
        }
        "mibz2-oneshot" => {
            let data = std::fs::read(path).map_err(|e| e.to_string())?;
            let out = mibz2::decode(&data).map_err(|e| e.to_string())?;
            h.update(&out);
            last();
            Ok(())
        }
        #[cfg(mibz2_alternatives)]
        "fbz" => {
            let opts = fbz::DecodeOptions {
                format: fbz::DecodeFormat::Bzip2,
                threads,
                ..Default::default()
            };
            drain(
                fbz::Reader::open(path, opts).map_err(|e| e.to_string())?,
                h,
                last,
            )
        }
        _ => Err(format!("unknown arm {arm} (in this build of the harness)")),
    }
}

/// A program's run: the result, the wall time, its busy threads, and the CPU it spent
/// before it was ready to decode
fn piped(mut cmd: Command, ready: bool, h: &mut Hash) -> (Result<(), String>, f64, Busy, f64) {
    let mut t0 = Instant::now();
    let mut p = match cmd.stdout(Stdio::piped()).spawn() {
        Ok(p) => p,
        Err(e) => {
            return (
                Err(format!("{cmd:?}: {e}")),
                0.0,
                (0, String::new(), 0.0, 0.0, String::new()),
                0.0,
            )
        }
    };
    let w = Threads::watch(p.id());
    let mut out = p.stdout.take().unwrap();
    let mut startup = Ok(0.0);
    if ready {
        let mut line = Vec::new();
        let mut b = [0u8];
        while out.read(&mut b).is_ok_and(|n| n == 1) && b[0] != b'\n' {
            line.push(b[0]);
        }
        let line = String::from_utf8_lossy(&line).to_string();
        startup = line
            .strip_prefix("ready ")
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("no ready line ({line:?})"));
        t0 = Instant::now();
    }
    let r = drain(out, h, &|| ());
    let threads = w.finish(false);
    let st = p.wait().unwrap();
    let wall = t0.elapsed().as_secs_f64();
    let r = r.and(startup.clone().map(|_| ())).and_then(|()| {
        if st.success() {
            Ok(())
        } else {
            Err(format!("{:?}: {st}", cmd.get_program()))
        }
    });
    (r, wall, threads, startup.unwrap_or(0.0))
}

fn child(arm: &str, threads: usize, path: &str) {
    let mut h = Hash::new();
    let external = program(arm, threads, path);
    let is_program = external.is_some();
    let (res, wall, (nthreads, per_thread, runq, sampler, sampler_cpus), startup) = match external {
        Some((cmd, ready)) => piped(cmd, ready, &mut h),
        // (a 1-thread in-process run has one thread to account for, so no sampler at all:
        // reading another thread's /proc entries cost a 12 ms run up to 17% (#69). Its split
        // is its own thread's CPU, RUSAGE_THREAD, and its run-queue wait its own schedstat)
        None if threads == 1 => {
            let t0 = Instant::now();
            let res = decode(arm, threads, path, &mut h, &|| ());
            let wall = t0.elapsed().as_secs_f64();
            let wait = run_delay(std::path::Path::new("/proc/thread-self/schedstat")) as f64 * 1e-9;
            let main = format!("main={:.2}", rusage(1).0);
            (res, wall, (1, main, wait, 0.0, "none".into()), 0.0)
        }
        None => {
            let w = Threads::watch(std::process::id());
            let t0 = Instant::now();
            let res = decode(arm, threads, path, &mut h, &|| w.sample());
            let wall = t0.elapsed().as_secs_f64();
            (res, wall, w.finish(true), 0.0)
        }
    };
    let (cpu_self, rss_self) = rusage(0);
    let (cpu_kids, rss_kids) = rusage(-1);
    // (a program's arm: its own peak RSS, and its CPU with this process's hashing, less
    // its interpreter's start; every arm's less the sampler's own, which is ours)
    let rss = if is_program { rss_kids } else { rss_self };
    let err = res.err().unwrap_or_default().replace('"', "'");
    println!(
        "{{\"wall\":{wall:.4},\"cpu\":{:.4},\"rss_kb\":{rss},\"out\":{},\"hash\":\"{}\",\"threads_busy\":{nthreads},\"thread_cpu\":\"{per_thread}\",\"runq_s\":{runq:.4},\"sampler_s\":{sampler:.4},\"sampler_cpus\":\"{sampler_cpus}\",\"err\":\"{err}\"}}",
        cpu_self + cpu_kids - startup - sampler,
        h.n,
        h.finish()
    );
}

/// (busy, steal, total) jiffies over all CPUs
fn stat() -> (u64, u64, u64) {
    let s = std::fs::read_to_string("/proc/stat").unwrap();
    let f: Vec<u64> = s
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse().unwrap())
        .collect();
    // user nice system idle iowait irq softirq steal
    let busy = f[0] + f[1] + f[2] + f[5] + f[6] + f[7];
    (busy, f[7], f[..8].iter().sum())
}

fn load1() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

const HZ: f64 = 100.0;

/// Busy jiffies of each CPU
fn per_cpu() -> Vec<u64> {
    let s = std::fs::read_to_string("/proc/stat").unwrap();
    s.lines()
        .filter(|l| l.starts_with("cpu") && l.as_bytes().get(3).is_some_and(u8::is_ascii_digit))
        .map(|l| {
            let f: Vec<u64> = l
                .split_whitespace()
                .skip(1)
                .map(|x| x.parse().unwrap())
                .collect();
            f[0] + f[1] + f[2] + f[5] + f[6] + f[7]
        })
        .collect()
}

/// Cores the other processes keep busy, over `secs` while nothing of ours runs: in all,
/// and the busiest single CPU's share
fn others_busy(secs: f64) -> (f64, f64) {
    let a = per_cpu();
    let t = Instant::now();
    std::thread::sleep(Duration::from_secs_f64(secs));
    let b = per_cpu();
    let dt = t.elapsed().as_secs_f64() * HZ;
    let each: Vec<f64> = a.iter().zip(&b).map(|(x, y)| (y - x) as f64 / dt).collect();
    (each.iter().sum(), each.iter().cloned().fold(0.0, f64::max))
}

/// A CPU list ("0-3,6"), as sysfs and /proc write them
fn cpu_list(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in s.trim().split(',').filter(|p| !p.is_empty()) {
        match part.split_once('-') {
            Some((a, b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<usize>(), b.parse::<usize>()) {
                    out.extend(a..=b);
                }
            }
            None => out.extend(part.parse::<usize>().ok()),
        }
    }
    out
}

/// The CPUs this process may run on (its Cpus_allowed_list), grouped by physical core
/// (sysfs's thread_siblings_list), each core's CPUs in order, the cores in order of their
/// first CPU; a CPU whose siblings can't be read is a core of its own
fn cores() -> Vec<Vec<usize>> {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let allowed = status
        .lines()
        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
        .map(cpu_list)
        .unwrap_or_else(|| {
            (0..std::thread::available_parallelism().map_or(1, |n| n.get())).collect()
        });
    group(&allowed, |c| {
        std::fs::read_to_string(format!(
            "/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list"
        ))
        .ok()
        .map(|s| cpu_list(&s))
    })
}

/// The allowed CPUs grouped by core, given each CPU's siblings (None: a core of its own)
fn group(allowed: &[usize], siblings: impl Fn(usize) -> Option<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for &c in allowed {
        let sib = siblings(c).unwrap_or_else(|| vec![c]);
        let mut core: Vec<usize> = sib.into_iter().filter(|x| allowed.contains(x)).collect();
        if !core.contains(&c) {
            core = vec![c];
        }
        if !out.contains(&core) {
            out.push(core);
        }
    }
    out.sort();
    out
}

/// `--spread`'s CPUs for `t` threads: the first CPU of each of the first `t` cores, where
/// there are at least `t` (and `t` > 1); None runs unpinned
fn spread_list(cores: &[Vec<usize>], t: usize) -> Option<String> {
    (t > 1 && t <= cores.len()).then(|| {
        cores[..t]
            .iter()
            .map(|c| c[0].to_string())
            .collect::<Vec<_>>()
            .join(",")
    })
}

/// The cgroup's CPU throttling so far: (periods throttled, time throttled in µs), from
/// cpu.stat (cgroup v2's `throttled_usec`, or v1's cpu controller's `throttled_time` in
/// ns); None where neither can be read
fn throttled() -> Option<(u64, u64)> {
    let cg = std::fs::read_to_string("/proc/self/cgroup").ok()?;
    let mut paths = Vec::new();
    for l in cg.lines() {
        let mut f = l.splitn(3, ':');
        let (id, ctl, path) = (f.next()?, f.next()?, f.next()?);
        if id == "0" && ctl.is_empty() {
            paths.push((format!("/sys/fs/cgroup{path}/cpu.stat"), 1));
        } else if ctl.split(',').any(|c| c == "cpu") {
            paths.push((format!("/sys/fs/cgroup/{ctl}{path}/cpu.stat"), 1000));
        }
    }
    for (p, per_us) in paths {
        let Ok(st) = std::fs::read_to_string(&p) else {
            continue;
        };
        let get = |k: &str| {
            st.lines()
                .find_map(|l| l.strip_prefix(k)?.trim().parse::<u64>().ok())
        };
        if let Some(n) = get("nr_throttled ") {
            let t = get("throttled_usec ").or_else(|| get("throttled_time ").map(|ns| ns / per_us));
            return Some((n, t.unwrap_or(0)));
        }
    }
    None
}

/// The CPU pressure stall time so far (µs: some task runnable and waiting for a CPU,
/// /proc/pressure/cpu's `some total`); None without PSI. A stall here is waiting for a
/// CPU, never a core waiting on memory: none says nothing about cache or memory contention
fn psi_cpu() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/pressure/cpu").ok()?;
    let l = s.lines().find(|l| l.starts_with("some"))?;
    l.split_whitespace()
        .find_map(|f| f.strip_prefix("total=")?.parse().ok())
}

/// A JSON number, or null
fn opt(v: Option<f64>) -> String {
    v.map_or("null".into(), |x| format!("{x:.3}"))
}

/// `ab env`: the machine as a run sees it, one JSON line. The run-queue counter is
/// checked, not assumed: twice as many threads as CPUs spin for 0.3 s, and it counts if
/// their summed wait is at least a quarter of their summed time off a CPU (most of it
/// should be wait); a counter that reads zero there is never read as no wait
fn env() {
    let cores = cores();
    let ncpu: usize = cores.iter().map(Vec::len).sum();
    let quota = std::fs::read_to_string("/sys/fs/cgroup/cpu.max")
        .ok()
        .and_then(|s| {
            let mut f = s.split_whitespace();
            let (q, p) = (f.next()?, f.next()?.parse::<f64>().ok()?);
            Some(if q == "max" {
                None
            } else {
                Some(q.parse::<f64>().ok()? / p)
            })
        })
        .or_else(|| {
            let q: f64 = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_quota_us")
                .ok()?
                .trim()
                .parse()
                .ok()?;
            let p: f64 = std::fs::read_to_string("/sys/fs/cgroup/cpu/cpu.cfs_period_us")
                .ok()?
                .trim()
                .parse()
                .ok()?;
            Some(if q < 0.0 { None } else { Some(q / p) })
        });
    let spin = 2 * ncpu.max(1);
    let t0 = Instant::now();
    let hs: Vec<_> = (0..spin)
        .map(|_| {
            std::thread::spawn(move || {
                let t = Instant::now();
                let mut x = 0u64;
                while t.elapsed() < Duration::from_millis(300) {
                    x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
                }
                std::hint::black_box(x);
                let st = std::fs::read_to_string("/proc/thread-self/schedstat").unwrap_or_default();
                let f: Vec<u64> = st
                    .split_whitespace()
                    .filter_map(|x| x.parse().ok())
                    .collect();
                (
                    t.elapsed().as_secs_f64(),
                    f.first().copied(),
                    f.get(1).copied(),
                )
            })
        })
        .collect();
    let r: Vec<_> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    let _ = t0;
    let off: f64 = r
        .iter()
        .map(|(w, on, _)| w - on.unwrap_or(0) as f64 * 1e-9)
        .sum();
    let wait: f64 = r.iter().map(|(_, _, w)| w.unwrap_or(0) as f64 * 1e-9).sum();
    let counts = r.iter().all(|x| x.2.is_some()) && off > 0.0 && wait >= 0.25 * off;
    let list = |v: &[usize]| {
        v.iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    println!(
        "{{\"cpus\":{ncpu},\"cores\":{},\"core_cpus\":[{}],\"cpu_quota\":{},\"throttling\":{},\"psi_cpu\":{},\"runq\":{{\"counts\":{counts},\"spin_threads\":{spin},\"off_cpu_s\":{off:.3},\"wait_s\":{wait:.3}}}}}",
        cores.len(),
        cores.iter().map(|c| format!("\"{}\"", list(c))).collect::<Vec<_>>().join(","),
        match quota {
            Some(Some(q)) => format!("{q:.2}"),
            _ => "null".into(),
        },
        throttled().is_some(),
        psi_cpu().is_some(),
    );
}

fn json_str(line: &str, key: &str) -> Option<String> {
    let k = format!("\"{key}\":");
    let i = line.find(&k)? + k.len();
    let rest = &line[i..];
    if let Some(r) = rest.strip_prefix('"') {
        Some(r[..r.find('"')?].to_string())
    } else {
        let end = rest.find([',', '}']).unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn json_f(line: &str, key: &str) -> f64 {
    json_str(line, key)
        .and_then(|s| s.parse().ok())
        .unwrap_or(f64::NAN)
}

fn run(args: &[String]) {
    let out_path = &args[0];
    let mut arms = vec!["mibz2".to_string(), "pbz2".into(), "lbzip2".into()];
    let mut threads = vec![1usize, 4];
    let mut rounds = 5;
    let mut max_busy = 1.5;
    let mut settle = 2.0;
    let mut pin = "1".to_string();
    let mut spread = false;
    let mut capped: BTreeMap<String, usize> = BTreeMap::new();
    let mut files = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--arms" => {
                arms = args[i + 1].split(',').map(String::from).collect();
                i += 1;
            }
            "--threads" => {
                threads = args[i + 1].split(',').map(|x| x.parse().unwrap()).collect();
                i += 1;
            }
            "--rounds" => {
                rounds = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--pin" => {
                pin = args[i + 1].clone();
                i += 1;
            }
            "--spread" => spread = true,
            "--max-busy" => {
                max_busy = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--settle" => {
                settle = args[i + 1].parse().unwrap();
                i += 1;
            }
            "--cap" => {
                for c in args[i + 1].split(',') {
                    let (arm, n) = c.split_once('=').expect("--cap ARM=ROUNDS,..");
                    capped.insert(arm.to_string(), n.parse().unwrap());
                }
                i += 1;
            }
            f => files.push(f.to_string()),
        }
        i += 1;
    }
    let me = std::env::current_exe().unwrap();
    let mut out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out_path)
        .unwrap();
    let fewest = threads.iter().copied().min().unwrap_or(1);
    // (--spread: a run of t threads, where there are t cores, on each core's first CPU)
    let cores = cores();
    let spread_cpus = |t: usize| spread.then(|| spread_list(&cores, t)).flatten();
    for round in 0..rounds {
        for file in &files {
            for &t in &threads {
                for arm in arm_order(round, arms.len()).into_iter().map(|i| &arms[i]) {
                    let name = arm.split_once('@').map_or(arm.as_str(), |x| x.0);
                    // (a single-threaded arm once a round, at the fewest threads; a capped
                    // arm in its first rounds only)
                    if (!parallel(name) && t != fewest)
                        || capped.get(arm).is_some_and(|&n| round >= n)
                    {
                        continue;
                    }
                    let waited = Instant::now();
                    // (and no CPU more than half busy: a 1-thread run is pinned to one, and
                    // a pinned job of someone else's would share it)
                    let (mut busy, mut one) = others_busy(settle);
                    while busy > max_busy || one > 0.5 {
                        eprintln!("  waiting: others keep {busy:.2} cores busy, one CPU {one:.2} (load {:.2})", load1());
                        std::thread::sleep(Duration::from_secs(20));
                        (busy, one) = others_busy(settle);
                    }
                    let (name, exe) = match arm.split_once('@') {
                        Some((n, e)) => (n.to_string(), std::path::PathBuf::from(e)),
                        None => (arm.clone(), me.clone()),
                    };
                    let cpus = if t == 1 {
                        Some(pin.clone())
                    } else {
                        spread_cpus(t)
                    };
                    let mut cmd = match &cpus {
                        Some(list) => {
                            let mut c = Command::new("taskset");
                            c.args(["-c", list]).arg(&exe);
                            c
                        }
                        None => Command::new(&exe),
                    };
                    cmd.args(["child", &name, &t.to_string(), file]);
                    let load = load1();
                    let (b0, s0, tot0) = stat();
                    let (th0, psi0) = (throttled(), psi_cpu());
                    let w0 = Instant::now();
                    let o = cmd.output().unwrap();
                    let span = w0.elapsed().as_secs_f64();
                    let (b1, s1, tot1) = stat();
                    let (th1, psi1) = (throttled(), psi_cpu());
                    let throttle = th0.zip(th1).map(|(a, b)| (b.0 - a.0, b.1 - a.1));
                    let psi_ms = psi0.zip(psi1).map(|(a, b)| (b - a) as f64 / 1000.0);
                    let line = String::from_utf8_lossy(&o.stdout);
                    let line = line.trim();
                    let cpu = json_f(line, "cpu");
                    let others = ((b1 - b0) as f64 / HZ - cpu).max(0.0) / span;
                    let steal = 100.0 * (s1 - s0) as f64 / (tot1 - tot0).max(1) as f64;
                    let base = std::path::Path::new(file)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let rec = format!(
                        "{{\"round\":{round},\"corpus\":\"{base}\",\"threads\":{t},\"arm\":\"{arm}\",\"parallel\":{},\"cpus\":\"{}\",\"load1\":{load:.2},\"others_busy\":{others:.2},\"pre_busy\":{busy:.2},\"steal_pct\":{steal:.1},\"throttled_n\":{},\"throttled_ms\":{},\"psi_cpu_ms\":{},\"waited_s\":{:.0},{}",
                        parallel(&name),
                        cpus.as_deref().unwrap_or("all"),
                        opt(throttle.map(|x| x.0 as f64)),
                        opt(throttle.map(|x| x.1 as f64 / 1000.0)),
                        opt(psi_ms),
                        waited.elapsed().as_secs_f64() - settle,
                        line.strip_prefix('{').unwrap_or("\"broken\":1}")
                    );
                    eprintln!(
                        "r{round} {base:<12} t={t} {arm:<16} wall {:>7.3} cpu {:>7.3} rss {:>5.0}MB others {others:.2} steal {steal:.1}% {}",
                        json_f(line, "wall"),
                        cpu,
                        json_f(line, "rss_kb") / 1024.0,
                        json_str(line, "err").unwrap_or_default()
                    );
                    writeln!(out, "{rec}").unwrap();
                    out.flush().unwrap();
                }
            }
        }
    }
}

/// The arms' order in round `round` (indices into the arms): rotated a step each round,
/// and reversed in every other cycle of `n` rounds, so that over 2n rounds each arm runs
/// in each position equally often. (Before bench 3.0.0 the order was reversed in every
/// other round, which with 2 arms undid the rotation: head ran first in every round of
/// every pull request's bench.)
fn arm_order(round: usize, n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    if n == 0 {
        return order;
    }
    order.rotate_left(round % n);
    if (round / n) % 2 == 1 {
        order.reverse();
    }
    order
}

/// The two-sided 95% t quantile for `df` degrees of freedom
fn t95(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    if df == 0 {
        f64::NAN
    } else {
        T.get(df - 1).copied().unwrap_or(1.96)
    }
}

/// The geometric mean of paired ratios and its 95% interval
fn ratio_ci(r: &[f64]) -> (f64, f64, f64) {
    let l: Vec<f64> = r.iter().map(|x| x.ln()).collect();
    let n = l.len() as f64;
    let m = l.iter().sum::<f64>() / n;
    let sd = (l.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let h = t95(l.len() - 1) * sd / n.sqrt();
    (m.exp(), (m - h).exp(), (m + h).exp())
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn summary(args: &[String]) {
    let text = std::fs::read_to_string(&args[0]).unwrap();
    let reference = args
        .iter()
        .position(|a| a == "--ref")
        .map(|i| args[i + 1].clone());
    // (corpus, threads) -> arm -> round -> line
    let mut by: BTreeMap<(String, u64), BTreeMap<String, BTreeMap<u64, String>>> = BTreeMap::new();
    let mut arm_order: Vec<String> = Vec::new();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let all_threads: std::collections::BTreeSet<u64> =
        lines.iter().map(|l| json_f(l, "threads") as u64).collect();
    for l in &lines {
        let arm = json_str(l, "arm").unwrap();
        if !arm_order.contains(&arm) {
            arm_order.push(arm.clone());
        }
        // (a single-threaded arm's run stands at every thread count)
        let t = json_f(l, "threads") as u64;
        let at: Vec<u64> = if json_str(l, "parallel").as_deref() == Some("false") {
            all_threads.iter().copied().collect()
        } else {
            vec![t]
        };
        for t in at {
            by.entry((json_str(l, "corpus").unwrap(), t))
                .or_default()
                .entry(arm.clone())
                .or_default()
                .insert(json_f(l, "round") as u64, l.to_string());
        }
    }
    let reference = reference.unwrap_or_else(|| arm_order[0].clone());
    for ((corpus, t), arms) in &by {
        let ref_runs = arms.get(&reference);
        let ref_hash = ref_runs
            .and_then(|r| r.values().next())
            .and_then(|l| json_str(l, "hash"));
        let out_mb = ref_runs
            .and_then(|r| r.values().next())
            .map_or(f64::NAN, |l| json_f(l, "out") / 1e6);
        println!("\n### {corpus}, {t} thread{} ({out_mb:.0} MB out; ratios: arm / {reference}, paired by round)\n", if *t == 1 { "" } else { "s" });
        println!("| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |");
        println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|");
        for arm in arm_order.iter().filter(|a| arms.contains_key(*a)) {
            let runs = &arms[arm];
            let col = |k: &str| runs.values().map(|l| json_f(l, k)).collect::<Vec<f64>>();
            let (mut w, mut c, mut r, mut o) =
                (col("wall"), col("cpu"), col("rss_kb"), col("others_busy"));
            let best = |v: &[f64]| v.iter().cloned().fold(f64::MAX, f64::min);
            let errs: Vec<String> = runs
                .values()
                .filter_map(|l| json_str(l, "err"))
                .filter(|e| !e.is_empty())
                .collect();
            let same = runs.values().all(|l| json_str(l, "hash") == ref_hash) && errs.is_empty();
            let pair = |k: &str| {
                let rr: Vec<f64> = runs
                    .iter()
                    .filter_map(|(round, l)| {
                        ref_runs?.get(round).map(|rl| json_f(l, k) / json_f(rl, k))
                    })
                    .collect();
                if arm == &reference {
                    "1".to_string()
                } else if rr.len() >= 2 {
                    let (g, lo, hi) = ratio_ci(&rr);
                    format!("{g:.3} [{lo:.3}, {hi:.3}]")
                } else {
                    "-".into()
                }
            };
            // (a single-threaded arm's 1-thread runs, at another thread count)
            let serial = runs
                .values()
                .next()
                .is_some_and(|l| json_f(l, "threads") as u64 != *t);
            println!(
                "| {arm}{} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.0} MB | {} | {} | {:.2} | {} |",
                if serial { " (single-threaded: its 1-thread runs)" } else { "" },
                runs.len(),
                best(&w),
                median(&mut w),
                best(&c),
                median(&mut c),
                median(&mut r) / 1024.0,
                pair("wall"),
                pair("cpu"),
                median(&mut o),
                if same {
                    "yes".to_string()
                } else {
                    format!("NO {errs:?}")
                }
            );
        }
    }
}

fn gen(args: &[String]) {
    let dir = std::path::Path::new(&args[0]);
    let scale: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    std::fs::create_dir_all(dir).unwrap();
    for (name, data) in common::corpora(scale) {
        let z = common::compress(&data, 9);
        let p = dir.join(format!("{}.bz2", name.replace(' ', "")));
        // (a new file: never written through a hard link to someone else's)
        let _ = std::fs::remove_file(&p);
        std::fs::write(&p, &z).unwrap();
        eprintln!("{}: {} -> {} bytes", p.display(), data.len(), z.len());
    }
}

fn main() {
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--bench")
        .collect();
    match args.first().map(String::as_str) {
        Some("child") => child(&args[1], args[2].parse().unwrap(), &args[3]),
        Some("run") => run(&args[1..]),
        Some("summary") => summary(&args[1..]),
        Some("gen") => gen(&args[1..]),
        Some("env") => env(),
        _ => eprintln!("usage: ab gen DIR [SCALE] | ab run OUT.jsonl [--arms a,b] [--threads 1,4] [--rounds N] [--settle S] FILE.. | ab summary OUT.jsonl [--ref ARM] | ab child ARM T FILE | ab env"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_arm_runs_in_every_position_equally() {
        for n in 1..=8 {
            // counts[arm][position] over 2n rounds
            let mut counts = vec![vec![0; n]; n];
            for round in 0..2 * n {
                let order = arm_order(round, n);
                let mut seen = order.clone();
                seen.sort();
                assert_eq!(seen, (0..n).collect::<Vec<_>>(), "a permutation");
                for (pos, &arm) in order.iter().enumerate() {
                    counts[arm][pos] += 1;
                }
            }
            for row in &counts {
                assert!(row.iter().all(|&c| c == 2), "n {n}: {counts:?}");
            }
        }
        // two arms: head (0) and base (1) alternate first, in a balanced pattern
        let first: Vec<usize> = (0..4).map(|r| arm_order(r, 2)[0]).collect();
        assert_eq!(first, vec![0, 1, 1, 0]);
    }

    #[test]
    fn cpu_lists() {
        assert_eq!(cpu_list("0-3,6\n"), vec![0, 1, 2, 3, 6]);
        assert_eq!(cpu_list("5"), vec![5]);
        assert_eq!(cpu_list(""), Vec::<usize>::new());
    }

    #[test]
    fn spread_takes_one_cpu_a_core() {
        // CI's runners: 4 CPUs on 2 cores, siblings 0-1 | 2-3
        let sib = |c: usize| Some(if c < 2 { vec![0, 1] } else { vec![2, 3] });
        let cores = group(&[0, 1, 2, 3], sib);
        assert_eq!(cores, vec![vec![0, 1], vec![2, 3]]);
        assert_eq!(spread_list(&cores, 2).as_deref(), Some("0,2"));
        // more threads than cores, or one thread: unpinned (1 thread has its own pin)
        assert_eq!(spread_list(&cores, 4), None);
        assert_eq!(spread_list(&cores, 1), None);
        // siblings interleaved (0,4 | 1,5 | ...), as many Intel parts number them
        let sib = |c: usize| Some(vec![c % 4, c % 4 + 4]);
        let cores = group(&(0..8).collect::<Vec<_>>(), sib);
        assert_eq!(spread_list(&cores, 4).as_deref(), Some("0,1,2,3"));
        // a CPU set that allows one sibling of each core
        let cores = group(&[0, 2], |c| {
            Some(if c < 2 { vec![0, 1] } else { vec![2, 3] })
        });
        assert_eq!(cores, vec![vec![0], vec![2]]);
        // siblings unreadable: each CPU a core of its own
        assert_eq!(group(&[0, 1], |_| None), vec![vec![0], vec![1]]);
    }
}
