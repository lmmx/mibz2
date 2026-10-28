//! The contention fingerprint's competitors and its calibration (bench/fingerprint.py):
//! deterministic, bounded in memory and time, std only.
//!
//! ```text
//! stress chase [--sizes 256K,384K,..] [--ms 20]
//!     a pointer chase (one random cycle through 64-byte lines, each load's address the
//!     last load's value) at each size: one JSON line a size, {"bytes","ns","cpu"}, ns
//!     the median of 3 timed stretches of `ms`/3 each after a warming lap, cpu the least
//!     of their thread CPU time over wall time (under 1: it was preempted)
//! stress compute|cache|stream [--bytes N] [--timeout S]
//!     compute  a register-only integer loop: 4 multiply-rotate chains, no memory traffic
//!     cache    random read-modify-writes of 64-byte lines over a working set of N bytes
//!              (default 8M): 8 independent xorshift (LFSR) streams, unrolled, so the
//!              accesses overlap (Bubble-Up's bubble)
//!     stream   a sequential read and write over two arrays of N bytes each (default 64M):
//!              b[i] = a[i] + 1, then a[i] = b[i] + 1, STREAM-like
//! ```
//!
//! A stressor touches its memory, prints `{"ready":..}`, then runs until its stdin closes
//! (the orchestrator exits, or closes the pipe), or until `--timeout` seconds (default 300),
//! when it exits with status 3. For every line read on stdin it prints
//! `{"t":SECONDS,"n":WORK}`: the seconds since ready and the work done so far (compute:
//! iterations of one chain; cache: read-modify-writes; stream: bytes read plus bytes
//! written, without the stores' line reads), so its rate over any window can be read.

use std::hint::black_box;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// "256K", "8M", "1G" or bytes
fn size(s: &str) -> usize {
    let (n, m) = match s.chars().last() {
        Some('K' | 'k') => (&s[..s.len() - 1], 1 << 10),
        Some('M' | 'm') => (&s[..s.len() - 1], 1 << 20),
        Some('G' | 'g') => (&s[..s.len() - 1], 1 << 30),
        _ => (s, 1),
    };
    let v: f64 = n
        .parse()
        .unwrap_or_else(|_| fail(&format!("not a size: {s}")));
    (v * m as f64) as usize
}

fn fail(msg: &str) -> ! {
    eprintln!("stress: {msg}");
    std::process::exit(2)
}

#[inline(always)]
fn xorshift(x: &mut u64) -> u64 {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    *x
}

/// An index below `n` (< 2^32) from the high half of a 64-bit random word, without a division
#[inline(always)]
fn below(x: u64, n: usize) -> usize {
    (((x >> 32) * n as u64) >> 32) as usize
}

#[repr(C)]
struct Timespec {
    sec: i64,
    nsec: i64,
}

extern "C" {
    fn clock_gettime(clock: i32, ts: *mut Timespec) -> i32;
}

/// This thread's CPU time, seconds (CLOCK_THREAD_CPUTIME_ID)
fn thread_cpu() -> f64 {
    let mut ts = Timespec { sec: 0, nsec: 0 };
    // SAFETY: a plain out-parameter of the size the kernel writes (struct timespec, LP64)
    unsafe { clock_gettime(3, &mut ts) };
    ts.sec as f64 + ts.nsec as f64 * 1e-9
}

const SIZES: &str = "256K,384K,512K,768K,1M,1536K,2M,3M,4M,6M,8M,12M,16M,24M,32M,48M,64M";

fn chase(sizes: &[usize], ms: f64) {
    let max = *sizes.iter().max().unwrap();
    // one u64 a line used (its first word: the next line's word index); a line a node
    let mut buf = vec![0u64; max / 8];
    let mut perm: Vec<u32> = Vec::new();
    let mut rng = 0x2545_f491_4f6c_dd1du64;
    for &bytes in sizes {
        let nodes = (bytes / 64).max(2);
        // Sattolo's shuffle: a uniformly random cyclic permutation, one cycle through all
        perm.clear();
        perm.extend(0..nodes as u32);
        for i in (1..nodes).rev() {
            let j = below(xorshift(&mut rng), i);
            perm.swap(i, j);
        }
        for (i, &p) in perm.iter().enumerate() {
            buf[i * 8] = p as u64 * 8;
        }
        let mut p = 0usize;
        let step = |p: &mut usize, k: usize, buf: &[u64]| {
            for _ in 0..k {
                *p = buf[*p] as usize;
            }
        };
        // a warming lap (at most 2^18 loads)
        step(&mut p, nodes.min(1 << 18), &buf);
        let mut ns = Vec::new();
        let mut cpu = 1f64;
        for _ in 0..3 {
            let t0 = Instant::now();
            let c0 = thread_cpu();
            let mut n = 0usize;
            let budget = Duration::from_secs_f64(ms / 3.0 / 1e3);
            while t0.elapsed() < budget {
                step(&mut p, 4096, &buf);
                n += 4096;
            }
            let wall = t0.elapsed().as_secs_f64();
            cpu = cpu.min((thread_cpu() - c0) / wall);
            ns.push(wall * 1e9 / n as f64);
        }
        black_box(p);
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // (cpu: the least of the stretches' CPU over wall; under 1, the chase was preempted)
        println!("{{\"bytes\":{bytes},\"ns\":{:.3},\"cpu\":{cpu:.3}}}", ns[1]);
    }
}

/// A competitor: its work counter, read by the stdin thread; stdin closing ends the process
fn serve(done: Arc<AtomicU64>, t0: Instant) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line = String::new();
        let mut lock = stdin.lock();
        loop {
            line.clear();
            match lock.read_line(&mut line) {
                Ok(0) | Err(_) => std::process::exit(0),
                Ok(_) => {
                    let n = done.load(Relaxed);
                    let t = t0.elapsed().as_secs_f64();
                    let mut o = std::io::stdout().lock();
                    let _ = writeln!(o, "{{\"t\":{t:.6},\"n\":{n}}}");
                    let _ = o.flush();
                }
            }
        }
    });
}

fn ready(kind: &str, bytes: usize, unit: &str) -> (Arc<AtomicU64>, Instant) {
    let done = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    println!("{{\"ready\":\"{kind}\",\"bytes\":{bytes},\"unit\":\"{unit}\"}}");
    let _ = std::io::stdout().flush();
    serve(done.clone(), t0);
    (done, t0)
}

fn check_timeout(t0: Instant, timeout: Duration) {
    if t0.elapsed() > timeout {
        eprintln!("stress: the {:.0} s timeout passed", timeout.as_secs_f64());
        std::process::exit(3);
    }
}

fn compute(timeout: Duration) {
    let (done, t0) = ready("compute", 0, "iterations");
    let mut x = [0x9e37_79b9_7f4a_7c15u64, 3, 5, 7];
    let mut n = 0u64;
    loop {
        for _ in 0..(1 << 16) {
            for v in x.iter_mut() {
                *v = v
                    .wrapping_mul(0xd134_2543_de82_ef95)
                    .wrapping_add(1)
                    .rotate_left(17);
            }
        }
        x = black_box(x);
        n += 1 << 16;
        done.store(n, Relaxed);
        check_timeout(t0, timeout);
    }
}

fn cache(bytes: usize, timeout: Duration) {
    let lines = (bytes / 64).max(1);
    if lines >= 1 << 32 {
        fail("cache: at most 256 GB");
    }
    let mut buf = vec![0u64; lines * 8];
    for i in (0..buf.len()).step_by(8) {
        buf[i] = i as u64; // every line touched (and so every page)
    }
    let (done, t0) = ready("cache", lines * 64, "rmw");
    let mut s = [0u64; 8];
    for (k, v) in s.iter_mut().enumerate() {
        *v = 0x9e37_79b9_7f4a_7c15u64.wrapping_mul(k as u64 + 1) | 1;
    }
    let mut n = 0u64;
    loop {
        for _ in 0..4096 {
            // 8 independent streams: 8 read-modify-writes in flight at once
            for v in s.iter_mut() {
                let i = below(xorshift(v), lines) * 8;
                buf[i] = buf[i].wrapping_add(1);
            }
        }
        n += 4096 * 8;
        done.store(n, Relaxed);
        check_timeout(t0, timeout);
    }
}

fn stream(bytes: usize, timeout: Duration) {
    let len = (bytes / 8).max(1024);
    let mut a = vec![1u64; len];
    let mut b = vec![0u64; len];
    for i in (0..len).step_by(512) {
        b[i] = 1; // every page touched
    }
    let (done, t0) = ready("stream", len * 8, "bytes");
    let mut n = 0u64;
    const CHUNK: usize = 8192;
    loop {
        // a -> b, then b -> a, a chunk at a time (the counter kept current)
        for pass in 0..2 {
            let (src, dst) = if pass == 0 {
                (&a, &mut b)
            } else {
                (&b, &mut a)
            };
            for (s, d) in src.chunks(CHUNK).zip(dst.chunks_mut(CHUNK)) {
                for (x, y) in s.iter().zip(d.iter_mut()) {
                    *y = x.wrapping_add(1);
                }
                n += 16 * s.len() as u64;
                done.store(n, Relaxed);
            }
            black_box(&mut a);
            black_box(&mut b);
            check_timeout(t0, timeout);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter().position(|a| a == name).map(|i| {
            args.get(i + 1)
                .cloned()
                .unwrap_or_else(|| fail(&format!("{name} needs a value")))
        })
    };
    let timeout = Duration::from_secs_f64(flag("--timeout").map_or(300.0, |s| {
        s.parse().unwrap_or_else(|_| fail("--timeout: seconds"))
    }));
    match args.first().map(String::as_str) {
        Some("chase") => {
            let sizes: Vec<usize> = flag("--sizes").as_deref().unwrap_or(SIZES).split(',').map(size).collect();
            let ms = flag("--ms").map_or(20.0, |s| s.parse().unwrap_or_else(|_| fail("--ms: milliseconds")));
            if sizes.iter().any(|&s| s > 1 << 30) {
                fail("chase: at most 1G");
            }
            chase(&sizes, ms)
        }
        Some("compute") => compute(timeout),
        Some("cache") => cache(size(flag("--bytes").as_deref().unwrap_or("8M")), timeout),
        Some("stream") => stream(size(flag("--bytes").as_deref().unwrap_or("64M")), timeout),
        _ => fail("usage: stress chase [--sizes 256K,..] [--ms 20] | stress compute|cache|stream [--bytes N] [--timeout S]"),
    }
}
