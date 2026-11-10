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
//!     --machine-state  also record, beside each run, its cycles and instructions and the
//!                      clock and package temperature before and after (`machine`: off by
//!                      default, recorded and never judged)
//!     --sysfs-state    only the parent's half of `--machine-state`: the clock, governor,
//!                      EPP, turbo and package temperature read from sysfs before each run
//!                      is spawned and after it exits, with their timestamps. The child is
//!                      untouched: no counters, no CPU-set sampling, `AB_MACHINE_STATE`
//!                      cleared (bench 3.5.0; recorded and never judged)
//!     --hold-cpu N|auto
//!                      hold one CPU busy around each run (bench 3.8.0; off by default, and
//!                      for workstation runs: an idle machine starts a decode in a slow mode,
//!                      cold-start/20261110T102700Z): a spinner, a thread of this process
//!                      pinned to CPU N, started and seen running there before the settle
//!                      gate, held through the decode until the child is reaped, then stopped
//!                      and joined (on an error or a panic too; as a thread it dies with this
//!                      process). `auto` takes the last CPU of the last core the run's CPUs
//!                      leave free, else a CPU the run doesn't use (an SMT sibling), else
//!                      the last CPU (`pick_hold`); `overlap` records whether its core shares
//!                      a CPU with the run's (always so for an unpinned run). The gate and
//!                      `others_busy` take the spinner's own CPU off (its raw readings kept),
//!                      and its CPU is never the decoder's. Never passed to the child, so an
//!                      ARM@EXE build without the option runs as before. Recorded as
//!                      `held_spinner`: {cpu, auto, overlap, decoder_cpus, owner, cpu_s,
//!                      pre_busy_raw, pre_one_raw, others_busy_raw}
//!     --hold-max-threads T
//!                      hold only around runs of at most T threads (default: every run)
//! ab summary OUT.jsonl [--ref ARM]   best and median wall and CPU, peak RSS, and paired
//!                                    ratios against ARM (geometric mean, 95% CI over rounds)
//! ab child ARM THREADS FILE [--hold-cpu N|auto]
//!                                    one run (what `run` spawns, never with the option).
//!                                    `--hold-cpu`, for a driver spawning the child itself:
//!                                    the child holds its own spinner thread, started before
//!                                    the sampler and the counters and stopped before its CPU
//!                                    is read, that CPU taken off the decode's and left out
//!                                    of the per-thread split; `auto` picks against the CPUs
//!                                    the child may run on (`held_spinner`, owner `child`)
//! ab env                             the machine as the runs see it, one JSON line: the
//!                                    CPUs allowed, their physical cores, the cgroup's CPU
//!                                    quota, and whether the run-queue counter counts
//!                                    (and `machine`: the cpufreq driver, governor, EPP,
//!                                    turbo, and what perf can count). Active: it spins
//!                                    2 x CPUs threads for 0.3 s and opens perf counters
//! ab env --facts                     passive: the same machine facts read from procfs,
//!                                    sysfs and sched_getaffinity alone, no spin thread and
//!                                    no perf counter opened; the CPUs allowed (the
//!                                    affinity and Cpus_allowed_list, which must agree), the
//!                                    cores, and the CPU quota of the process's effective
//!                                    cgroup hierarchy, every ancestor read (v1 or v2):
//!                                    established unlimited, a limit, or unknown with why
//!                                    (`facts`; bench 3.5.0)
//! ab settle [--seconds N] [--interval S]
//!                                    untimed: sample the clock and the package temperature
//!                                    until they hold within a registered tolerance or N
//!                                    seconds pass, one JSON line (`machine::settle`)
//! ab sample [--count N] [--interval S]
//!                                    untimed and read-only: exactly N readings of the
//!                                    machine's state (default 10), S seconds apart (default
//!                                    1), one JSON line each, whatever they read; it never
//!                                    stops early, waits for nothing and never fails
//!                                    (`machine::sample`)
//! ab telemetry-check                 untimed: the machine-state recorder's failure paths
//!                                    (a failed enable or disable, a missing counter,
//!                                    multiplexing, the user-only fallback) and coverage
//!                                    (inheritance, the region), one JSON line a case;
//!                                    exits 1 if any case FAILs (`machine::check`)
//! ```
//!
//! Arms, block-parallel unless said:
//!
//! - `mibz2-cells@PATH` (experimental cells admission plus shipped fallback, sequential
//!   only). Build its separate harness with `cargo rustc --release --example ab --
//!   --cfg mibz2_cells_bench`, then copy it to PATH. The ordinary build omits this arm
//!   to keep its generated default-reader code independent of the experimental reader;
//!   the library type itself is always compiled, with no feature or default change;
//! - `mibz2` (this build: `BzReader::new`, or `BzReader::parallel`, the cooperative
//!   policy's plan since it became the default), `mibz2-scalar` (the portable kernels),
//!   `mibz2-workers` (THREADS workers of a block each, `Plan::Workers`: the decoder before
//!   the policy);
//! - `mibz2-coop2`, `-coop3`, `-coop4` (the cooperative decoder, `bwt::Cooperative`:
//!   THREADS cores as THREADS / K teams of K, `BzReader::parallel_planned`, 2 blocks in
//!   flight a team), `mibz2-team2`, `-team3`, `-team4` (the same plans with the default's
//!   window, `parallel::SLOTS_PER_THREAD` blocks a team) and
//!   `mibz2-auto` (the cooperative policy's plan for THREADS, `Plan::for_threads`: today's
//!   workers below 6 physical cores, else teams of 2 capped by the L3);
//! - `mibz2-pipe2`, `-pipe3`, `-pipe4` (research: the pipelined team,
//!   `BzReader::parallel_pipelined`, its leader's RLE1, CRC and next move-to-front overlapped
//!   with its helpers' build and walk: THREADS / K teams of K in the teamK arms' window);
//! - `mibz2-pipe2-park`, `-pipe3-park`, `-pipe4-park`, and the same with `-spin2k`,
//!   `-small` and `-both` (research: a pipeK arm with its levers,
//!   `BzReader::parallel_pipelined_levers`: `park`, its waits for the build's end parking
//!   after the same 4,096 spins; `spin2k`, those waits yielding after 2,000 spins, the
//!   spin-budget control; `small`, its leader staging on after a block below `MIN_COOP`;
//!   `both`, park and small together), each its pipeK arm's plan and window;
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
//! With `--machine-state` only (bench 3.3.0; the `machine` module): the run's cycles,
//! instructions and task clock (perf, inherited by its threads, enabled one after another
//! just before the wall's clock starts and disabled one after another just after it stops:
//! `perf_skew_ns` bounds the difference), cycles over the task clock, and the clock, governor, EPP,
//! turbo and package temperature before and after it; each null with a reason where it
//! can't be read.
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
//!
//! **The child's record is schema `ab-child/2`** (bench 3.6.0): beside its figures, `out`
//! and `hash`, and `err` (the error's text, empty on success), it carries `schema`, and
//! `err_kind` and `err_block` (a decoder error's kind and block, from the reader's
//! `mibz2::Error`; both null on success, or where the error carries no block). A decoder
//! error is reported in the record, and the child exits normally. For untimed checks only,
//! never set by `ab run`: `AB_SHA256=1` adds `sha256`, a SHA-256 of the output (null
//! without it), and `AB_READ_BUF=BYTES` sets the read buffer (4 MiB without it).

#[cfg(mibz2_cells_bench)]
mod ab_cells;

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
    fn sched_getaffinity(pid: i32, size: usize, mask: *mut u64) -> i32;
    fn sched_getcpu() -> i32;
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

/// (voluntary, involuntary) context switches of this process (who 0), every thread's,
/// live and exited, summed (`machine`'s per-run telemetry, bench 3.4.0)
fn ctx_switches() -> (i64, i64) {
    let mut r = Rusage::default();
    // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
    unsafe { getrusage(0, &mut r) };
    // (ru_nvcsw and ru_nivcsw: the last two of the 14 longs after the two timevals)
    (r.rest[12], r.rest[13])
}

/// A streaming 64-bit hash of the output (4 lanes of a multiply-rotate over 8-byte
/// words; the same however the bytes are cut into reads)
struct Hash {
    h: [u64; 4],
    tail: [u8; 32],
    nt: usize,
    n: u64,
    /// with `AB_SHA256=1` only (untimed checks): a SHA-256 of the same stream, beside the
    /// 64-bit hash, for byte-exact identity
    sha: Option<Sha256>,
}

impl Hash {
    fn new() -> Self {
        Hash {
            h: [1, 2, 3, 4],
            tail: [0; 32],
            nt: 0,
            n: 0,
            sha: std::env::var("AB_SHA256")
                .is_ok_and(|v| v == "1")
                .then(Sha256::new),
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
        if let Some(s) = &mut self.sha {
            s.update(b);
        }
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

/// SHA-256 (FIPS 180-4), streaming: a byte-exact output identity for untimed checks, only
/// with `AB_SHA256=1` (never set by a timed run)
struct Sha256 {
    st: [u32; 8],
    buf: [u8; 64],
    nb: usize,
    len: u64,
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
        Sha256 {
            st: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buf: [0; 64],
            nb: 0,
            len: 0,
        }
    }

    fn compress(st: &mut [u32; 8], b: &[u8]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(b[4 * i..4 * i + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *st;
        for (&k, &wi) in Self::K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k)
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
        for (x, y) in st.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *x = x.wrapping_add(y);
        }
    }

    fn update(&mut self, mut b: &[u8]) {
        self.len += b.len() as u64;
        if self.nb > 0 {
            let k = (64 - self.nb).min(b.len());
            self.buf[self.nb..self.nb + k].copy_from_slice(&b[..k]);
            self.nb += k;
            b = &b[k..];
            if self.nb < 64 {
                return;
            }
            let t = self.buf;
            Self::compress(&mut self.st, &t);
            self.nb = 0;
        }
        let mut c = b.chunks_exact(64);
        for x in &mut c {
            Self::compress(&mut self.st, x);
        }
        let r = c.remainder();
        self.buf[..r.len()].copy_from_slice(r);
        self.nb = r.len();
    }

    fn hex(&self) -> String {
        let mut st = self.st;
        let mut tail = self.buf[..self.nb].to_vec();
        tail.push(0x80);
        while tail.len() % 64 != 56 {
            tail.push(0);
        }
        tail.extend_from_slice(&(self.len * 8).to_be_bytes());
        for blk in tail.chunks_exact(64) {
            Self::compress(&mut st, blk);
        }
        st.iter().map(|x| format!("{x:08x}")).collect()
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
    /// the CPUs seen (`--machine-state` only)
    cpus: Option<SeenCpus>,
}

/// A schedstat file's run-queue wait (its second field, ns), 0 where it can't be read
fn run_delay(path: &std::path::Path) -> u64 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.split_whitespace().nth(1)?.parse().ok())
        .unwrap_or(0)
}

/// The CPUs the sampler saw a run's threads on (`--machine-state` only): each thread's last
/// CPU (/proc stat's field 39) at each sample, the sampler's own left out. A sample, not an
/// account: a thread can run on a CPU between two samples and never be seen there
type SeenCpus = std::sync::Arc<std::sync::Mutex<std::collections::BTreeSet<u32>>>;

fn sample_tasks(pid: u32, seen: &std::sync::Mutex<Seen>, cpus: Option<&SeenCpus>) {
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
        if let Some(c) = cpus {
            if name != "sampler" && name != "held-spinner" {
                if let Some(cpu) = f.get(36).and_then(|x| x.parse::<u32>().ok()) {
                    c.lock().unwrap().insert(cpu);
                }
            }
        }
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
    fn watch(pid: u32, see_cpus: bool) -> Threads {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let seen = std::sync::Arc::new(std::sync::Mutex::new(BTreeMap::new()));
        let cpus: Option<SeenCpus> = see_cpus.then(Default::default);
        let (s, m, c) = (stop.clone(), seen.clone(), cpus.clone());
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
                        sample_tasks(pid, &m, c.as_ref());
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
            cpus,
        }
    }
    /// The set the sampler fills with the CPUs it sees (`--machine-state` only), read after
    /// `finish`
    fn cpu_set(&self) -> Option<SeenCpus> {
        self.cpus.clone()
    }
    /// A sample now (an in-process decoder's, once its last byte is read and before it is
    /// dropped, while its threads are still there to read)
    fn sample(&self) {
        if self.handle.is_some() {
            sample_tasks(self.pid, &self.seen, self.cpus.as_ref());
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
            sample_tasks(self.pid, &self.seen, self.cpus.as_ref());
        }
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let (sampler, away) = handle.join().unwrap();
        let m = self.seen.lock().unwrap();
        let runq = m
            .values()
            .filter(|x| x.0 != "sampler" && x.0 != "held-spinner")
            .map(|x| x.2 as f64 * 1e-9)
            .sum();
        // (in this process, the thread whose id is the process's is the reading thread)
        let mut v: Vec<(u64, String)> = m
            .iter()
            .filter(|(_, x)| x.0 != "sampler" && x.0 != "held-spinner")
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

/// `--hold-cpu`'s choice: a CPU number, or `auto` (`pick_hold`)
#[derive(Clone, Copy, Debug, PartialEq)]
enum Hold {
    Cpu(usize),
    Auto,
}

impl Hold {
    fn parse(v: &str) -> Hold {
        match v {
            "auto" => Hold::Auto,
            n => Hold::Cpu(n.parse().unwrap_or_else(|_| {
                eprintln!("ab: --hold-cpu takes a CPU number or `auto`, not {n:?}");
                std::process::exit(2)
            })),
        }
    }
}

/// `--hold-cpu auto`'s CPU, given the allowed CPUs grouped by core (`cores`) and the CPUs
/// the decoder may run on: the last CPU of the last core the decoder can't use at all; else
/// the last CPU the decoder can't use (an SMT sibling of one of its CPUs); else the last of
/// the decoder's own. With whether the held CPU's core shares a CPU with the decoder's
fn pick_hold(cores: &[Vec<usize>], decoder: &[usize]) -> Option<(usize, bool)> {
    let free_core = cores
        .iter()
        .rev()
        .find(|c| !c.iter().any(|x| decoder.contains(x)));
    if let Some(&cpu) = free_core.and_then(|c| c.last()) {
        return Some((cpu, false));
    }
    let all: Vec<usize> = cores.iter().flatten().copied().collect();
    if let Some(&cpu) = all.iter().rev().find(|x| !decoder.contains(x)) {
        return Some((cpu, true));
    }
    all.last().map(|&cpu| (cpu, true))
}

/// Whether CPU `cpu`'s core (`siblings`: its thread_siblings_list, itself where unread)
/// shares a CPU with the decoder's
fn hold_overlaps(cpu: usize, siblings: &[usize], decoder: &[usize]) -> bool {
    decoder.contains(&cpu) || siblings.iter().any(|x| decoder.contains(x))
}

/// The held spinner (bench 3.8.0, W1 of docs/journal/2026-11-10-next-experiments.md): one
/// thread of the process that holds it, pinned to one CPU and spinning there, so the machine
/// never goes wholly idle around a decode (cold-start/20261110T102700Z: a decode started
/// after the machine went idle ran at about 2.1× the wall; one spinner held throughout kept
/// 30 of 30 runs fast). A thread, never a process: it can't outlive its owner. It is started,
/// and its affinity and CPU confirmed, before the settle gate; stopped and joined after the
/// child is reaped, or on drop (an error, a panic). Its CPU is its own (`cpu_s`), never the
/// decoder's.
struct Held {
    cpu: usize,
    auto: bool,
    overlap: bool,
    /// the CPUs the decoder may run on, as the choice saw them
    decoder: String,
    tid: u32,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<f64>>,
}

impl Held {
    /// The spinner on `cpu`, once it runs there (Err: the kernel refused the CPU, or the
    /// thread wasn't seen on it within a second)
    fn start(cpu: usize, auto: bool, overlap: bool) -> Result<Held, String> {
        if cpu >= 1024 {
            return Err(format!("CPU {cpu} is beyond a 1024-bit CPU set"));
        }
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let s = stop.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<u32, String>>(1);
        let handle = std::thread::Builder::new()
            .name("held-spinner".into())
            .spawn(move || {
                let ready = (|| {
                    let mut mask = [0u64; 16];
                    mask[cpu / 64] |= 1 << (cpu % 64);
                    // SAFETY: a cpu_set_t of 1024 bits, read or written by the kernel; pid 0
                    // is the calling thread
                    if unsafe { sched_setaffinity(0, std::mem::size_of_val(&mask), mask.as_ptr()) }
                        != 0
                    {
                        return Err(format!(
                            "sched_setaffinity to CPU {cpu}: {}",
                            std::io::Error::last_os_error()
                        ));
                    }
                    let mut got = [0u64; 16];
                    // SAFETY: as above
                    if unsafe {
                        sched_getaffinity(0, std::mem::size_of_val(&got), got.as_mut_ptr())
                    } < 0
                    {
                        return Err(format!(
                            "sched_getaffinity: {}",
                            std::io::Error::last_os_error()
                        ));
                    }
                    if got != mask {
                        return Err(format!("the affinity read back is not CPU {cpu} alone"));
                    }
                    let t = Instant::now();
                    // SAFETY: no arguments; the CPU the calling thread runs on
                    while unsafe { sched_getcpu() } != cpu as i32 {
                        if t.elapsed() > Duration::from_secs(1) {
                            return Err(format!("not seen running on CPU {cpu} within 1 s"));
                        }
                        std::thread::yield_now();
                    }
                    // (/proc/thread-self is "PID/task/TID")
                    std::fs::read_link("/proc/thread-self")
                        .ok()
                        .and_then(|p| p.file_name()?.to_str()?.parse().ok())
                        .ok_or_else(|| "no thread id from /proc/thread-self".to_string())
                })();
                let ok = ready.is_ok();
                let _ = tx.send(ready);
                // (a plain increment, as bench/cold_start_spin.c's `hold`, the stop flag read
                // every 4,096 of them)
                let mut x = 0u64;
                while ok && !s.load(std::sync::atomic::Ordering::Relaxed) {
                    for _ in 0..4096 {
                        x = std::hint::black_box(x.wrapping_add(1));
                    }
                }
                rusage(1).0
            })
            .map_err(|e| format!("spawning the spinner: {e}"))?;
        let mut held = Held {
            cpu,
            auto,
            overlap,
            decoder: String::new(),
            tid: 0,
            stop,
            handle: Some(handle),
        };
        match rx.recv() {
            Ok(Ok(tid)) => {
                held.tid = tid;
                Ok(held)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("the spinner ended before it was ready".into()),
        }
    }

    /// `hold` for a decoder on the CPUs `decoder`, among `cores` (`online_cores`)
    fn for_decoder(hold: Hold, cores: &[Vec<usize>], decoder: &[usize]) -> Result<Held, String> {
        let (cpu, auto, overlap) = match hold {
            Hold::Cpu(cpu) => {
                let sib = std::fs::read_to_string(format!(
                    "/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list"
                ))
                .map(|s| cpu_list(&s))
                .unwrap_or_default();
                (cpu, false, hold_overlaps(cpu, &sib, decoder))
            }
            Hold::Auto => {
                let (cpu, overlap) =
                    pick_hold(cores, decoder).ok_or("no CPU to hold: none allowed")?;
                (cpu, true, overlap)
            }
        };
        let mut held = Held::start(cpu, auto, overlap)?;
        held.decoder = decoder
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Ok(held)
    }

    /// Its CPU so far, in ticks (/proc's utime + stime)
    fn ticks(&self) -> u64 {
        std::fs::read_to_string(format!("/proc/self/task/{}/stat", self.tid))
            .ok()
            .and_then(|st| {
                let f: Vec<u64> = st
                    .rsplit_once(')')?
                    .1
                    .split_whitespace()
                    .skip(11)
                    .take(2)
                    .filter_map(|x| x.parse().ok())
                    .collect();
                (f.len() == 2).then(|| f[0] + f[1])
            })
            .unwrap_or(0)
    }

    /// Stop it and wait for it: its own CPU seconds (RUSAGE_THREAD, exact)
    fn stop(mut self) -> f64 {
        self.halt()
    }

    fn halt(&mut self) -> f64 {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.handle.take().map_or(0.0, |h| h.join().unwrap_or(0.0))
    }

    /// The record's `held_spinner` fields before its CPU is known
    fn facts(&self) -> String {
        format!(
            "\"cpu\":{},\"auto\":{},\"overlap\":{},\"decoder_cpus\":\"{}\"",
            self.cpu, self.auto, self.overlap, self.decoder
        )
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        self.halt();
    }
}

/// A census reading: thread `tid` is now called `name`. The latest name wins: a new thread
/// carries its parent's name (`ab`) until it names itself, and a reading can fall between
fn census_saw(seen: &mut BTreeMap<u64, String>, tid: u64, name: &str) {
    seen.insert(tid, name.trim().to_string());
}

#[cfg(test)]
mod json_field_tests {
    use super::*;

    #[test]
    fn a_nested_field_of_the_same_name_is_not_read() {
        let rec = r#"{"threads":10,"note":"a \"cpu\":7 text","held_spinner":{"cpu":19,"cpu_s":2.1},"wall":0.138,"cpu":0.9824,"err":""}"#;
        assert_eq!(json_f(rec, "cpu"), 0.9824);
        assert_eq!(json_f(rec, "wall"), 0.138);
        assert_eq!(json_str(rec, "err").as_deref(), Some(""));
        assert!(json_f(rec, "cpu_s").is_nan());
        let plain = r#"{"wall":0.5,"cpu":0.5,"rss_kb":1}"#;
        assert_eq!(json_f(plain, "cpu"), 0.5);
        assert_eq!(json_f(plain, "rss_kb"), 1.0);
    }
}

#[cfg(test)]
mod child_record_tests {
    use super::*;

    fn sha(chunks: &[&[u8]]) -> String {
        let mut s = Sha256::new();
        for c in chunks {
            s.update(c);
        }
        s.hex()
    }

    /// FIPS 180-4's vectors, and the same bytes however they are cut into reads
    #[test]
    fn sha256_known_answers_and_any_cut() {
        assert_eq!(
            sha(&[b""]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha(&[b"abc"]),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let two = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            sha(&[two]),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        let million = vec![b'a'; 1_000_000];
        let want = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";
        assert_eq!(sha(&[&million]), want);
        let cuts: Vec<&[u8]> = million
            .chunks(63)
            .chain(std::iter::once(&b""[..]))
            .collect();
        assert_eq!(sha(&cuts), want);
        let (a, b) = million.split_at(64);
        let (b, c) = b.split_at(1);
        assert_eq!(sha(&[a, b, c]), want);
    }

    /// A decoder error's kind and block, and nulls where there is none
    #[test]
    fn the_error_identity() {
        *ERR_ID.lock().unwrap() = None;
        assert_eq!(
            err_id_json(),
            ",\"schema\":\"ab-child/2\",\"err_kind\":null,\"err_block\":null"
        );
        let e = mibz2::Error {
            kind: mibz2::ErrorKind::BlockCrc {
                stored: 1,
                computed: 2,
            },
            block: Some(41),
            bit: Some(80),
        };
        record_err_id(&std::io::Error::from(e));
        assert_eq!(
            err_id_json(),
            ",\"schema\":\"ab-child/2\",\"err_kind\":\"BlockCrc\",\"err_block\":41"
        );
        let e = mibz2::Error::new(mibz2::ErrorKind::Truncated);
        record_err_id(&std::io::Error::from(e));
        assert_eq!(
            err_id_json(),
            ",\"schema\":\"ab-child/2\",\"err_kind\":\"Truncated\",\"err_block\":null"
        );
        // an error that is not the decoder's leaves the identity as it was
        record_err_id(&std::io::Error::other("a reader's own"));
        assert!(err_id_json().contains("Truncated"));
        *ERR_ID.lock().unwrap() = None;
    }

    #[test]
    fn the_read_buffer() {
        assert_eq!(read_buf(None), 4 << 20);
        assert_eq!(read_buf(Some("4096")), 4096);
        assert_eq!(read_buf(Some("0")), 4 << 20);
        assert_eq!(read_buf(Some("x")), 4 << 20);
    }
}

#[cfg(test)]
mod census_tests {
    use super::*;

    #[test]
    fn a_thread_renamed_after_its_first_reading_counts_by_its_own_name() {
        let mut seen = BTreeMap::new();
        census_saw(&mut seen, 7, "ab\n");
        census_saw(&mut seen, 8, "mibz2-0\n");
        census_saw(&mut seen, 7, "mibz2-coop-1\n");
        census_saw(&mut seen, 8, "mibz2-0\n");
        let names: Vec<&str> = seen.values().map(String::as_str).collect();
        assert_eq!(names, ["mibz2-coop-1", "mibz2-0"]);
    }
}

/// The names of every thread a child process runs, read from outside it, by its parent, every
/// AB_CENSUS_MS milliseconds (default 5) until it exits: the same observer for every arm,
/// whichever build the child is. Only names: a thread's CPU is the child's own report
struct Census {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: std::thread::JoinHandle<BTreeMap<u64, String>>,
}

impl Census {
    fn watch(pid: u32) -> Census {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let s = stop.clone();
        let every: u64 = std::env::var("AB_CENSUS_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5)
            .max(1);
        let handle = std::thread::Builder::new()
            .name("census".into())
            .spawn(move || {
                let mut seen = BTreeMap::new();
                let dir = format!("/proc/{pid}/task");
                while !s.load(std::sync::atomic::Ordering::Relaxed) {
                    let Ok(tasks) = std::fs::read_dir(&dir) else {
                        break;
                    };
                    for t in tasks.flatten() {
                        let Some(tid) = t.file_name().to_str().and_then(|x| x.parse().ok()) else {
                            continue;
                        };
                        if let Ok(name) = std::fs::read_to_string(t.path().join("comm")) {
                            census_saw(&mut seen, tid, &name);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(every));
                }
                seen
            })
            .unwrap();
        Census { stop, handle }
    }

    /// Every thread seen, by name, sorted (one entry a thread)
    fn finish(self) -> String {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let seen: BTreeMap<u64, String> = self.handle.join().unwrap_or_default();
        let mut names: Vec<String> = seen.into_values().map(|n| n.replace('"', "'")).collect();
        names.sort_unstable();
        names.join(" ")
    }
}

/// Every byte of `r` through `h`; `last` runs after the last byte, while `r` (and a
/// decoder's threads) still lives
fn drain(mut r: impl Read, h: &mut Hash, last: &dyn Fn()) -> Result<(), String> {
    // (`AB_READ_BUF=BYTES`, untimed checks only: read-buffer variants of an exact-error
    // check; unset, the 4 MiB every run has always read with)
    let mut buf = vec![0u8; read_buf(std::env::var("AB_READ_BUF").ok().as_deref())];
    loop {
        match r.read(&mut buf) {
            Ok(0) => {
                last();
                return Ok(());
            }
            Ok(n) => h.update(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => {
                record_err_id(&e);
                return Err(e.to_string());
            }
        }
    }
}

/// The read buffer's size: `AB_READ_BUF`'s, where it is a positive number, else 4 MiB
fn read_buf(v: Option<&str>) -> usize {
    v.and_then(|v| v.parse().ok())
        .filter(|&n: &usize| n > 0)
        .unwrap_or(4 << 20)
}

/// The decoder's error, as the child's record reports it apart from its text: its kind's
/// name and its block (`mibz2::Error`'s, where the reader's error carries one), so that an
/// untimed check can compare an expected error exactly (record schema `ab-child/2`)
static ERR_ID: std::sync::Mutex<Option<(String, Option<u64>)>> = std::sync::Mutex::new(None);

fn record_err_id(e: &std::io::Error) {
    if let Some(d) = e.get_ref().and_then(|x| x.downcast_ref::<mibz2::Error>()) {
        let name = format!("{:?}", d.kind);
        let name = name
            .split(['(', ' ', '{'])
            .next()
            .unwrap_or_default()
            .to_string();
        *ERR_ID.lock().unwrap_or_else(|p| p.into_inner()) = Some((name, d.block));
    }
}

/// `,"err_kind":…,"err_block":…` for the child's record (null where there is none)
fn err_id_json() -> String {
    let mut s = String::from(",\"schema\":\"ab-child/2\"");
    s += &match ERR_ID.lock().unwrap_or_else(|p| p.into_inner()).clone() {
        Some((k, b)) => format!(
            ",\"err_kind\":\"{k}\",\"err_block\":{}",
            b.map_or("null".into(), |b| b.to_string())
        ),
        None => ",\"err_kind\":null,\"err_block\":null".into(),
    };
    s
}

/// Whether an arm decodes a stream's blocks in parallel. A single-threaded arm (libbzip2
/// and its Rust port, which have no thread count) runs once a round on each file, at the
/// smallest thread count (pinned to one CPU, as every 1-thread run), and the summaries
/// pair that run with the reference's at every thread count
fn parallel(arm: &str) -> bool {
    #[cfg(mibz2_cells_bench)]
    if arm == "mibz2-cells" {
        return false;
    }
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
        #[cfg(mibz2_cells_bench)]
        "mibz2-cells" => drain(ab_cells::CellsReader::new(open()?), h, last),
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
        "mibz2-coop2" | "mibz2-coop3" | "mibz2-coop4" | "mibz2-team2" | "mibz2-team3"
        | "mibz2-team4" | "mibz2-auto" | "mibz2-workers" => {
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
            // the coopK arms keep 2 blocks in flight a team, as they always have; the teamK
            // arms keep the default's window, SLOTS_PER_THREAD a team (what
            // `BzReader::parallel` gives the policy's plan), so a teamK arm differs from
            // `mibz2` in its plan alone
            let slots = if arm.starts_with("mibz2-team") {
                mibz2::parallel::SLOTS_PER_THREAD * plan.workers()
            } else {
                2 * plan.workers()
            };
            drain(
                mibz2::BzReader::parallel_planned(open()?, plan, slots, mibz2::Kernels::detect()),
                h,
                last,
            )
        }
        // the pipelined team (research, `StreamDecoder::pipelined`): THREADS cores as
        // THREADS / K teams of K, as the teamK arms, and their window (SLOTS_PER_THREAD a
        // team), so a pipeK arm differs from its teamK arm in the pipeline alone
        "mibz2-pipe2" | "mibz2-pipe3" | "mibz2-pipe4" => {
            let k = (arm.as_bytes()[arm.len() - 1] - b'0') as usize;
            let teams = (threads / k).max(1);
            let slots = mibz2::parallel::SLOTS_PER_THREAD * teams;
            drain(
                mibz2::BzReader::parallel_pipelined(
                    open()?,
                    teams,
                    k,
                    slots,
                    mibz2::Kernels::detect(),
                ),
                h,
                last,
            )
        }
        // the pipelined team's levers (research, `StreamDecoder::pipelined_levers`): a
        // pipeK arm's plan and window, with lever A (park), lever B (small) or both
        "mibz2-pipe2-park" | "mibz2-pipe3-park" | "mibz2-pipe4-park" | "mibz2-pipe2-small"
        | "mibz2-pipe3-small" | "mibz2-pipe4-small" | "mibz2-pipe2-both" | "mibz2-pipe3-both"
        | "mibz2-pipe4-both" | "mibz2-pipe2-spin2k" | "mibz2-pipe3-spin2k"
        | "mibz2-pipe4-spin2k" => {
            let (base, lever) = arm.rsplit_once('-').expect("mibz2-pipeK-LEVER");
            let k = (base.as_bytes()[base.len() - 1] - b'0') as usize;
            let teams = (threads / k).max(1);
            let slots = mibz2::parallel::SLOTS_PER_THREAD * teams;
            let levers = mibz2::parallel::Levers {
                park: lever == "park" || lever == "both",
                small: lever == "small" || lever == "both",
                spin2k: lever == "spin2k",
            };
            drain(
                mibz2::BzReader::parallel_pipelined_levers(
                    open()?,
                    teams,
                    k,
                    slots,
                    mibz2::Kernels::detect(),
                    levers,
                ),
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
    let w = Threads::watch(p.id(), false);
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

fn child(arm: &str, threads: usize, path: &str, hold: Option<Hold>) {
    // (`ab child ... --hold-cpu`, for a driver that spawns the child itself: the spinner is
    // this process's thread, started first, before the sampler and the counters, and its CPU
    // taken off the decode's. `ab run --hold-cpu` holds its own and never passes the option)
    let held = hold.map(|h| {
        let decoder: Vec<usize> = cores().into_iter().flatten().collect();
        Held::for_decoder(h, &online_cores(), &decoder).unwrap_or_else(|e| {
            eprintln!("ab child: --hold-cpu: {e}");
            std::process::exit(2)
        })
    });
    // (the `walk-stats` feature only: `AB_WALK_STATS=FILE` writes the walks' records there
    // once the child is done, never during the decode)
    #[cfg(feature = "walk-stats")]
    struct WalkStats;
    #[cfg(feature = "walk-stats")]
    impl Drop for WalkStats {
        fn drop(&mut self) {
            if let Ok(path) = std::env::var("AB_WALK_STATS") {
                let lines = mibz2::walk_stats::take();
                std::fs::write(&path, lines.join("\n") + "\n").expect("AB_WALK_STATS");
            }
        }
    }
    #[cfg(feature = "walk-stats")]
    let _walk_stats = WalkStats;
    let mut h = Hash::new();
    let external = program(arm, threads, path);
    let is_program = external.is_some();
    // (`ab run --machine-state` only: the counters, opened before and read after the timed
    // region, enabled and disabled at its edges; None leaves every path as it was)
    let counted = machine::requested();
    let mut perf = None;
    // (with the counters: the region's affinity, context switches and the CPUs seen)
    let mut region: Option<machine::Region> = None;
    let (res, wall, (nthreads, per_thread, runq, sampler, sampler_cpus), startup) = match external {
        Some((cmd, ready)) => {
            if counted {
                perf = Some(machine::Perf::skipped("not counted for a program's arm"));
            }
            piped(cmd, ready, &mut h)
        }
        // (a 1-thread in-process run has one thread to account for, so no sampler at all:
        // reading another thread's /proc entries cost a 12 ms run up to 17% (#69). Its split
        // is its own thread's CPU, RUSAGE_THREAD, and its run-queue wait its own schedstat)
        None if threads == 1 => {
            perf = counted.then(machine::Perf::open);
            if let Some(p) = &perf {
                region = Some(machine::Region::start(None));
                p.enable();
            }
            let t0 = Instant::now();
            let res = decode(arm, threads, path, &mut h, &|| ());
            let wall = t0.elapsed().as_secs_f64();
            if let Some(p) = &perf {
                p.disable();
            }
            if let Some(r) = &mut region {
                r.stop();
            }
            let wait = run_delay(std::path::Path::new("/proc/thread-self/schedstat")) as f64 * 1e-9;
            let main = format!("main={:.2}", rusage(1).0);
            (res, wall, (1, main, wait, 0.0, "none".into()), 0.0)
        }
        None => {
            let w = Threads::watch(std::process::id(), counted);
            let seen_cpus = w.cpu_set();
            // (opened after the sampler starts, so it is never counted)
            perf = counted.then(machine::Perf::open);
            if let Some(p) = &perf {
                region = Some(machine::Region::start(seen_cpus));
                p.enable();
            }
            let t0 = Instant::now();
            let res = decode(arm, threads, path, &mut h, &|| w.sample());
            let wall = t0.elapsed().as_secs_f64();
            if let Some(p) = &perf {
                p.disable();
            }
            if let Some(r) = &mut region {
                r.stop();
            }
            (res, wall, w.finish(true), 0.0)
        }
    };
    // (stopped, and its CPU known, before the process's CPU is read)
    let held = held.map(|h| (h.facts(), h.stop()));
    let (cpu_self, rss_self) = rusage(0);
    let (cpu_kids, rss_kids) = rusage(-1);
    // (a program's arm: its own peak RSS, and its CPU with this process's hashing, less
    // its interpreter's start; every arm's less the sampler's own, which is ours)
    let rss = if is_program { rss_kids } else { rss_self };
    let err = res.err().unwrap_or_default().replace('"', "'");
    let perf = perf.map(machine::Perf::finish).unwrap_or_default()
        + &region.map(machine::Region::json).unwrap_or_default();
    let (held, held_s) = held.map_or((String::new(), 0.0), |(facts, s)| {
        (
            format!(",\"held_spinner\":{{{facts},\"owner\":\"child\",\"cpu_s\":{s:.4}}}"),
            s,
        )
    });
    println!(
        "{{\"wall\":{wall:.4},\"cpu\":{:.4},\"rss_kb\":{rss},\"out\":{},\"hash\":\"{}\",\"threads_busy\":{nthreads},\"thread_cpu\":\"{per_thread}\",\"runq_s\":{runq:.4},\"sampler_s\":{sampler:.4},\"sampler_cpus\":\"{sampler_cpus}\"{perf}{held}{},\"err\":\"{err}\"}}",
        cpu_self + cpu_kids - startup - sampler - held_s,
        h.n,
        h.finish(),
        err_id_json()
            + &h.sha
                .as_ref()
                .map_or(",\"sha256\":null".into(), |s| format!(",\"sha256\":\"{}\"", s.hex()))
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

/// Busy jiffies of each CPU, by its number
fn per_cpu() -> Vec<(usize, u64)> {
    let s = std::fs::read_to_string("/proc/stat").unwrap();
    s.lines()
        .filter(|l| l.starts_with("cpu") && l.as_bytes().get(3).is_some_and(u8::is_ascii_digit))
        .map(|l| {
            let mut it = l.split_whitespace();
            let cpu = it.next().unwrap()[3..].parse().unwrap();
            let f: Vec<u64> = it.map(|x| x.parse().unwrap()).collect();
            (cpu, f[0] + f[1] + f[2] + f[5] + f[6] + f[7])
        })
        .collect()
}

/// Cores the other processes keep busy, over `secs` while nothing of ours runs: in all,
/// and the busiest single CPU's share. With a held spinner (`--hold-cpu`), its own CPU over
/// the same window is taken off its CPU's share, so the known helper doesn't hold the gate
/// shut; then also the raw readings, the spinner's CPU left in
fn others_busy(secs: f64, held: Option<&Held>) -> (f64, f64, Option<(f64, f64)>) {
    let a = per_cpu();
    let h0 = held.map(Held::ticks);
    let t = Instant::now();
    std::thread::sleep(Duration::from_secs_f64(secs));
    let b = per_cpu();
    let h1 = held.map(Held::ticks);
    let dt = t.elapsed().as_secs_f64() * HZ;
    let mut each: Vec<(usize, f64)> = a
        .iter()
        .zip(&b)
        .map(|((c, x), (_, y))| (*c, y.saturating_sub(*x) as f64 / dt))
        .collect();
    let sum_max = |e: &[(usize, f64)]| {
        (
            e.iter().map(|x| x.1).sum::<f64>(),
            e.iter().map(|x| x.1).fold(0.0, f64::max),
        )
    };
    let raw = sum_max(&each);
    let Some(((h, x), y)) = held.zip(h0).zip(h1) else {
        return (raw.0, raw.1, None);
    };
    let spun = y.saturating_sub(x) as f64 / dt;
    for e in each.iter_mut().filter(|e| e.0 == h.cpu) {
        e.1 = (e.1 - spun).max(0.0);
    }
    let (busy, one) = sum_max(&each);
    (busy, one, Some(raw))
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

/// The online CPUs grouped by physical core, as `cores` groups the allowed ones: where
/// `--hold-cpu auto` looks, since the spinner may run beyond the CPUs a decoder is pinned to
fn online_cores() -> Vec<Vec<usize>> {
    let online = std::fs::read_to_string("/sys/devices/system/cpu/online")
        .map(|s| cpu_list(&s))
        .unwrap_or_default();
    if online.is_empty() {
        return cores();
    }
    group(&online, |c| {
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
/// A timestamp or null
fn opt_ns(v: &Result<u128, String>) -> String {
    v.as_ref().map_or("null".into(), |x| x.to_string())
}

fn opt(v: Option<f64>) -> String {
    v.map_or("null".into(), |x| format!("{x:.3}"))
}

thread_local! {
    /// The spin threads `env_line` has started on this thread (`facts`'s test reads it: the
    /// passive path must start none, the active one does)
    static SPIN_SPAWNS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// `ab env`: the machine as a run sees it, one JSON line. The run-queue counter is
/// checked, not assumed: twice as many threads as CPUs spin for 0.3 s, and it counts if
/// their summed wait is at least a quarter of their summed time off a CPU (most of it
/// should be wait); a counter that reads zero there is never read as no wait. Active: the
/// spin loads every CPU, and `machine`'s `perf` field opens (and closes) perf counters.
/// `ab env --facts` (`facts`) is the passive reading
fn env_line() -> String {
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
            SPIN_SPAWNS.with(|n| n.set(n.get() + 1));
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
    let line = format!(
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
    machine::extend_env(&line)
}

/// `ab env --facts` (bench 3.5.0; docs/journal/2026-11-09-run-fed-leader.md §R11): the
/// machine facts a preflight needs, read passively. Procfs, sysfs, cgroupfs and
/// `sched_getaffinity` only: no thread is started, nothing spins, and no perf counter is
/// opened (`machine`'s `perf` field is null, "not probed"). Every fact is established or
/// null with why; nothing is defaulted.
///
/// - **The CPUs allowed** (`cpus_allowed`): `sched_getaffinity` and `/proc/self/status`'s
///   `Cpus_allowed_list`, each read strictly, and they must agree; `cpus` is their count.
/// - **The cores** (`cores`, `core_cpus`): the allowed CPUs grouped by
///   `thread_siblings_list`, every one of them read (a CPU whose siblings can't be read is
///   no core of its own here: the count is null).
/// - **The CPU quota** (`cpu_quota`): of this process's effective cgroup hierarchy, as
///   `mibz2::parallel::cpu_quota` establishes it on `perf/service-guarded` (#302: Unlimited,
///   Cpus or Unknown, fail closed), ported here with cgroup v1 covered too: the hierarchy
///   that holds the `cpu` controller (a v1 one, where any v1 hierarchy names it, else the
///   unified v2 one), the process's cgroup in it, then every directory up to the mount
///   point, each level's limit parsed strictly and the tightest kept; the mount point must
///   be the hierarchy's true root (v2: no `cgroup.type`; v1: its `release_agent`, which only
///   a hierarchy's root has). Anything not so established is `unknown`, with why.
mod facts {
    use std::path::{Path, PathBuf};

    use super::machine::js;

    extern "C" {
        fn sched_getaffinity(pid: i32, size: usize, mask: *mut u64) -> i32;
    }

    /// A cgroup's CPU quota, as `cpu_quota` establishes it
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Quota {
        /// established: no limit on this process's cgroup or any ancestor, up to and
        /// including the hierarchy's true root
        Unlimited,
        /// established: the tightest limit on the way up is this many CPUs' time
        Cpus(f64),
        /// not established: the preflight refuses
        Unknown,
    }

    /// The quota with what established it: the hierarchy, every level read, or why not
    pub struct QuotaFacts {
        pub quota: Quota,
        pub hierarchy: Option<&'static str>,
        pub levels: Vec<String>,
        pub why: Option<String>,
    }

    impl QuotaFacts {
        pub fn json(&self) -> String {
            let (state, cpus) = match self.quota {
                Quota::Unlimited => ("unlimited", "null".to_string()),
                Quota::Cpus(c) => ("limited", format!("{c}")),
                Quota::Unknown => ("unknown", "null".to_string()),
            };
            format!(
                "{{\"state\":\"{state}\",\"cpus\":{cpus},\"hierarchy\":{},\"levels\":[{}],\"why\":{}}}",
                self.hierarchy.map_or("null".into(), js),
                self.levels.iter().map(|l| js(l)).collect::<Vec<_>>().join(","),
                self.why.as_deref().map_or("null".into(), js),
            )
        }
    }

    fn unknown(hierarchy: Option<&'static str>, why: impl Into<String>) -> QuotaFacts {
        QuotaFacts {
            quota: Quota::Unknown,
            hierarchy,
            levels: Vec::new(),
            why: Some(why.into()),
        }
    }

    /// The tree read from: `/` live, a fake tree in tests. Paths are absolute, as procfs
    /// and the mount table write them
    pub struct Root(PathBuf);

    impl Root {
        pub fn at(p: impl AsRef<Path>) -> Root {
            Root(p.as_ref().to_path_buf())
        }
        fn path(&self, abs: &str) -> PathBuf {
            self.0.join(abs.trim_start_matches('/'))
        }
        fn read(&self, abs: &str) -> std::io::Result<String> {
            std::fs::read_to_string(self.path(abs))
        }
        fn is_dir(&self, abs: &str) -> bool {
            self.path(abs).is_dir()
        }
        fn is_file(&self, abs: &str) -> bool {
            self.path(abs).is_file()
        }
    }

    /// A cgroup path from /proc/self/cgroup, accepted only plain and absolute
    fn plain(path: &str) -> bool {
        path.starts_with('/') && !path.contains("..") && !path.contains(' ') && !path.contains('\\')
    }

    /// This process's CPU quota, established or `Unknown` with why (Linux only)
    pub fn cpu_quota(r: &Root) -> QuotaFacts {
        if !cfg!(target_os = "linux") {
            return unknown(None, "not Linux");
        }
        let cg = match r.read("/proc/self/cgroup") {
            Ok(s) => s,
            Err(e) => return unknown(None, format!("/proc/self/cgroup: {e}")),
        };
        let mut v1 = Vec::new();
        for l in cg.lines().filter(|l| !l.is_empty()) {
            let mut f = l.splitn(3, ':');
            let (Some(_), Some(ctl), Some(path)) = (f.next(), f.next(), f.next()) else {
                return unknown(None, format!("/proc/self/cgroup: an unparsable line {l:?}"));
            };
            if ctl.split(',').any(|c| c == "cpu") {
                v1.push(path.to_string());
            }
        }
        let (hier, got) = match v1.as_slice() {
            [] => ("v2", v2(r, &cg)),
            [path] => ("v1", self::v1(r, path)),
            _ => {
                return unknown(
                    Some("v1"),
                    "several cgroup v1 entries name the cpu controller",
                )
            }
        };
        match got {
            Ok((quota, levels)) => QuotaFacts {
                quota,
                hierarchy: Some(hier),
                levels,
                why: None,
            },
            Err(why) => unknown(Some(hier), why),
        }
    }

    /// The directories from the mount point down to the process's cgroup
    fn levels(root: &str, point: &str, path: &str) -> Result<Vec<String>, String> {
        if root.contains('\\') || point.contains('\\') || !point.starts_with('/') {
            return Err(format!(
                "an escaped or relative mount ({root:?} at {point:?})"
            ));
        }
        let rel = if root == "/" {
            path
        } else {
            let r = path.strip_prefix(root).ok_or_else(|| {
                format!("the cgroup {path:?} is not under the mount's root {root:?}")
            })?;
            if !(r.is_empty() || r.starts_with('/')) {
                return Err(format!(
                    "the cgroup {path:?} is not under the mount's root {root:?}"
                ));
            }
            r
        };
        let point = point.trim_end_matches('/');
        let point = if point.is_empty() { "/" } else { point };
        let mut out = vec![point.to_string()];
        let mut at = point.trim_end_matches('/').to_string();
        for part in rel.split('/').filter(|p| !p.is_empty()) {
            at = format!("{at}/{part}");
            out.push(at.clone());
        }
        Ok(out)
    }

    /// The one mount of `fstype` whose super options satisfy `want`: (root, mount point)
    fn one_mount(
        r: &Root,
        fstype: &str,
        want: impl Fn(&str) -> bool,
    ) -> Result<(String, String), String> {
        let info = r
            .read("/proc/self/mountinfo")
            .map_err(|e| format!("/proc/self/mountinfo: {e}"))?;
        let mut found = info.lines().filter_map(|l| {
            let (pre, post) = l.split_once(" - ")?;
            let mut p = post.split_whitespace();
            let (t, _src, opts) = (p.next()?, p.next()?, p.next().unwrap_or(""));
            (t == fstype && want(opts)).then_some(pre)
        });
        let pre = found.next().ok_or_else(|| format!("no {fstype} mount"))?;
        if found.next().is_some() {
            return Err(format!("several {fstype} mounts"));
        }
        let f: Vec<&str> = pre.split_whitespace().collect();
        match (f.get(3), f.get(4)) {
            (Some(root), Some(point)) => Ok((root.to_string(), point.to_string())),
            _ => Err(format!("an unparsable {fstype} mount")),
        }
    }

    /// Positive decimal, strictly (no sign, no space)
    fn pos(t: &str) -> Option<u64> {
        (!t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
            .then(|| t.parse::<u64>().ok())
            .flatten()
            .filter(|&n| n > 0)
    }

    /// The tighter of two established quotas
    fn tighter(a: Quota, b: Quota) -> Quota {
        match (a, b) {
            (Quota::Cpus(x), Quota::Cpus(y)) => Quota::Cpus(x.min(y)),
            (Quota::Unlimited, b) => b,
            (a, _) => a,
        }
    }

    /// A v2 `cpu.max`, strictly: one line, `max PERIOD` or `QUOTA PERIOD`
    pub fn parse_cpu_max(s: &str) -> Quota {
        let line = s.strip_suffix('\n').unwrap_or(s);
        match line.split(' ').collect::<Vec<_>>().as_slice() {
            [q, p] => match (pos(p), *q) {
                (Some(_), "max") => Quota::Unlimited,
                (Some(p), q) => pos(q).map_or(Quota::Unknown, |q| Quota::Cpus(q as f64 / p as f64)),
                (None, _) => Quota::Unknown,
            },
            _ => Quota::Unknown,
        }
    }

    /// cgroup v2, as #302's `cpu_quota_of`: the one `0::` entry and the one `cgroup2` mount
    fn v2(r: &Root, cg: &str) -> Result<(Quota, Vec<String>), String> {
        use std::io::ErrorKind::NotFound;
        let mut entries = cg.lines().filter_map(|l| l.strip_prefix("0::"));
        let path = entries
            .next()
            .ok_or("no cgroup v2 entry and no v1 cpu controller")?
            .trim()
            .to_string();
        if entries.next().is_some() || !plain(&path) {
            return Err(format!("an ambiguous or escaped cgroup v2 entry {path:?}"));
        }
        let (root, point) = one_mount(r, "cgroup2", |_| true)?;
        let levels = levels(&root, &point, &path)?;
        let mut q = Quota::Unlimited;
        for dir in &levels {
            if !r.is_dir(dir) {
                return Err(format!("{dir}: not a directory"));
            }
            let f = format!("{}/cpu.max", dir.trim_end_matches('/'));
            match r.read(&f) {
                Ok(text) => match parse_cpu_max(&text) {
                    Quota::Unknown => return Err(format!("{f} reads {text:?}")),
                    b => q = tighter(q, b),
                },
                // (the cpu controller not enabled at that level: no limit there)
                Err(e) if e.kind() == NotFound => {}
                Err(e) => return Err(format!("{f}: {e}")),
            }
        }
        let top = format!("{}/cgroup.type", levels[0].trim_end_matches('/'));
        match r.read(&top) {
            Err(e) if e.kind() == NotFound => Ok((q, levels)),
            Ok(_) => Err(format!(
                "{top} exists: the mount is a cgroup below the hierarchy's root, whose ancestors can't be read"
            )),
            Err(e) => Err(format!("{top}: {e}")),
        }
    }

    /// cgroup v1: the hierarchy holding the cpu controller, its bandwidth files at every
    /// level (`cpu.cfs_quota_us`, -1 for none, and `cpu.cfs_period_us`), each required
    fn v1(r: &Root, path: &str) -> Result<(Quota, Vec<String>), String> {
        let path = path.trim();
        if !plain(path) {
            return Err(format!("an escaped cgroup v1 entry {path:?}"));
        }
        let (root, point) = one_mount(r, "cgroup", |o| o.split(',').any(|x| x == "cpu"))?;
        let levels = levels(&root, &point, path)?;
        let mut q = Quota::Unlimited;
        for dir in &levels {
            if !r.is_dir(dir) {
                return Err(format!("{dir}: not a directory"));
            }
            let d = dir.trim_end_matches('/');
            let read = |name: &str| -> Result<String, String> {
                let f = format!("{d}/{name}");
                let s = r.read(&f).map_err(|e| format!("{f}: {e}"))?;
                Ok(s.strip_suffix('\n').unwrap_or(&s).to_string())
            };
            let (quota, period) = (read("cpu.cfs_quota_us")?, read("cpu.cfs_period_us")?);
            let p =
                pos(&period).ok_or_else(|| format!("{d}/cpu.cfs_period_us reads {period:?}"))?;
            let here = if quota == "-1" {
                Quota::Unlimited
            } else {
                Quota::Cpus(
                    pos(&quota).ok_or_else(|| format!("{d}/cpu.cfs_quota_us reads {quota:?}"))?
                        as f64
                        / p as f64,
                )
            };
            q = tighter(q, here);
        }
        let top = format!("{}/release_agent", levels[0].trim_end_matches('/'));
        if r.is_file(&top) {
            Ok((q, levels))
        } else {
            Err(format!(
                "no {top}: the mount is not the hierarchy's root, whose ancestors can't be read"
            ))
        }
    }

    /// A CPU list ("0-3,6"), strictly: every part a CPU or an ascending range, at least one
    pub fn cpu_list(s: &str) -> Option<Vec<usize>> {
        let n = |t: &str| {
            (!t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
                .then(|| t.parse::<usize>().ok())
                .flatten()
        };
        let mut out = Vec::new();
        for part in s.trim().split(',') {
            match part.split_once('-') {
                Some((a, b)) => {
                    let (a, b) = (n(a)?, n(b)?);
                    if a > b {
                        return None;
                    }
                    out.extend(a..=b);
                }
                None => out.push(n(part)?),
            }
        }
        out.sort_unstable();
        let len = out.len();
        out.dedup();
        (len == out.len() && !out.is_empty()).then_some(out)
    }

    /// This process's CPU affinity mask (`sched_getaffinity`), as CPU ids
    pub fn affinity() -> Result<Vec<usize>, String> {
        let mut mask = [0u64; 64];
        // SAFETY: an out-parameter of the size given (4096 CPUs' bits)
        let rc = unsafe { sched_getaffinity(0, std::mem::size_of_val(&mask), mask.as_mut_ptr()) };
        if rc < 0 {
            return Err(format!(
                "sched_getaffinity failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let cpus: Vec<usize> = (0..mask.len() * 64)
            .filter(|&c| mask[c / 64] >> (c % 64) & 1 == 1)
            .collect();
        if cpus.is_empty() {
            return Err("sched_getaffinity: an empty mask".into());
        }
        Ok(cpus)
    }

    /// The CPUs allowed: the affinity mask and Cpus_allowed_list, which must agree
    pub fn allowed(r: &Root, mask: Result<Vec<usize>, String>) -> Result<Vec<usize>, String> {
        let mask = mask?;
        let status = r
            .read("/proc/self/status")
            .map_err(|e| format!("/proc/self/status: {e}"))?;
        let line = status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
            .ok_or("no Cpus_allowed_list in /proc/self/status")?;
        let listed =
            cpu_list(line).ok_or_else(|| format!("Cpus_allowed_list reads {:?}", line.trim()))?;
        if listed != mask {
            return Err(format!(
                "Cpus_allowed_list ({} CPUs) and sched_getaffinity ({} CPUs) disagree",
                listed.len(),
                mask.len()
            ));
        }
        Ok(listed)
    }

    /// The allowed CPUs grouped by core, every CPU's siblings read
    pub fn cores(r: &Root, allowed: &[usize]) -> Result<Vec<Vec<usize>>, String> {
        let mut sib = std::collections::BTreeMap::new();
        for &c in allowed {
            let f = format!("/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list");
            let s = r.read(&f).map_err(|e| format!("{f}: {e}"))?;
            let v = cpu_list(&s).ok_or_else(|| format!("{f} reads {:?}", s.trim()))?;
            if !v.contains(&c) {
                return Err(format!("{f} does not list cpu{c} itself"));
            }
            sib.insert(c, v);
        }
        Ok(super::group(allowed, |c| sib.get(&c).cloned()))
    }

    /// The facts line, from the tree `r` and the affinity mask given
    pub fn line(
        r: &Root,
        mask: Result<Vec<usize>, String>,
        throttling: bool,
        psi: bool,
        machine: &str,
    ) -> String {
        let ids = |v: &[usize]| v.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
        let allowed = allowed(r, mask);
        let cores = allowed
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|a| cores(r, a));
        let mut f = vec!["\"facts\":\"ab env --facts/1\"".to_string()];
        match &allowed {
            Ok(a) => {
                f.push(format!("\"cpus\":{}", a.len()));
                f.push(format!("\"cpus_allowed\":[{}]", ids(a)));
            }
            Err(why) => {
                f.push(format!(
                    "\"cpus\":null,\"cpus_allowed\":null,\"cpus_allowed_why\":{}",
                    js(why)
                ));
            }
        }
        match &cores {
            Ok(c) => {
                f.push(format!("\"cores\":{}", c.len()));
                let cc: Vec<String> = c.iter().map(|x| js(&ids(x))).collect();
                f.push(format!("\"core_cpus\":[{}]", cc.join(",")));
            }
            Err(why) => f.push(format!(
                "\"cores\":null,\"core_cpus\":null,\"cores_why\":{}",
                js(why)
            )),
        }
        f.push(format!("\"cpu_quota\":{}", cpu_quota(r).json()));
        f.push(format!("\"throttling\":{throttling},\"psi_cpu\":{psi}"));
        f.push("\"spin_threads\":0,\"perf_probed\":false".into());
        f.push(format!("\"machine\":{machine}"));
        format!("{{{}}}", f.join(","))
    }

    /// `ab env --facts`, live
    pub fn live() -> String {
        let machine = super::machine::Sysfs::live()
            .env_json("\"perf\":null,\"perf_why\":\"not probed: ab env --facts opens no counter\"");
        line(
            &Root::at("/"),
            affinity(),
            super::throttled().is_some(),
            super::psi_cpu().is_some(),
            &machine,
        )
    }
    // (the end of mod facts: its test reads the text above for what it must not call)
}

/// A top-level field of a one-line JSON record, as text. Only the record's own fields are
/// read: a nested object's field of the same name (`held_spinner`'s `cpu`, the CPU it held)
/// is skipped, as are strings.
fn json_str(line: &str, key: &str) -> Option<String> {
    let k = format!("\"{key}\":");
    let b = line.as_bytes();
    let (mut depth, mut in_str, mut esc, mut at) = (0usize, false, false, None);
    for (j, &c) in b.iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
            continue;
        }
        match c {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.saturating_sub(1),
            b'"' => {
                if depth == 1 && line[j..].starts_with(&k) {
                    at = Some(j + k.len());
                    break;
                }
                in_str = true;
            }
            _ => {}
        }
    }
    let rest = &line[at?..];
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
    let mut machine_state = false;
    let mut sysfs_state = false;
    let mut hold: Option<Hold> = None;
    let mut hold_max = usize::MAX;
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
            "--machine-state" => machine_state = true,
            "--sysfs-state" => sysfs_state = true,
            "--hold-cpu" => {
                hold = Some(Hold::parse(&args[i + 1]));
                i += 1;
            }
            "--hold-max-threads" => {
                hold_max = args[i + 1].parse().unwrap();
                i += 1;
            }
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
    let online = hold.map(|_| online_cores()).unwrap_or_default();
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
                    let cpus = if t == 1 {
                        Some(pin.clone())
                    } else {
                        spread_cpus(t)
                    };
                    let mask: Vec<usize> = match &cpus {
                        Some(list) => cpu_list(list),
                        None => cores.iter().flatten().copied().collect(),
                    };
                    // (--hold-cpu: the spinner, this process's own thread, held from before
                    // the gate until the child is reaped; never passed to the child, so an
                    // ARM@EXE build without the option runs as before)
                    let held = match hold.filter(|_| t <= hold_max) {
                        Some(h) => match Held::for_decoder(h, &online, &mask) {
                            Ok(x) => Some(x),
                            Err(e) => {
                                eprintln!("ab run: --hold-cpu: {e}");
                                std::process::exit(2);
                            }
                        },
                        None => None,
                    };
                    // (and no CPU more than half busy: a 1-thread run is pinned to one, and
                    // a pinned job of someone else's would share it. The held spinner's own
                    // CPU is taken off, its raw readings kept)
                    let (mut busy, mut one, mut raw) = others_busy(settle, held.as_ref());
                    while busy > max_busy || one > 0.5 {
                        eprintln!("  waiting: others keep {busy:.2} cores busy, one CPU {one:.2} (load {:.2})", load1());
                        std::thread::sleep(Duration::from_secs(20));
                        (busy, one, raw) = others_busy(settle, held.as_ref());
                    }
                    let (name, exe) = match arm.split_once('@') {
                        Some((n, e)) => (n.to_string(), std::path::PathBuf::from(e)),
                        None => (arm.clone(), me.clone()),
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
                    // (--machine-state: the child counts its cycles, and the clock and the
                    // package temperature are read before and after, outside the run's
                    // other readings; without it the variable is cleared, so an inherited
                    // one never turns the counters on. --sysfs-state: the parent's readings
                    // alone, the child exactly as without either flag)
                    let sysfs = (machine_state || sysfs_state).then(machine::Sysfs::live);
                    if machine_state {
                        cmd.env(machine::ENV, "1");
                    } else {
                        cmd.env_remove(machine::ENV);
                    }
                    let state_pre = sysfs.as_ref().map(|s| s.state(&mask).json());
                    let t_spawn = machine::unix_ns();
                    let load = load1();
                    let spun0 = held.as_ref().map(Held::ticks);
                    let (b0, s0, tot0) = stat();
                    let (th0, psi0) = (throttled(), psi_cpu());
                    let w0 = Instant::now();
                    // (every arm's child is watched alike from here, whatever build it is:
                    // the names of the threads it runs, read from /proc, for the plan. A
                    // child's own figures, wall, CPU and RSS among them, are its own)
                    let child = cmd
                        .stdout(std::process::Stdio::piped())
                        .stderr(std::process::Stdio::inherit())
                        .spawn()
                        .unwrap();
                    let census = Census::watch(child.id());
                    let o = child.wait_with_output().unwrap();
                    let observed = census.finish();
                    // (how it ended, the signal's number included: `termination`, bench 3.5.0)
                    let term = Term::of(o.status);
                    let span = w0.elapsed().as_secs_f64();
                    let t_exit = machine::unix_ns();
                    let (b1, s1, tot1) = stat();
                    let (th1, psi1) = (throttled(), psi_cpu());
                    // (the spinner's CPU over the run, taken off the others' busy cores)
                    let spun = spun0
                        .zip(held.as_ref().map(Held::ticks))
                        .map_or(0.0, |(a, b)| b.saturating_sub(a) as f64 / HZ);
                    let state = match (&sysfs, state_pre) {
                        (Some(s), Some(pre)) => format!(
                            "\"machine_pre\":{pre},\"t_spawn_unix_ns\":{},\"t_exit_unix_ns\":{},\"machine_post\":{},",
                            opt_ns(&t_spawn),
                            opt_ns(&t_exit),
                            s.state(&mask).json()
                        ),
                        _ => String::new(),
                    };
                    let throttle = th0.zip(th1).map(|(a, b)| (b.0 - a.0, b.1 - a.1));
                    let psi_ms = psi0.zip(psi1).map(|(a, b)| (b - a) as f64 / 1000.0);
                    let line = String::from_utf8_lossy(&o.stdout);
                    let line = line.trim();
                    // (the record's tail: the termination, then the child's own fields)
                    let fields = format!(
                        "{},{}",
                        term.json(),
                        line.strip_prefix('{').unwrap_or("\"broken\":1}")
                    );
                    let cpu = json_f(line, "cpu");
                    let others_raw = ((b1 - b0) as f64 / HZ - cpu).max(0.0) / span;
                    let others = ((b1 - b0) as f64 / HZ - cpu - spun).max(0.0) / span;
                    let held_json = held.map_or(String::new(), |h| {
                        let facts = h.facts();
                        let (raw_busy, raw_one) = raw.unwrap_or((busy, one));
                        format!(
                            "\"held_spinner\":{{{facts},\"owner\":\"run\",\"cpu_s\":{:.4},\"pre_busy_raw\":{raw_busy:.2},\"pre_one_raw\":{raw_one:.2},\"others_busy_raw\":{others_raw:.2}}},",
                            h.stop()
                        )
                    });
                    let steal = 100.0 * (s1 - s0) as f64 / (tot1 - tot0).max(1) as f64;
                    let base = std::path::Path::new(file)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let rec = format!(
                        "{{\"round\":{round},\"corpus\":\"{base}\",\"threads\":{t},\"arm\":\"{arm}\",\"parallel\":{},\"cpus\":\"{}\",\"load1\":{load:.2},\"others_busy\":{others:.2},\"pre_busy\":{busy:.2},\"steal_pct\":{steal:.1},\"throttled_n\":{},\"throttled_ms\":{},\"psi_cpu_ms\":{},\"waited_s\":{:.0},\"observed_threads\":\"{observed}\",{held_json}{state}{}",
                        parallel(&name),
                        cpus.as_deref().unwrap_or("all"),
                        opt(throttle.map(|x| x.0 as f64)),
                        opt(throttle.map(|x| x.1 as f64 / 1000.0)),
                        opt(psi_ms),
                        waited.elapsed().as_secs_f64() - settle,
                        fields
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

/// How a run's child ended (bench 3.5.0): success, a nonzero exit code, or the signal that
/// killed it. Recorded in every run's record (`termination`), so output written before a
/// failure is never taken for a run that succeeded
#[derive(Debug, Clone, Copy, PartialEq)]
enum Term {
    Success,
    Code(i32),
    Signal(i32),
    /// neither a code nor a signal (not on Unix): never success
    Unknown,
}

impl Term {
    fn of(status: std::process::ExitStatus) -> Term {
        use std::os::unix::process::ExitStatusExt;
        match (status.code(), status.signal()) {
            (Some(0), _) => Term::Success,
            (Some(c), _) => Term::Code(c),
            (None, Some(s)) => Term::Signal(s),
            (None, None) => Term::Unknown,
        }
    }
    /// The record's field: `"termination":"success"`, `"exit N"`, `"signal N"` or `"unknown"`
    fn json(self) -> String {
        let t = match self {
            Term::Success => "success".to_string(),
            Term::Code(c) => format!("exit {c}"),
            Term::Signal(s) => format!("signal {s}"),
            Term::Unknown => "unknown".to_string(),
        };
        format!("\"termination\":\"{t}\"")
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
        Some("child") => {
            let hold = match args.get(4).map(String::as_str) {
                Some("--hold-cpu") => Some(Hold::parse(args.get(5).map_or("", String::as_str))),
                Some(a) => panic!("ab child: unknown argument {a}"),
                None => None,
            };
            child(&args[1], args[2].parse().unwrap(), &args[3], hold)
        }
        Some("run") => run(&args[1..]),
        Some("summary") => summary(&args[1..]),
        Some("gen") => gen(&args[1..]),
        Some("env") => match args.get(1).map(String::as_str) {
            None => println!("{}", env_line()),
            Some("--facts") if args.len() == 2 => println!("{}", facts::live()),
            Some(a) => panic!("ab env: unknown argument {a} (ab env [--facts])"),
        },
        Some("settle") => {
            let cpus: Vec<usize> = cores().into_iter().flatten().collect();
            machine::settle(&args[1..], &cpus)
        }
        Some("sample") => {
            let cpus: Vec<usize> = cores().into_iter().flatten().collect();
            machine::sample(&args[1..], &cpus)
        }
        Some("telemetry-check") => std::process::exit(machine::check()),
        _ => eprintln!("usage: ab gen DIR [SCALE] | ab run OUT.jsonl [--arms a,b] [--threads 1,4] [--rounds N] [--settle S] [--machine-state | --sysfs-state] [--hold-cpu N|auto [--hold-max-threads T]] FILE.. | ab summary OUT.jsonl [--ref ARM] | ab child ARM T FILE [--hold-cpu N|auto] | ab env | ab settle [--seconds N] [--interval S] | ab sample [--count N] [--interval S] | ab telemetry-check"),
    }
}

/// The machine's state beside a run (bench 3.3.0): recorded, never judged. Opt-in: `ab run
/// --machine-state` reads it, and sets AB_MACHINE_STATE=1 on each child so the child counts
/// its cycles; without the flag the parent clears that variable on its children, and
/// neither the record nor any timed path changes.
///
/// - **The child's counters** (`Perf`): `perf_event_open` on this process (pid 0, any CPU),
///   inherited by every thread and process it starts after the open, each counter opened
///   disabled before the timed region, enabled just before its clock starts, disabled just
///   after it stops, and read after that: hardware cycles and instructions, and the software
///   task clock (the counted threads' CPU time over the same region). Cycles count user and
///   kernel mode where `perf_event_paranoid` allows it (at most 1, or root), else user mode
///   only (`perf_scope` says which). The sampler thread of a parallel run is started before
///   the counters open, so it is never counted. A program's arm is not counted (its process
///   is spawned inside the region, beside the sampler, and its start-up can't be separated).
///   The effective clock is cycles over the task clock (GHz), null where a counter was
///   multiplexed (counted less than the whole region).
/// - **The clock and the package temperature** (`Sysfs`), read by the parent before the run
///   starts and after it ends, outside every reading of the run: the mean `scaling_cur_freq`
///   over the run's CPUs, each cpufreq policy's governor and energy-performance preference
///   (over the policies holding the run's CPUs), the turbo state, and the package
///   temperature.
///
/// Added in bench 3.4.0, so each observation says when, over what and in which units:
/// - beside each sysfs reading, its time (`t_unix_ns`, CLOCK_REALTIME, at the reading's
///   start, and `read_s`, how long it took), each CPU's own `scaling_cur_freq` in kHz by CPU
///   id (`cpu_khz`, a policy's requested or reported frequency, which the kernel documents
///   may not be the hardware's), each policy's id, CPUs, driver, governor and EPP
///   (`policies`), and the ACPI platform profile; the run's `t_spawn_unix_ns` and
///   `t_exit_unix_ns`, so each reading's distance from the run is explicit;
/// - in the child: reference cycles (`ref_cycles`, a constant-rate count, so cycles over
///   them is the counted threads' clock against that reference), each counter's raw time
///   enabled and running (`perf_times_ns`, never scaled), its coverage (`perf_coverage`), the
///   CPUs the child may run on (`cpus_allowed`), the context switches across the region
///   (`ctxsw_voluntary`, `ctxsw_involuntary`: the whole process's, the sampler's included)
///   and the CPUs the sampler saw its threads on (`cpus_seen`: a sample, not an account);
/// - `eff_ghz` is null where cycles were counted in user mode only, since the task clock's
///   scope then isn't shown to match; and null where cycles or the task clock was
///   multiplexed. Only those two decide it: instructions' and reference cycles' validity is
///   their own (`perf_multiplexed`). Cycles over `eff_ghz` gives back the task clock: it is
///   no independent frequency correction;
/// - every failed enable or disable is recorded (`perf_ctl_errors`, with its errno), and
///   that counter's value is null with the failure as its reason;
/// - the counters are enabled, and disabled, one after another (`perf_skew_ns`: how long each
///   sequence took, and the order), so their boundaries differ by up to that much;
/// - `ab sample`: a fixed number of readings at a fixed interval, judging nothing;
/// - `ab telemetry-check`: the recorder's failure paths and coverage, untimed.
///
/// Every field is null with a `_why` beside it where it can't be read: never a guess.
mod machine {
    use std::path::{Path, PathBuf};

    /// Set on a child by `ab run --machine-state`: the child counts its cycles
    pub const ENV: &str = "AB_MACHINE_STATE";

    /// Whether this child was asked to count its cycles
    pub fn requested() -> bool {
        std::env::var(ENV).is_ok_and(|v| !v.is_empty() && v != "0")
    }

    /// A JSON string literal
    pub fn js(s: &str) -> String {
        let mut o = String::with_capacity(s.len() + 2);
        o.push('"');
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o.push('"');
        o
    }

    /// `"key":value` and, where there's no value, `"key_why":reason` beside a null
    fn field<T>(key: &str, v: &Result<T, String>, show: impl Fn(&T) -> String) -> String {
        match v {
            Ok(x) => format!("\"{key}\":{}", show(x)),
            Err(why) => format!("\"{key}\":null,\"{key}_why\":{}", js(why)),
        }
    }

    // ---------------------------------------------------------------------------------
    // sysfs

    /// The sysfs (and procfs) tree the state is read from: `/` live, a fake tree in tests
    pub struct Sysfs {
        root: PathBuf,
    }

    /// The machine's state at one moment, each reading or why there's none
    pub struct State {
        /// when the reading began (ns since the Unix epoch, CLOCK_REALTIME), and how long
        /// it took (s): a reading is of its moment, not of the run beside it
        pub t_unix_ns: Result<u128, String>,
        pub read_s: f64,
        /// each CPU's own scaling_cur_freq (kHz, as sysfs gives it), by CPU id: the policy's
        /// last requested or reported frequency, which the kernel documents may not be the
        /// hardware's actual frequency
        pub cpu_khz: Vec<(usize, Result<u64, String>)>,
        /// each cpufreq policy holding any of the CPUs, by id, with its own settings
        pub policies: Vec<Policy>,
        /// the ACPI platform profile, where the firmware has one
        pub platform_profile: Result<String, String>,
        /// the mean scaling_cur_freq (MHz) and how many of the CPUs it is over (bench 3.3.0;
        /// a mean over the allowed CPUs at one moment, not a workload's frequency)
        pub freq_mhz: Result<(f64, usize), String>,
        pub governor: Result<String, String>,
        pub epp: Result<String, String>,
        /// turbo allowed, and the file that says so
        pub turbo: Result<(bool, String), String>,
        /// °C, and where it was read
        pub pkg_temp_c: Result<(f64, String), String>,
    }

    /// One cpufreq policy's identity and settings, each read raw or why not
    pub struct Policy {
        /// its directory's name (`policy0`)
        pub id: String,
        /// affected_cpus (else related_cpus), as sysfs lists them
        pub cpus: Result<String, String>,
        pub driver: Result<String, String>,
        pub governor: Result<String, String>,
        pub epp: Result<String, String>,
    }

    impl Policy {
        fn json(&self) -> String {
            let f = [
                format!("\"policy\":{}", js(&self.id)),
                field("cpus", &self.cpus, |x| js(x)),
                field("driver", &self.driver, |x| js(x)),
                field("governor", &self.governor, |x| js(x)),
                field("epp", &self.epp, |x| js(x)),
            ];
            format!("{{{}}}", f.join(","))
        }
    }

    /// The time now, ns since the Unix epoch, or why not
    pub fn unix_ns() -> Result<u128, String> {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .map_err(|e| format!("the clock reads before the epoch: {e}"))
    }

    impl State {
        pub fn json(&self) -> String {
            let khz: Vec<String> = self
                .cpu_khz
                .iter()
                .map(|(c, v)| {
                    format!(
                        "\"{c}\":{}",
                        v.as_ref().map_or("null".into(), |x| x.to_string())
                    )
                })
                .collect();
            let khz_why: Vec<String> = self
                .cpu_khz
                .iter()
                .filter_map(|(c, v)| v.as_ref().err().map(|w| format!("\"{c}\":{}", js(w))))
                .collect();
            let policies: Vec<String> = self.policies.iter().map(Policy::json).collect();
            let mut raw = vec![
                field("t_unix_ns", &self.t_unix_ns, |x| x.to_string()),
                format!("\"read_s\":{:.6}", self.read_s),
                format!("\"cpu_khz\":{{{}}}", khz.join(",")),
            ];
            if !khz_why.is_empty() {
                raw.push(format!("\"cpu_khz_why\":{{{}}}", khz_why.join(",")));
            }
            raw.push(format!("\"policies\":[{}]", policies.join(",")));
            raw.push(field("platform_profile", &self.platform_profile, |x| js(x)));
            let mut f = vec![
                field("freq_mhz", &self.freq_mhz, |x| format!("{:.1}", x.0)),
                field("governor", &self.governor, |x| js(x)),
                field("epp", &self.epp, |x| js(x)),
                field("turbo", &self.turbo, |x| x.0.to_string()),
                field("pkg_temp_c", &self.pkg_temp_c, |x| format!("{:.1}", x.0)),
            ];
            if let Ok((_, n)) = &self.freq_mhz {
                f.insert(1, format!("\"freq_cpus\":{n}"));
            }
            if let Ok((_, src)) = &self.turbo {
                f.push(format!("\"turbo_src\":{}", js(src)));
            }
            if let Ok((_, src)) = &self.pkg_temp_c {
                f.push(format!("\"temp_src\":{}", js(src)));
            }
            f.extend(raw);
            format!("{{{}}}", f.join(","))
        }
    }

    impl Sysfs {
        pub fn live() -> Self {
            Sysfs::at("/")
        }
        pub fn at(root: impl AsRef<Path>) -> Self {
            Sysfs {
                root: root.as_ref().to_path_buf(),
            }
        }
        fn path(&self, rel: &str) -> PathBuf {
            self.root.join(rel.trim_start_matches('/'))
        }
        fn read(&self, rel: &str) -> Option<String> {
            std::fs::read_to_string(self.path(rel))
                .ok()
                .map(|s| s.trim().to_string())
        }
        /// The entries of a directory whose names start with `prefix`, in numeric order of
        /// what follows it (policy2 before policy10)
        fn numbered(&self, dir: &str, prefix: &str) -> Vec<String> {
            let mut v: Vec<(u64, String)> = std::fs::read_dir(self.path(dir))
                .map(|d| {
                    d.filter_map(|e| {
                        let n = e.ok()?.file_name().to_string_lossy().to_string();
                        Some((n.strip_prefix(prefix)?.parse().ok()?, n))
                    })
                    .collect()
                })
                .unwrap_or_default();
            v.sort();
            v.into_iter().map(|x| x.1).collect()
        }

        /// The mean scaling_cur_freq over `cpus`, in MHz, and how many it is over
        pub fn freq_mhz(&self, cpus: &[usize]) -> Result<(f64, usize), String> {
            let khz: Vec<f64> = cpus
                .iter()
                .filter_map(|c| {
                    self.read(&format!(
                        "sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq"
                    ))?
                    .parse()
                    .ok()
                })
                .collect();
            if khz.is_empty() {
                return Err(format!(
                    "no cpufreq/scaling_cur_freq for any of the {} CPUs (no cpufreq driver: a VM or container)",
                    cpus.len()
                ));
            }
            Ok((
                khz.iter().sum::<f64>() / khz.len() as f64 / 1000.0,
                khz.len(),
            ))
        }

        /// The cpufreq policies holding any of `cpus` (every policy where None)
        pub fn policies(&self, cpus: Option<&[usize]>) -> Vec<String> {
            let dir = "sys/devices/system/cpu/cpufreq";
            self.numbered(dir, "policy")
                .into_iter()
                .filter(|p| {
                    let Some(cpus) = cpus else { return true };
                    let held = self
                        .read(&format!("{dir}/{p}/affected_cpus"))
                        .or_else(|| self.read(&format!("{dir}/{p}/related_cpus")))
                        .map(|s| {
                            s.split_whitespace()
                                .flat_map(super::cpu_list)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    held.iter().any(|c| cpus.contains(c))
                })
                .collect()
        }

        /// A policy attribute over the policies holding `cpus`: its one value, or each
        /// distinct value with how many policies have it ("powersave (6), performance (2)")
        pub fn policy_attr(&self, cpus: Option<&[usize]>, attr: &str) -> Result<String, String> {
            let ps = self.policies(cpus);
            if ps.is_empty() {
                return Err("no cpufreq policies (sys/devices/system/cpu/cpufreq/policy*)".into());
            }
            let mut seen: Vec<(String, usize)> = Vec::new();
            for p in &ps {
                let v = self
                    .read(&format!("sys/devices/system/cpu/cpufreq/{p}/{attr}"))
                    .unwrap_or_else(|| "absent".into());
                match seen.iter_mut().find(|x| x.0 == v) {
                    Some(x) => x.1 += 1,
                    None => seen.push((v, 1)),
                }
            }
            if seen.len() == 1 && seen[0].0 == "absent" {
                return Err(format!("no {attr} in any of {} cpufreq policies", ps.len()));
            }
            Ok(if seen.len() == 1 {
                seen.remove(0).0
            } else {
                seen.iter()
                    .map(|(v, n)| format!("{v} ({n})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        }

        /// Whether turbo (boost) is allowed: intel_pstate's no_turbo, else cpufreq's boost
        pub fn turbo(&self) -> Result<(bool, String), String> {
            if let Some(v) = self.read("sys/devices/system/cpu/intel_pstate/no_turbo") {
                return Ok((v == "0", format!("intel_pstate/no_turbo={v}")));
            }
            if let Some(v) = self.read("sys/devices/system/cpu/cpufreq/boost") {
                return Ok((v == "1", format!("cpufreq/boost={v}")));
            }
            Err("neither intel_pstate/no_turbo nor cpufreq/boost".into())
        }

        /// The package temperature (°C; the hottest package's, where there are several):
        /// a thermal zone of type x86_pkg_temp, else hwmon's coretemp "Package id N", else
        /// k10temp's Tctl (AMD's nearest)
        pub fn pkg_temp_c(&self) -> Result<(f64, String), String> {
            let milli = |s: Option<String>| s?.parse::<f64>().ok().map(|m| m / 1000.0);
            let mut best: Option<(f64, String)> = None;
            let keep = |best: &mut Option<(f64, String)>, t: f64, src: String| {
                if best.as_ref().map_or(true, |b| t > b.0) {
                    *best = Some((t, src));
                }
            };
            let tz = "sys/class/thermal";
            for z in self.numbered(tz, "thermal_zone") {
                if self.read(&format!("{tz}/{z}/type")).as_deref() == Some("x86_pkg_temp") {
                    if let Some(t) = milli(self.read(&format!("{tz}/{z}/temp"))) {
                        keep(&mut best, t, format!("{z} (x86_pkg_temp)"));
                    }
                }
            }
            if best.is_none() {
                let hw = "sys/class/hwmon";
                for h in self.numbered(hw, "hwmon") {
                    let name = self.read(&format!("{hw}/{h}/name")).unwrap_or_default();
                    let want = |label: &str| match name.as_str() {
                        "coretemp" => label.starts_with("Package id"),
                        "k10temp" => label == "Tctl",
                        _ => false,
                    };
                    for i in 1..=64 {
                        let Some(label) = self.read(&format!("{hw}/{h}/temp{i}_label")) else {
                            continue;
                        };
                        if want(&label) {
                            if let Some(t) = milli(self.read(&format!("{hw}/{h}/temp{i}_input"))) {
                                keep(&mut best, t, format!("{h} ({name} {label})"));
                            }
                        }
                    }
                }
            }
            best.ok_or_else(|| {
                "no x86_pkg_temp thermal zone and no coretemp or k10temp hwmon".into()
            })
        }

        /// Each CPU's own scaling_cur_freq (kHz), or why not
        pub fn cpu_khz(&self, cpus: &[usize]) -> Vec<(usize, Result<u64, String>)> {
            cpus.iter()
                .map(|&c| {
                    let rel = format!("sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq");
                    let v = match self.read(&rel) {
                        Some(x) => x.parse().map_err(|_| format!("{rel} reads {x:?}")),
                        None => Err(format!("no {rel}")),
                    };
                    (c, v)
                })
                .collect()
        }

        /// Each policy holding any of `cpus`, its identity and settings read raw
        pub fn policy_list(&self, cpus: &[usize]) -> Vec<Policy> {
            let dir = "sys/devices/system/cpu/cpufreq";
            let get = |p: &str, attr: &str| -> Result<String, String> {
                self.read(&format!("{dir}/{p}/{attr}"))
                    .ok_or_else(|| format!("no {dir}/{p}/{attr}"))
            };
            self.policies(Some(cpus))
                .into_iter()
                .map(|p| Policy {
                    cpus: get(&p, "affected_cpus").or_else(|_| get(&p, "related_cpus")),
                    driver: get(&p, "scaling_driver"),
                    governor: get(&p, "scaling_governor"),
                    epp: get(&p, "energy_performance_preference"),
                    id: p,
                })
                .collect()
        }

        /// The state over the run's CPUs
        pub fn state(&self, cpus: &[usize]) -> State {
            let t0 = std::time::Instant::now();
            let t_unix_ns = unix_ns();
            let mut st = self.state_at(cpus, t_unix_ns);
            st.read_s = t0.elapsed().as_secs_f64();
            st
        }

        fn state_at(&self, cpus: &[usize], t_unix_ns: Result<u128, String>) -> State {
            State {
                t_unix_ns,
                read_s: 0.0,
                cpu_khz: self.cpu_khz(cpus),
                policies: self.policy_list(cpus),
                platform_profile: self
                    .read("sys/firmware/acpi/platform_profile")
                    .ok_or_else(|| "no sys/firmware/acpi/platform_profile".to_string()),
                freq_mhz: self.freq_mhz(cpus),
                governor: self.policy_attr(Some(cpus), "scaling_governor"),
                epp: self.policy_attr(Some(cpus), "energy_performance_preference"),
                turbo: self.turbo(),
                pkg_temp_c: self.pkg_temp_c(),
            }
        }

        /// `ab env`'s static fields, over every policy: the cpufreq driver, governor, EPP,
        /// turbo, and perf's availability (`perf`: what `Perf::open` would count here)
        pub fn env_json(&self, perf: &str) -> String {
            let driver = self
                .policies(None)
                .first()
                .and_then(|p| {
                    self.read(&format!(
                        "sys/devices/system/cpu/cpufreq/{p}/scaling_driver"
                    ))
                })
                .ok_or_else(|| "no cpufreq policy's scaling_driver".to_string());
            let paranoid = self
                .read("proc/sys/kernel/perf_event_paranoid")
                .and_then(|s| s.parse::<i64>().ok())
                .ok_or_else(|| "no /proc/sys/kernel/perf_event_paranoid".to_string());
            let f = [
                field("cpufreq_driver", &driver, |x| js(x)),
                field(
                    "governor",
                    &self.policy_attr(None, "scaling_governor"),
                    |x| js(x),
                ),
                field(
                    "epp",
                    &self.policy_attr(None, "energy_performance_preference"),
                    |x| js(x),
                ),
                field("turbo", &self.turbo(), |x| x.0.to_string()),
                field("perf_event_paranoid", &paranoid, |x| x.to_string()),
                perf.to_string(),
            ];
            format!("{{{}}}", f.join(","))
        }
    }

    /// `ab env`'s line with the machine's static fields added as `"machine":{...}`
    pub fn extend_env(line: &str) -> String {
        let fields = Sysfs::live().env_json(&Perf::probe());
        match line.trim_end().strip_suffix('}') {
            Some(head) => format!("{head},\"machine\":{fields}}}"),
            None => line.to_string(),
        }
    }

    // ---------------------------------------------------------------------------------
    // the settle check

    /// `ab settle`'s registered tolerance: the clock (the mean over the allowed CPUs) within
    /// SETTLE_FREQ_REL of its window's mean, and the package temperature within
    /// SETTLE_TEMP_C (max - min), over the last SETTLE_WINDOW samples
    pub const SETTLE_FREQ_REL: f64 = 0.03;
    pub const SETTLE_TEMP_C: f64 = 2.0;
    pub const SETTLE_WINDOW: usize = 5;

    /// Whether a window of readings is settled (None: nothing readable to judge by). A
    /// reading absent from the whole window is left out; one absent in part of it is not
    /// settled
    pub fn settled(freqs: &[Option<f64>], temps: &[Option<f64>]) -> Option<bool> {
        let steady = |v: &[Option<f64>], ok: &dyn Fn(f64, f64, f64) -> bool| -> Option<bool> {
            if v.len() < SETTLE_WINDOW || v.iter().all(Option::is_none) {
                return if v.iter().all(Option::is_none) {
                    None
                } else {
                    Some(false)
                };
            }
            let w = &v[v.len() - SETTLE_WINDOW..];
            if w.iter().any(Option::is_none) {
                return Some(false);
            }
            let w: Vec<f64> = w.iter().flatten().copied().collect();
            let (lo, hi) = w
                .iter()
                .fold((f64::MAX, f64::MIN), |a, &x| (a.0.min(x), a.1.max(x)));
            let mean = w.iter().sum::<f64>() / w.len() as f64;
            Some(ok(lo, hi, mean))
        };
        let f = steady(freqs, &|lo, hi, m| {
            (hi - m).abs().max((m - lo).abs()) <= SETTLE_FREQ_REL * m
        });
        let t = steady(temps, &|lo, hi, _| hi - lo <= SETTLE_TEMP_C);
        match (f, t) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(true) && b.unwrap_or(true)),
        }
    }

    /// `ab settle [--seconds N] [--interval S]`: untimed; samples the clock and the package
    /// temperature every S seconds (default 1) until settled or N seconds pass (default
    /// 120), and prints one JSON line. It never fails a run: a caller decides what to do
    pub fn settle(args: &[String], cpus: &[usize]) {
        let mut seconds = 120.0;
        let mut interval = 1.0;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--seconds" => seconds = args[i + 1].parse().expect("--seconds N"),
                "--interval" => interval = args[i + 1].parse().expect("--interval S"),
                a => panic!("ab settle: unknown argument {a}"),
            }
            i += 2;
        }
        let s = Sysfs::live();
        let t0 = std::time::Instant::now();
        let (mut freqs, mut temps) = (Vec::new(), Vec::new());
        let mut last = s.state(cpus);
        let verdict = loop {
            freqs.push(last.freq_mhz.as_ref().ok().map(|x| x.0));
            temps.push(last.pkg_temp_c.as_ref().ok().map(|x| x.0));
            let v = settled(&freqs, &temps);
            if v != Some(false) || t0.elapsed().as_secs_f64() + interval > seconds {
                break v;
            }
            std::thread::sleep(std::time::Duration::from_secs_f64(interval));
            last = s.state(cpus);
        };
        let why = match verdict {
            None => {
                ",\"settled_why\":\"neither the clock nor the package temperature can be read\""
            }
            Some(false) => ",\"settled_why\":\"timed out\"",
            Some(true) => "",
        };
        println!(
            "{{\"settled\":{}{why},\"waited_s\":{:.1},\"samples\":{},\"state\":{},\"tolerance\":{{\"freq_rel\":{SETTLE_FREQ_REL},\"temp_c\":{SETTLE_TEMP_C},\"window\":{SETTLE_WINDOW},\"interval_s\":{interval}}},\"timeout_s\":{seconds}}}",
            verdict.map_or("null".into(), |v| v.to_string()),
            t0.elapsed().as_secs_f64(),
            freqs.len(),
            last.json(),
        );
    }

    /// `ab sample [--count N] [--interval S]`: untimed and read-only. Exactly N readings of
    /// the state over the allowed CPUs (default 10), S seconds apart (default 1), one JSON
    /// line each, whatever they read. Unlike `settle` it judges nothing: it never stops
    /// early, never waits for a condition, and never fails, so its length is fixed before
    /// it starts (bench 3.4.0)
    pub fn sample(args: &[String], cpus: &[usize]) {
        let mut count: usize = 10;
        let mut interval = 1.0;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--count" => count = args[i + 1].parse().expect("--count N"),
                "--interval" => interval = args[i + 1].parse().expect("--interval S"),
                a => panic!("ab sample: unknown argument {a}"),
            }
            i += 2;
        }
        let s = Sysfs::live();
        let list = cpus
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",");
        for k in 0..count {
            if k > 0 {
                std::thread::sleep(std::time::Duration::from_secs_f64(interval));
            }
            println!(
                "{{\"sample\":{k},\"count\":{count},\"interval_s\":{interval},\"cpus\":\"{list}\",\"state\":{}}}",
                s.state(cpus).json()
            );
        }
    }

    /// The child's timed region beside its counters (`--machine-state` only): the CPUs it
    /// may run on, its context switches across the region, and the CPUs the sampler saw
    /// its threads on (bench 3.4.0)
    pub struct Region {
        allowed: Result<String, String>,
        ctx0: (i64, i64),
        ctx: Option<(i64, i64)>,
        cpus: Option<super::SeenCpus>,
    }

    impl Region {
        /// Just before the counters are enabled
        pub fn start(cpus: Option<super::SeenCpus>) -> Region {
            let allowed = std::fs::read_to_string("/proc/self/status")
                .map_err(|e| format!("/proc/self/status: {e}"))
                .and_then(|s| {
                    s.lines()
                        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
                        .map(|v| v.trim().to_string())
                        .ok_or_else(|| "no Cpus_allowed_list in /proc/self/status".into())
                });
            Region {
                allowed,
                ctx0: super::ctx_switches(),
                ctx: None,
                cpus,
            }
        }
        /// Just after they are disabled
        pub fn stop(&mut self) {
            let c = super::ctx_switches();
            self.ctx = Some((c.0 - self.ctx0.0, c.1 - self.ctx0.1));
        }
        /// The record's fields: a leading comma, then the allowed CPUs, the context
        /// switches (voluntary, involuntary) and the CPUs seen
        pub fn json(self) -> String {
            let ctx = self
                .ctx
                .ok_or_else(|| "the region was not stopped".to_string());
            let seen = match &self.cpus {
                Some(c) => {
                    let c = c.lock().unwrap();
                    if c.is_empty() {
                        Err("the sampler saw no thread on any CPU".to_string())
                    } else {
                        Ok(c.iter().map(u32::to_string).collect::<Vec<_>>().join(","))
                    }
                }
                None => Err("no sampler (a 1-thread run, pinned to its allowed CPU)".into()),
            };
            let f = [
                field("cpus_allowed", &self.allowed, |x| js(x)),
                field("ctxsw_voluntary", &ctx, |x| x.0.to_string()),
                field("ctxsw_involuntary", &ctx, |x| x.1.to_string()),
                field("cpus_seen", &seen, |x| js(x)),
            ];
            format!(",{}", f.join(","))
        }
    }

    // ---------------------------------------------------------------------------------
    // the child's counters

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    const SYS_PERF_EVENT_OPEN: Option<i64> = Some(298);
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    const SYS_PERF_EVENT_OPEN: Option<i64> = Some(241);
    #[cfg(not(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )))]
    const SYS_PERF_EVENT_OPEN: Option<i64> = None;

    extern "C" {
        // (libc's own signatures: the descriptors are read and closed through std's File)
        fn syscall(num: std::ffi::c_long, ...) -> std::ffi::c_long;
        fn ioctl(fd: std::ffi::c_int, request: std::ffi::c_ulong, ...) -> std::ffi::c_int;
    }

    const TYPE_HARDWARE: u64 = 0;
    const TYPE_SOFTWARE: u64 = 1;
    const HW_CPU_CYCLES: u64 = 0;
    const HW_INSTRUCTIONS: u64 = 1;
    const HW_REF_CPU_CYCLES: u64 = 9;
    const SW_TASK_CLOCK: u64 = 1;
    const IOC_ENABLE: u64 = 0x2400;
    const IOC_DISABLE: u64 = 0x2401;

    /// A descriptor as a File, which closes it when dropped
    ///
    /// # Safety
    /// `fd` is open, owned by the caller, and never used or closed again after this
    unsafe fn owned(fd: i32) -> std::fs::File {
        use std::os::fd::FromRawFd;
        unsafe { std::fs::File::from_raw_fd(fd) }
    }

    /// One counter: an open descriptor, or why it couldn't be opened
    type Counter = Result<i32, String>;

    /// The child's counters, or why there are none
    pub struct Perf {
        cycles: Counter,
        instructions: Counter,
        /// reference cycles: counted at a constant rate whatever the core's clock (bench
        /// 3.4.0); cycles over them is the counted threads' clock over that reference
        ref_cycles: Counter,
        task_clock: Counter,
        /// "user+kernel" or "user"
        scope: &'static str,
        /// each failed enable or disable: (counter, request, errno), in order (bench 3.4.0;
        /// a counter whose control failed is invalid, whatever it reads)
        ctl: std::cell::RefCell<Vec<(&'static str, &'static str, i32)>>,
        /// how long the enables, and the disables, took from the first counter's to the
        /// last's (ns): the counters' boundaries are that far apart, at most
        skew_ns: std::cell::Cell<(Option<u64>, Option<u64>)>,
    }

    /// The counters, in the order they are enabled and disabled
    const ORDER: [&str; 4] = ["task_clock", "ref_cycles", "instructions", "cycles"];

    /// One counter's reading: its raw value, time enabled and time running (ns)
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Reading {
        pub value: u64,
        pub enabled: u64,
        pub running: u64,
    }

    /// Where a counter's reading failed (D066): it couldn't be opened here (UNAVAILABLE in
    /// the check), or it was opened and then its enable, disable or read failed (INCOMPLETE)
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Stage {
        Open,
        Control,
        Read,
    }

    /// A failed reading: its stage and its reason
    #[derive(Clone, Debug, PartialEq)]
    pub struct Fault {
        pub stage: Stage,
        pub why: String,
    }

    /// A counter's reading, or where and why it failed
    pub type Got = Result<Reading, Fault>;

    fn fault(stage: Stage, why: impl Into<String>) -> Fault {
        Fault {
            stage,
            why: why.into(),
        }
    }

    impl Reading {
        /// Whether it counted the whole of the time it was enabled (not multiplexed)
        pub fn whole(&self) -> bool {
            self.enabled > 0 && self.running == self.enabled
        }
    }

    /// Cycles over the task clock (GHz), or why there's no matched ratio. Only the two
    /// counters it divides decide it: instructions' validity is its own (`perf_multiplexed`,
    /// `perf_ctl_errors`), and any use of them with cycles needs it separately. Note that
    /// this ratio isn't an independent correction: cycles over it gives back the task clock
    pub fn eff_ghz(
        cy: &Result<Reading, String>,
        tc: &Result<Reading, String>,
        user_only: bool,
    ) -> Result<f64, String> {
        match (cy, tc) {
            (Err(e), _) => Err(format!("no cycles: {e}")),
            (_, Err(e)) => Err(format!("no task clock: {e}")),
            // (the task clock's exclusion of kernel time isn't established, so a user-mode
            // cycle count over it is no matched ratio)
            _ if user_only => Err(
                "cycles counted in user mode only, and the task clock is not shown to be: no matched ratio"
                    .into(),
            ),
            (Ok(c), Ok(t)) if !c.whole() || !t.whole() => Err(format!(
                "multiplexed: cycles counted {:.1}% and the task clock {:.1}% of their enabled time",
                share(c),
                share(t)
            )),
            (Ok(c), Ok(t)) if t.value > 0 => Ok(c.value as f64 / t.value as f64),
            _ => Err("no task clock".into()),
        }
    }

    fn share(r: &Reading) -> f64 {
        if r.enabled == 0 {
            0.0
        } else {
            100.0 * r.running as f64 / r.enabled as f64
        }
    }

    /// What an errno from perf_event_open means here
    fn why(errno: i32, paranoid: &str) -> String {
        match errno {
            1 | 13 => format!("perf_event_open not permitted (perf_event_paranoid={paranoid})"),
            2 | 95 => {
                "no such event here (no hardware PMU: a VM or container without a virtual PMU)"
                    .into()
            }
            38 => "perf_event_open not implemented by this kernel".into(),
            e => format!("perf_event_open failed (errno {e})"),
        }
    }

    thread_local! {
        /// Every `perf_event_open` this thread has attempted through `open` (`facts`'s test
        /// reads it: the passive path must attempt none, the active one does)
        pub static PERF_OPENS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    fn open(kind: u64, config: u64, user_only: bool) -> Result<i32, i32> {
        PERF_OPENS.with(|n| n.set(n.get() + 1));
        let Some(nr) = SYS_PERF_EVENT_OPEN else {
            return Err(38);
        };
        // struct perf_event_attr, its first 64 bytes (PERF_ATTR_SIZE_VER0), the rest zero:
        // type and size, config, sample period and type (none), read_format (the total
        // times enabled and running, so a multiplexed count shows), then the flags:
        // disabled, inherit, exclude_kernel where user only, exclude_hv
        let mut attr = [0u64; 8];
        attr[0] = kind | (64 << 32);
        attr[1] = config;
        attr[4] = 1 | 2;
        attr[5] = 1 | 2 | if user_only { 1 << 5 } else { 0 } | 1 << 6;
        // SAFETY: the attribute is a valid perf_event_attr of the size it declares; pid 0,
        // any CPU, no group, close-on-exec (PERF_FLAG_FD_CLOEXEC)
        let fd = unsafe {
            syscall(
                nr as std::ffi::c_long,
                attr.as_ptr(),
                0i64,
                -1i64,
                -1i64,
                8u64,
            )
        };
        if fd < 0 {
            Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0))
        } else {
            Ok(fd as i32)
        }
    }

    impl Perf {
        /// The counters, opened disabled: user and kernel mode where allowed, else user only
        pub fn open() -> Perf {
            Perf::open_events(
                (TYPE_HARDWARE, HW_CPU_CYCLES),
                (TYPE_HARDWARE, HW_INSTRUCTIONS),
                (TYPE_HARDWARE, HW_REF_CPU_CYCLES),
            )
        }
        /// `open` with the events counted as cycles and instructions given (a test counts
        /// software events in their place, where there's no hardware PMU)
        fn open_events(cyc: (u64, u64), ins: (u64, u64), refc: (u64, u64)) -> Perf {
            Perf::open_mode(cyc, ins, refc, false)
        }
        /// `open_events`, in user mode only where `user_only` (the fallback, forced: `ab
        /// telemetry-check` exercises it on a machine that would count both modes)
        fn open_mode(cyc: (u64, u64), ins: (u64, u64), refc: (u64, u64), user_only: bool) -> Perf {
            let paranoid = std::fs::read_to_string("/proc/sys/kernel/perf_event_paranoid")
                .map_or("unreadable".into(), |s| s.trim().to_string());
            let first = if user_only {
                Err(13)
            } else {
                open(cyc.0, cyc.1, false)
            };
            let (cycles, scope) = match first {
                Ok(fd) => (Ok(fd), "user+kernel"),
                Err(1 | 13) => (
                    open(cyc.0, cyc.1, true).map_err(|e| why(e, &paranoid)),
                    "user",
                ),
                Err(e) => (Err(why(e, &paranoid)), "user+kernel"),
            };
            let user = scope == "user";
            let (instructions, ref_cycles, task_clock) = if cycles.is_ok() {
                (
                    open(ins.0, ins.1, user).map_err(|e| why(e, &paranoid)),
                    open(refc.0, refc.1, user).map_err(|e| why(e, &paranoid)),
                    open(TYPE_SOFTWARE, SW_TASK_CLOCK, user).map_err(|e| why(e, &paranoid)),
                )
            } else {
                let no = cycles.as_ref().err().cloned().unwrap_or_default();
                (
                    Err(format!("not opened: cycles {no}")),
                    Err(format!("not opened: cycles {no}")),
                    Err(format!("not opened: cycles {no}")),
                )
            };
            Perf {
                cycles,
                instructions,
                ref_cycles,
                task_clock,
                scope,
                ctl: Default::default(),
                skew_ns: Default::default(),
            }
        }
        /// A child that wasn't asked to count, or can't (a program's arm): every field null
        pub fn skipped(reason: &str) -> Perf {
            Perf {
                cycles: Err(reason.into()),
                instructions: Err(reason.into()),
                ref_cycles: Err(reason.into()),
                task_clock: Err(reason.into()),
                scope: "user+kernel",
                ctl: Default::default(),
                skew_ns: Default::default(),
            }
        }
        fn counter(&self, name: &str) -> &Counter {
            match name {
                "task_clock" => &self.task_clock,
                "ref_cycles" => &self.ref_cycles,
                "instructions" => &self.instructions,
                _ => &self.cycles,
            }
        }
        /// The request on every open counter, in ORDER; each failure kept, with its errno.
        /// Returns how long it took from the first ioctl's start to the last's end
        fn each(&self, req: u64) -> Option<u64> {
            let op = if req == IOC_ENABLE {
                "enable"
            } else {
                "disable"
            };
            let mut first = None;
            for name in ORDER {
                let Ok(fd) = self.counter(name) else { continue };
                first.get_or_insert_with(std::time::Instant::now);
                // SAFETY: an open perf event descriptor; the request takes no argument
                let r = unsafe { ioctl(*fd, req as std::ffi::c_ulong, 0u64) };
                if r != 0 {
                    let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                    self.ctl.borrow_mut().push((name, op, errno));
                }
            }
            first.map(|t| t.elapsed().as_nanos() as u64)
        }
        /// Just before the timed region's clock starts
        #[inline]
        pub fn enable(&self) {
            let s = self.each(IOC_ENABLE);
            self.skew_ns.set((s, self.skew_ns.get().1));
        }
        /// Just after it stops
        #[inline]
        pub fn disable(&self) {
            let s = self.each(IOC_DISABLE);
            self.skew_ns.set((self.skew_ns.get().0, s));
        }
        /// The record's fields (after `disable`), the descriptors closed: a leading comma,
        /// then cycles, instructions, the task clock (s), the scope, the least share of
        /// the region a counter ran for, and the effective clock (GHz)
        pub fn finish(self) -> String {
            self.collect().1
        }
        /// `finish`, with each counter's reading beside the record's fields: cycles,
        /// instructions, reference cycles and the task clock, in that order
        #[allow(clippy::type_complexity)]
        pub fn collect(self) -> (Vec<(&'static str, Got)>, String) {
            let ctl = self.ctl.take();
            let failed = |name: &str| -> Option<String> {
                let v: Vec<String> = ctl
                    .iter()
                    .filter(|x| x.0 == name)
                    .map(|x| format!("{} failed (errno {})", x.1, x.2))
                    .collect();
                (!v.is_empty()).then(|| v.join(", "))
            };
            let get = |name: &str| -> Got {
                let fd = *self
                    .counter(name)
                    .as_ref()
                    .map_err(|e| fault(Stage::Open, e.clone()))?;
                // SAFETY: an open perf event descriptor this Perf owns, taken once: the File
                // closes it when dropped
                let mut file = unsafe { owned(fd) };
                // (a counter whose enable or disable failed is invalid, whatever it reads)
                if let Some(why) = failed(name) {
                    return Err(fault(Stage::Control, format!("its {why}")));
                }
                // (the value, time enabled and time running: read_format 3)
                let mut raw = [0u8; 24];
                std::io::Read::read_exact(&mut file, &mut raw)
                    .map_err(|e| fault(Stage::Read, format!("reading the counter: {e}")))?;
                let w = |i: usize| u64::from_ne_bytes(raw[8 * i..8 * i + 8].try_into().unwrap());
                Ok(Reading {
                    value: w(0),
                    enabled: w(1),
                    running: w(2),
                })
            };
            let got = [
                ("cycles", get("cycles")),
                ("instructions", get("instructions")),
                ("ref_cycles", get("ref_cycles")),
                ("task_clock", get("task_clock")),
            ];
            let why = |g: &Got| g.clone().map_err(|f| f.why);
            let (cy, ins, rc, tc) = (
                why(&got[0].1),
                why(&got[1].1),
                why(&got[2].1),
                why(&got[3].1),
            );
            let all = [
                ("cycles", &cy),
                ("instructions", &ins),
                ("ref_cycles", &rc),
                ("task_clock", &tc),
            ];
            let running = all
                .iter()
                .filter_map(|r| r.1.as_ref().ok().map(|x| share(x) / 100.0))
                .fold(None, |a: Option<f64>, x| Some(a.map_or(x, |a| a.min(x))));
            let eff = eff_ghz(&cy, &tc, self.scope == "user");
            let measured = self.cycles.is_ok();
            let mut f = vec![
                field("cycles", &cy, |x| x.value.to_string()),
                field("instructions", &ins, |x| x.value.to_string()),
                field("ref_cycles", &rc, |x| x.value.to_string()),
                field("task_clock_s", &tc, |x| {
                    format!("{:.4}", x.value as f64 * 1e-9)
                }),
            ];
            if measured {
                f.push(format!("\"perf_scope\":\"{}\"", self.scope));
                // (each counter's time enabled and time running, ns, raw: never scaled; a
                // value is the raw count, never scaled up for multiplexing)
                let t = |r: &Result<Reading, String>| {
                    r.as_ref()
                        .map_or("null".into(), |x| format!("[{},{}]", x.enabled, x.running))
                };
                f.push(format!(
                    "\"perf_times_ns\":{{\"cycles\":{},\"instructions\":{},\"ref_cycles\":{},\"task_clock\":{}}}",
                    t(&cy),
                    t(&ins),
                    t(&rc),
                    t(&tc)
                ));
                let mux: Vec<String> = all
                    .iter()
                    .filter(|r| r.1.as_ref().is_ok_and(|x| !x.whole()))
                    .map(|r| js(r.0))
                    .collect();
                f.push(format!("\"perf_multiplexed\":[{}]", mux.join(",")));
                let errs: Vec<String> = ctl
                    .iter()
                    .map(|x| {
                        format!(
                            "{{\"counter\":\"{}\",\"op\":\"{}\",\"errno\":{}}}",
                            x.0, x.1, x.2
                        )
                    })
                    .collect();
                f.push(format!("\"perf_ctl_errors\":[{}]", errs.join(",")));
                let (se, sd) = self.skew_ns.get();
                let ns = |v: Option<u64>| v.map_or("null".into(), |x| x.to_string());
                f.push(format!(
                    "\"perf_skew_ns\":{{\"enable\":{},\"disable\":{},\"order\":\"{}\"}}",
                    ns(se),
                    ns(sd),
                    ORDER.join(",")
                ));
                f.push("\"perf_coverage\":\"the opening thread and every thread it starts after the counters open (inherited), each followed on any CPU; not threads started before (the sampler)\"".into());
            }
            f.push(format!(
                "\"perf_running\":{}",
                running.map_or("null".into(), |x| format!("{x:.6}"))
            ));
            f.push(field("eff_ghz", &eff, |x| format!("{x:.3}")));
            (got.into_iter().collect(), format!(",{}", f.join(",")))
        }
        /// `ab env`'s `perf` field: what the counters would count here
        pub fn probe() -> String {
            let p = Perf::open();
            let r = match &p.cycles {
                Ok(_) => format!(
                    "\"perf\":{{\"cycles\":true,\"scope\":\"{}\",\"instructions\":{},\"ref_cycles\":{},\"task_clock\":{}}}",
                    p.scope,
                    p.instructions.is_ok(),
                    p.ref_cycles.is_ok(),
                    p.task_clock.is_ok()
                ),
                Err(e) => format!("\"perf\":{{\"cycles\":false,\"why\":{}}}", js(e)),
            };
            for fd in [&p.task_clock, &p.ref_cycles, &p.instructions, &p.cycles]
                .into_iter()
                .flatten()
            {
                // SAFETY: an open descriptor this Perf owns, closed once (by the File's drop)
                drop(unsafe { owned(*fd) });
            }
            r
        }
    }

    // ---------------------------------------------------------------------------------
    // the recorder's own check (bench 3.4.0; its specification:
    // docs/journal/2026-11-09-telemetry-check-registration.md)

    /// This thread's own (user, system) CPU seconds, RUSAGE_THREAD, or why not: a failed
    /// call is an error, never a zero
    pub fn thread_times() -> Result<(f64, f64), String> {
        let mut r = crate::Rusage::default();
        // SAFETY: a plain out-parameter of the size the kernel writes (struct rusage, LP64)
        let rc = unsafe { crate::getrusage(1, &mut r) };
        if rc != 0 {
            return Err(format!(
                "getrusage(RUSAGE_THREAD) failed (errno {})",
                std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
            ));
        }
        let t = |tv: [i64; 2]| tv[0] as f64 + tv[1] as f64 * 1e-6;
        Ok((t(r.utime), t(r.stime)))
    }

    /// The wall-time bound on each spin: whatever its CPU budget, a spin gives up after this
    /// long, and its case is INCOMPLETE (a machine too busy to give it the CPU, or failed
    /// accounting, never a pass)
    pub const SPIN_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

    /// Spin for `ms` of this thread's own CPU time, checked, within SPIN_DEADLINE of wall
    /// time
    pub fn spin(ms: u64) -> Result<(), String> {
        let deadline = std::time::Instant::now() + SPIN_DEADLINE;
        let cpu = || thread_times().map(|t| t.0 + t.1);
        let t0 = cpu()?;
        let mut x = 0u64;
        while cpu()? - t0 < ms as f64 * 1e-3 {
            if std::time::Instant::now() > deadline {
                return Err(format!(
                    "{ms} ms of CPU not reached within {} s of wall time",
                    SPIN_DEADLINE.as_secs()
                ));
            }
            for _ in 0..10_000 {
                x = std::hint::black_box(x.wrapping_add(1));
            }
        }
        Ok(())
    }

    const SW_CPU_CLOCK: u64 = 0;
    const SW: (u64, u64) = (TYPE_SOFTWARE, SW_TASK_CLOCK);

    /// One case's line: its name, its status, what it saw, and its fields. The statuses:
    /// PASS (the property checked holds), FAIL (it doesn't: the recorder is wrong here),
    /// OBSERVED (a behaviour shown, with nothing it is checked against), UNAVAILABLE (the
    /// events it needs can't be opened here, with why) and INCOMPLETE (it started but
    /// couldn't finish validly: a failed control or read, multiplexing, a deadline)
    fn case(name: &str, status: &str, detail: &str, fields: &str) -> String {
        format!(
            "{{\"case\":{},\"status\":\"{status}\",\"detail\":{},\"fields\":{{{}}}}}",
            js(name),
            js(detail),
            fields.trim_start_matches(',')
        )
    }

    /// A reading's JSON: raw value, enabled and running, or null with its reason
    fn reading_json(name: &str, r: &Got) -> String {
        match r {
            Ok(x) => format!(
                "\"{name}\":{{\"value\":{},\"enabled_ns\":{},\"running_ns\":{}}}",
                x.value, x.enabled, x.running
            ),
            Err(e) => format!(
                "\"{name}\":null,\"{name}_why\":{},\"{name}_stage\":\"{:?}\"",
                js(&e.why),
                e.stage
            ),
        }
    }

    /// The case statuses' exit code: 1 if any FAILs, else 3 if any is INCOMPLETE, else 0
    pub fn exit_code(statuses: &[&str]) -> i32 {
        if statuses.contains(&"FAIL") {
            1
        } else if statuses.contains(&"INCOMPLETE") {
            3
        } else {
            0
        }
    }

    /// The per-counter inheritance witness: counts with the work in a thread started inside
    /// the region (`inside`) against the same work in a thread started before the counters
    /// open (`before`), with the opener idle in both. A counter passes only if both readings
    /// are valid and whole, and `inside` is at least ten times `before` (and nonzero):
    /// removing that counter's inheritance makes `inside` the opener's idle count, and fails
    pub fn inherit_status(inside: &Got, before: &Got) -> (&'static str, String) {
        // (both regions' faults first: a control or read failure after a successful open is
        // INCOMPLETE, whichever region it is in and whatever the other shows; only then an
        // event that couldn't be opened is UNAVAILABLE)
        let faults: Vec<(&str, &Fault)> = [("inside", inside), ("before", before)]
            .into_iter()
            .filter_map(|(k, r)| r.as_ref().err().map(|f| (k, f)))
            .collect();
        let say = |fs: &[(&str, &Fault)]| {
            fs.iter()
                .map(|(k, f)| format!("{k}: {:?}: {}", f.stage, f.why))
                .collect::<Vec<_>>()
                .join("; ")
        };
        if faults.iter().any(|(_, f)| f.stage != Stage::Open) {
            return ("INCOMPLETE", say(&faults));
        }
        if !faults.is_empty() {
            return ("UNAVAILABLE", say(&faults));
        }
        let (Ok(a), Ok(b)) = (inside, before) else {
            unreachable!("no faults, so both readings are valid")
        };
        if !a.whole() || !b.whole() {
            (
                "INCOMPLETE",
                "multiplexed: the counts don't cover their enabled time".into(),
            )
        } else if a.value > 0 && a.value >= 10 * b.value.max(1) {
            (
                "PASS",
                format!(
                    "{} counted with the work in a thread started in the region, {} with it in a thread started before",
                    a.value, b.value
                ),
            )
        } else {
            (
                "FAIL",
                format!(
                    "{} counted with the work in a thread started in the region, {} with it in a thread started before: no inheritance shown",
                    a.value, b.value
                ),
            )
        }
    }

    /// The live multiplexing case's status, from the record's readings and its fields
    /// (D066): any control or read failure on an opened counter is INCOMPLETE, before
    /// multiplexing is judged; a counter that couldn't be opened is left out. Then: no
    /// multiplexed counter is OBSERVED ("not exercised"), never a pass; a multiplexed
    /// counter is PASS only if the record lists it and eff_ghz follows its rule
    pub fn multiplex_status(rs: &[(&'static str, Got)], f: &str) -> (&'static str, String) {
        let broken: Vec<String> = rs
            .iter()
            .filter_map(|(n, r)| match r {
                Err(x) if x.stage != Stage::Open => Some(format!("{n}: {:?}: {}", x.stage, x.why)),
                _ => None,
            })
            .collect();
        if !broken.is_empty() {
            return (
                "INCOMPLETE",
                format!("a control or read failed: {}", broken.join("; ")),
            );
        }
        let partial: Vec<&str> = rs
            .iter()
            .filter(|r| r.1.as_ref().is_ok_and(|x| !x.whole()))
            .map(|r| r.0)
            .collect();
        if partial.is_empty() {
            return (
                "OBSERVED",
                "multiplexing not exercised: every valid counter ran its whole enabled time".into(),
            );
        }
        let mux = f
            .split_once("\"perf_multiplexed\":[")
            .and_then(|x| x.1.split_once(']'))
            .map_or("", |x| x.0);
        let listed = partial.iter().all(|n| mux.contains(&format!("\"{n}\"")));
        let divides = partial.iter().any(|n| *n == "cycles" || *n == "task_clock");
        if listed && (!divides || f.contains("\"eff_ghz\":null")) {
            (
                "PASS",
                format!(
                    "multiplexed: {}; the record lists them and eff_ghz follows",
                    partial.join(", ")
                ),
            )
        } else {
            (
                "FAIL",
                format!(
                    "multiplexed: {}; the record doesn't say so",
                    partial.join(", ")
                ),
            )
        }
    }

    /// One inheritance region: `before` true runs the work (80 ms of CPU) in a thread
    /// started before the counters open, else in one started inside the region; the opener
    /// only waits. The readings, or why the region couldn't be run
    #[allow(clippy::type_complexity)]
    fn inherit_region(
        open: &dyn Fn() -> Perf,
        before: bool,
    ) -> Result<Vec<(&'static str, Got)>, String> {
        let (go, wait) = std::sync::mpsc::channel::<()>();
        let early = before.then(|| {
            std::thread::spawn(move || {
                wait.recv().map_err(|_| "never started".to_string())?;
                spin(80)
            })
        });
        let p = open();
        if let Err(e) = p.cycles.clone() {
            drop(go);
            if let Some(h) = early {
                let _ = h.join();
            }
            return Ok(p
                .collect()
                .0
                .into_iter()
                .map(|(n, _)| (n, Err(fault(Stage::Open, e.clone()))))
                .collect());
        }
        p.enable();
        let r = match early {
            Some(h) => {
                let _ = go.send(());
                h.join()
            }
            None => std::thread::spawn(|| spin(80)).join(),
        };
        p.disable();
        let readings = p.collect().0;
        r.map_err(|_| "the worker panicked".to_string())??;
        Ok(readings)
    }

    /// `ab telemetry-check`: one JSON line a case (statuses at `case`), then exit 1 if any
    /// case FAILs, else 3 if any is INCOMPLETE, else 0. Every case is bounded: each spin by
    /// SPIN_DEADLINE, the whole check by the recipe's outer timeout
    pub fn check() -> i32 {
        let mut statuses: Vec<&'static str> = Vec::new();
        let mut out = |status: &'static str, line: String| {
            statuses.push(status);
            println!("{line}");
        };

        // control-failure: a descriptor that isn't a perf event's, so its enable and disable
        // fail; that counter must be invalid, the rest valid
        {
            let mut p = Perf::open_events(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW);
            if let Err(e) = &p.cycles {
                let e = e.clone();
                drop(p.finish());
                out(
                    "UNAVAILABLE",
                    case("control-failure", "UNAVAILABLE", &e, ""),
                );
            } else {
                use std::os::fd::IntoRawFd;
                if let Ok(fd) = &p.instructions {
                    // SAFETY: an open descriptor this Perf owns, replaced below, closed once
                    drop(unsafe { owned(*fd) });
                }
                p.instructions = std::fs::File::open("/dev/null")
                    .map(|f| f.into_raw_fd())
                    .map_err(|e| e.to_string());
                p.enable();
                let s = spin(5);
                p.disable();
                let f = p.finish();
                let ok = f.contains(
                    "\"instructions\":null,\"instructions_why\":\"its enable failed (errno",
                ) && f.contains("disable failed (errno")
                    && f.contains("{\"counter\":\"instructions\",\"op\":\"enable\"")
                    && f.contains("{\"counter\":\"instructions\",\"op\":\"disable\"")
                    && !f.contains("\"cycles\":null");
                let (st, d) = match s {
                    Err(e) => ("INCOMPLETE", e),
                    Ok(()) if ok => (
                        "PASS",
                        "a failed enable and disable are recorded, and only that counter is invalid"
                            .into(),
                    ),
                    Ok(()) => (
                        "FAIL",
                        "a failed enable or disable was not recorded as such".into(),
                    ),
                };
                out(st, case("control-failure", st, &d, &f));
            }
        }

        // missing-counter
        {
            let p = Perf::open_events(
                (TYPE_HARDWARE, 0xfff0),
                (TYPE_HARDWARE, 0xfff1),
                (TYPE_HARDWARE, 0xfff2),
            );
            p.enable();
            p.disable();
            let f = p.finish();
            let ok = [
                "cycles",
                "instructions",
                "ref_cycles",
                "task_clock_s",
                "eff_ghz",
            ]
            .iter()
            .all(|k| f.contains(&format!("\"{k}\":null,\"{k}_why\":\"")))
                && f.contains("\"perf_running\":null")
                && !f.contains("perf_scope");
            let st = if ok { "PASS" } else { "FAIL" };
            out(
                st,
                case(
                    "missing-counter",
                    st,
                    "an event that can't be opened: every field null with its reason, never zero",
                    &f,
                ),
            );
        }

        // multiplex-predicate
        {
            let r = |value, running| Reading {
                value,
                enabled: 1_000,
                running,
            };
            let a = eff_ghz(&Ok(r(2_000, 500)), &Ok(r(1_000, 1_000)), false);
            let b = eff_ghz(&Ok(r(4_000, 1_000)), &Ok(r(1_000, 500)), false);
            let c = eff_ghz(&Ok(r(4_000, 1_000)), &Ok(r(1_000, 1_000)), false);
            let d = eff_ghz(&Ok(r(4_000, 1_000)), &Ok(r(1_000, 1_000)), true);
            let ok = a.as_ref().is_err_and(|e| e.starts_with("multiplexed"))
                && b.as_ref().is_err_and(|e| e.starts_with("multiplexed"))
                && c == Ok(4.0)
                && d.as_ref().is_err_and(|e| e.contains("user mode only"));
            let st = if ok { "PASS" } else { "FAIL" };
            let q = |x: &Result<f64, String>| js(&format!("{x:?}"));
            out(
                st,
                case(
                    "multiplex-predicate",
                    st,
                    "eff_ghz: null for multiplexed cycles or task clock, and in user mode only",
                    &format!(
                        "\"a\":{},\"b\":{},\"c\":{},\"d\":{}",
                        q(&a),
                        q(&b),
                        q(&c),
                        q(&d)
                    ),
                ),
            );
        }

        // multiplex-live: 24 more cycle counters, each opened and enabled with its result
        // checked; whether the record's counters were in fact multiplexed, and if so that
        // the record says so
        {
            let p = Perf::open();
            if let Err(e) = &p.cycles {
                let e = format!("no hardware cycles: {e}");
                drop(p.finish());
                out("UNAVAILABLE", case("multiplex-live", "UNAVAILABLE", &e, ""));
            } else {
                let user = p.scope == "user";
                let attempted = 24;
                let opened: Vec<i32> = (0..attempted)
                    .filter_map(|_| open(TYPE_HARDWARE, HW_CPU_CYCLES, user).ok())
                    .collect();
                let enabled = opened
                    .iter()
                    // SAFETY: an open perf event descriptor; the request takes no argument
                    .filter(|fd| unsafe { ioctl(**fd, IOC_ENABLE as std::ffi::c_ulong, 0u64) } == 0)
                    .count();
                p.enable();
                let s = spin(50);
                p.disable();
                for fd in &opened {
                    // SAFETY: an open descriptor owned here, closed once
                    drop(unsafe { owned(*fd) });
                }
                let (rs, f) = p.collect();
                let counts = format!(
                    "{attempted} extra cycle counters attempted, {} opened, {enabled} enabled",
                    opened.len()
                );
                let (st, d) = match s {
                    Err(e) => ("INCOMPLETE", format!("{counts}; {e}")),
                    Ok(()) => {
                        let (st, d) = multiplex_status(&rs, &f);
                        (st, format!("{counts}; {d}"))
                    }
                };
                out(st, case("multiplex-live", st, &d, &f));
            }
        }

        // user-only-fallback: the fallback forced; no matched ratio may be claimed
        {
            let p = Perf::open_mode(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW, true);
            if let Err(e) = &p.cycles {
                let e = e.clone();
                drop(p.finish());
                out(
                    "UNAVAILABLE",
                    case("user-only-fallback", "UNAVAILABLE", &e, ""),
                );
            } else {
                p.enable();
                let s = spin(5);
                p.disable();
                let f = p.finish();
                let ok = f.contains("\"perf_scope\":\"user\"")
                    && f.contains(
                        "\"eff_ghz\":null,\"eff_ghz_why\":\"cycles counted in user mode only",
                    );
                let (st, d) = match s {
                    Err(e) => ("INCOMPLETE", e),
                    Ok(()) if ok => (
                        "PASS",
                        "the fallback forced: scope user, no matched ratio claimed".into(),
                    ),
                    Ok(()) => (
                        "FAIL",
                        "the forced fallback claimed a ratio or lost its scope".into(),
                    ),
                };
                out(st, case("user-only-fallback", st, &d, &f));
            }
        }

        // task-clock-modes: a task clock in user mode only beside one in both modes, over
        // syscall-heavy work, every control and read checked, with this thread's own user
        // and system time over the same work. A ratio only from two valid, whole readings
        {
            let pu = Perf::open_mode(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW, true);
            let pa = Perf::open_mode(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW, false);
            if pu.cycles.is_err() || pa.cycles.is_err() || pa.scope != "user+kernel" {
                let why = match (&pu.cycles, &pa.cycles) {
                    (Err(e), _) | (_, Err(e)) => e.clone(),
                    _ => "a both-mode task clock is not permitted here".into(),
                };
                drop(pu.finish());
                drop(pa.finish());
                out(
                    "UNAVAILABLE",
                    case("task-clock-modes", "UNAVAILABLE", &why, ""),
                );
            } else {
                let t0 = thread_times();
                pu.enable();
                pa.enable();
                let deadline = std::time::Instant::now() + SPIN_DEADLINE;
                let mut work: Result<(), String> = Ok(());
                match std::fs::File::open("/dev/zero") {
                    Err(e) => work = Err(format!("/dev/zero: {e}")),
                    Ok(mut z) => {
                        let mut b = [0u8; 64];
                        loop {
                            match (thread_times(), t0.clone()) {
                                (Ok(t), Ok(s)) if (t.0 + t.1) - (s.0 + s.1) >= 0.05 => break,
                                (Err(e), _) | (_, Err(e)) => {
                                    work = Err(e.clone());
                                    break;
                                }
                                _ => {}
                            }
                            if std::time::Instant::now() > deadline {
                                work = Err("50 ms of CPU not reached within the deadline".into());
                                break;
                            }
                            if let Err(e) = std::io::Read::read(&mut z, &mut b) {
                                work = Err(format!("reading /dev/zero: {e}"));
                                break;
                            }
                        }
                    }
                }
                pa.disable();
                pu.disable();
                let t1 = thread_times();
                let (ru, fu) = pu.collect();
                let (ra, fa) = pa.collect();
                let (u, a) = (ru[0].1.clone(), ra[0].1.clone());
                let times = match (&t0, &t1) {
                    (Ok(s), Ok(e)) => Ok((e.0 - s.0, e.1 - s.1)),
                    (Err(x), _) | (_, Err(x)) => Err(x.clone()),
                };
                let fields = format!(
                    "{},{},{},\"user_only_record\":{{{}}},\"both_record\":{{{}}}",
                    reading_json("user_only", &u),
                    reading_json("both", &a),
                    match &times {
                        Ok((us, ss)) =>
                            format!("\"thread_user_s\":{us:.4},\"thread_system_s\":{ss:.4}"),
                        Err(e) => format!("\"thread_user_s\":null,\"thread_times_why\":{}", js(e)),
                    },
                    fu.trim_start_matches(','),
                    fa.trim_start_matches(',')
                );
                let (st, d) = match (&work, &u, &a, &times) {
                    (Err(e), ..) => ("INCOMPLETE", e.clone()),
                    (_, Err(e), _, _) | (_, _, Err(e), _) => {
                        ("INCOMPLETE", format!("{:?}: {}", e.stage, e.why))
                    }
                    (_, _, _, Err(e)) => ("INCOMPLETE", e.clone()),
                    // (both nonzero, as registered: over 50 ms of this thread's CPU, a zero
                    // either way is no reading of the flag)
                    (_, Ok(x), Ok(y), _)
                        if !x.whole() || !y.whole() || x.value == 0 || y.value == 0 =>
                    {
                        (
                            "INCOMPLETE",
                            "a reading was multiplexed or zero: no ratio".into(),
                        )
                    }
                    (_, Ok(x), Ok(y), Ok((us, ss))) => {
                        let r = x.value as f64 / y.value as f64;
                        let sys = ss / (us + ss).max(1e-9);
                        let reading = if sys < 0.2 {
                            "too little kernel time to say anything of the user-only flag"
                        } else if r > 0.9 {
                            "consistent with this kernel's task clock not excluding kernel time"
                        } else {
                            "consistent with the user-only task clock excluding kernel time"
                        };
                        (
                            "OBSERVED",
                            format!("the user-only task clock read {r:.3} of the both-mode one, over work {:.0}% in system time: {reading}", 100.0 * sys),
                        )
                    }
                };
                out(st, case("task-clock-modes", st, &d, &fields));
            }
        }

        // region-task-clock: the task clock (software) counts 80 ms in a thread started in
        // the region and 20 ms in the opener, none of 120 ms outside the region, none of a
        // thread started before the counters open. The task clock only: the hardware
        // counters' region coverage is not validated here
        {
            let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let (go, wait) = std::sync::mpsc::channel::<()>();
            let s2 = stop.clone();
            let early = std::thread::spawn(move || -> Result<(), String> {
                if wait.recv().is_err() {
                    return Ok(());
                }
                while !s2.load(std::sync::atomic::Ordering::Relaxed) {
                    spin(5)?;
                }
                Ok(())
            });
            let p = Perf::open_events(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW);
            if let Err(e) = &p.cycles {
                let e = e.clone();
                drop(go);
                let _ = early.join();
                drop(p.finish());
                out(
                    "UNAVAILABLE",
                    case("region-task-clock", "UNAVAILABLE", &e, ""),
                );
            } else {
                let mut s = spin(60);
                p.enable();
                let _ = go.send(());
                let inner = std::thread::spawn(|| spin(80)).join();
                let own = spin(20);
                p.disable();
                stop.store(true, std::sync::atomic::Ordering::Relaxed);
                let bg = early.join();
                let after = spin(60);
                for r in [
                    inner.unwrap_or_else(|_| Err("panicked".into())),
                    own,
                    bg.unwrap_or_else(|_| Err("panicked".into())),
                    after,
                ] {
                    if s.is_ok() {
                        s = r;
                    }
                }
                let (rs, f) = p.collect();
                let tc = rs[3].1.clone();
                let (st, d) = match (&s, &tc) {
                    (Err(e), _) => ("INCOMPLETE", e.clone()),
                    (_, Err(e)) => ("INCOMPLETE", format!("{:?}: {}", e.stage, e.why)),
                    (_, Ok(x)) if !x.whole() => ("INCOMPLETE", "multiplexed".into()),
                    (_, Ok(x)) => {
                        let v = x.value as f64 * 1e-9;
                        let st = if (0.090..0.130).contains(&v) {
                            "PASS"
                        } else {
                            "FAIL"
                        };
                        (st, format!("the task clock read {v:.4} s: 0.100 expected (80 ms in a thread started in the region, 20 ms in the opener), none of the 120 ms outside it, none of the thread started before. The task clock only: the hardware counters' region coverage is not validated here"))
                    }
                };
                out(st, case("region-task-clock", st, &d, &f));
            }
        }

        // inheritance, each counter on its own: software events, then the real counters
        for (mode, hw) in [("software", false), ("hardware", true)] {
            let open = move || {
                if hw {
                    Perf::open()
                } else {
                    Perf::open_events(SW, (TYPE_SOFTWARE, SW_CPU_CLOCK), SW)
                }
            };
            let (inside, before) = (inherit_region(&open, false), inherit_region(&open, true));
            match (inside, before) {
                (Err(e), _) | (_, Err(e)) => {
                    out(
                        "INCOMPLETE",
                        case(&format!("inheritance-{mode}"), "INCOMPLETE", &e, ""),
                    );
                }
                (Ok(a), Ok(b)) => {
                    for ((name, ra), (_, rb)) in a.iter().zip(b.iter()) {
                        let (st, d) = inherit_status(ra, rb);
                        let what = if hw {
                            (*name).to_string()
                        } else {
                            format!("{name} (a software stand-in)")
                        };
                        out(
                            st,
                            case(
                                &format!("inheritance-{mode}-{name}"),
                                st,
                                &format!("{what}: {d}"),
                                &format!(
                                    "{},{}",
                                    reading_json("inside", ra),
                                    reading_json("before", rb)
                                ),
                            ),
                        );
                    }
                }
            }
        }
        exit_code(&statuses)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// A fake tree under the test's own directory, removed when dropped
        struct Tree(PathBuf);
        impl Tree {
            fn new(name: &str) -> Tree {
                let p =
                    std::env::temp_dir().join(format!("ab-machine-{}-{name}", std::process::id()));
                let _ = std::fs::remove_dir_all(&p);
                std::fs::create_dir_all(&p).unwrap();
                Tree(p)
            }
            fn put(&self, rel: &str, v: &str) -> &Self {
                let p = self.0.join(rel);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, format!("{v}\n")).unwrap();
                self
            }
        }
        impl Drop for Tree {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        fn intel(t: &Tree) {
            for c in 0..4 {
                t.put(
                    &format!("sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq"),
                    &format!("{}", 4_000_000 + 100_000 * c),
                );
                let p = format!("sys/devices/system/cpu/cpufreq/policy{c}");
                t.put(&format!("{p}/affected_cpus"), &c.to_string())
                    .put(
                        &format!("{p}/scaling_governor"),
                        if c < 3 { "powersave" } else { "performance" },
                    )
                    .put(
                        &format!("{p}/energy_performance_preference"),
                        "balance_performance",
                    )
                    .put(&format!("{p}/scaling_driver"), "intel_pstate");
            }
            t.put("sys/devices/system/cpu/intel_pstate/no_turbo", "0")
                .put("sys/class/thermal/thermal_zone0/type", "acpitz")
                .put("sys/class/thermal/thermal_zone0/temp", "99000")
                .put("sys/class/thermal/thermal_zone1/type", "x86_pkg_temp")
                .put("sys/class/thermal/thermal_zone1/temp", "71000")
                .put("proc/sys/kernel/perf_event_paranoid", "2")
                .put("sys/firmware/acpi/platform_profile", "balanced");
        }

        #[test]
        fn reads_an_intel_tree() {
            let t = Tree::new("intel");
            intel(&t);
            let s = Sysfs::at(&t.0);
            let (mhz, n) = s.freq_mhz(&[1, 2]).unwrap();
            assert_eq!((mhz, n), (4150.0, 2));
            assert_eq!(s.policies(Some(&[1, 2])), vec!["policy1", "policy2"]);
            assert_eq!(
                s.policy_attr(Some(&[0, 1]), "scaling_governor").unwrap(),
                "powersave"
            );
            assert_eq!(
                s.policy_attr(None, "scaling_governor").unwrap(),
                "powersave (3), performance (1)"
            );
            assert_eq!(
                s.policy_attr(Some(&[3]), "energy_performance_preference")
                    .unwrap(),
                "balance_performance"
            );
            assert_eq!(s.turbo().unwrap(), (true, "intel_pstate/no_turbo=0".into()));
            // the package zone, never another (acpitz)
            assert_eq!(
                s.pkg_temp_c().unwrap(),
                (71.0, "thermal_zone1 (x86_pkg_temp)".into())
            );
            let j = s.state(&[0, 1, 2, 3]).json();
            assert!(j.contains("\"freq_mhz\":4150.0,\"freq_cpus\":4"), "{j}");
            assert!(
                j.contains("\"turbo\":true") && j.contains("\"pkg_temp_c\":71.0"),
                "{j}"
            );
            assert!(!j.contains("_why"), "{j}");
            // the raw readings beside the mean: each CPU's own, each policy's own
            assert!(
                j.contains("\"cpu_khz\":{\"0\":4000000,\"1\":4100000,\"2\":4200000,\"3\":4300000}"),
                "{j}"
            );
            assert!(
                j.contains("{\"policy\":\"policy3\",\"cpus\":\"3\",\"driver\":\"intel_pstate\",\"governor\":\"performance\",\"epp\":\"balance_performance\"}"),
                "{j}"
            );
            assert!(j.contains("\"platform_profile\":\"balanced\""), "{j}");
            assert!(
                j.contains("\"t_unix_ns\":1") && j.contains("\"read_s\":"),
                "{j}"
            );
            // only the policies holding the CPUs asked for
            let j1 = s.state(&[1]).json();
            assert!(j1.contains("\"policies\":[{\"policy\":\"policy1\""), "{j1}");
            assert!(!j1.contains("policy2"), "{j1}");
            let e = s.env_json("\"perf\":{}");
            assert!(e.contains("\"cpufreq_driver\":\"intel_pstate\""), "{e}");
            assert!(e.contains("\"perf_event_paranoid\":2"), "{e}");
        }

        #[test]
        fn reads_hwmon_and_boost() {
            let t = Tree::new("amd");
            t.put("sys/devices/system/cpu/cpufreq/boost", "0")
                .put("sys/class/hwmon/hwmon0/name", "nvme")
                .put("sys/class/hwmon/hwmon0/temp1_label", "Composite")
                .put("sys/class/hwmon/hwmon0/temp1_input", "40000")
                .put("sys/class/hwmon/hwmon2/name", "coretemp")
                .put("sys/class/hwmon/hwmon2/temp1_label", "Package id 0")
                .put("sys/class/hwmon/hwmon2/temp1_input", "64000")
                .put("sys/class/hwmon/hwmon2/temp2_label", "Core 0")
                .put("sys/class/hwmon/hwmon2/temp2_input", "90000")
                .put("sys/class/hwmon/hwmon10/name", "coretemp")
                .put("sys/class/hwmon/hwmon10/temp1_label", "Package id 1")
                .put("sys/class/hwmon/hwmon10/temp1_input", "66500");
            let s = Sysfs::at(&t.0);
            assert_eq!(s.turbo().unwrap(), (false, "cpufreq/boost=0".into()));
            // the hotter package, never a core's reading
            assert_eq!(
                s.pkg_temp_c().unwrap(),
                (66.5, "hwmon10 (coretemp Package id 1)".into())
            );
            let k = Tree::new("k10");
            k.put("sys/class/hwmon/hwmon0/name", "k10temp")
                .put("sys/class/hwmon/hwmon0/temp1_label", "Tctl")
                .put("sys/class/hwmon/hwmon0/temp1_input", "55250");
            assert_eq!(Sysfs::at(&k.0).pkg_temp_c().unwrap().0, 55.25);
        }

        #[test]
        fn absent_is_null_with_a_reason() {
            let t = Tree::new("empty");
            let s = Sysfs::at(&t.0);
            let j = s.state(&[0, 1]).json();
            for k in [
                "freq_mhz",
                "governor",
                "epp",
                "turbo",
                "pkg_temp_c",
                "platform_profile",
            ] {
                assert!(
                    j.contains(&format!("\"{k}\":null,\"{k}_why\":\"")),
                    "{k}: {j}"
                );
            }
            // each CPU absent by its id, with its reason; never a zero
            assert!(j.contains("\"cpu_khz\":{\"0\":null,\"1\":null}"), "{j}");
            assert!(j.contains("\"cpu_khz_why\":{\"0\":\"no sys/"), "{j}");
            assert!(j.contains("\"policies\":[]"), "{j}");
            assert!(!j.contains("freq_cpus") && !j.contains("temp_src"), "{j}");
            // a driver with no EPP (acpi-cpufreq): the governor read, the EPP null
            t.put(
                "sys/devices/system/cpu/cpufreq/policy0/affected_cpus",
                "0 1",
            )
            .put(
                "sys/devices/system/cpu/cpufreq/policy0/scaling_governor",
                "schedutil",
            );
            assert_eq!(
                s.policy_attr(Some(&[1]), "scaling_governor").unwrap(),
                "schedutil"
            );
            assert!(s
                .policy_attr(Some(&[1]), "energy_performance_preference")
                .is_err());
            // CPUs outside every policy: none of the policies
            assert!(s.policy_attr(Some(&[7]), "scaling_governor").is_err());
            let e = s.env_json("\"perf\":{}");
            assert!(
                e.contains("\"perf_event_paranoid\":null,\"perf_event_paranoid_why\""),
                "{e}"
            );
        }

        #[test]
        fn the_record_fields_exist_when_unavailable() {
            let f = Perf::skipped("not measured for a program's arm").finish();
            assert!(f.starts_with(','));
            let rec: String = format!("{{\"wall\":1.0{f}}}");
            for k in [
                "cycles",
                "instructions",
                "ref_cycles",
                "task_clock_s",
                "eff_ghz",
            ] {
                assert!(
                    rec.contains(&format!("\"{k}\":null,\"{k}_why\":\"")),
                    "{k}: {rec}"
                );
            }
            assert!(rec.contains("\"perf_running\":null"), "{rec}");
            assert!(!rec.contains("perf_scope"), "{rec}");
            assert!(!rec.contains("perf_times_ns"), "{rec}");
            // whatever this machine allows, opening and reading gives the same fields: each
            // a value or null with its reason
            let p = Perf::open();
            p.enable();
            std::hint::black_box((0..100_000u64).sum::<u64>());
            p.disable();
            let f = p.finish();
            for k in [
                "cycles",
                "instructions",
                "ref_cycles",
                "task_clock_s",
                "eff_ghz",
            ] {
                let present =
                    f.contains(&format!("\"{k}\":")) && !f.contains(&format!("\"{k}\":null"));
                let why = f.contains(&format!("\"{k}\":null,\"{k}_why\":\""));
                assert!(present ^ why, "{k}: {f}");
            }
        }

        /// The counters' mechanics where there's no hardware PMU: software events (the task
        /// clock, the CPU clock) in place of cycles and instructions, so opening disabled,
        /// inheriting into a thread started in the region, and counting nothing outside
        /// it are checked on any Linux machine
        #[test]
        fn counts_threads_started_in_the_region_and_nothing_outside_it() {
            // (spinning for `ms` of this thread's own CPU time, RUSAGE_THREAD, not wall
            // time, so a preempted thread on a busy machine spins as long)
            let spin = |ms: u64| {
                let t = crate::rusage(1).0;
                let mut x = 0u64;
                while crate::rusage(1).0 - t < ms as f64 * 1e-3 {
                    for _ in 0..10_000 {
                        x = std::hint::black_box(x.wrapping_add(1));
                    }
                }
            };
            let p = Perf::open_events(
                (TYPE_SOFTWARE, SW_TASK_CLOCK),
                (TYPE_SOFTWARE, 0),
                (TYPE_SOFTWARE, 0),
            );
            if p.cycles.is_err() {
                eprintln!("no software events here: {:?}", p.cycles);
                return;
            }
            spin(60); // before the region: not counted
            p.enable();
            std::thread::spawn(move || spin(80)).join().unwrap();
            spin(20);
            p.disable();
            spin(60); // after it: not counted
            let f = p.finish();
            let num = |k: &str| -> f64 {
                let i = f.find(&format!("\"{k}\":")).unwrap() + k.len() + 3;
                f[i..].split([',', '}']).next().unwrap().parse().unwrap()
            };
            // the "cycles" stand-in is the task clock in ns: the thread's 80 ms and the
            // main thread's 20 ms, and none of the 120 ms outside
            let ns = num("cycles");
            assert!((95e6..115e6).contains(&ns), "{ns} ns: {f}");
            assert!((0.095..0.115).contains(&num("task_clock_s")), "{f}");
            assert_eq!(num("perf_running"), 1.0, "{f}");
            // task clock over task clock: 1 cycle a ns, where the scopes match; in user
            // mode only, no matched ratio is claimed
            if f.contains("\"perf_scope\":\"user\"") {
                assert!(
                    f.contains(
                        "\"eff_ghz\":null,\"eff_ghz_why\":\"cycles counted in user mode only"
                    ),
                    "{f}"
                );
            } else {
                assert!((num("eff_ghz") - 1.0).abs() < 0.01, "{f}");
            }
            // the raw times enabled and running, for every counter
            assert!(f.contains("\"perf_times_ns\":{\"cycles\":["), "{f}");
            assert!(f.contains("\"perf_coverage\":"), "{f}");
        }

        #[test]
        fn env_line_gains_a_machine_field() {
            let l = extend_env("{\"cpus\":4,\"runq\":{\"counts\":true}}");
            assert!(
                l.starts_with("{\"cpus\":4,\"runq\":{\"counts\":true},\"machine\":{"),
                "{l}"
            );
            assert!(
                l.ends_with("}}") && l.contains("\"perf\":{\"cycles\":"),
                "{l}"
            );
        }

        #[test]
        fn settles_within_the_tolerance() {
            let f = |v: &[f64]| v.iter().map(|&x| Some(x)).collect::<Vec<_>>();
            let none = vec![None; 6];
            // too few samples, then a steady window
            assert_eq!(settled(&f(&[4000.0; 3]), &none), Some(false));
            assert_eq!(settled(&f(&[4000.0; 5]), &none), Some(true));
            // the clock moving more than 3%: not settled
            assert_eq!(
                settled(&f(&[4000.0, 4000.0, 4000.0, 4000.0, 4400.0]), &none),
                Some(false)
            );
            // a settled clock with a rising temperature: not settled
            assert_eq!(
                settled(&f(&[4000.0; 5]), &f(&[60.0, 61.0, 62.0, 63.0, 64.0])),
                Some(false)
            );
            assert_eq!(
                settled(&f(&[4000.0; 5]), &f(&[63.0, 64.0, 63.5, 64.0, 63.0])),
                Some(true)
            );
            // nothing readable: no verdict
            assert_eq!(settled(&none, &none), None);
            // a reading lost partway: not settled
            let mut gap = f(&[4000.0; 5]);
            gap[3] = None;
            assert_eq!(settled(&gap, &none), Some(false));
        }

        /// `ab telemetry-check` on whatever this machine has: no case FAILs (a case without
        /// the events it needs is UNAVAILABLE)
        #[test]
        fn the_recorder_check_passes_where_it_can() {
            // (no case FAILs; on a loaded runner a case may be INCOMPLETE, exit 3)
            assert_ne!(check(), 1);
        }

        #[test]
        fn exit_codes_keep_fail_and_incomplete_apart() {
            assert_eq!(exit_code(&["PASS", "UNAVAILABLE", "OBSERVED"]), 0);
            assert_eq!(exit_code(&["PASS", "INCOMPLETE"]), 3);
            assert_eq!(exit_code(&["INCOMPLETE", "FAIL"]), 1);
        }

        /// The inheritance witness fails when a counter's inheritance is removed (the inside
        /// region counts only the idle opener), and never passes on a null, a failed control
        /// or a multiplexed reading
        #[test]
        fn inheritance_witness_negative_controls() {
            let r = |value, running| -> Got {
                Ok(Reading {
                    value,
                    enabled: 100,
                    running,
                })
            };
            let f = |stage, why: &str| -> Got { Err(fault(stage, why)) };
            assert_eq!(inherit_status(&r(80_000, 100), &r(500, 100)).0, "PASS");
            // inheritance removed: inside is the opener's idle count, as before
            assert_eq!(inherit_status(&r(500, 100), &r(480, 100)).0, "FAIL");
            assert_eq!(inherit_status(&r(0, 100), &r(0, 100)).0, "FAIL");
            // never a pass on a missing, failed or multiplexed reading
            let unopened = f(Stage::Open, "no such event here");
            assert_eq!(inherit_status(&unopened, &r(1, 100)).0, "UNAVAILABLE");
            let ctl = f(Stage::Control, "its enable failed (errno 25)");
            assert_eq!(inherit_status(&r(80_000, 100), &ctl).0, "INCOMPLETE");
            assert_eq!(inherit_status(&r(80_000, 50), &r(1, 100)).0, "INCOMPLETE");
            // D066's witnesses: a failed read is INCOMPLETE, not UNAVAILABLE; and an
            // unavailable first reading can't hide a control failure in the second
            let eio = f(
                Stage::Read,
                "reading the counter: Input/output error (os error 5)",
            );
            assert_eq!(inherit_status(&eio, &r(1, 100)).0, "INCOMPLETE");
            assert_eq!(inherit_status(&r(80_000, 100), &eio).0, "INCOMPLETE");
            let (st, why) = inherit_status(&unopened, &ctl);
            assert_eq!(st, "INCOMPLETE");
            assert!(
                why.contains("inside: Open") && why.contains("before: Control"),
                "{why}"
            );
            // with no independent FAIL, those make the check's exit 3
            let statuses = [
                "PASS",
                "UNAVAILABLE",
                inherit_status(&eio, &r(1, 100)).0,
                inherit_status(&unopened, &ctl).0,
            ];
            assert_eq!(exit_code(&statuses), 3);
        }

        /// D066's multiplexing witnesses: failed controls or reads on opened counters are
        /// INCOMPLETE before multiplexing is judged; only valid, exercised multiplexing
        /// listed in the record passes; none exercised is OBSERVED
        #[test]
        fn multiplex_status_negative_controls() {
            let whole = |v| -> Got {
                Ok(Reading {
                    value: v,
                    enabled: 100,
                    running: 100,
                })
            };
            let half = |v| -> Got {
                Ok(Reading {
                    value: v,
                    enabled: 100,
                    running: 50,
                })
            };
            let f = |stage, why: &str| -> Got { Err(fault(stage, why)) };
            let names = ["cycles", "instructions", "ref_cycles", "task_clock"];
            let rec =
                |v: [Got; 4]| -> Vec<(&'static str, Got)> { names.into_iter().zip(v).collect() };
            // every opened counter's control failed: INCOMPLETE, never "whole enabled time"
            let all_failed = rec([
                f(Stage::Control, "its enable failed (errno 9)"),
                f(Stage::Control, "its enable failed (errno 9)"),
                f(Stage::Read, "reading the counter: os error 5"),
                f(Stage::Control, "its enable failed (errno 9)"),
            ]);
            let none = "\"perf_multiplexed\":[],\"eff_ghz\":null";
            assert_eq!(multiplex_status(&all_failed, none).0, "INCOMPLETE");
            // cycles' control failed while instructions multiplexed: INCOMPLETE, not PASS
            let mixed = rec([
                f(Stage::Control, "its disable failed (errno 9)"),
                half(50),
                whole(10),
                whole(10),
            ]);
            let listed = "\"perf_multiplexed\":[\"instructions\"],\"eff_ghz\":null";
            assert_eq!(multiplex_status(&mixed, listed).0, "INCOMPLETE");
            // a counter that couldn't be opened is left out
            let unopened = rec([
                whole(10),
                half(50),
                f(Stage::Open, "no such event here"),
                whole(10),
            ]);
            assert_eq!(multiplex_status(&unopened, listed).0, "PASS");
            // nothing multiplexed: OBSERVED, not exercised
            let clean = rec([whole(1), whole(1), whole(1), whole(1)]);
            let (st, d) = multiplex_status(&clean, none);
            assert_eq!(st, "OBSERVED");
            assert!(d.contains("not exercised"), "{d}");
            // multiplexed but not listed, or cycles multiplexed with eff_ghz standing: FAIL
            assert_eq!(multiplex_status(&unopened, none).0, "FAIL");
            let cyc = rec([half(5), whole(1), whole(1), whole(1)]);
            let standing = "\"perf_multiplexed\":[\"cycles\"],\"eff_ghz\":1.000";
            assert_eq!(multiplex_status(&cyc, standing).0, "FAIL");
            // with no independent FAIL, the two D066 witnesses make the exit 3
            let statuses = [
                "PASS",
                multiplex_status(&all_failed, none).0,
                multiplex_status(&mixed, listed).0,
            ];
            assert_eq!(exit_code(&statuses), 3);
        }

        #[test]
        fn strings_are_escaped() {
            assert_eq!(js("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
        }
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

    /// A fake procfs, sysfs and cgroupfs under the test's own directory, removed when dropped
    struct Fake(std::path::PathBuf);
    impl Fake {
        fn new(name: &str) -> Fake {
            let p = std::env::temp_dir().join(format!("ab-facts-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Fake(p)
        }
        fn put(&self, rel: &str, v: &str) -> &Self {
            let p = self.0.join(rel.trim_start_matches('/'));
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, v).unwrap();
            self
        }
        fn dir(&self, rel: &str) -> &Self {
            std::fs::create_dir_all(self.0.join(rel.trim_start_matches('/'))).unwrap();
            self
        }
        fn root(&self) -> facts::Root {
            facts::Root::at(&self.0)
        }
    }
    impl Drop for Fake {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const MOUNT_V2: &str = "30 23 0:26 / /sys/fs/cgroup rw,nosuid - cgroup2 cgroup2 rw\n";

    /// cgroup v2, the process in /user.slice/run.scope, its root unlimited
    fn v2(name: &str) -> Fake {
        let t = Fake::new(name);
        t.put("/proc/self/cgroup", "0::/user.slice/run.scope\n")
            .put("/proc/self/mountinfo", MOUNT_V2)
            .dir("/sys/fs/cgroup/user.slice/run.scope")
            .put("/sys/fs/cgroup/user.slice/cpu.max", "max 100000\n")
            .put(
                "/sys/fs/cgroup/user.slice/run.scope/cpu.max",
                "max 100000\n",
            )
            .put("/sys/fs/cgroup/user.slice/cgroup.type", "domain\n")
            .put(
                "/sys/fs/cgroup/user.slice/run.scope/cgroup.type",
                "domain\n",
            );
        t
    }

    const MOUNT_V1: &str = "36 23 0:28 / /sys/fs/cgroup rw - tmpfs tmpfs rw,mode=755\n\
        37 36 0:29 / /sys/fs/cgroup/cpu,cpuacct rw - cgroup cgroup rw,cpu,cpuacct\n\
        40 36 0:32 / /sys/fs/cgroup/memory rw - cgroup cgroup rw,memory\n\
        46 36 0:38 / /sys/fs/cgroup/unified rw - cgroup2 cgroup2 rw\n";

    /// cgroup v1 (hybrid), the process in /job, every level unlimited
    fn v1(name: &str) -> Fake {
        let t = Fake::new(name);
        let h = "/sys/fs/cgroup/cpu,cpuacct";
        t.put(
            "/proc/self/cgroup",
            "4:memory:/\n3:cpu,cpuacct:/job\n0::/\n",
        )
        .put("/proc/self/mountinfo", MOUNT_V1)
        .put(&format!("{h}/release_agent"), "")
        .put(&format!("{h}/cpu.cfs_quota_us"), "-1\n")
        .put(&format!("{h}/cpu.cfs_period_us"), "100000\n")
        .put(&format!("{h}/job/cpu.cfs_quota_us"), "-1\n")
        .put(&format!("{h}/job/cpu.cfs_period_us"), "100000\n");
        t
    }

    #[test]
    fn the_quota_is_established_unlimited_only_through_every_level() {
        use facts::Quota::*;
        let q = |t: &Fake| facts::cpu_quota(&t.root());
        // positive controls: v2 and v1, every level read and none limited
        let t = v2("v2-ok");
        let got = q(&t);
        assert_eq!(got.quota, Unlimited, "{:?}", got.why);
        assert_eq!(got.hierarchy, Some("v2"));
        assert_eq!(
            got.levels,
            [
                "/sys/fs/cgroup",
                "/sys/fs/cgroup/user.slice",
                "/sys/fs/cgroup/user.slice/run.scope"
            ]
        );
        assert!(got
            .json()
            .starts_with("{\"state\":\"unlimited\",\"cpus\":null,\"hierarchy\":\"v2\""));
        let t = v1("v1-ok");
        let got = q(&t);
        assert_eq!(got.quota, Unlimited, "{:?}", got.why);
        assert_eq!(got.hierarchy, Some("v1"));
        assert_eq!(got.levels.len(), 2);
        // (the Researcher's case: a limited child under an unlimited root, which a fixed
        // read of /sys/fs/cgroup/cpu.max calls unlimited) negative controls: limited
        let t = v2("v2-child-limited");
        t.put(
            "/sys/fs/cgroup/user.slice/run.scope/cpu.max",
            "400000 100000\n",
        );
        assert_eq!(q(&t).quota, Cpus(4.0));
        assert!(q(&t)
            .json()
            .starts_with("{\"state\":\"limited\",\"cpus\":4,"));
        let t = v2("v2-tightest");
        t.put("/sys/fs/cgroup/user.slice/cpu.max", "200000 100000\n")
            .put(
                "/sys/fs/cgroup/user.slice/run.scope/cpu.max",
                "800000 100000\n",
            );
        assert_eq!(q(&t).quota, Cpus(2.0));
        let t = v1("v1-child-limited");
        t.put("/sys/fs/cgroup/cpu,cpuacct/job/cpu.cfs_quota_us", "50000\n");
        assert_eq!(q(&t).quota, Cpus(0.5));
        // (v2: a level without cpu.max has the controller off there, no limit)
        let t = v2("v2-no-controller");
        std::fs::remove_file(t.0.join("sys/fs/cgroup/user.slice/cpu.max")).unwrap();
        assert_eq!(q(&t).quota, Unlimited);
        // unknown controls: each refuses to establish anything
        let unknown: Vec<(&str, Box<dyn Fn(&Fake)>)> = vec![
            (
                "no /proc/self/cgroup",
                Box::new(|t| {
                    std::fs::remove_file(t.0.join("proc/self/cgroup")).unwrap();
                }),
            ),
            (
                "no mountinfo",
                Box::new(|t| {
                    std::fs::remove_file(t.0.join("proc/self/mountinfo")).unwrap();
                }),
            ),
            (
                "an unparsable cpu.max",
                Box::new(|t| {
                    t.put("/sys/fs/cgroup/user.slice/cpu.max", "max\n");
                }),
            ),
            (
                "a fractional cpu.max",
                Box::new(|t| {
                    t.put("/sys/fs/cgroup/user.slice/cpu.max", "1.5 100000\n");
                }),
            ),
            (
                "an unreadable cpu.max",
                Box::new(|t| {
                    let p = t.0.join("sys/fs/cgroup/user.slice/run.scope/cpu.max");
                    std::fs::remove_file(&p).unwrap();
                    std::fs::create_dir(&p).unwrap();
                }),
            ),
            (
                "a missing level",
                Box::new(|t| {
                    t.put("/proc/self/cgroup", "0::/user.slice/gone.scope\n");
                }),
            ),
            (
                "a namespaced mount (cgroup.type at the top)",
                Box::new(|t| {
                    t.put("/sys/fs/cgroup/cgroup.type", "domain\n");
                }),
            ),
            (
                "two cgroup2 mounts",
                Box::new(|t| {
                    t.put(
                        "/proc/self/mountinfo",
                        &format!("{MOUNT_V2}{}", MOUNT_V2.replace("30 23", "31 23")),
                    );
                }),
            ),
            (
                "two v2 entries",
                Box::new(|t| {
                    t.put(
                        "/proc/self/cgroup",
                        "0::/user.slice\n0::/user.slice/run.scope\n",
                    );
                }),
            ),
            (
                "an escaped path",
                Box::new(|t| {
                    t.put("/proc/self/cgroup", "0::/user.slice/../run.scope\n");
                }),
            ),
            (
                "no v2 entry, no v1 cpu",
                Box::new(|t| {
                    t.put("/proc/self/cgroup", "4:memory:/\n");
                }),
            ),
        ];
        for (what, mutate) in unknown {
            let t = v2(&format!("v2-{}", what.len()));
            mutate(&t);
            let got = q(&t);
            assert_eq!(got.quota, Unknown, "{what}: {:?}", got.levels);
            assert!(got.why.is_some(), "{what}");
            assert!(
                got.json()
                    .starts_with("{\"state\":\"unknown\",\"cpus\":null"),
                "{what}"
            );
        }
        let h = "sys/fs/cgroup/cpu,cpuacct";
        let unknown_v1: Vec<(&str, Box<dyn Fn(&Fake)>)> = vec![
            (
                "no release_agent (not the hierarchy's root)",
                Box::new(move |t| {
                    std::fs::remove_file(t.0.join(format!("{h}/release_agent"))).unwrap();
                }),
            ),
            (
                "no cfs_quota_us at a level",
                Box::new(move |t| {
                    std::fs::remove_file(t.0.join(format!("{h}/job/cpu.cfs_quota_us"))).unwrap();
                }),
            ),
            (
                "a zero period",
                Box::new(move |t| {
                    t.put(&format!("/{h}/cpu.cfs_period_us"), "0\n");
                }),
            ),
            (
                "a quota of -2",
                Box::new(move |t| {
                    t.put(&format!("/{h}/job/cpu.cfs_quota_us"), "-2\n");
                }),
            ),
            (
                "two v1 cpu entries",
                Box::new(|t| {
                    t.put("/proc/self/cgroup", "3:cpu,cpuacct:/job\n2:cpu:/job\n");
                }),
            ),
            (
                "no v1 cpu mount",
                Box::new(|t| {
                    t.put("/proc/self/mountinfo", MOUNT_V2);
                }),
            ),
            (
                "a mount below the hierarchy's root",
                Box::new(|t| {
                    t.put(
                        "/proc/self/mountinfo",
                        &MOUNT_V1.replace("0:29 / ", "0:29 /other "),
                    );
                }),
            ),
        ];
        for (what, mutate) in unknown_v1 {
            let t = v1(&format!("v1-{}", what.len()));
            mutate(&t);
            let got = q(&t);
            assert_eq!(got.quota, Unknown, "{what}: {:?}", got.levels);
            assert_eq!(got.hierarchy, Some("v1"), "{what}");
        }
    }

    #[test]
    fn the_allowed_cpus_are_the_affinity_and_the_status_agreeing() {
        let t = Fake::new("allowed");
        t.put("/proc/self/status", "Name:\tab\nCpus_allowed_list:\t0-3\n");
        let r = t.root();
        assert_eq!(
            facts::allowed(&r, Ok(vec![0, 1, 2, 3])),
            Ok(vec![0, 1, 2, 3])
        );
        assert!(facts::allowed(&r, Ok(vec![0, 1]))
            .unwrap_err()
            .contains("disagree"));
        assert!(facts::allowed(&r, Err("no mask".into())).is_err());
        t.put("/proc/self/status", "Cpus_allowed_list:\t0-3,x\n");
        assert!(facts::allowed(&r, Ok(vec![0, 1, 2, 3])).is_err());
        assert_eq!(facts::cpu_list("0-3,6\n"), Some(vec![0, 1, 2, 3, 6]));
        for bad in ["", "3-1", "0,0", "a", "1-", " - "] {
            assert_eq!(facts::cpu_list(bad), None, "{bad:?}");
        }
        // the live process: whatever this machine is, the two sources agree
        let live = facts::allowed(&facts::Root::at("/"), facts::affinity());
        assert!(live.as_ref().is_ok_and(|v| !v.is_empty()), "{live:?}");
    }

    #[test]
    fn the_facts_line_is_null_with_why_where_a_fact_is_not_read() {
        let t = v2("line");
        t.put("/proc/self/status", "Cpus_allowed_list:\t0-3\n");
        for c in 0..4 {
            t.put(
                &format!("/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list"),
                if c % 2 == 0 { &"0,2"[..] } else { "1,3" },
            );
        }
        let l = facts::line(&t.root(), Ok(vec![0, 1, 2, 3]), false, true, "{}");
        assert!(l.starts_with("{\"facts\":\"ab env --facts/1\",\"cpus\":4,\"cpus_allowed\":[0,1,2,3],\"cores\":2,\"core_cpus\":[\"0,2\",\"1,3\"],\"cpu_quota\":{\"state\":\"unlimited\""), "{l}");
        assert!(
            l.ends_with("\"spin_threads\":0,\"perf_probed\":false,\"machine\":{}}"),
            "{l}"
        );
        std::fs::remove_file(t.0.join("sys/devices/system/cpu/cpu3/topology/thread_siblings_list"))
            .unwrap();
        let l = facts::line(&t.root(), Ok(vec![0, 1, 2, 3]), false, true, "{}");
        assert!(
            l.contains("\"cores\":null,\"core_cpus\":null,\"cores_why\":"),
            "{l}"
        );
        let l = facts::line(&t.root(), Ok(vec![0, 1]), false, true, "{}");
        assert!(
            l.contains("\"cpus\":null,\"cpus_allowed\":null,\"cpus_allowed_why\":"),
            "{l}"
        );
    }

    /// The passive path's witness, at run time and by its text. At run time: `facts::live`
    /// on this thread attempts no `perf_event_open` through ab's one wrapper
    /// (`machine::PERF_OPENS`) and starts no spin thread (`SPIN_SPAWNS`), while the active
    /// `env_line`, through the same counters, does both (the positive control). By its text:
    /// the module calls no counter, spawns no thread and spins nothing. Neither shows what a
    /// library call might do underneath (sysfs reads, `sched_getaffinity`): only ab's own
    /// counter and spin paths are witnessed
    #[test]
    fn the_facts_path_opens_no_counter_and_spawns_no_spin_thread() {
        let opens = || machine::PERF_OPENS.with(std::cell::Cell::get);
        let spins = || SPIN_SPAWNS.with(std::cell::Cell::get);
        let (o0, s0) = (opens(), spins());
        let l = facts::live();
        assert_eq!(
            (opens(), spins()),
            (o0, s0),
            "the passive path opened or spun: {l}"
        );
        assert!(l.contains("\"perf\":null,\"perf_why\":\"not probed"), "{l}");
        let active = env_line();
        assert!(
            opens() > o0,
            "the positive control opened nothing: {active}"
        );
        assert!(spins() > s0, "the positive control spun nothing: {active}");
        let src = include_str!("ab.rs");
        let start = src.find("\nmod facts {").expect("mod facts");
        let end = src.find("// (the end of mod facts").expect("its end");
        let body = &src[start..end];
        for banned in [
            "Perf::",
            "Perf {",
            "perf_event_open",
            "probe()",
            "fn open(",
            "thread::spawn",
            "spin(",
            "extend_env",
            "env_line",
        ] {
            assert!(!body.contains(banned), "mod facts names {banned}");
        }
    }

    /// The producer keeps every child's termination: `ab run` itself, its children stand-in
    /// scripts (ARM@EXE) that write a valid-looking record and then succeed, exit 17, or are
    /// killed by a signal, and one that writes nothing (the controls). Output before a
    /// failure is recorded with that failure, never as a success
    #[test]
    fn the_run_records_each_childs_termination() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("ab-term-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let rec = r#"echo '{"wall":0.5,"cpu":0.5,"rss_kb":1,"out":3,"hash":"0123456789abcdef"}'"#;
        let mut arms = Vec::new();
        for (name, tail) in [
            ("ok", format!("{rec}\nexit 0")),
            ("seventeen", format!("{rec}\nexit 17")),
            ("killed", format!("{rec}\nkill -KILL $$")),
            ("empty", "exit 0".to_string()),
        ] {
            let p = dir.join(name);
            std::fs::write(&p, format!("#!/bin/sh\n{tail}\n")).unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            arms.push(format!("{name}@{}", p.display()));
        }
        let corpus = dir.join("c.bz2");
        std::fs::write(&corpus, b"BZh9").unwrap();
        let out = dir.join("runs.jsonl");
        let args: Vec<String> = [
            out.to_str().unwrap(),
            "--arms",
            &arms.join(","),
            "--threads",
            "2",
            "--rounds",
            "1",
            "--max-busy",
            "64",
            "--settle",
            "0.2",
            corpus.to_str().unwrap(),
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        run(&args);
        let text = std::fs::read_to_string(&out).unwrap();
        let by = |arm: &str| {
            text.lines()
                .find(|l| l.contains(&format!("\"arm\":\"{arm}@")))
                .unwrap_or_else(|| panic!("no {arm} record in {text}"))
                .to_string()
        };
        let ok = by("ok");
        assert!(ok.contains("\"termination\":\"success\""), "{ok}");
        assert!(ok.contains("\"hash\":\"0123456789abcdef\""), "{ok}");
        let r = by("seventeen");
        assert!(r.contains("\"termination\":\"exit 17\""), "{r}");
        assert!(
            r.contains("\"hash\":\"0123456789abcdef\""),
            "its output kept beside: {r}"
        );
        let r = by("killed");
        assert!(r.contains("\"termination\":\"signal 9\""), "{r}");
        assert!(r.contains("\"hash\":\"0123456789abcdef\""), "{r}");
        let r = by("empty");
        assert!(r.contains("\"termination\":\"success\""), "{r}");
        assert!(r.contains("\"broken\":1"), "{r}");
        let _ = std::fs::remove_dir_all(&dir);
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

    #[test]
    fn hold_auto_avoids_the_decoders_cores() {
        // the i9: 10 cores, siblings c and c + 10
        let cores = group(&(0..20).collect::<Vec<_>>(), |c| {
            Some(vec![c % 10, c % 10 + 10])
        });
        let spread = |t: usize| cpu_list(&spread_list(&cores, t).unwrap());
        // 4 threads spread on cores 0-3: the last free core's last CPU
        assert_eq!(pick_hold(&cores, &spread(4)), Some((19, false)));
        // 10 threads on every core: no free core, so a sibling, the overlap reported
        assert_eq!(pick_hold(&cores, &spread(10)), Some((19, true)));
        // a 1-thread run pinned to CPU 1
        assert_eq!(pick_hold(&cores, &[1]), Some((19, false)));
        // a decoder on core 9's CPU 19 alone: its sibling 9 is on the same core
        assert_eq!(pick_hold(&cores, &[19]), Some((18, false)));
        // unpinned (every CPU): the last, overlapping
        assert_eq!(
            pick_hold(&cores, &(0..20).collect::<Vec<_>>()),
            Some((19, true))
        );
        // CI's 4 CPUs on 2 cores, 2 threads spread on 0,2: no free core; 3, 2's sibling
        let ci = group(&[0, 1, 2, 3], |c| {
            Some(if c < 2 { vec![0, 1] } else { vec![2, 3] })
        });
        assert_eq!(pick_hold(&ci, &[0, 2]), Some((3, true)));
        assert_eq!(pick_hold(&ci, &[1]), Some((3, false)));
        assert_eq!(pick_hold(&[], &[]), None);
        // an explicit CPU: its core against the decoder's CPUs
        assert!(hold_overlaps(19, &[9, 19], &[9]));
        assert!(hold_overlaps(3, &[], &[3]));
        assert!(!hold_overlaps(19, &[9, 19], &[0, 1, 2]));
    }

    /// The held spinners among this process's threads now
    fn spinners() -> usize {
        std::fs::read_dir("/proc/self/task")
            .unwrap()
            .flatten()
            .filter(|e| {
                std::fs::read_to_string(e.path().join("comm"))
                    .is_ok_and(|c| c.trim() == "held-spinner")
            })
            .count()
    }

    /// Whether thread `tid` of this process is still there
    fn alive(tid: u32) -> bool {
        std::path::Path::new(&format!("/proc/self/task/{tid}")).exists()
    }

    /// Whether `cond` holds within 2 s. A joined thread's /proc entry can outlast the join
    /// briefly (the join returns when the kernel clears its tid, before its task is released)
    fn soon(cond: impl Fn() -> bool) -> bool {
        let t = std::time::Instant::now();
        while t.elapsed() < std::time::Duration::from_secs(2) {
            if cond() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        cond()
    }

    #[test]
    fn held_spinner_starts_holds_and_stops() {
        // (one test, so the tests' own threads don't come and go between its readings)
        let cores = cores();
        let mine: Vec<usize> = cores.iter().flatten().copied().collect();
        let (cpu, _) = pick_hold(&cores, &mine[..1]).unwrap();
        assert_eq!(spinners(), 0);
        let h = Held::start(cpu, true, false).expect("the spinner holds a CPU it may use");
        let tid = h.tid;
        assert!(alive(tid));
        let comm = std::fs::read_to_string(format!("/proc/self/task/{tid}/comm")).unwrap();
        assert_eq!(comm.trim(), "held-spinner");
        let status = std::fs::read_to_string(format!("/proc/self/task/{tid}/status")).unwrap();
        let allowed = status
            .lines()
            .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
            .map(cpu_list)
            .unwrap();
        assert_eq!(allowed, vec![cpu]);
        // it spins: its CPU grows while it is held, and the gate takes it off its CPU
        let t0 = h.ticks();
        let (busy, _, raw) = others_busy(0.3, Some(&h));
        let (raw_busy, _) = raw.expect("the raw readings beside the accounted ones");
        assert!(h.ticks() > t0, "the spinner ran");
        assert!(busy <= raw_busy);
        assert!(h.facts().contains(&format!("\"cpu\":{cpu},\"auto\":true")));
        let spun = h.stop();
        // (only that its CPU was read: how much it gets depends on the runner's load)
        assert!(spun > 0.0, "its own CPU, {spun} s");
        assert!(soon(|| !alive(tid)), "stopped and joined");
        assert!(soon(|| spinners() == 0), "no thread left behind");

        // a failing child (it exits 1) under the spinner: stopped after the reap, as `run`
        let h = Held::start(cpu, false, false).unwrap();
        let tid = h.tid;
        let st = std::process::Command::new("sh")
            .args(["-c", "exit 1"])
            .status()
            .unwrap();
        assert!(!st.success());
        assert!(alive(tid), "held until the child is reaped");
        h.stop();
        assert!(soon(|| !alive(tid)));

        // an error or a panic while it is held: dropped, so stopped and joined
        let tid = std::panic::catch_unwind(|| {
            let h = Held::start(cpu, false, false).unwrap();
            let tid = h.tid;
            assert!(alive(tid));
            std::panic::panic_any(tid);
        })
        .unwrap_err()
        .downcast::<u32>()
        .map(|b| *b)
        .unwrap();
        assert!(soon(|| !alive(tid)), "the unwind stopped it");
        assert!(soon(|| spinners() == 0), "no thread left behind");

        // a CPU it may not run on: refused, and nothing left running
        assert!(Held::start(4096, false, false).is_err());
        let not_allowed = (0..1024).find(|c| !mine.contains(c)).unwrap();
        assert!(Held::start(not_allowed, false, false).is_err());
        assert!(soon(|| spinners() == 0), "no thread left behind");
    }
}
