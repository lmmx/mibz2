//! Research only (`pooled-scan`): one file decoded by the plain workers, pooled or as shipped,
//! with its wall, its output's size and digest, and the pooled scan's counts.
//!
//!     cargo run --release --features pooled-scan --example pooled_scan -- FILE.bz2 THREADS [ROUNDS]
//!
//! With `--finder`, the kill test's K4 (pooled-scan/registration.md §7): the magic finder alone
//! over the whole file on the calling thread (the recipe pins it), one untimed pass to warm it
//! and the standard 100 ms pre-spin, then REPEATS timed passes, as one JSON line: each pass's ns, the median, and the interval
//! between the order statistics that cover the median with at least 95% (for 11, the 2nd and
//! the 10th: 98.8%), with the magics found.
//!
//!     cargo run --release --features pooled-scan --example pooled_scan -- --finder FILE.bz2 [REPEATS]
use mibz2::{pooled_scan, Kernels, StreamDecoder};
use std::time::Instant;

fn decode(data: &[u8], threads: usize, pooled: bool) -> (f64, u64, u64, pooled_scan::Stats) {
    pooled_scan::set(pooled);
    let dec = StreamDecoder::workers(threads, 4 * threads, Kernels::detect(), true);
    pooled_scan::set(false);
    let _ = pooled_scan::take();
    let t = Instant::now();
    let (mut n, mut h) = (0u64, 0xcbf2_9ce4_8422_2325u64);
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(4 << 20) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 4 << 20];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break,
                Ok(k) => {
                    n += k as u64;
                    for &b in &buf[..k] {
                        h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
                    }
                }
                Err(e) => panic!("decode: {e}"),
            }
        }
    });
    let wall = t.elapsed().as_secs_f64();
    (wall, n, h, pooled_scan::take())
}

/// The order statistics' ranks (1-based) that cover the median of `n` with at least 95%:
/// the widest-inward pair (j, n + 1 − j) with 1 − 2 P(Bin(n, ½) < j) ≥ 0.95, and its coverage
fn median_interval(n: usize) -> (usize, usize, f64) {
    let total = 2f64.powi(n as i32);
    let mut best = (1, n, 1.0 - 2.0 / total);
    let mut c = 1f64; // C(n, 0)
    let mut below = 0f64; // P(Bin < j) * 2^n
    for j in 1..=n / 2 {
        below += c;
        c = c * (n - j + 1) as f64 / j as f64;
        let cover = 1.0 - 2.0 * below / total;
        if cover >= 0.95 {
            best = (j, n + 1 - j, cover);
        }
    }
    best
}

fn finder(file: &str, repeats: usize) {
    let data = std::fs::read(file).expect("read the input");
    let f = pooled_scan::Finder::detect();
    let mut v = Vec::new();
    // (one pass untimed: the page cache, the tables and the vector's capacity warm)
    f.find(&data, 0, 0, data.len(), &mut v);
    let (blocks, ends) = (
        v.iter().filter(|c| !c.end).count(),
        v.iter().filter(|c| c.end).count(),
    );
    // (the standard warm-up, journal §81: the thread spun 100 ms, its clock up, before the first
    // timed pass)
    let spin = Instant::now();
    let mut x = 0u64;
    while spin.elapsed().as_millis() < 100 {
        x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(1));
    }
    let mut ns = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        v.clear();
        let t = Instant::now();
        f.find(&data, 0, 0, data.len(), &mut v);
        ns.push(t.elapsed().as_nanos() as u64);
        assert_eq!(v.len(), blocks + ends, "every pass finds the same magics");
    }
    let mut s = ns.clone();
    s.sort_unstable();
    let med = if repeats % 2 == 1 {
        s[repeats / 2] as f64
    } else {
        (s[repeats / 2 - 1] + s[repeats / 2]) as f64 / 2.0
    };
    let (lo, hi, cover) = median_interval(repeats);
    let cpu = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|t| {
            t.rsplit(')')
                .next()?
                .split_whitespace()
                .nth(36)?
                .parse::<i64>()
                .ok()
        })
        .unwrap_or(-1);
    println!(
        "{{\"kind\":\"finder\",\"bytes\":{},\"avx2\":{},\"cpu\":{cpu},\"repeats\":{repeats},\"ns\":{:?},\"median_ns\":{med:.0},\"interval_ns\":[{},{}],\"interval_ranks\":[{lo},{hi}],\"interval_coverage\":{cover:.4},\"gb_s\":{:.3},\"blocks\":{blocks},\"ends\":{ends}}}",
        data.len(),
        f.avx2(),
        ns,
        s[lo - 1],
        s[hi - 1],
        data.len() as f64 / med
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.get(1).map(String::as_str) == Some("--finder") {
        let repeats = a.get(3).map_or(11, |r| r.parse().expect("REPEATS"));
        return finder(&a[2], repeats);
    }
    let data = std::fs::read(&a[1]).expect("read the input");
    let threads: usize = a[2].parse().expect("THREADS");
    let rounds: usize = a.get(3).map_or(3, |r| r.parse().expect("ROUNDS"));
    for r in 0..rounds {
        let s = decode(&data, threads, false);
        let p = decode(&data, threads, true);
        assert_eq!((s.1, s.2), (p.1, p.2), "the same bytes");
        println!(
            "round {r}: shipped {:.3} s, pooled {:.3} s ({:.3}x); {} bytes; pooled {:?}",
            s.0,
            p.0,
            p.0 / s.0,
            p.1,
            p.3
        );
    }
}
