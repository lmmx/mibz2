//! Bandwidth titration, a diagnostic (docs/journal/2026-11-10-bandwidth-titration.md): is the
//! decoder's throughput at a given thread count set by the machine's memory bandwidth? It
//! times the shipped decoder, unchanged, beside threads that take a controlled share of that
//! bandwidth and nothing else.
//!
//! ```text
//! bw_titrate FILE.bz2 ROUNDS THREADS LEVELS HOGS REPS   one JSON line a cell (stdout)
//!     THREADS  decoder thread counts, e.g. 1,4,8,10 (0 is added: the hogs alone)
//!     LEVELS   hog duty in percent, e.g. 0,33,66,100
//!     HOGS     hog threads
//!     REPS     decodes of FILE a cell
//! ```
//!
//! **A hog** owns a 64 MiB buffer and repeats a 2 ms period: for the level's share of it, it
//! fills the buffer with non-temporal stores (written to memory without being read first, and
//! without passing through the caches); for the rest it spins. So at every level, 0 included, a
//! hog keeps one CPU busy and holds no cache: only the memory traffic changes with the level.
//! Its bytes are counted, and each cell reports the rate it reached. The hogs are pinned to the
//! CPUs of one physical core (the highest-numbered), so they share that core's fill buffers
//! with each other and with no decoder thread the scheduler keeps off a busy CPU. The decoder's
//! threads are not pinned, and the process's own CPU mask is untouched, so the decoder plans
//! its threads as it does for any user.
//!
//! **A cell** sets the level, waits 100 ms, then times REPS decodes of the file through
//! `BzReader::parallel` (`BzReader::new` at one thread) into a 1 MiB buffer, as `ab` drains a
//! decoder. Rounds rotate the order of the thread counts and of the levels.

use std::io::Cursor;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

const HOG_BYTES: usize = 64 << 20;
const CHUNK: usize = 256 << 10;
const PERIOD: Duration = Duration::from_millis(2);

static LEVEL: AtomicUsize = AtomicUsize::new(0);
static STOP: AtomicBool = AtomicBool::new(false);
static WRITTEN: AtomicU64 = AtomicU64::new(0);

/// `CHUNK` bytes from `p`, written without reading them and past the caches where the CPU can
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn fill(p: *mut u8, v: u8) {
    use std::arch::x86_64::*;
    let x = _mm256_set1_epi8(v as i8);
    let mut i = 0;
    while i < CHUNK {
        _mm256_stream_si256(p.add(i) as *mut __m256i, x);
        i += 32;
    }
    _mm_sfence();
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn fill(p: *mut u8, v: u8) {
    std::ptr::write_bytes(p, v, CHUNK);
}

/// 32 bytes, aligned as a non-temporal 256-bit store needs
#[repr(align(32))]
#[derive(Clone, Copy)]
struct Aligned(#[allow(dead_code)] [u8; 32]);

/// The CPUs of the highest-numbered physical core (its SMT siblings), where Linux lists them
fn hog_cpus() -> Vec<usize> {
    let mut last = None;
    for cpu in 0..4096 {
        let path = format!("/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list");
        match std::fs::read_to_string(path) {
            Ok(s) => last = Some(s),
            Err(_) => break,
        }
    }
    let mut cpus = Vec::new();
    for part in last
        .unwrap_or_default()
        .trim()
        .split(',')
        .filter(|p| !p.is_empty())
    {
        let mut ends = part
            .split('-')
            .map(|x| x.parse::<usize>().unwrap_or(usize::MAX));
        let (a, b) = (ends.next().unwrap(), ends.next());
        cpus.extend(a..=b.unwrap_or(a));
    }
    cpus.retain(|&c| c < 1024);
    cpus
}

/// Pin the calling thread to one CPU: true if the kernel accepted it
#[cfg(target_os = "linux")]
fn pin(cpu: usize) -> bool {
    extern "C" {
        fn sched_setaffinity(pid: i32, size: usize, mask: *const u8) -> i32;
    }
    let mut mask = [0u8; 128];
    mask[cpu / 8] |= 1 << (cpu % 8);
    unsafe { sched_setaffinity(0, mask.len(), mask.as_ptr()) == 0 }
}

#[cfg(not(target_os = "linux"))]
fn pin(_: usize) -> bool {
    false
}

fn hog(cpu: Option<usize>) {
    if let Some(c) = cpu {
        assert!(pin(c), "could not pin a hog to CPU {c}");
    }
    // touched once, so no page is first mapped inside a cell
    let mut buf = vec![Aligned([0u8; 32]); HOG_BYTES / 32];
    let p = buf.as_mut_ptr() as *mut u8;
    for i in (0..HOG_BYTES).step_by(CHUNK) {
        unsafe { fill(p.add(i), 1) };
    }
    let (mut at, mut v) = (0usize, 0u8);
    while !STOP.load(Relaxed) {
        let start = Instant::now();
        let busy = PERIOD * LEVEL.load(Relaxed) as u32 / 100;
        let mut n = 0u64;
        while start.elapsed() < busy {
            unsafe { fill(p.add(at), v) };
            at = (at + CHUNK) % HOG_BYTES;
            v = v.wrapping_add(1);
            n += CHUNK as u64;
        }
        WRITTEN.fetch_add(n, Relaxed);
        while start.elapsed() < PERIOD {
            std::hint::spin_loop();
        }
    }
    std::hint::black_box(&buf);
}

/// One decode of `data` on `threads` threads: the bytes read
fn decode(data: &Arc<[u8]>, threads: usize, buf: &mut [u8]) -> u64 {
    let mut n = 0u64;
    let mut drain = |mut r: Box<dyn Read>| loop {
        match r.read(buf).expect("decode") {
            0 => break,
            k => n += k as u64,
        }
    };
    if threads == 1 {
        drain(Box::new(mibz2::BzReader::new(Cursor::new(data.clone()))));
    } else {
        drain(Box::new(mibz2::BzReader::parallel(
            Cursor::new(data.clone()),
            threads,
        )));
    }
    n
}

fn list(s: &str) -> Vec<usize> {
    s.split(',').map(|x| x.parse().expect("a number")).collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 7 {
        eprintln!("usage: bw_titrate FILE.bz2 ROUNDS THREADS LEVELS HOGS REPS");
        std::process::exit(2);
    }
    #[cfg(target_arch = "x86_64")]
    assert!(is_x86_feature_detected!("avx"), "the hog's stores are AVX");
    let data: Arc<[u8]> = std::fs::read(&a[1]).expect("read").into();
    let rounds: usize = a[2].parse().unwrap();
    let mut threads = vec![0];
    threads.extend(list(&a[3]));
    let levels = list(&a[4]);
    assert!(levels.iter().all(|&l| l <= 100));
    let hogs: usize = a[5].parse().unwrap();
    let reps: usize = a[6].parse().unwrap();
    // the hogs share one physical core's CPUs, so the decoder keeps the others whole
    let cpus = hog_cpus();
    eprintln!("bw_titrate: {hogs} hogs on CPUs {cpus:?} (one physical core's)");
    let mut buf = vec![0u8; 1 << 20];
    // the decoded size every cell must reproduce
    let want = decode(&data, 1, &mut buf);
    std::thread::scope(|s| {
        for h in 0..hogs {
            let cpu = (!cpus.is_empty()).then(|| cpus[h % cpus.len()]);
            s.spawn(move || hog(cpu));
        }
        // the hogs' buffers are touched before the first cell
        std::thread::sleep(Duration::from_millis(500));
        for round in 0..rounds {
            for ti in 0..threads.len() {
                let t = threads[(ti + round) % threads.len()];
                for li in 0..levels.len() {
                    let level = levels[(li + round + ti) % levels.len()];
                    LEVEL.store(level, Relaxed);
                    std::thread::sleep(Duration::from_millis(100));
                    let (w0, t0) = (WRITTEN.load(Relaxed), Instant::now());
                    let mut bytes = 0;
                    if t == 0 {
                        std::thread::sleep(Duration::from_millis(500));
                    } else {
                        for _ in 0..reps {
                            let n = decode(&data, t, &mut buf);
                            assert_eq!(n, want, "a decode of another size");
                            bytes += n;
                        }
                    }
                    let (ns, w1) = (t0.elapsed().as_nanos(), WRITTEN.load(Relaxed));
                    println!(
                        "{{\"round\":{round},\"threads\":{t},\"level\":{level},\"hogs\":{hogs},\"wall_ns\":{ns},\"bytes\":{bytes},\"hog_bytes\":{}}}",
                        w1 - w0
                    );
                }
            }
        }
        STOP.store(true, Relaxed);
    });
}
