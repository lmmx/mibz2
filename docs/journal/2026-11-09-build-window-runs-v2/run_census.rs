//! The run census behind docs/journal/2026-11-09-build-window-runs-v2.md (scratch, not
//! built: copy into examples/ to run). Usage: run_census FILE.bz2..
//! For each file, the 16-row windows of every block's last column by their number of runs
//! (1 to 16), and the share of windows each classifier step decides.
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
fn main() {
    let k = Kernels::detect();
    for f in std::env::args().skip(1) {
        let data = std::fs::read(&f).unwrap();
        let mut sc = Scanner::new(SliceSource(&data));
        let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
        let mut hist = [0u64; 17];
        let mut blocks = 0;
        loop {
            let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap()
            else {
                break;
            };
            mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            syms = b.syms;
            blocks += 1;
            for w in ll.chunks_exact(16) {
                let r = 1 + (1..16).filter(|&j| w[j] != w[j - 1]).count();
                hist[r] += 1;
            }
        }
        let tot: u64 = hist.iter().sum();
        let pct = |x: u64| 100.0 * x as f64 / tot as f64;
        println!("{f}: {blocks} blocks, {tot} windows");
        println!(
            "  1 run {:.2}%, 2-6 runs {:.2}%, 7-15 runs {:.2}%, 16 runs {:.2}%",
            pct(hist[1]),
            pct(hist[2..=6].iter().sum()),
            pct(hist[7..=15].iter().sum()),
            pct(hist[16])
        );
        println!("  by runs: {:?}", &hist[1..]);
    }
}
