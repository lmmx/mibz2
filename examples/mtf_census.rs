//! A census of MTF's input (research, not a benchmark): for a .bz2's blocks, every
//! RUNA/RUNB symbol and every move-to-front index the MTF stage decodes, the runs' lengths,
//! symbols per byte of `ll` and of output, the memory-path pattern for a register front of
//! 16, 32 or 64 bytes, and how predictable the run/byte branch is from its own history.
//!
//! ```text
//! mtf_census FILE.bz2 [BLOCK...]   (no BLOCK: every block, summed)
//! ```
//!
//! Each block is also decoded (`decode_block`), so its `ll` and output lengths are the
//! decoder's own, and the census's own reconstruction of the runs is checked against
//! `mtf::mtf`'s `ll` length.

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, mtf, Kernels, Scratch};

/// The history lengths the branch-predictability bound is computed for: of the branch's
/// own outcomes, and of 2-bit symbol classes
const HIST: [usize; 6] = [0, 1, 2, 4, 8, 12];
const CLASS_HIST: [usize; 6] = [0, 1, 2, 4, 6, 8];
/// Register fronts: the list's first F bytes held in registers, the rest in memory
const FRONTS: [usize; 4] = [16, 32, 48, 64];

#[derive(Clone)]
struct Census {
    blocks: u64,
    /// symbols before the end-of-block symbol
    syms: u64,
    runa: u64,
    runb: u64,
    /// byte symbols (an MTF index of at least 1) by index
    idx: [u64; 256],
    /// runs (maximal groups of RUNA/RUNB symbols) by length in bytes, log2 buckets
    run_len: [u64; 24],
    /// runs by their count of RUNA/RUNB symbols
    run_syms: [u64; 24],
    runs: u64,
    run_bytes: u64,
    /// runs of at most 64 bytes (the two-store path)
    short_runs: u64,
    ll: u64,
    out: u64,
    /// per front F: byte symbols at or past F, and those whose previous byte symbol was too
    far: [u64; 4],
    far_after_far: [u64; 4],
    /// per front F: byte symbols past F with no byte symbol past F within the previous
    /// 1, 2, 4, 8 byte symbols (how far apart the memory path's steps are)
    far_gap: [[u64; 4]; 4],
    /// for the count's read-modify-write: byte symbols whose byte was last counted 2, 3,
    /// 4-7, 8+ symbols before (runs included as one step)
    count_dist: [u64; 4],
    /// the run/byte branch: best static k-history predictor's misses, per HIST
    hist_miss: [u64; 6],
    /// the same, with 2-bit symbol classes (run, index < 16, < 32, >= 32) as the history
    class_miss: [u64; 6],
    /// symbols whose kind (run/byte) differs from the previous symbol's
    switches: u64,
}

impl Census {
    fn new() -> Self {
        Census {
            blocks: 0,
            syms: 0,
            runa: 0,
            runb: 0,
            idx: [0; 256],
            run_len: [0; 24],
            run_syms: [0; 24],
            runs: 0,
            run_bytes: 0,
            short_runs: 0,
            ll: 0,
            out: 0,
            far: [0; 4],
            far_after_far: [0; 4],
            far_gap: [[0; 4]; 4],
            count_dist: [0; 4],
            hist_miss: [0; 6],
            class_miss: [0; 6],
            switches: 0,
        }
    }

    fn add(&mut self, o: &Census) {
        self.blocks += o.blocks;
        self.syms += o.syms;
        self.runa += o.runa;
        self.runb += o.runb;
        for i in 0..256 {
            self.idx[i] += o.idx[i];
        }
        for i in 0..24 {
            self.run_len[i] += o.run_len[i];
            self.run_syms[i] += o.run_syms[i];
        }
        self.runs += o.runs;
        self.run_bytes += o.run_bytes;
        self.short_runs += o.short_runs;
        self.ll += o.ll;
        self.out += o.out;
        for f in 0..4 {
            self.far[f] += o.far[f];
            self.far_after_far[f] += o.far_after_far[f];
            for g in 0..4 {
                self.far_gap[f][g] += o.far_gap[f][g];
            }
            self.count_dist[f] += o.count_dist[f];
        }
        for h in 0..6 {
            self.hist_miss[h] += o.hist_miss[h];
            self.class_miss[h] += o.class_miss[h];
        }
        self.switches += o.switches;
    }
}

fn log2_bucket(x: u64) -> usize {
    (63 - x.max(1).leading_zeros() as usize).min(23)
}

/// The best static predictor with k bits of history (trained on the sequence itself): for
/// each context, the minority outcome's count is its misses. An optimistic bound for a
/// history predictor of that depth; `ctx_bits` bits of context per step.
fn static_misses(outcome: &[u8], ctx: &[u8], ctx_bits: usize, k: usize) -> u64 {
    let width = ctx_bits * k;
    assert!(width <= 24);
    let mut table = vec![[0u32; 2]; 1 << width];
    let mask = (1u64 << width) - 1;
    let mut h = 0u64;
    for (i, &o) in outcome.iter().enumerate() {
        table[(h & mask) as usize][o as usize] += 1;
        h = (h << ctx_bits) | ctx[i] as u64;
    }
    table.iter().map(|c| c[0].min(c[1]) as u64).sum()
}

fn census_block(b: &mibz2::block::Symbols) -> (Census, usize) {
    let mut c = Census::new();
    c.blocks = 1;
    // a list of the bytes, to know each output's byte for the count distances
    let mut list: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut last_counted = [u64::MAX; 256];
    let (mut run, mut weight, mut nrun) = (0u64, 1u64, 0usize);
    let mut ll = 0u64;
    // per front: byte symbols since the last far one
    let mut since_far = [u64::MAX; 4];
    let mut prev_far = [false; 4];
    let mut is_run = Vec::with_capacity(b.syms.len());
    let mut class = Vec::with_capacity(b.syms.len());
    let mut step = 0u64;
    let mut prev_kind: Option<bool> = None;
    for &s in &b.syms {
        if s == b.eob {
            if run != 0 {
                c.runs += 1;
                c.run_bytes += run;
                c.run_len[log2_bucket(run)] += 1;
                c.run_syms[nrun.min(23)] += 1;
                c.short_runs += (run <= 64) as u64;
                ll += run;
            }
            break;
        }
        c.syms += 1;
        let kind = s <= 1;
        if prev_kind.is_some_and(|p| p != kind) {
            c.switches += 1;
        }
        prev_kind = Some(kind);
        is_run.push(kind as u8);
        if s <= 1 {
            if s == 0 {
                c.runa += 1;
            } else {
                c.runb += 1;
            }
            run += (s as u64 + 1) * weight;
            weight <<= 1;
            nrun += 1;
            class.push(0);
            continue;
        }
        if run != 0 {
            c.runs += 1;
            c.run_bytes += run;
            c.run_len[log2_bucket(run)] += 1;
            c.run_syms[nrun.min(23)] += 1;
            c.short_runs += (run <= 64) as u64;
            ll += run;
            let byte = list[0];
            last_counted[byte as usize] = step;
            step += 1;
            (run, weight, nrun) = (0, 1, 0);
        }
        let nn = (s - 1) as usize;
        c.idx[nn] += 1;
        class.push(if nn < 16 {
            1
        } else if nn < 32 {
            2
        } else {
            3
        });
        for (f, &front) in FRONTS.iter().enumerate() {
            let far = nn >= front;
            if far {
                c.far[f] += 1;
                if prev_far[f] {
                    c.far_after_far[f] += 1;
                }
                // gaps: none within the last 1, 2, 4, 8 byte symbols
                for (g, &w) in [1u64, 2, 4, 8].iter().enumerate() {
                    if since_far[f] >= w {
                        c.far_gap[f][g] += 1;
                    }
                }
                since_far[f] = 0;
            } else {
                since_far[f] = since_far[f].saturating_add(1);
            }
            prev_far[f] = far;
        }
        let v = list[nn];
        list.copy_within(0..nn, 1);
        list[0] = v;
        let d = step.wrapping_sub(last_counted[v as usize]);
        if last_counted[v as usize] != u64::MAX {
            c.count_dist[match d {
                0..=2 => 0,
                3 => 1,
                4..=7 => 2,
                _ => 3,
            }] += 1;
        }
        last_counted[v as usize] = step;
        step += 1;
        ll += 1;
    }
    for h in 0..6 {
        c.hist_miss[h] = static_misses(&is_run, &is_run, 1, HIST[h]);
        c.class_miss[h] = static_misses(&is_run, &class, 2, CLASS_HIST[h]);
    }
    c.ll = ll;
    (c, ll as usize)
}

fn pct(a: u64, b: u64) -> String {
    format!("{:.2}%", 100.0 * a as f64 / b.max(1) as f64)
}

fn report(name: &str, c: &Census) {
    let bytesyms: u64 = c.idx.iter().sum();
    let runsyms = c.runa + c.runb;
    println!("== {name}: {} blocks", c.blocks);
    println!(
        "symbols {} (before EOB): RUNA {} + RUNB {} = {} run symbols ({}), byte symbols {} ({})",
        c.syms,
        c.runa,
        c.runb,
        runsyms,
        pct(runsyms, c.syms),
        bytesyms,
        pct(bytesyms, c.syms)
    );
    println!(
        "ll bytes {}, output bytes {} (RLE1 x{:.4}); symbols per ll byte {:.4}, per output byte {:.4}; byte symbols per output byte {:.4}",
        c.ll,
        c.out,
        c.out as f64 / c.ll as f64,
        c.syms as f64 / c.ll as f64,
        c.syms as f64 / c.out as f64,
        bytesyms as f64 / c.out as f64
    );
    println!(
        "runs {} ({} bytes, {} of ll; mean {:.2} bytes, {:.3} run symbols per run); runs <= 64 bytes {}",
        c.runs,
        c.run_bytes,
        pct(c.run_bytes, c.ll),
        c.run_bytes as f64 / c.runs.max(1) as f64,
        runsyms as f64 / c.runs.max(1) as f64,
        pct(c.short_runs, c.runs)
    );
    print!("run length (bytes), log2 buckets:");
    for (i, &n) in c.run_len.iter().enumerate() {
        if n > 0 {
            print!(" [{}..{}]:{} ({})", 1u64 << i, (2u64 << i) - 1, n, pct(n, c.runs));
        }
    }
    println!();
    print!("run symbols per run:");
    for (i, &n) in c.run_syms.iter().enumerate() {
        if n > 0 {
            print!(" {i}:{n} ({})", pct(n, c.runs));
        }
    }
    println!();
    let r = |lo: usize, hi: usize| c.idx[lo..=hi].iter().sum::<u64>();
    let classes = [
        ("1", 1, 1),
        ("2", 2, 2),
        ("3", 3, 3),
        ("4-7", 4, 7),
        ("8-15", 8, 15),
        ("16-31", 16, 31),
        ("32-63", 32, 63),
        ("64-127", 64, 127),
        ("128-255", 128, 255),
    ];
    print!("MTF index of byte symbols (index 0 is a run):");
    for (n, lo, hi) in classes {
        print!(" {n}:{} ({})", r(lo, hi), pct(r(lo, hi), bytesyms));
    }
    println!();
    println!(
        "  as shares of all symbols: run {} | 1 {} | 2-15 {} | 16-31 {} | 32-255 {}",
        pct(runsyms, c.syms),
        pct(r(1, 1), c.syms),
        pct(r(2, 15), c.syms),
        pct(r(16, 31), c.syms),
        pct(r(32, 255), c.syms)
    );
    let mean: f64 = c
        .idx
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / bytesyms as f64;
    let moved: f64 = c
        .idx
        .iter()
        .enumerate()
        .filter(|&(i, _)| i >= 32)
        .map(|(i, &n)| n as f64 * i as f64)
        .sum::<f64>();
    println!(
        "  mean index {:.2}; bytes memmoved by the >= 32 path {:.0} ({:.2} a byte symbol)",
        mean,
        moved,
        moved / bytesyms as f64
    );
    for (f, &front) in FRONTS.iter().enumerate() {
        println!(
            "front {front}: byte symbols past it {} ({} of byte symbols, {} of symbols); after another past it {} ({} of them); none past it in the previous 1/2/4/8 byte symbols: {} / {} / {} / {}",
            c.far[f],
            pct(c.far[f], bytesyms),
            pct(c.far[f], c.syms),
            c.far_after_far[f],
            pct(c.far_after_far[f], c.far[f]),
            pct(c.far_gap[f][0], c.far[f]),
            pct(c.far_gap[f][1], c.far[f]),
            pct(c.far_gap[f][2], c.far[f]),
            pct(c.far_gap[f][3], c.far[f]),
        );
    }
    let cd: u64 = c.count_dist.iter().sum();
    println!(
        "count RMW distance (outputs since the byte's last count): 2: {} | 3: {} | 4-7: {} | 8+: {}",
        pct(c.count_dist[0], cd),
        pct(c.count_dist[1], cd),
        pct(c.count_dist[2], cd),
        pct(c.count_dist[3], cd)
    );
    println!(
        "run/byte kind switches {} ({} of symbols)",
        c.switches,
        pct(c.switches, c.syms)
    );
    print!("run/byte branch, best static predictor misses per symbol, by history length (own outcomes):");
    for (h, &k) in HIST.iter().enumerate() {
        print!(" k={k}: {}", pct(c.hist_miss[h], c.syms));
    }
    println!();
    print!("  (history of classes run/<16/<32/>=32):");
    for (h, &k) in CLASS_HIST.iter().enumerate() {
        print!(" k={k}: {}", pct(c.class_miss[h], c.syms));
    }
    println!();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let want: Vec<u64> = a[2..].iter().map(|s| s.parse().unwrap()).collect();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut total = Census::new();
    let mut scratch: Scratch = Scratch::default();
    let mut out = Vec::new();
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    // the full index histogram, for the record
    loop {
        let (index, b) = match sc.next(syms).unwrap() {
            Event::Block { index, symbols, .. } => (index, symbols),
            Event::Done => break,
        };
        if want.is_empty() || want.contains(&index) {
            let (mut c, n) = census_block(&b);
            let m = mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            assert_eq!(m, n, "the census's ll length");
            out.clear();
            decode_block(&b, k, &mut scratch, &mut out).unwrap();
            c.out = out.len() as u64;
            if !want.is_empty() {
                report(&format!("block {index}"), &c);
            }
            total.add(&c);
        }
        syms = b.syms;
    }
    report(&format!("{} (selected blocks summed)", a[1]), &total);
    print!("index histogram (index:count, byte symbols):");
    for (i, &n) in total.idx.iter().enumerate() {
        if n > 0 {
            print!(" {i}:{n}");
        }
    }
    println!();
}
