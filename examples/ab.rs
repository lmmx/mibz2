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
//!                      thread count, the arms' order rotated and reversed round to round
//!     --cap ARM=N,..   run ARM (as given in --arms) in the first N rounds only
//!     --pin CPU        the CPU 1-thread runs are pinned to (default 1)
//!     --max-busy C     before each run, wait until the other processes keep at most C
//!                      cores busy (default 1.5; sampled over 2 s while nothing of ours runs)
//!     --settle S       that sample's length in seconds (default 2; CI's runners, which
//!                      run nothing else, take a shorter one)
//! ab summary OUT.jsonl [--ref ARM]   best and median wall and CPU, peak RSS, and paired
//!                                    ratios against ARM (geometric mean, 95% CI over rounds)
//! ab child ARM THREADS FILE          one run (what `run` spawns)
//! ```
//!
//! Arms, block-parallel unless said:
//!
//! - `mibz2` (this build: `BzReader::new`, or `BzReader::parallel`), `mibz2-scalar` (the
//!   portable kernels);
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
//!   `StreamingBz2Read`, on the CPUs it may run on); `fbz` (`fbz::Reader`, `threads: T`).
//!
//! A single-threaded arm runs once a round on each file, at the fewest threads, and the
//! summaries pair that run with the reference's at every thread count.
//!
//! Recorded a run: wall (the decode, from opening the file to the last byte read), CPU
//! (user + sys), peak RSS, output bytes and hash; and around it, the 1-minute load, the
//! cores the other processes kept busy during the run (the machine's busy time less the
//! run's own CPU) and the steal.

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
}

/// (CPU seconds, peak RSS in KiB) of this process (who 0) or its waited-for children (-1)
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

/// Each thread's CPU (ticks) of process `pid`, sampled every 50 ms until `stop`, the last
/// sample of each thread kept (a thread's CPU only grows)
struct Threads {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    seen: std::sync::Arc<std::sync::Mutex<BTreeMap<String, u64>>>,
    handle: Option<std::thread::JoinHandle<()>>,
    pid: u32,
}

fn sample_tasks(pid: u32, seen: &std::sync::Mutex<BTreeMap<String, u64>>) {
    let Ok(dir) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return;
    };
    let mut m = seen.lock().unwrap();
    for e in dir.flatten() {
        let Ok(st) = std::fs::read_to_string(e.path().join("stat")) else {
            continue;
        };
        // fields after the ")" of the name: state is 3rd, utime 14th, stime 15th
        let Some(rest) = st.rsplit_once(')').map(|x| x.1) else {
            continue;
        };
        let f: Vec<&str> = rest.split_whitespace().collect();
        let ticks = f.get(11).and_then(|u| u.parse::<u64>().ok()).unwrap_or(0)
            + f.get(12).and_then(|u| u.parse::<u64>().ok()).unwrap_or(0);
        let name = std::fs::read_to_string(e.path().join("comm")).unwrap_or_default();
        let key = format!("{}:{}", name.trim(), e.file_name().to_string_lossy());
        let v = m.entry(key).or_insert(0);
        *v = (*v).max(ticks);
    }
}

impl Threads {
    fn watch(pid: u32) -> Threads {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let seen = std::sync::Arc::new(std::sync::Mutex::new(BTreeMap::new()));
        let (s, m) = (stop.clone(), seen.clone());
        let handle = std::thread::Builder::new()
            .name("sampler".into())
            .spawn(move || {
                while !s.load(std::sync::atomic::Ordering::Relaxed) {
                    sample_tasks(pid, &m);
                    std::thread::sleep(Duration::from_millis(50));
                }
            })
            .unwrap();
        Threads {
            stop,
            seen,
            handle: Some(handle),
            pid,
        }
    }
    /// The threads that ran (more than 2% of the busiest's CPU), and each one's CPU
    /// seconds, busiest first (the sampler left out; "main" is the thread reading the
    /// decoder and hashing its output)
    fn finish(mut self, own: bool) -> (usize, String) {
        if own {
            sample_tasks(self.pid, &self.seen);
        }
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.handle.take().unwrap().join().unwrap();
        let me = format!("ab:{}", self.pid);
        let m = self.seen.lock().unwrap();
        let mut v: Vec<(u64, String)> = m
            .iter()
            .filter(|(k, _)| !k.starts_with("sampler:"))
            .map(|(k, &t)| {
                (
                    t,
                    if own && *k == me {
                        "main".to_string()
                    } else {
                        k.clone()
                    },
                )
            })
            .collect();
        v.sort_by(|a, b| b.cmp(a));
        let top = v.first().map_or(0, |x| x.0);
        let busy: Vec<String> = v
            .iter()
            .filter(|x| x.0 * 50 > top)
            .map(|(t, k)| format!("{}={:.2}", k.split(':').next().unwrap(), *t as f64 / HZ))
            .collect();
        (busy.len(), busy.join(" "))
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

/// Whether an arm decodes a stream's blocks in parallel. A single-threaded arm (libbzip2
/// and its Rust port, which have no thread count) runs once a round on each file, at the
/// smallest thread count (pinned to one CPU, as every 1-thread run), and the summaries
/// pair that run with the reference's at every thread count
fn parallel(arm: &str) -> bool {
    !matches!(arm, "libbzip2" | "libbz2-rs")
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
        "indexed_bzip2" => {
            let mut c =
                Command::new(std::env::var("AB_PYTHON").unwrap_or_else(|_| "python3".into()));
            c.args(["bench/ib.py", &t, path]);
            Some((c, true))
        }
        _ => None,
    }
}

/// An in-process arm's decode of the file at `path`, every byte through `h`
fn decode(arm: &str, threads: usize, path: &str, h: &mut Hash) -> Result<(), String> {
    let open = || std::fs::File::open(path).map_err(|e| e.to_string());
    match arm {
        "mibz2" if threads == 1 => drain(mibz2::BzReader::new(open()?), h),
        "mibz2" => drain(mibz2::BzReader::parallel(open()?, threads), h),
        "mibz2-scalar" if threads == 1 => drain(
            mibz2::BzReader::with_kernels(open()?, mibz2::Kernels::scalar()),
            h,
        ),
        "mibz2-scalar" => drain(
            mibz2::BzReader::parallel_with(open()?, threads, 2 * threads, mibz2::Kernels::scalar()),
            h,
        ),
        "pbz2" if threads == 1 => drain(pbz2::DecoderReader::new(open()?), h),
        "pbz2" => drain(pbz2::ParallelDecoder::with_threads(open()?, threads), h),
        // libbzip2 1.0.8 (C: the bzip2 crate's bundled source, the tests' oracle), one
        // stream after another, as `bzip2 -dc` and Python's bz2 decode a file
        "libbzip2" => drain(bzip2::read::MultiBzDecoder::new(open()?), h),
        // (bench/alternatives' build of this harness only)
        #[cfg(mibz2_alternatives)]
        "libbz2-rs" => drain(bzip2_rs::read::MultiBzDecoder::new(open()?), h),
        // (no thread count: it takes the CPUs this process may run on, so a 1-thread
        // run's pin decides)
        #[cfg(mibz2_alternatives)]
        "lbzip2-rs" => drain(
            lbzip2::reader::StreamingBz2Read::new(open()?).map_err(|e| format!("{e:?}"))?,
            h,
        ),
        #[cfg(mibz2_alternatives)]
        "fbz" => {
            let opts = fbz::DecodeOptions {
                format: fbz::DecodeFormat::Bzip2,
                threads,
                ..Default::default()
            };
            drain(fbz::Reader::open(path, opts).map_err(|e| e.to_string())?, h)
        }
        _ => Err(format!("unknown arm {arm} (in this build of the harness)")),
    }
}

/// A program's run: the result, the wall time, its busy threads, and the CPU it spent
/// before it was ready to decode
fn piped(
    mut cmd: Command,
    ready: bool,
    h: &mut Hash,
) -> (Result<(), String>, f64, (usize, String), f64) {
    let mut t0 = Instant::now();
    let mut p = match cmd.stdout(Stdio::piped()).spawn() {
        Ok(p) => p,
        Err(e) => return (Err(format!("{cmd:?}: {e}")), 0.0, (0, String::new()), 0.0),
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
    let r = drain(out, h);
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
    let (res, wall, (nthreads, per_thread), startup) = match external {
        Some((cmd, ready)) => piped(cmd, ready, &mut h),
        None => {
            let w = Threads::watch(std::process::id());
            let t0 = Instant::now();
            let res = decode(arm, threads, path, &mut h);
            let wall = t0.elapsed().as_secs_f64();
            (res, wall, w.finish(true), 0.0)
        }
    };
    let (cpu_self, rss_self) = rusage(0);
    let (cpu_kids, rss_kids) = rusage(-1);
    // (a program's arm: its own peak RSS, and its CPU with this process's hashing, less
    // its interpreter's start)
    let rss = if is_program { rss_kids } else { rss_self };
    let err = res.err().unwrap_or_default().replace('"', "'");
    println!(
        "{{\"wall\":{wall:.4},\"cpu\":{:.4},\"rss_kb\":{rss},\"out\":{},\"hash\":\"{}\",\"threads_busy\":{nthreads},\"thread_cpu\":\"{per_thread}\",\"err\":\"{err}\"}}",
        cpu_self + cpu_kids - startup,
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
    for round in 0..rounds {
        for file in &files {
            for &t in &threads {
                // the arms' order rotated a step each round, and reversed every other
                let mut order: Vec<&String> = arms.iter().collect();
                order.rotate_left(round % arms.len());
                if round % 2 == 1 {
                    order.reverse();
                }
                for arm in order {
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
                    let mut cmd = if t == 1 {
                        let mut c = Command::new("taskset");
                        c.args(["-c", &pin]).arg(&exe);
                        c
                    } else {
                        Command::new(&exe)
                    };
                    cmd.args(["child", &name, &t.to_string(), file]);
                    let load = load1();
                    let (b0, s0, tot0) = stat();
                    let w0 = Instant::now();
                    let o = cmd.output().unwrap();
                    let span = w0.elapsed().as_secs_f64();
                    let (b1, s1, tot1) = stat();
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
                        "{{\"round\":{round},\"corpus\":\"{base}\",\"threads\":{t},\"arm\":\"{arm}\",\"parallel\":{},\"load1\":{load:.2},\"others_busy\":{others:.2},\"pre_busy\":{busy:.2},\"steal_pct\":{steal:.1},\"waited_s\":{:.0},{}",
                        parallel(&name),
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
        _ => eprintln!("usage: ab gen DIR [SCALE] | ab run OUT.jsonl [--arms a,b] [--threads 1,4] [--rounds N] [--settle S] FILE.. | ab summary OUT.jsonl [--ref ARM] | ab child ARM T FILE"),
    }
}
