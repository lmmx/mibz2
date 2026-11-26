//! Research only (`pooled-scan`): one file decoded by the plain workers, pooled or as shipped,
//! with its wall, its output's size and digest, and the pooled scan's counts.
//!
//!     cargo run --release --features pooled-scan --example pooled_scan -- FILE.bz2 THREADS [ROUNDS]
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

fn main() {
    let a: Vec<String> = std::env::args().collect();
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
