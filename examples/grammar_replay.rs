//! The grammar replay harness (`--features grammar-replay`; src/cells_replay.rs): every lazy
//! Cells block of each file, its grammar written by the shipped writer (W16) and by the
//! 12-byte (W12, as #506 stores them), 10-byte (W10) and 8-byte (W8) rule writers, with the tables' fit and the §6 and §7
//! censuses (#145, LOG 6924013198, item 1).
//!
//!     cargo run --release --features grammar-replay --example grammar_replay -- REPS FILE...
use mibz2::bwt::LazyCells;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let reps: usize = a.get(1).and_then(|v| v.parse().ok()).expect("REPS FILE...");
    mibz2::replay_reps(reps);
    let k = mibz2::Kernels::detect();
    for f in &a[2..] {
        let data = std::fs::read(f).expect("a readable file");
        let before = mibz2::replay_stats();
        mibz2::decode_with::<LazyCells>(&data, k).expect("a valid bzip2 stream");
        let (s, words) = mibz2::replay_stats();
        let (b, w0) = before;
        let d = |x: u64, y: u64| (x - y) as f64;
        let r = d(s.letters, b.letters);
        let g = d(s.rules, b.rules);
        println!(
            "== {f}: {} blocks replayed ({} skipped), {} letters, {} bytes, {} reps",
            s.blocks - b.blocks,
            s.skipped - b.skipped,
            s.letters - b.letters,
            s.bytes - b.bytes,
            reps
        );
        println!(
            "rules: {:.4} a letter (G/r); stored: 16 B {:.2}, 12 B {:.2}, 10 B {:.2}, 8 B {:.2} bytes a letter",
            g / r,
            16.0 * g / r,
            12.0 * g / r,
            10.0 * g / r,
            8.0 * g / r
        );
        let ns: Vec<f64> = (0..4).map(|i| d(s.ns[i], b.ns[i])).collect();
        println!(
            "writers (the fastest of the reps a block, summed): W16 {:.1} ms, W12 {:.1} ms ({:.3}), W10 {:.1} ms ({:.3}), W8 {:.1} ms ({:.3})",
            ns[0] / 1e6,
            ns[1] / 1e6,
            ns[1] / ns[0],
            ns[3] / 1e6,
            ns[3] / ns[0],
            ns[2] / 1e6,
            ns[2] / ns[0]
        );
        println!(
            "bytes differing from W16's: W12 {} blocks, W10 {} blocks, W8 {} blocks",
            s.differ[0] - b.differ[0],
            s.differ[2] - b.differ[2],
            s.differ[1] - b.differ[1]
        );
        let blocks = s.blocks - b.blocks;
        println!("fit in the touched top slots ({:.2} B a letter): 4 B a rule {}/{blocks} (past it {:.3} B a letter), 8 B a rule {}/{blocks} (past it {:.3} B a letter); in the whole room: {}/{blocks}, {}/{blocks}",
                 d(s.touched, b.touched) / r, s.fit[0] - b.fit[0], d(s.past[0], b.past[0]) / r,
                 s.fit[1] - b.fit[1], d(s.past[1], b.past[1]) / r, s.room[0] - b.room[0], s.room[1] - b.room[1]);
        let wd: Vec<f64> = (0..18).map(|i| d(words[i], w0[i])).collect();
        let made: f64 = wd.iter().sum();
        let tiny: f64 = (2..=4).map(|l| l as f64 * wd[l]).sum();
        let tiny3: f64 = (2..=3).map(|l| l as f64 * wd[l]).sum();
        let arena: f64 = (2..=16).map(|l| l as f64 * wd[l]).sum();
        println!("§6 words made: {:.4} a letter; kept as bytes by length 2..16: {}; kept as rules {:.4} a letter",
                 made / r, (2..=16).map(|l| format!("{}:{:.4}", l, wd[l] / r)).collect::<Vec<_>>().join(" "), wd[17] / r);
        println!("§6 arena bytes {:.3} a letter, of them in words of 2..4 bytes {:.3} ({:.1}%), of 2..3 bytes {:.3}",
                 arena / r, tiny / r, 100.0 * tiny / arena, tiny3 / r);
        println!("§7 runs of consecutive rules sharing the left child: {} runs, {:.1}% of rules; the right: {} runs, {:.1}% of rules",
                 s.runs[0] - b.runs[0], 100.0 * d(s.run_rules[0], b.run_rules[0]) / g,
                 s.runs[1] - b.runs[1], 100.0 * d(s.run_rules[1], b.run_rules[1]) / g);
        println!(
            "§7 the greedy partition's runs by length 1..8, 9+: {}",
            (1..10)
                .map(|i| (s.run_len[i] - b.run_len[i]).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        println!("§7 grouping saves, a letter: header 8 B + index 4 B: {:.3} B over 16-byte rules, {:.3} B over 8-byte; header 16 B + index 4 B: {:.3}, {:.3}",
                 d(s.saved16[0], b.saved16[0]) / r, d(s.saved8[0], b.saved8[0]) / r, d(s.saved16[1], b.saved16[1]) / r, d(s.saved8[1], b.saved8[1]) / r);
    }
}
