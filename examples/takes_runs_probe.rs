#![forbid(unsafe_code)]
//! Smooth-zone `explore` probe (zero `src/` edits, `#![forbid(unsafe_code)]`):
//! Post-#1080 `runs::takes_runs` (`Column` vs `Runs` route boundary) dissection and
//! candidate O(1) pre-MTF admission rule evaluation across 14 deterministic corpora.
//!
//! # Frozen Discovery / Stop Test (frozen BEFORE reading any timing data)
//! - **PASS** (open follow-on P2 `idea` claim `post-1080-takes-runs` on `src/runs.rs`):
//!   at least one simple, zero-extra-memory O(1) pre-MTF rule (100% byte+CRC exact across
//!   all 14 corpora, using only `block::Symbols` fields + at most `takes_runs`'s existing
//!   `<= 4096`-symbol sampled run-share check) cuts MTF + inverse-BWT CPU (steal-free
//!   `/proc/thread-self/schedstat`, `taskset -c 0`, CPU-calibrated inner passes
//!   `>= 240 ms/sample`, 5 rotated interleaved repeats) by:
//!   - `>= 10.0%` (disjoint FASTER) on `mdjournal` AND `>= 10.0%` (disjoint FASTER) on `elf`,
//!   - with no regression `> +1.5%` (and zero disjoint SLOWER) across any of the other 12
//!     corpora (`wikidata`, `text`, `prose`, `pysrc`, `rustsrc`, `json`, `logs`, `random`,
//!     `records`, `runs`, `onebyte`, `pattern`),
//!   - across ALL THREE builds: baseline x86-64, `-C target-cpu=native`, and JCC-32B-aligned
//!     (`-C llvm-args=-x86-branches-within-32B-boundaries`).
//! - **STOP** (record a scoped dead-end row in `frontier/dead-ends.tsv`): no O(1) pre-MTF
//!   rule clears the `>= 10.0%` disjoint bar on both `mdjournal` and `elf` across all 3 builds
//!   without regressing any of the other 12 corpora by `> +1.5%`.
//!
//! # Usage
//! ```bash
//! cargo run --release --features moonshot --example takes_runs_probe -- [--reps 5] [--eval-rules] [--dump-blocks] FILE.bz2...
//! ```

#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::block::Symbols;
    use mibz2::bwt::{ForcedOpenCells, InverseBwt, OpenCells, Segmented};
    use mibz2::crc;
    use mibz2::mtf;
    use mibz2::rle;
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::fs::File;
    use std::os::unix::fs::FileExt;
    use std::path::Path;
    use std::time::Instant;

    const WINDOW: usize = 64;
    const WINDOWS: usize = 64;
    const LIMIT: u32 = 16;
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;

    /// Pure-safe steal-free Linux on-CPU thread clock via `/proc/thread-self/schedstat`.
    struct CpuClock {
        file: Option<File>,
    }

    impl CpuClock {
        fn open() -> Self {
            let file = File::open("/proc/thread-self/schedstat")
                .or_else(|_| File::open("/proc/self/schedstat"))
                .ok();
            Self { file }
        }

        fn now_ns(&self) -> u64 {
            let Some(ref f) = self.file else {
                return 0;
            };
            let mut buf = [0u8; 128];
            let Ok(n) = f.read_at(&mut buf, 0) else {
                return 0;
            };
            let Ok(s) = std::str::from_utf8(&buf[..n]) else {
                return 0;
            };
            s.split_whitespace()
                .next()
                .and_then(|tok| tok.parse::<u64>().ok())
                .unwrap_or(0)
        }
    }

    fn median_f64(vals: &mut [f64]) -> f64 {
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = vals.len();
        if n == 0 {
            return 0.0;
        }
        if n % 2 == 1 {
            vals[n / 2]
        } else {
            0.5 * (vals[n / 2 - 1] + vals[n / 2])
        }
    }

    /// Sampled `(run_syms, sampled_total)` using `src/runs.rs::takes_runs_slice`'s exact
    /// 64 x 64 = 4,096-symbol evenly spaced window layout (O(1), zero allocation).
    fn sample_run_share(syms: &[u16]) -> (usize, usize) {
        let len = syms.len();
        if len == 0 {
            return (0, 0);
        }
        if len <= WINDOW * WINDOWS {
            let runs: usize = syms.iter().map(|&s| usize::from(s <= 1)).sum();
            (runs, len)
        } else {
            let step = len / WINDOWS;
            let mut runs = 0usize;
            for i in 0..WINDOWS {
                let w = &syms[i * step..i * step + WINDOW];
                runs += w.iter().map(|&s| usize::from(s <= 1)).sum::<usize>();
            }
            (runs, WINDOW * WINDOWS)
        }
    }

    /// Exact early-stopping match of `src/open.rs::within_cap` for combined R2_2tier_m22 check.
    fn within_cap_live(records: &[u16], p: u32, cap_m: u32, cap_strips: usize) -> bool {
        let (mut pos, mut m, mut ns) = (0u32, 0u32, 0usize);
        let mut i = 0;
        while i < records.len() {
            let rec = records[i];
            let c = rec >> 8;
            let mut width = u32::from(rec & 0xFF);
            i += 1;
            while i < records.len() && (records[i] >> 8) == c {
                width += u32::from(records[i] & 0xFF);
                i += 1;
            }
            if width <= LIMIT {
                m += width;
            } else if p.wrapping_sub(pos) >= width {
                ns += 1;
            } else {
                m += 1;
                ns += usize::from(pos < p) + usize::from(p + 1 < pos + width);
            }
            if m > cap_m || ns > cap_strips {
                return false;
            }
            pos += width;
        }
        true
    }

    /// Estimated `Runs` scratch bytes allocated by `front_avx2` for a block with `syms_len`
    /// symbols and `max_len` bound (`runs: Vec<u16>` + `lit: Vec<u64>` + `one: Vec<u64>`).
    fn runs_scratch_bytes(syms_len: usize, max_len: usize) -> usize {
        let slack = max_len / 127 + 1;
        let runs_bytes = (syms_len + slack + 2) * 2;
        let words = syms_len / 64 + 3;
        let bitmap_bytes = words * 8 * 2;
        runs_bytes + bitmap_bytes
    }

    struct BlockInfo {
        idx: usize,
        shipped_runs: bool,
        syms_len: usize,
        max_len: usize,
        in_use: usize,
        n: usize,
        recs: usize,
        run_frac_exact: f64,
        run_frac_sample: f64,
        lit_ratio_est: f64,
        t_col_mtf_us: f64,
        t_col_inv_us: f64,
        t_run_front_us: f64,
        t_run_inv_us: f64,
    }

    impl BlockInfo {
        fn syms_ratio(&self) -> f64 {
            self.syms_len as f64 / self.max_len.max(1) as f64
        }

        fn col_total_us(&self) -> f64 {
            self.t_col_mtf_us + self.t_col_inv_us
        }

        fn run_total_us(&self) -> f64 {
            self.t_run_front_us + self.t_run_inv_us
        }
    }

    fn scan_blocks(data: &[u8]) -> Vec<Symbols> {
        let mut sc = Scanner::new(SliceSource(data));
        let mut blocks = Vec::new();
        while let Event::Block { symbols, .. } = sc.next(Vec::new()).expect("scan valid bzip2") {
            blocks.push(symbols);
        }
        blocks
    }

    fn dissect_corpus(
        name: &str,
        blocks: &[Symbols],
        reps: usize,
        k: Kernels,
        dump_blocks: bool,
    ) -> Vec<BlockInfo> {
        let mut open_ship = OpenCells::default();
        let mut seg = Segmented::default();
        let mut r = Runs::default();
        let mut ll_col = Vec::new();
        let mut ll_run = Vec::new();
        let mut ll_open = Vec::new();
        let mut out_rle = Vec::new();
        let mut counts = [0u32; 256];
        let mut infos = Vec::with_capacity(blocks.len());

        for (idx, b) in blocks.iter().enumerate() {
            let shipped_runs = runs::takes_runs(b);
            let syms_len = b.syms.len();
            let max_len = b.max_len;
            let in_use = usize::from(b.eob.saturating_sub(1));

            let run_exact: usize = b.syms.iter().map(|&s| usize::from(s <= 1)).sum();
            let run_frac_exact = run_exact as f64 / syms_len.max(1) as f64;
            let (rs, st) = sample_run_share(&b.syms);
            let run_frac_sample = rs as f64 / st.max(1) as f64;
            let syms_ratio = syms_len as f64 / max_len.max(1) as f64;
            let lit_ratio_est = syms_ratio * (1.0 - run_frac_sample);

            // 100% byte + CRC exactness check across Column, Runs+Seg, and Runs+OpenCells.
            let n_col = mtf::mtf(b, &mut ll_col, &mut counts, k).expect("mtf::mtf");
            assert!(seg.invert_in_place(&mut ll_col, &counts, b.orig_ptr));
            let n_run = runs::front(b, &mut r, k).expect("runs::front");
            assert_eq!(n_col, n_run, "{name} blk {idx}: n mismatch");
            let recs = r.records().len();
            assert!(seg.invert_runs(&r, n_run, b.orig_ptr, &mut ll_run, k));
            assert_eq!(ll_col, ll_run, "{name} blk {idx}: Column != Runs+Seg");
            assert!(open_ship.invert_runs(&r, n_run, b.orig_ptr, &mut ll_open, k));
            assert_eq!(ll_col, ll_open, "{name} blk {idx}: Column != OpenCells");
            out_rle.clear();
            rle::unrle(&ll_col, &mut out_rle, k).expect("unrle");
            assert_eq!(
                crc::crc32(&out_rle, k),
                b.crc,
                "{name} blk {idx}: CRC mismatch"
            );

            // Interleaved per-block phase timing (rotating order to cancel warmth bias).
            let mut col_mtf_v = Vec::with_capacity(reps);
            let mut col_inv_v = Vec::with_capacity(reps);
            let mut run_front_v = Vec::with_capacity(reps);
            let mut run_inv_v = Vec::with_capacity(reps);

            for rep in 0..reps {
                let (mut tcm, mut tci, mut trf, mut tri) = (0.0, 0.0, 0.0, 0.0);
                for step in 0..2 {
                    if (rep + step) % 2 == 0 {
                        let t0 = Instant::now();
                        let nc = mtf::mtf(b, &mut ll_col, &mut counts, k).unwrap();
                        tcm = t0.elapsed().as_nanos() as f64 / 1e3;
                        let t1 = Instant::now();
                        let ok = seg.invert_in_place(&mut ll_col, &counts, b.orig_ptr);
                        tci = t1.elapsed().as_nanos() as f64 / 1e3;
                        std::hint::black_box((nc, ok, ll_col.len()));
                    } else {
                        let t0 = Instant::now();
                        let nr = runs::front(b, &mut r, k).unwrap();
                        trf = t0.elapsed().as_nanos() as f64 / 1e3;
                        let t1 = Instant::now();
                        let ok = seg.invert_runs(&r, nr, b.orig_ptr, &mut ll_run, k);
                        tri = t1.elapsed().as_nanos() as f64 / 1e3;
                        std::hint::black_box((nr, ok, ll_run.len()));
                    }
                }
                col_mtf_v.push(tcm);
                col_inv_v.push(tci);
                run_front_v.push(trf);
                run_inv_v.push(tri);
            }

            let info = BlockInfo {
                idx,
                shipped_runs,
                syms_len,
                max_len,
                in_use,
                n: n_run,
                recs,
                run_frac_exact,
                run_frac_sample,
                lit_ratio_est,
                t_col_mtf_us: median_f64(&mut col_mtf_v),
                t_col_inv_us: median_f64(&mut col_inv_v),
                t_run_front_us: median_f64(&mut run_front_v),
                t_run_inv_us: median_f64(&mut run_inv_v),
            };
            if dump_blocks && info.n >= 100_000 {
                eprintln!(
                    "BLK\t{name}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.3}",
                    info.idx,
                    if info.shipped_runs { "Runs" } else { "Col" },
                    info.n,
                    info.syms_ratio(),
                    info.run_frac_sample,
                    info.run_frac_exact,
                    info.lit_ratio_est,
                    info.recs as f64 / info.n.max(1) as f64,
                    info.in_use,
                    info.t_col_mtf_us,
                    info.t_col_inv_us,
                    info.t_run_front_us,
                    info.t_run_inv_us,
                    info.run_total_us() / info.col_total_us().max(0.001)
                );
            }
            infos.push(info);
        }

        infos
    }

    #[derive(Clone, Copy)]
    enum Rule {
        Shipped,
        AlwaysColSeg,
        /// syms <= 0.75 * max_len && run_frac >= 1/32
        Syms75Rs32,
        /// syms <= 0.85 * max_len && run_frac >= 1/32
        Syms85Rs32,
        /// syms <= 0.85 * max_len && run_frac >= 1/16
        Syms85Rs16,
        /// lit_ratio_est <= 0.60 && run_frac >= 1/16
        Lit60Rs16,
        /// lit_ratio_est <= 0.65 && run_frac >= 1/32
        Lit65Rs32,
        /// lit_ratio_est <= 0.70 && run_frac >= 1/16
        Lit70Rs16,
        /// syms <= max_len && run_frac >= 1/16 (no 2*len half-bound)
        Rs16NoHalf,
        /// Combined: Lit65Rs32 + #1145's R2_2tier_m22
        ComboLit65R2m22,
        /// Combined: Syms85Rs16 + #1145's R2_2tier_m22
        ComboSyms85R2m22,
    }

    impl Rule {
        fn name(self) -> &'static str {
            match self {
                Self::Shipped => "shipped",
                Self::AlwaysColSeg => "always_col_seg",
                Self::Syms75Rs32 => "T1_syms75_rs32",
                Self::Syms85Rs32 => "T2_syms85_rs32",
                Self::Syms85Rs16 => "T3_syms85_rs16",
                Self::Lit60Rs16 => "T4_lit60_rs16",
                Self::Lit65Rs32 => "T5_lit65_rs32",
                Self::Lit70Rs16 => "T6_lit70_rs16",
                Self::Rs16NoHalf => "T7_rs16_nohalf",
                Self::ComboLit65R2m22 => "COMBO_T5_R2m22",
                Self::ComboSyms85R2m22 => "COMBO_T3_R2m22",
            }
        }

        fn uses_r2_open(self) -> bool {
            matches!(self, Self::ComboLit65R2m22 | Self::ComboSyms85R2m22)
        }

        /// Live O(1) pre-MTF route decision using ONLY `block::Symbols` (timed in the loop).
        fn takes_runs_live(self, b: &Symbols) -> bool {
            if b.max_len >= (1 << 24) {
                return false;
            }
            let len = b.syms.len();
            match self {
                Self::Shipped => runs::takes_runs(b),
                Self::AlwaysColSeg => false,
                Self::Syms75Rs32 => {
                    if len * 4 > b.max_len * 3 {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    rs * 32 >= st
                }
                Self::Syms85Rs32 => {
                    if len * 20 > b.max_len * 17 {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    rs * 32 >= st
                }
                Self::Syms85Rs16 | Self::ComboSyms85R2m22 => {
                    if len * 20 > b.max_len * 17 {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    rs * 16 >= st
                }
                Self::Lit60Rs16 => {
                    if len > b.max_len {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    if rs * 16 < st {
                        return false;
                    }
                    // lit_est = len * (st - rs) / st <= 0.60 * max_len
                    (len as u128) * ((st - rs) as u128) * 5
                        <= (b.max_len as u128) * (st as u128) * 3
                }
                Self::Lit65Rs32 | Self::ComboLit65R2m22 => {
                    if len > b.max_len {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    if rs * 32 < st {
                        return false;
                    }
                    // lit_est <= 0.65 * max_len
                    (len as u128) * ((st - rs) as u128) * 20
                        <= (b.max_len as u128) * (st as u128) * 13
                }
                Self::Lit70Rs16 => {
                    if len > b.max_len {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    if rs * 16 < st {
                        return false;
                    }
                    // lit_est <= 0.70 * max_len
                    (len as u128) * ((st - rs) as u128) * 10
                        <= (b.max_len as u128) * (st as u128) * 7
                }
                Self::Rs16NoHalf => {
                    if len > b.max_len {
                        return false;
                    }
                    let (rs, st) = sample_run_share(&b.syms);
                    rs * 16 >= st
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_rule_pass(
        blocks: &[Symbols],
        rule: Rule,
        k: Kernels,
        open_ship: &mut OpenCells,
        open_forced: &mut ForcedOpenCells,
        seg: &mut Segmented,
        r: &mut Runs,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> (usize, usize) {
        let mut total_bytes = 0usize;
        let mut peak_runs_bytes = 0usize;
        for b in blocks {
            if rule.takes_runs_live(b) {
                let n = runs::front(b, r, k).unwrap();
                peak_runs_bytes = peak_runs_bytes.max(runs_scratch_bytes(b.syms.len(), b.max_len));
                let p = b.orig_ptr;
                if rule.uses_r2_open() {
                    let recs = r.records().len();
                    let take_open = if n == 0
                        || n > 900_000
                        || (p as usize) >= n
                        || recs * 64 < n
                        || recs * 20 >= n * 3
                    {
                        false
                    } else if recs * 10 < n {
                        true
                    } else {
                        let cap_m = DENSE_M.min(((n as u64) * 22 / 100) as u32);
                        within_cap_live(r.records(), p, cap_m, DENSE_STRIPS)
                    };
                    let ok = if take_open {
                        open_forced.invert_runs(r, n, p, ll, k)
                    } else {
                        seg.invert_runs(r, n, p, ll, k)
                    };
                    debug_assert!(ok);
                } else {
                    let ok = open_ship.invert_runs(r, n, p, ll, k);
                    debug_assert!(ok);
                }
                total_bytes += ll.len();
            } else {
                let _n = mtf::mtf(b, ll, counts, k).unwrap();
                let ok = seg.invert_in_place(ll, counts, b.orig_ptr);
                debug_assert!(ok);
                total_bytes += ll.len();
            }
        }
        (std::hint::black_box(total_bytes), peak_runs_bytes)
    }

    fn eval_rules_corpus(name: &str, blocks: &[Symbols], reps: usize, k: Kernels) {
        let rules = [
            Rule::Shipped,
            Rule::AlwaysColSeg,
            Rule::Syms75Rs32,
            Rule::Syms85Rs32,
            Rule::Syms85Rs16,
            Rule::Lit60Rs16,
            Rule::Lit65Rs32,
            Rule::Lit70Rs16,
            Rule::Rs16NoHalf,
            Rule::ComboLit65R2m22,
            Rule::ComboSyms85R2m22,
        ];

        let mut open_ship = OpenCells::default();
        let mut open_forced = ForcedOpenCells::default();
        let mut seg = Segmented::default();
        let mut r = Runs::default();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let clock = CpuClock::open();

        // Calibrate inner passes using steal-free CPU so each sample >= ~240 ms.
        let c_cal = clock.now_ns();
        let w_cal = Instant::now();
        run_rule_pass(
            blocks,
            Rule::Shipped,
            k,
            &mut open_ship,
            &mut open_forced,
            &mut seg,
            &mut r,
            &mut ll,
            &mut counts,
        );
        let one_cpu_ms = {
            let dc = clock.now_ns().saturating_sub(c_cal) as f64 / 1e6;
            if dc > 1.0 {
                dc
            } else {
                w_cal.elapsed().as_secs_f64() * 1e3
            }
        };
        let inner = ((240.0 / one_cpu_ms.max(1.0)).ceil() as usize).clamp(1, 12);

        let mut wall_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(reps); rules.len()];
        let mut cpu_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(reps); rules.len()];
        let mut peak_kb: Vec<usize> = vec![0; rules.len()];

        for rep in 0..reps {
            for offset in 0..rules.len() {
                let ri = (rep + offset) % rules.len();
                let rule = rules[ri];
                let c0 = clock.now_ns();
                let w0 = Instant::now();
                let mut pk = 0usize;
                for _ in 0..inner {
                    let (_, p) = run_rule_pass(
                        blocks,
                        rule,
                        k,
                        &mut open_ship,
                        &mut open_forced,
                        &mut seg,
                        &mut r,
                        &mut ll,
                        &mut counts,
                    );
                    pk = pk.max(p);
                }
                let dw = w0.elapsed().as_secs_f64() * 1e3 / inner as f64;
                let dc = clock.now_ns().saturating_sub(c0) as f64 / 1e6 / inner as f64;
                wall_ms[ri].push(dw);
                cpu_ms[ri].push(dc);
                peak_kb[ri] = peak_kb[ri].max(pk / 1024);
            }
        }

        let mut ship_c = cpu_ms[0].clone();
        let mut ship_w = wall_ms[0].clone();
        let ship_c_med = median_f64(&mut ship_c);
        let ship_c_min = ship_c[0];
        let ship_c_max = *ship_c.last().unwrap();
        let ship_w_med = median_f64(&mut ship_w);

        for (ri, rule) in rules.iter().enumerate() {
            let mut cv = cpu_ms[ri].clone();
            let mut wv = wall_ms[ri].clone();
            let cm = median_f64(&mut cv);
            let cmin = cv[0];
            let cmax = *cv.last().unwrap();
            let wm = median_f64(&mut wv);
            let wmin = wv[0];
            let wmax = *wv.last().unwrap();
            let c_pct = (cm - ship_c_med) / ship_c_med * 100.0;
            let w_pct = (wm - ship_w_med) / ship_w_med * 100.0;
            let disj = if ri == 0 {
                "BASE"
            } else if cmax < ship_c_min {
                "FASTER"
            } else if cmin > ship_c_max {
                "SLOWER"
            } else {
                "OVERLAP"
            };
            println!(
                "RULE\t{name}\t{}\t{inner}\t{}\t{cm:.2}\t[{cmin:.2}..{cmax:.2}]\t{c_pct:+.2}%\t{disj}\t{wm:.2}\t[{wmin:.2}..{wmax:.2}]\t{w_pct:+.2}%",
                rule.name(),
                peak_kb[ri]
            );
        }
    }

    pub fn main() {
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        let mut reps = 5usize;
        let mut dump_blocks = false;
        let mut eval_rules = false;
        let mut rules_only = false;
        let mut files = Vec::new();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--reps" => {
                    i += 1;
                    reps = args[i].parse().expect("parse --reps");
                }
                "--dump-blocks" => dump_blocks = true,
                "--eval-rules" => eval_rules = true,
                "--rules-only" => {
                    eval_rules = true;
                    rules_only = true;
                }
                _ => files.push(std::mem::take(&mut args[i])),
            }
            i += 1;
        }
        if files.is_empty() {
            eprintln!("usage: takes_runs_probe [--reps N] [--dump-blocks] [--eval-rules] [--rules-only] FILE.bz2...");
            std::process::exit(2);
        }

        let k = Kernels::detect();
        println!(
            "# takes_runs_probe target_avx2={} reps={reps} rules_only={rules_only}",
            cfg!(target_feature = "avx2")
        );

        let mut all_corpora: Vec<(String, Vec<Symbols>, Vec<BlockInfo>)> = Vec::new();

        if !rules_only {
            println!(
                "SUMMARY\tcorpus\tblks\tfull\tship_col\tship_runs\tcol_mtf_ms\tcol_inv_ms\trun_front_ms\trun_inv_ms\trun_over_col\tcol_run_wins\tsyms_r_col_med\trunf_col_med\tlit_r_col_med"
            );
            if dump_blocks {
                eprintln!(
                    "BLK\tcorpus\tidx\tship\tn\tsyms_r\trun_f\tlit_r\trn\tin_use\tcol_mtf_us\tcol_inv_us\trun_front_us\trun_inv_us\trun_over_col"
                );
            }
        }

        for path in &files {
            let data = std::fs::read(path).expect("read bz2");
            let name = Path::new(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let blocks = scan_blocks(&data);
            if rules_only {
                all_corpora.push((name, blocks, Vec::new()));
                continue;
            }
            let infos = dissect_corpus(&name, &blocks, reps, k, dump_blocks);
            let full: Vec<&BlockInfo> = infos.iter().filter(|b| b.n >= 100_000).collect();
            let col_blks: Vec<&BlockInfo> =
                full.iter().copied().filter(|b| !b.shipped_runs).collect();
            let run_blks = full.len() - col_blks.len();

            let col_mtf_ms: f64 = infos.iter().map(|b| b.t_col_mtf_us).sum::<f64>() / 1e3;
            let col_inv_ms: f64 = infos.iter().map(|b| b.t_col_inv_us).sum::<f64>() / 1e3;
            let run_front_ms: f64 = infos.iter().map(|b| b.t_run_front_us).sum::<f64>() / 1e3;
            let run_inv_ms: f64 = infos.iter().map(|b| b.t_run_inv_us).sum::<f64>() / 1e3;
            let col_tot = col_mtf_ms + col_inv_ms;
            let run_tot = run_front_ms + run_inv_ms;
            let col_run_wins = col_blks
                .iter()
                .filter(|b| b.run_total_us() < b.col_total_us())
                .count();

            let mut sr: Vec<f64> = col_blks.iter().map(|b| b.syms_ratio()).collect();
            let mut rf: Vec<f64> = col_blks.iter().map(|b| b.run_frac_sample).collect();
            let mut lr: Vec<f64> = col_blks.iter().map(|b| b.lit_ratio_est).collect();

            println!(
                "SUMMARY\t{name}\t{}\t{}\t{}\t{}\t{col_mtf_ms:.2}\t{col_inv_ms:.2}\t{run_front_ms:.2}\t{run_inv_ms:.2}\t{:.3}\t{col_run_wins}/{}\t{:.3}\t{:.3}\t{:.3}",
                infos.len(),
                full.len(),
                col_blks.len(),
                run_blks,
                run_tot / col_tot.max(1e-9),
                col_blks.len(),
                median_f64(&mut sr),
                median_f64(&mut rf),
                median_f64(&mut lr),
            );
            all_corpora.push((name, blocks, infos));
        }

        if !rules_only {
            let all_full: Vec<&BlockInfo> = all_corpora
                .iter()
                .flat_map(|(_, _, v)| v.iter())
                .filter(|b| b.n >= 100_000)
                .collect();

            println!("\n# CROSSOVER BY syms_ratio = syms.len() / max_len (all full blocks)");
            println!(
                "SR_BIN\tbin\tblks\trun_wins\tcol_mtf\tcol_inv\trun_frt\trun_inv\trun_over_col\trunf_med\tlit_med"
            );
            let sr_edges = [0.0, 0.30, 0.50, 0.60, 0.70, 0.80, 0.90, 0.98, 1.05];
            for w in sr_edges.windows(2) {
                let bin: Vec<&BlockInfo> = all_full
                    .iter()
                    .copied()
                    .filter(|b| b.syms_ratio() >= w[0] && b.syms_ratio() < w[1])
                    .collect();
                if bin.is_empty() {
                    continue;
                }
                let rw = bin
                    .iter()
                    .filter(|b| b.run_total_us() < b.col_total_us())
                    .count();
                let cm: f64 = bin.iter().map(|b| b.t_col_mtf_us).sum::<f64>() / 1e3;
                let ci: f64 = bin.iter().map(|b| b.t_col_inv_us).sum::<f64>() / 1e3;
                let rf: f64 = bin.iter().map(|b| b.t_run_front_us).sum::<f64>() / 1e3;
                let ri: f64 = bin.iter().map(|b| b.t_run_inv_us).sum::<f64>() / 1e3;
                let mut rfs: Vec<f64> = bin.iter().map(|b| b.run_frac_sample).collect();
                let mut lrs: Vec<f64> = bin.iter().map(|b| b.lit_ratio_est).collect();
                println!(
                    "SR_BIN\t[{:.2},{:.2})\t{}\t{rw}\t{cm:.1}\t{ci:.1}\t{rf:.1}\t{ri:.1}\t{:.3}\t{:.3}\t{:.3}",
                    w[0],
                    w[1],
                    bin.len(),
                    (rf + ri) / (cm + ci).max(1e-9),
                    median_f64(&mut rfs),
                    median_f64(&mut lrs)
                );
            }

            println!("\n# CROSSOVER BY lit_ratio_est = syms_ratio * (1 - run_frac_sample) (all full blocks)");
            println!(
                "LR_BIN\tbin\tblks\trun_wins\tcol_mtf\tcol_inv\trun_frt\trun_inv\trun_over_col\trunf_med\tsr_med"
            );
            let lr_edges = [0.0, 0.25, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 1.05];
            for w in lr_edges.windows(2) {
                let bin: Vec<&BlockInfo> = all_full
                    .iter()
                    .copied()
                    .filter(|b| b.lit_ratio_est >= w[0] && b.lit_ratio_est < w[1])
                    .collect();
                if bin.is_empty() {
                    continue;
                }
                let rw = bin
                    .iter()
                    .filter(|b| b.run_total_us() < b.col_total_us())
                    .count();
                let cm: f64 = bin.iter().map(|b| b.t_col_mtf_us).sum::<f64>() / 1e3;
                let ci: f64 = bin.iter().map(|b| b.t_col_inv_us).sum::<f64>() / 1e3;
                let rf: f64 = bin.iter().map(|b| b.t_run_front_us).sum::<f64>() / 1e3;
                let ri: f64 = bin.iter().map(|b| b.t_run_inv_us).sum::<f64>() / 1e3;
                let mut rfs: Vec<f64> = bin.iter().map(|b| b.run_frac_sample).collect();
                let mut srs: Vec<f64> = bin.iter().map(|b| b.syms_ratio()).collect();
                println!(
                    "LR_BIN\t[{:.2},{:.2})\t{}\t{rw}\t{cm:.1}\t{ci:.1}\t{rf:.1}\t{ri:.1}\t{:.3}\t{:.3}\t{:.3}",
                    w[0],
                    w[1],
                    bin.len(),
                    (rf + ri) / (cm + ci).max(1e-9),
                    median_f64(&mut rfs),
                    median_f64(&mut srs)
                );
            }

            println!("\n# CROSSOVER BY run_frac_sample (all full blocks with syms_ratio >= 0.45)");
            println!(
                "RF_BIN\tbin\tblks\trun_wins\tcol_mtf\tcol_inv\trun_frt\trun_inv\trun_over_col\tsr_med\tlit_med"
            );
            let rf_edges = [0.0, 0.01, 0.03125, 0.0625, 0.125, 0.25, 0.40, 1.01];
            for w in rf_edges.windows(2) {
                let bin: Vec<&BlockInfo> = all_full
                    .iter()
                    .copied()
                    .filter(|b| {
                        b.syms_ratio() >= 0.45
                            && b.run_frac_sample >= w[0]
                            && b.run_frac_sample < w[1]
                    })
                    .collect();
                if bin.is_empty() {
                    continue;
                }
                let rw = bin
                    .iter()
                    .filter(|b| b.run_total_us() < b.col_total_us())
                    .count();
                let cm: f64 = bin.iter().map(|b| b.t_col_mtf_us).sum::<f64>() / 1e3;
                let ci: f64 = bin.iter().map(|b| b.t_col_inv_us).sum::<f64>() / 1e3;
                let rf: f64 = bin.iter().map(|b| b.t_run_front_us).sum::<f64>() / 1e3;
                let ri: f64 = bin.iter().map(|b| b.t_run_inv_us).sum::<f64>() / 1e3;
                let mut srs: Vec<f64> = bin.iter().map(|b| b.syms_ratio()).collect();
                let mut lrs: Vec<f64> = bin.iter().map(|b| b.lit_ratio_est).collect();
                println!(
                    "RF_BIN\t[{:.4},{:.4})\t{}\t{rw}\t{cm:.1}\t{ci:.1}\t{rf:.1}\t{ri:.1}\t{:.3}\t{:.3}\t{:.3}",
                    w[0],
                    w[1],
                    bin.len(),
                    (rf + ri) / (cm + ci).max(1e-9),
                    median_f64(&mut srs),
                    median_f64(&mut lrs)
                );
            }
        }

        if eval_rules {
            println!(
                "\n# WHOLE-CORPUS RULE EVALUATION (MTF/front + InverseBwt, interleaved, schedstat CPU + wall)"
            );
            println!(
                "RULE\tcorpus\trule\tinner\tpeak_runs_kb\tcpu_med_ms\tcpu_range\tcpu_pct\tcpu_disj\twall_med_ms\twall_range\twall_pct"
            );
            for (name, blocks, _) in &all_corpora {
                eval_rules_corpus(name, blocks, reps, k);
            }
        }
    }
}

#[cfg(feature = "moonshot")]
fn main() {
    imp::main();
}

#[cfg(not(feature = "moonshot"))]
fn main() {
    eprintln!("takes_runs_probe: build with --features moonshot");
    std::process::exit(2);
}
