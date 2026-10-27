//! An interleaved A/B harness for whole decoders: each run is one process decoding one
//! file from disk, end to end (headers, Huffman, MTF, the T-vector, the walk and its
//! joining, RLE1, CRCs), its output hashed so every arm's bytes are compared.
//!
//! ```text
//! ab gen DIR [SCALE]                 the crate's corpora (tests/common), libbzip2 -9, into DIR
//! ab run OUT.jsonl [options] FILE..  time the arms on each file; one JSON line a run
//!     --arms a,b,c     arms (default mibz2,pbz2,lbzip2); ARM@EXE runs ARM in another
//!                      build of this harness (a baseline commit's), so builds compare
//!                      in the same process shape
//!     --threads 1,4    thread counts (1-thread runs are pinned to one CPU, so a
//!                      decoder's helper threads share its core)
//!     --rounds N       rounds (default 5); each round runs every arm once per file and
//!                      thread count, the arms' order rotated and reversed round to round
//!     --pin CPU        the CPU 1-thread runs are pinned to (default 1)
//!     --max-busy C     before each run, wait until the other processes keep at most C
//!                      cores busy (default 1.5; sampled over 2 s while nothing of ours runs)
//! ab summary OUT.jsonl [--ref ARM]   best and median wall and CPU, peak RSS, and paired
//!                                    ratios against ARM (geometric mean, 95% CI over rounds)
//! ab child ARM THREADS FILE          one run (what `run` spawns)
//! ```
//!
//! Arms: `mibz2` (this build: `BzReader::new`, or `BzReader::parallel`), `mibz2-scalar`
//! (the portable kernels), `pbz2` (pbz2 0.2.0, a dev-dependency only: `DecoderReader`, or
//! `ParallelDecoder::with_threads`), `lbzip2` (`lbzip2 -dc -nT`, its output read through a
//! pipe and hashed here, its CPU and peak RSS its own).
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
        Hash { h: [1, 2, 3, 4], tail: [0; 32], nt: 0, n: 0 }
    }
    #[inline]
    fn block(&mut self, b: &[u8]) {
        for (i, w) in b.chunks_exact(8).enumerate() {
            let w = u64::from_le_bytes(w.try_into().unwrap());
            self.h[i] = (self.h[i] ^ w).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(31);
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
    let Ok(dir) = std::fs::read_dir(format!("/proc/{pid}/task")) else { return };
    let mut m = seen.lock().unwrap();
    for e in dir.flatten() {
        let Ok(st) = std::fs::read_to_string(e.path().join("stat")) else { continue };
        // fields after the ")" of the name: state is 3rd, utime 14th, stime 15th
        let Some(rest) = st.rsplit_once(')').map(|x| x.1) else { continue };
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
        let handle = std::thread::Builder::new().name("sampler".into()).spawn(move || {
            while !s.load(std::sync::atomic::Ordering::Relaxed) {
                sample_tasks(pid, &m);
                std::thread::sleep(Duration::from_millis(50));
            }
        }).unwrap();
        Threads { stop, seen, handle: Some(handle), pid }
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
            .map(|(k, &t)| (t, if own && *k == me { "main".to_string() } else { k.clone() }))
            .collect();
        v.sort_by(|a, b| b.cmp(a));
        let top = v.first().map_or(0, |x| x.0);
        let busy: Vec<String> = v.iter().filter(|x| x.0 * 50 > top).map(|(t, k)| format!("{}={:.2}", k.split(':').next().unwrap(), *t as f64 / HZ)).collect();
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

fn child(arm: &str, threads: usize, path: &str) {
    let open = || std::fs::File::open(path).unwrap();
    let mut h = Hash::new();
    let watch = if arm == "lbzip2" { None } else { Some(Threads::watch(std::process::id())) };
    let mut kid_threads = None;
    let t0 = Instant::now();
    let res = match arm {
        "mibz2" if threads == 1 => drain(mibz2::BzReader::new(open()), &mut h),
        "mibz2" => drain(mibz2::BzReader::parallel(open(), threads), &mut h),
        "mibz2-scalar" if threads == 1 => drain(mibz2::BzReader::with_kernels(open(), mibz2::Kernels::scalar()), &mut h),
        "mibz2-scalar" => {
            drain(mibz2::BzReader::parallel_with(open(), threads, 2 * threads, mibz2::Kernels::scalar()), &mut h)
        }
        "pbz2" if threads == 1 => drain(pbz2::DecoderReader::new(open()), &mut h),
        "pbz2" => drain(pbz2::ParallelDecoder::with_threads(open(), threads), &mut h),
        "lbzip2" => {
            let mut p = Command::new("lbzip2")
                .args(["-dc", "-n", &threads.to_string(), path])
                .stdout(Stdio::piped())
                .spawn()
                .expect("lbzip2 on PATH");
            let w = Threads::watch(p.id());
            let r = drain(p.stdout.take().unwrap(), &mut h);
            kid_threads = Some(w.finish(false));
            let st = p.wait().unwrap();
            r.and_then(|()| if st.success() { Ok(()) } else { Err(format!("lbzip2: {st}")) })
        }
        _ => Err(format!("unknown arm {arm}")),
    };
    let wall = t0.elapsed().as_secs_f64();
    let (nthreads, per_thread) = match (watch, kid_threads) {
        (_, Some(k)) => k,
        (Some(w), None) => w.finish(true),
        _ => (0, String::new()),
    };
    let (cpu_self, rss_self) = rusage(0);
    let (cpu_kids, rss_kids) = rusage(-1);
    // (lbzip2's arm: its own peak RSS, and its CPU with this process's hashing)
    let rss = if arm == "lbzip2" { rss_kids } else { rss_self };
    let err = res.err().unwrap_or_default().replace('"', "'");
    println!(
        "{{\"wall\":{wall:.4},\"cpu\":{:.4},\"rss_kb\":{rss},\"out\":{},\"hash\":\"{}\",\"threads_busy\":{nthreads},\"thread_cpu\":\"{per_thread}\",\"err\":\"{err}\"}}",
        cpu_self + cpu_kids,
        h.n,
        h.finish()
    );
}

/// (busy, steal, total) jiffies over all CPUs
fn stat() -> (u64, u64, u64) {
    let s = std::fs::read_to_string("/proc/stat").unwrap();
    let f: Vec<u64> = s.lines().next().unwrap().split_whitespace().skip(1).map(|x| x.parse().unwrap()).collect();
    // user nice system idle iowait irq softirq steal
    let busy = f[0] + f[1] + f[2] + f[5] + f[6] + f[7];
    (busy, f[7], f[..8].iter().sum())
}

fn load1() -> f64 {
    std::fs::read_to_string("/proc/loadavg").unwrap().split_whitespace().next().unwrap().parse().unwrap()
}

const HZ: f64 = 100.0;

/// Busy jiffies of each CPU
fn per_cpu() -> Vec<u64> {
    let s = std::fs::read_to_string("/proc/stat").unwrap();
    s.lines()
        .filter(|l| l.starts_with("cpu") && l.as_bytes().get(3).is_some_and(u8::is_ascii_digit))
        .map(|l| {
            let f: Vec<u64> = l.split_whitespace().skip(1).map(|x| x.parse().unwrap()).collect();
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
    json_str(line, key).and_then(|s| s.parse().ok()).unwrap_or(f64::NAN)
}

fn run(args: &[String]) {
    let out_path = &args[0];
    let mut arms = vec!["mibz2".to_string(), "pbz2".into(), "lbzip2".into()];
    let mut threads = vec![1usize, 4];
    let mut rounds = 5;
    let mut max_busy = 1.5;
    let mut pin = "1".to_string();
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
            f => files.push(f.to_string()),
        }
        i += 1;
    }
    let me = std::env::current_exe().unwrap();
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_path).unwrap();
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
                    let waited = Instant::now();
                    // (and no CPU more than half busy: a 1-thread run is pinned to one, and
                    // a pinned job of someone else's would share it)
                    let (mut busy, mut one) = others_busy(2.0);
                    while busy > max_busy || one > 0.5 {
                        eprintln!("  waiting: others keep {busy:.2} cores busy, one CPU {one:.2} (load {:.2})", load1());
                        std::thread::sleep(Duration::from_secs(20));
                        (busy, one) = others_busy(2.0);
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
                    let base = std::path::Path::new(file).file_stem().unwrap().to_string_lossy().to_string();
                    let rec = format!(
                        "{{\"round\":{round},\"corpus\":\"{base}\",\"threads\":{t},\"arm\":\"{arm}\",\"load1\":{load:.2},\"others_busy\":{others:.2},\"pre_busy\":{busy:.2},\"steal_pct\":{steal:.1},\"waited_s\":{:.0},{}",
                        waited.elapsed().as_secs_f64() - 2.0,
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
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145, 2.131,
        2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
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
    let reference = args.iter().position(|a| a == "--ref").map(|i| args[i + 1].clone());
    // (corpus, threads) -> arm -> round -> line
    let mut by: BTreeMap<(String, u64), BTreeMap<String, BTreeMap<u64, String>>> = BTreeMap::new();
    let mut arm_order: Vec<String> = Vec::new();
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        let key = (json_str(l, "corpus").unwrap(), json_f(l, "threads") as u64);
        let arm = json_str(l, "arm").unwrap();
        if !arm_order.contains(&arm) {
            arm_order.push(arm.clone());
        }
        by.entry(key).or_default().entry(arm).or_default().insert(json_f(l, "round") as u64, l.to_string());
    }
    let reference = reference.unwrap_or_else(|| arm_order[0].clone());
    for ((corpus, t), arms) in &by {
        let ref_runs = arms.get(&reference);
        let ref_hash = ref_runs.and_then(|r| r.values().next()).and_then(|l| json_str(l, "hash"));
        let out_mb = ref_runs.and_then(|r| r.values().next()).map_or(f64::NAN, |l| json_f(l, "out") / 1e6);
        println!("\n### {corpus}, {t} thread{} ({out_mb:.0} MB out; ratios: arm / {reference}, paired by round)\n", if *t == 1 { "" } else { "s" });
        println!("| arm | n | wall best | wall median | CPU best | CPU median | peak RSS | wall ratio [95% CI] | CPU ratio [95% CI] | others busy | same bytes |");
        println!("|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|");
        for arm in arm_order.iter().filter(|a| arms.contains_key(*a)) {
            let runs = &arms[arm];
            let col = |k: &str| runs.values().map(|l| json_f(l, k)).collect::<Vec<f64>>();
            let (mut w, mut c, mut r, mut o) = (col("wall"), col("cpu"), col("rss_kb"), col("others_busy"));
            let best = |v: &[f64]| v.iter().cloned().fold(f64::MAX, f64::min);
            let errs: Vec<String> = runs.values().filter_map(|l| json_str(l, "err")).filter(|e| !e.is_empty()).collect();
            let same = runs.values().all(|l| json_str(l, "hash") == ref_hash) && errs.is_empty();
            let pair = |k: &str| {
                let rr: Vec<f64> = runs
                    .iter()
                    .filter_map(|(round, l)| ref_runs?.get(round).map(|rl| json_f(l, k) / json_f(rl, k)))
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
            println!(
                "| {arm} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.0} MB | {} | {} | {:.2} | {} |",
                runs.len(),
                best(&w),
                median(&mut w),
                best(&c),
                median(&mut c),
                median(&mut r) / 1024.0,
                pair("wall"),
                pair("cpu"),
                median(&mut o),
                if same { "yes".to_string() } else { format!("NO {errs:?}") }
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
    let args: Vec<String> = std::env::args().skip(1).filter(|a| a != "--bench").collect();
    match args.first().map(String::as_str) {
        Some("child") => child(&args[1], args[2].parse().unwrap(), &args[3]),
        Some("run") => run(&args[1..]),
        Some("summary") => summary(&args[1..]),
        Some("gen") => gen(&args[1..]),
        _ => eprintln!("usage: ab gen DIR [SCALE] | ab run OUT.jsonl [--arms a,b] [--threads 1,4] [--rounds N] FILE.. | ab summary OUT.jsonl [--ref ARM] | ab child ARM T FILE"),
    }
}
