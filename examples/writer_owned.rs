//! The one-scan writer against the shipped one (`--features owned-writer`; src/cells_owned.rs;
//! #145 QUEUED 6986629119, CHECKPOINT 6986665685 and LOG 6986965565 on #314): every Cells
//! block of each file, eager and lazy, its finished grammar written by both writers on the
//! same rules (the rules restored before each rep, uncharged), the fastest of REPS each,
//! interleaved, and the bytes compared. Full-period blocks (the one-scan schedule) and the
//! others (its fallback, the shipped writer) are reported apart, and the one-scan writer's
//! scan and command pass are timed apart.
//!
//!     cargo run --release --features owned-writer --example writer_owned -- MODE REPS KERNEL FILE...
//!
//! MODE: 0 both writers, timed; 1 the shipped writer alone; 2 the one-scan writer alone; 3
//! neither (for `perf stat`: a mode's counts less mode 3's are its writer's). KERNEL: eager or
//! lazy, as admitted, or forced-eager or forced-lazy, every block (the guards and the
//! periodic corpora reach the writers only so).
use mibz2::bwt::{Cells, ForcedCells, ForcedLazyCells, LazyCells};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let usage = "MODE REPS eager|lazy|forced-eager|forced-lazy FILE...";
    let mode: usize = a.get(1).and_then(|v| v.parse().ok()).expect(usage);
    let reps: usize = a.get(2).and_then(|v| v.parse().ok()).expect(usage);
    let kernel = a.get(3).expect(usage).clone();
    mibz2::owned_mode(mode, reps);
    let k = mibz2::Kernels::detect();
    for f in &a[4..] {
        let data = std::fs::read(f).expect("a readable file");
        let (ab0, ph0) = (mibz2::owned_ab(), mibz2::owned_phases());
        let t = std::time::Instant::now();
        let out = match kernel.as_str() {
            "eager" => mibz2::decode_with::<Cells>(&data, k),
            "lazy" => mibz2::decode_with::<LazyCells>(&data, k),
            "forced-eager" => mibz2::decode_with::<ForcedCells>(&data, k),
            "forced-lazy" => mibz2::decode_with::<ForcedLazyCells>(&data, k),
            _ => panic!("{usage}"),
        }
        .expect("a valid bzip2 stream");
        let secs = t.elapsed().as_secs_f64();
        let (ab, ph) = (mibz2::owned_ab(), mibz2::owned_phases());
        let d = |i: usize| ab[i] - ab0[i];
        let e = |i: usize| ph[i] - ph0[i];
        let ms = |ns: u64| ns as f64 / 1e6;
        println!(
            "{{\"file\":{f:?},\"kernel\":{kernel:?},\"mode\":{mode},\"reps\":{reps},\"bytes\":{},\"decode_s\":{secs:.3},\
             \"blocks\":{},\"full\":{},\"differ\":{},\"full_shipped_ms\":{:.3},\"full_owned_ms\":{:.3},\
             \"rules\":{},\"full_bytes\":{},\"fallback_shipped_ms\":{:.3},\"fallback_owned_ms\":{:.3},\
             \"fallback_bytes\":{},\"owned_calls_full\":{},\"owned_calls_fallback\":{},\"commands\":{},\
             \"scan_ms_all_calls\":{:.3},\"copy_ms_all_calls\":{:.3}}}",
            out.len(),
            d(0), d(1), d(2), ms(d(3)), ms(d(4)), d(5), d(6), ms(d(7)), ms(d(8)), d(9),
            e(0), e(1), e(3), ms(e(4)), ms(e(5))
        );
        if mode == 0 {
            for (i, b) in mibz2::owned_blocks().iter().enumerate() {
                println!(
                    "{{\"file\":{f:?},\"kernel\":{kernel:?},\"block\":{i},\"rows\":{},\"rules\":{},\"full\":{},\"shipped_ns\":{},\"owned_ns\":{},\"commands\":{}}}",
                    b[0], b[1], b[2], b[3], b[4], b[5]
                );
            }
            let (s, o) = (d(3) as f64, d(4) as f64);
            eprintln!(
                "{f} {kernel}: {} blocks ({} full period, {} differing); full-period writer time, \
                 the fastest of {reps} a block, summed: shipped {:.2} ms, one-scan {:.2} ms ({:.4}); \
                 fallback blocks: shipped {:.2} ms, one-scan {:.2} ms; one-scan phases over its {} \
                 full-period calls: scan {:.2} ms, commands {:.2} ms ({} commands, {} rules a call)",
                d(0), d(1), d(2), s / 1e6, o / 1e6, o / s, ms(d(7)), ms(d(8)), e(0),
                ms(e(4)), ms(e(5)), e(3) / e(0).max(1), e(2) / e(0).max(1)
            );
        }
    }
}
