//! `cargo bench`: decode speed on corpora with nothing of Wikidata (tests/common's
//! `corpora`, scaled up), each compressed by reference bzip2 (libbzip2 1.0.8) at level 9:
//! libbzip2 itself (the bzip2 crate, one thread) against this crate on one thread (SIMD
//! kernels, the portable scalar ones, and the plain walk in place of the segmented one)
//! and on every core. Best of 3; MB/s of decoded
//! output, and cycles a byte at the CPU's nominal clock where /proc/cpuinfo gives it.
//! The machine is printed first: report it with the figures.
//!
//! `cargo bench -- SCALE` sets the corpora's size (default 4: about 8-20 MB each).

#[path = "../tests/common/mod.rs"]
mod common;

use bunzip::{bwt, Kernels};
use std::io::Read;
use std::time::Instant;

fn best<F: FnMut() -> usize>(mut f: F) -> (f64, usize) {
    let mut t = f64::MAX;
    let mut n = 0;
    for _ in 0..3 {
        let s = Instant::now();
        n = f();
        t = t.min(s.elapsed().as_secs_f64());
    }
    (t, n)
}

fn main() {
    let scale: usize = std::env::args().skip(1).find_map(|a| a.parse().ok()).unwrap_or(4);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|c| c.lines().find(|l| l.starts_with("model name")).map(|l| l.split(':').nth(1).unwrap().trim().to_string()))
        .unwrap_or_else(|| std::env::consts::ARCH.to_string());
    let ghz: Option<f64> = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|c| c.lines().find(|l| l.starts_with("cpu MHz")).and_then(|l| l.split(':').nth(1)?.trim().parse::<f64>().ok()))
        .map(|mhz| mhz / 1000.0);
    println!("machine: {cpu}, {threads} threads, {} ({:?}), kernels {:?}", std::env::consts::ARCH,
             ghz.map(|g| format!("{g:.2} GHz")), Kernels::detect());
    println!("{:<10} {:>9} {:>9}  {:>14} {:>14} {:>14} {:>14} {:>14}", "corpus", "MB out", "ratio", "libbzip2 1t",
             "bunzip 1t", "scalar 1t", "plain walk 1t", format!("bunzip {threads}t"));
    for (name, data) in common::corpora(scale) {
        let z = common::compress(&data, 9);
        let mb = data.len() as f64 / 1e6;
        let (t_ref, n) = best(|| {
            let mut out = Vec::with_capacity(data.len());
            bzip2::read::MultiBzDecoder::new(&z[..]).read_to_end(&mut out).unwrap();
            out.len()
        });
        assert_eq!(n, data.len());
        let (t_one, n) = best(|| bunzip::decode(&z).unwrap().len());
        assert_eq!(n, data.len());
        let (t_scalar, n) = best(|| bunzip::decode_with::<bwt::Segmented>(&z, Kernels::scalar()).unwrap().len());
        assert_eq!(n, data.len());
        let (t_plain, n) = best(|| bunzip::decode_with::<bwt::Plain>(&z, Kernels::detect()).unwrap().len());
        assert_eq!(n, data.len());
        let (t_par, n) = best(|| bunzip::decode_parallel(&z, threads).unwrap().len());
        assert_eq!(n, data.len());
        let cell = |t: f64| match ghz {
            Some(g) => format!("{:>6.0} MB/s {:>4.1}c", mb / t, t * g * 1e9 / data.len() as f64),
            None => format!("{:>9.0} MB/s", mb / t),
        };
        println!("{name:<10} {mb:>9.1} {:>8.1}x  {:>14} {:>14} {:>14} {:>14} {:>14}   (1t: {:.2}x libbzip2)",
                 data.len() as f64 / z.len() as f64, cell(t_ref), cell(t_one), cell(t_scalar), cell(t_plain), cell(t_par),
                 t_ref / t_one);
    }
}
