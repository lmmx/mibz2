//! Research only: the machine's own memory ceiling, measured with the access pattern of the
//! inverse BWT's walk but none of its tables, so the knee journal can tell a workload's
//! plateau from the machine's (docs/journal/2026-10-29-libsais-knee.md).
//!
//! ```text
//! dram walk  THREADS SECONDS   each thread walks 32 dependent chains through one shared
//!                              random cyclic permutation of 2^27 u32 (512 MB, far past
//!                              the L3), each step a random line read, a prefetch of the
//!                              next, and one byte written to its own output, as the walk does
//! dram copy  THREADS SECONDS   each thread copies its own 64 MB buffer to another, over and
//!                              over: sequential reads and writes
//! ```
//!
//! Prints one line: mode, threads, seconds, steps (walk) or bytes copied, and the bytes the
//! access pattern implies (a 64-byte line a walk step, plus its byte written; a copy's bytes
//! read and written). Under `perf stat -D -1 --control fifo:CTL,ACK` with DRAM_PERF_CTL and
//! DRAM_PERF_ACK set to those fifos, it turns perf's counting on after its setup (the walk's
//! permutation, the buffers) and off when the measured seconds end, so the memory
//! controller's counts are the probe's alone (bench/dram.py).

use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const LANES: usize = 32;
const WALK_LOG2: u32 = 27;

fn xorshift(x: &mut u64) -> u64 {
    *x ^= *x << 13;
    *x ^= *x >> 7;
    *x ^= *x << 17;
    *x
}

/// A random cyclic permutation (Sattolo), so every chain is one cycle through all of it
fn cycle(n: usize) -> Vec<u32> {
    let mut p: Vec<u32> = (0..n as u32).collect();
    let mut x = 0x9e37_79b9_7f4a_7c15u64;
    for i in (1..n).rev() {
        let j = (xorshift(&mut x) % i as u64) as usize;
        p.swap(i, j);
    }
    p
}

#[inline(always)]
fn prefetch(p: *const u32) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_prefetch(p as *const i8, std::arch::x86_64::_MM_HINT_T0)
    };
    #[cfg(not(target_arch = "x86_64"))]
    let _ = p;
}

/// perf's counting on or off through its control fifo, waiting for its ack (a no-op unless
/// DRAM_PERF_CTL and DRAM_PERF_ACK are set)
fn perf_ctl(cmd: &str) {
    let (Ok(ctl), Ok(ack)) = (
        std::env::var("DRAM_PERF_CTL"),
        std::env::var("DRAM_PERF_ACK"),
    ) else {
        return;
    };
    let mut c = std::fs::OpenOptions::new()
        .write(true)
        .open(ctl)
        .expect("perf's control fifo");
    writeln!(c, "{cmd}").unwrap();
    let mut line = String::new();
    std::io::BufReader::new(std::fs::File::open(ack).expect("perf's ack fifo"))
        .read_line(&mut line)
        .unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("walk");
    let threads: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let secs: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(2.0);
    let stop = AtomicBool::new(false);
    match mode {
        "walk" => {
            let n = 1usize << WALK_LOG2;
            let t = cycle(n);
            let t = &t[..];
            perf_ctl("enable");
            let start = Instant::now();
            let steps: u64 = std::thread::scope(|s| {
                let hs: Vec<_> = (0..threads)
                    .map(|w| {
                        let stop = &stop;
                        s.spawn(move || {
                            let mut x = 0x2545_f491_4f6c_dd1du64 ^ (w as u64 + 1);
                            let mut rows: [u32; LANES] =
                                std::array::from_fn(|_| (xorshift(&mut x) % n as u64) as u32);
                            let mut out = vec![0u8; 1 << 20];
                            let (mut steps, mut at) = (0u64, 0usize);
                            while !stop.load(Ordering::Relaxed) {
                                for _ in 0..1024 {
                                    for k in 0..LANES {
                                        let q = t[rows[k] as usize];
                                        rows[k] = q;
                                        prefetch(&t[q as usize]);
                                        out[at] = q as u8;
                                        at = (at + 1) & ((1 << 20) - 1);
                                    }
                                }
                                steps += (1024 * LANES) as u64;
                            }
                            std::hint::black_box(&out);
                            steps
                        })
                    })
                    .collect();
                std::thread::sleep(Duration::from_secs_f64(secs));
                stop.store(true, Ordering::Relaxed);
                hs.into_iter().map(|h| h.join().unwrap()).sum()
            });
            let el = start.elapsed().as_secs_f64();
            perf_ctl("disable");
            println!(
                "walk threads {threads} secs {el:.3} steps {steps} lines_GB/s {:.2} steps_M/s {:.1}",
                steps as f64 * 64.0 / el / 1e9,
                steps as f64 / el / 1e6
            );
        }
        "copy" => {
            let len = 64usize << 20;
            let ready = std::sync::Barrier::new(threads + 1);
            let mut start = Instant::now();
            let copied: u64 = std::thread::scope(|s| {
                let hs: Vec<_> = (0..threads)
                    .map(|_| {
                        let (stop, ready) = (&stop, &ready);
                        s.spawn(move || {
                            let a = vec![1u8; len];
                            let mut b = vec![0u8; len];
                            b.copy_from_slice(&a); // every page touched before the timing
                            ready.wait();
                            let mut copied = 0u64;
                            while !stop.load(Ordering::Relaxed) {
                                b.copy_from_slice(&a);
                                std::hint::black_box(&b);
                                copied += len as u64;
                            }
                            copied
                        })
                    })
                    .collect();
                ready.wait();
                perf_ctl("enable");
                start = Instant::now();
                std::thread::sleep(Duration::from_secs_f64(secs));
                stop.store(true, Ordering::Relaxed);
                hs.into_iter().map(|h| h.join().unwrap()).sum()
            });
            let el = start.elapsed().as_secs_f64();
            perf_ctl("disable");
            println!(
                "copy threads {threads} secs {el:.3} bytes {copied} read+write_GB/s {:.2}",
                2.0 * copied as f64 / el / 1e9
            );
        }
        _ => panic!("dram walk|copy THREADS SECONDS"),
    }
}
