#![forbid(unsafe_code)]
//! Smooth-zone `explore` probe (zero `src/` edits, `#![forbid(unsafe_code)]`):
//! Post-#1080 `OpenCells` vs `Segmented` route-boundary and admission dissection across
//! real-world corpora.
//!
//! # Frozen Discovery / Stop Test (frozen BEFORE reading any timing data)
//! - **PASS** (open a follow-on `idea` claim with its own frozen stop test): if a simple
//!   O(1) / zero-extra-memory pre-route rule (using only features available at
//!   `dense_precheck` / `runs::front` time: `n`, `records`, `m` / retained thin rows,
//!   `thick` strips, `r/n`, `m/n`, `p`, or `syms.len() / max_len`) recovers:
//!   - `>= 5.0%` of total inverse-BWT + MTF worker CPU on `>= 2` off-Wikidata corpora, OR
//!   - `>= 3.0%` of total inverse-BWT + MTF worker CPU on primary `text`,
//!     with `0` Wikidata regression (`<= +0.2%`) and `0` regressions across all 9 corpora,
//!     disjoint on both baseline x86-64 and JCC-32B-aligned builds.
//! - **STOP** (record a scoped dead-end row in `frontier/dead-ends.tsv`): if the
//!   per-block oracle-route ceiling itself is `< 3.0%` on primary `text` and `< 5.0%`
//!   across off-Wikidata corpora (or no simple O(1)/zero-extra-memory rule clears the
//!   PASS bar without regressions).
//!
//! # Usage
//! ```bash
//! cargo run --release --features moonshot --example route_crossover_probe -- [FLAGS] FILE.bz2...
//! # Optional: also test future Open kernel features on master:
//! cargo run --release --features "moonshot a6b-endpoint-redesign forward-anchor-lanes" \
//!     --example route_crossover_probe -- [FLAGS] FILE.bz2...
//! ```
//! Flags:
//! - `--reps N`        interleaved repeats per block / corpus (default 5)
//! - `--dump-blocks`   print every full block's `(route, n, r/n, m/n, thick, t_open, t_seg, t_ship)`
//! - `--eval-rules`    run whole-corpus interleaved paired evaluation (wall + schedstat CPU)

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

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ShippedRoute {
        Column,
        OpenLow,
        OpenDense,
        RefusedDense,
        RefusedHigh,
        RefusedSparse,
    }

    impl ShippedRoute {
        fn as_str(self) -> &'static str {
            match self {
                Self::Column => "Column",
                Self::OpenLow => "OpenLow",
                Self::OpenDense => "OpenDense",
                Self::RefusedDense => "RefDense",
                Self::RefusedHigh => "RefHigh",
                Self::RefusedSparse => "RefSparse",
            }
        }

        fn is_open(self) -> bool {
            matches!(self, Self::OpenLow | Self::OpenDense)
        }
    }

    /// Exact match of `src/open.rs::within_cap` counts `(retained_m, thick_strips)`.
    fn open_counts(records: &[u16], p: u32) -> (u32, usize) {
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
            pos += width;
        }
        (m, ns)
    }

    fn classify_route(
        takes_runs: bool,
        recs: usize,
        n: usize,
        p: u32,
        m: u32,
        thick: usize,
    ) -> ShippedRoute {
        if !takes_runs {
            return ShippedRoute::Column;
        }
        if n == 0 || n > 900_000 || p as usize >= n {
            return ShippedRoute::RefusedHigh;
        }
        if recs * 64 < n {
            return ShippedRoute::RefusedSparse;
        }
        if recs * 20 < n * 3 {
            return ShippedRoute::OpenLow;
        }
        if recs * 25 < n * 7 {
            let cap_m = DENSE_M.min((n as u64 * 3 / 10) as u32);
            if m <= cap_m && thick <= DENSE_STRIPS {
                return ShippedRoute::OpenDense;
            }
            return ShippedRoute::RefusedDense;
        }
        ShippedRoute::RefusedHigh
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

    struct BlockMeas {
        idx: usize,
        route: ShippedRoute,
        n: usize,
        recs: usize,
        syms_len: usize,
        max_len: usize,
        m: u32,
        thick: usize,
        /// Shipped front time (runs::front for run-route, mtf::mtf for column-route)
        t_front_med_us: f64,
        /// Forced Open inverse time (on run-route records)
        t_open_med_us: f64,
        /// Forced Segmented inverse time (invert_runs for run-route, invert_in_place for col)
        t_seg_med_us: f64,
        /// Shipped inverse time (OpenCells::invert_runs for run-route, Seg::invert_in_place for col)
        t_ship_med_us: f64,
        /// For column-route blocks: runs::front + ForcedOpenCells vs runs::front + Segmented
        t_col_via_runs_front_med_us: f64,
        t_col_via_runs_open_med_us: f64,
        t_col_via_runs_seg_med_us: f64,
    }

    impl BlockMeas {
        fn rn(&self) -> f64 {
            self.recs as f64 / self.n.max(1) as f64
        }

        fn mn(&self) -> f64 {
            self.m as f64 / self.n.max(1) as f64
        }

        fn ship_mtf_inv_us(&self) -> f64 {
            self.t_front_med_us + self.t_ship_med_us
        }

        /// Oracle choosing min(Open, Seg) on run-route blocks; shipped Column on column-route.
        fn oracle_runs_mtf_inv_us(&self) -> f64 {
            if self.route == ShippedRoute::Column {
                self.ship_mtf_inv_us()
            } else {
                self.t_front_med_us + self.t_open_med_us.min(self.t_seg_med_us)
            }
        }

        /// Oracle choosing min across (Column+Seg, Runs+Open, Runs+Seg) on EVERY block.
        fn oracle_all_mtf_inv_us(&self) -> f64 {
            if self.route == ShippedRoute::Column {
                let col_cost = self.ship_mtf_inv_us();
                let run_open = self.t_col_via_runs_front_med_us + self.t_col_via_runs_open_med_us;
                let run_seg = self.t_col_via_runs_front_med_us + self.t_col_via_runs_seg_med_us;
                col_cost.min(run_open).min(run_seg)
            } else {
                self.oracle_runs_mtf_inv_us()
            }
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

    fn verify_and_measure_corpus(
        name: &str,
        blocks: &[Symbols],
        reps: usize,
        k: Kernels,
        dump_blocks: bool,
    ) -> Vec<BlockMeas> {
        let mut open_ship = OpenCells::default();
        let mut open_forced = ForcedOpenCells::default();
        let mut seg = Segmented::default();
        let mut r = Runs::default();
        let mut out_ship = Vec::new();
        let mut out_open = Vec::new();
        let mut out_seg = Vec::new();
        let mut out_rle = Vec::new();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];

        let mut results = Vec::with_capacity(blocks.len());
        let mut exact_checks = 0usize;

        for (idx, b) in blocks.iter().enumerate() {
            let takes = runs::takes_runs(b);
            let n = runs::front(b, &mut r, k).expect("runs::front succeeds");
            assert!((b.orig_ptr as usize) < n, "{name} blk {idx}: bad orig_ptr");
            let recs = r.records().len();
            let (m, thick) = open_counts(r.records(), b.orig_ptr);
            let route = classify_route(takes, recs, n, b.orig_ptr, m, thick);

            // 100% byte-exact verification across ALL routes for this block.
            assert!(
                open_ship.invert_runs(&r, n, b.orig_ptr, &mut out_ship, k),
                "{name} blk {idx}: OpenCells::invert_runs failed"
            );
            assert!(
                open_forced.invert_runs(&r, n, b.orig_ptr, &mut out_open, k),
                "{name} blk {idx}: ForcedOpenCells::invert_runs failed"
            );
            assert!(
                seg.invert_runs(&r, n, b.orig_ptr, &mut out_seg, k),
                "{name} blk {idx}: Segmented::invert_runs failed"
            );
            assert_eq!(
                out_ship, out_open,
                "{name} blk {idx}: OpenCells != ForcedOpenCells"
            );
            assert_eq!(
                out_ship, out_seg,
                "{name} blk {idx}: OpenCells != Segmented"
            );
            out_rle.clear();
            rle::unrle(&out_ship, &mut out_rle, k).expect("unrle");
            assert_eq!(
                crc::crc32(&out_rle, k),
                b.crc,
                "{name} blk {idx}: CRC mismatch"
            );
            if !takes {
                let nc = mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf::mtf");
                assert_eq!(nc, n);
                assert!(seg.invert_in_place(&mut ll, &counts, b.orig_ptr));
                assert_eq!(ll, out_ship, "{name} blk {idx}: Column Segmented != Runs");
            }
            exact_checks += 1;

            // Interleaved per-block timing (warmup already done by verification above).
            let mut front_v = Vec::with_capacity(reps);
            let mut open_v = Vec::with_capacity(reps);
            let mut seg_v = Vec::with_capacity(reps);
            let mut ship_v = Vec::with_capacity(reps);
            let mut col_rfront_v = Vec::with_capacity(reps);
            let mut col_ropen_v = Vec::with_capacity(reps);
            let mut col_rseg_v = Vec::with_capacity(reps);

            if takes {
                for rep in 0..reps {
                    let t0 = Instant::now();
                    let nf = runs::front(b, &mut r, k).unwrap();
                    let tf = t0.elapsed().as_nanos() as f64 / 1e3;
                    std::hint::black_box(nf);
                    front_v.push(tf);

                    // Rotate order of Open / Seg / Shipped across repeats to cancel order bias.
                    let order = [(rep % 3), ((rep + 1) % 3), ((rep + 2) % 3)];
                    let (mut to, mut ts, mut tsh) = (0.0, 0.0, 0.0);
                    for arm in order {
                        match arm {
                            0 => {
                                let t = Instant::now();
                                let ok =
                                    open_forced.invert_runs(&r, n, b.orig_ptr, &mut out_open, k);
                                to = t.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_open.len()));
                            }
                            1 => {
                                let t = Instant::now();
                                let ok = seg.invert_runs(&r, n, b.orig_ptr, &mut out_seg, k);
                                ts = t.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_seg.len()));
                            }
                            _ => {
                                let t = Instant::now();
                                let ok = open_ship.invert_runs(&r, n, b.orig_ptr, &mut out_ship, k);
                                tsh = t.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_ship.len()));
                            }
                        }
                    }
                    open_v.push(to);
                    seg_v.push(ts);
                    ship_v.push(tsh);
                }
            } else {
                // Column-route block: time shipped Column (mtf + Seg::invert_in_place) AND
                // runs::front + ForcedOpen / runs::front + Segmented::invert_runs.
                for rep in 0..reps {
                    let order = [rep % 2, (rep + 1) % 2];
                    let (mut tcm, mut tcs, mut trf, mut tro, mut trs) = (0.0, 0.0, 0.0, 0.0, 0.0);
                    for branch in order {
                        if branch == 0 {
                            let t = Instant::now();
                            let nc = mtf::mtf(b, &mut ll, &mut counts, k).unwrap();
                            tcm = t.elapsed().as_nanos() as f64 / 1e3;
                            let t2 = Instant::now();
                            let ok = seg.invert_in_place(&mut ll, &counts, b.orig_ptr);
                            tcs = t2.elapsed().as_nanos() as f64 / 1e3;
                            std::hint::black_box((nc, ok, ll.len()));
                        } else {
                            let t = Instant::now();
                            let nf = runs::front(b, &mut r, k).unwrap();
                            trf = t.elapsed().as_nanos() as f64 / 1e3;
                            std::hint::black_box(nf);
                            if rep % 2 == 0 {
                                let t2 = Instant::now();
                                let ok =
                                    open_forced.invert_runs(&r, n, b.orig_ptr, &mut out_open, k);
                                tro = t2.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_open.len()));
                                let t3 = Instant::now();
                                let ok = seg.invert_runs(&r, n, b.orig_ptr, &mut out_seg, k);
                                trs = t3.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_seg.len()));
                            } else {
                                let t3 = Instant::now();
                                let ok = seg.invert_runs(&r, n, b.orig_ptr, &mut out_seg, k);
                                trs = t3.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_seg.len()));
                                let t2 = Instant::now();
                                let ok =
                                    open_forced.invert_runs(&r, n, b.orig_ptr, &mut out_open, k);
                                tro = t2.elapsed().as_nanos() as f64 / 1e3;
                                std::hint::black_box((ok, out_open.len()));
                            }
                        }
                    }
                    front_v.push(tcm);
                    seg_v.push(tcs);
                    ship_v.push(tcs);
                    open_v.push(tro);
                    col_rfront_v.push(trf);
                    col_ropen_v.push(tro);
                    col_rseg_v.push(trs);
                }
            }

            let bm = BlockMeas {
                idx,
                route,
                n,
                recs,
                syms_len: b.syms.len(),
                max_len: b.max_len,
                m,
                thick,
                t_front_med_us: median_f64(&mut front_v),
                t_open_med_us: median_f64(&mut open_v),
                t_seg_med_us: median_f64(&mut seg_v),
                t_ship_med_us: median_f64(&mut ship_v),
                t_col_via_runs_front_med_us: median_f64(&mut col_rfront_v),
                t_col_via_runs_open_med_us: median_f64(&mut col_ropen_v),
                t_col_via_runs_seg_med_us: median_f64(&mut col_rseg_v),
            };

            if dump_blocks && bm.n >= 100_000 {
                let ratio = bm.t_open_med_us / bm.t_seg_med_us.max(0.001);
                eprintln!(
                    "BLK\t{name}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{}\t{:.4}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.3}",
                    bm.idx,
                    bm.route.as_str(),
                    bm.n,
                    bm.rn(),
                    bm.mn(),
                    bm.thick,
                    bm.syms_len as f64 / bm.max_len.max(1) as f64,
                    bm.t_front_med_us,
                    bm.t_open_med_us,
                    bm.t_seg_med_us,
                    bm.t_ship_med_us,
                    ratio
                );
            }
            results.push(bm);
        }

        assert_eq!(exact_checks, blocks.len());
        results
    }

    #[derive(Clone, Copy)]
    enum Rule {
        Shipped,
        AlwaysSeg,
        /// Oracle route precomputed per block from Phase 1 medians (on run-route blocks).
        OracleRuns,
        /// R1: Delete the DENSE band (0.15..0.28) and `within_cap`; Open iff 1/64 <= r/n < 0.15.
        RnLt015NoDense,
        /// R2: Two-tier zero-`runs::front`-edit rule:
        /// `r/n < 0.10` outright Open; `0.10 <= r/n < 0.15` live `within_cap(m <= max_mn*n)`;
        /// `r/n >= 0.15` straight to Segmented (no dense band).
        TwoTierMn {
            max_mn: f64,
        },
        /// R3: Simulates `thin_rows` tracked in `runs::front` (O(1) check, zero extra pass):
        /// Open iff `1/64 <= r/n < 0.15` and `m <= max_mn*n` and `thick <= DENSE_STRIPS`.
        O1ThinNoDense {
            max_mn: f64,
        },
        /// Diagnostic: test `runs::front + Segmented::invert_runs` on Column-route blocks.
        ColViaRunsSeg,
    }

    impl Rule {
        fn name(self) -> &'static str {
            match self {
                Self::Shipped => "shipped",
                Self::AlwaysSeg => "always_seg",
                Self::OracleRuns => "oracle_runs",
                Self::RnLt015NoDense => "R1_rn015_nodense",
                Self::TwoTierMn { max_mn } => {
                    if (max_mn - 0.20).abs() < 1e-6 {
                        "R2_2tier_m20"
                    } else if (max_mn - 0.22).abs() < 1e-6 {
                        "R2_2tier_m22"
                    } else if (max_mn - 0.25).abs() < 1e-6 {
                        "R2_2tier_m25"
                    } else {
                        "R2_2tier_custom"
                    }
                }
                Self::O1ThinNoDense { max_mn } => {
                    if (max_mn - 0.20).abs() < 1e-6 {
                        "R3_o1_m20"
                    } else if (max_mn - 0.22).abs() < 1e-6 {
                        "R3_o1_m22"
                    } else if (max_mn - 0.25).abs() < 1e-6 {
                        "R3_o1_m25"
                    } else {
                        "R3_o1_custom"
                    }
                }
                Self::ColViaRunsSeg => "diag_col_via_runs",
            }
        }
    }

    /// Exact early-stopping match of `src/open.rs::within_cap` for live two-tier benchmarking.
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

    /// Run one full pass of MTF/runs::front + InverseBwt over `blocks` under `rule`.
    #[allow(clippy::too_many_arguments)]
    fn run_rule_pass(
        blocks: &[Symbols],
        meas: &[BlockMeas],
        oracle_take_open: &[bool],
        rule: Rule,
        k: Kernels,
        open_ship: &mut OpenCells,
        open_forced: &mut ForcedOpenCells,
        seg: &mut Segmented,
        r: &mut Runs,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
    ) -> usize {
        let mut total_bytes = 0usize;
        for (i, b) in blocks.iter().enumerate() {
            if runs::takes_runs(b) {
                let n = runs::front(b, r, k).unwrap();
                let p = b.orig_ptr;
                let recs = r.records().len();
                match rule {
                    Rule::Shipped | Rule::ColViaRunsSeg => {
                        let ok = open_ship.invert_runs(r, n, p, ll, k);
                        debug_assert!(ok);
                    }
                    Rule::AlwaysSeg => {
                        let ok = seg.invert_runs(r, n, p, ll, k);
                        debug_assert!(ok);
                    }
                    Rule::OracleRuns => {
                        let ok = if oracle_take_open[i] {
                            open_forced.invert_runs(r, n, p, ll, k)
                        } else {
                            seg.invert_runs(r, n, p, ll, k)
                        };
                        debug_assert!(ok);
                    }
                    Rule::RnLt015NoDense => {
                        let take_open = n > 0
                            && n <= 900_000
                            && (p as usize) < n
                            && recs * 64 >= n
                            && recs * 20 < n * 3;
                        let ok = if take_open {
                            open_forced.invert_runs(r, n, p, ll, k)
                        } else {
                            seg.invert_runs(r, n, p, ll, k)
                        };
                        debug_assert!(ok);
                    }
                    Rule::TwoTierMn { max_mn } => {
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
                            let cap_m = DENSE_M.min(((n as f64) * max_mn) as u32);
                            within_cap_live(r.records(), p, cap_m, DENSE_STRIPS)
                        };
                        let ok = if take_open {
                            open_forced.invert_runs(r, n, p, ll, k)
                        } else {
                            seg.invert_runs(r, n, p, ll, k)
                        };
                        debug_assert!(ok);
                    }
                    Rule::O1ThinNoDense { max_mn } => {
                        let bm = &meas[i];
                        let take_open = n > 0
                            && n <= 900_000
                            && (p as usize) < n
                            && recs * 64 >= n
                            && recs * 20 < n * 3
                            && (bm.m as f64) <= (n as f64) * max_mn
                            && bm.thick <= DENSE_STRIPS;
                        let ok = if take_open {
                            open_forced.invert_runs(r, n, p, ll, k)
                        } else {
                            seg.invert_runs(r, n, p, ll, k)
                        };
                        debug_assert!(ok);
                    }
                }
                total_bytes += ll.len();
            } else if matches!(rule, Rule::ColViaRunsSeg) {
                let n = runs::front(b, r, k).unwrap();
                let ok = seg.invert_runs(r, n, b.orig_ptr, ll, k);
                debug_assert!(ok);
                total_bytes += ll.len();
            } else {
                let _n = mtf::mtf(b, ll, counts, k).unwrap();
                let ok = seg.invert_in_place(ll, counts, b.orig_ptr);
                debug_assert!(ok);
                total_bytes += ll.len();
            }
        }
        std::hint::black_box(total_bytes)
    }

    fn eval_rules_corpus(
        name: &str,
        blocks: &[Symbols],
        meas: &[BlockMeas],
        reps: usize,
        k: Kernels,
    ) {
        let oracle_take_open: Vec<bool> = meas
            .iter()
            .map(|bm| {
                bm.route != ShippedRoute::Column
                    && bm.n > 0
                    && bm.n <= 900_000
                    && bm.t_open_med_us < bm.t_seg_med_us
            })
            .collect();

        let rules = [
            Rule::Shipped,
            Rule::OracleRuns,
            Rule::AlwaysSeg,
            Rule::RnLt015NoDense,
            Rule::TwoTierMn { max_mn: 0.20 },
            Rule::TwoTierMn { max_mn: 0.22 },
            Rule::TwoTierMn { max_mn: 0.25 },
            Rule::O1ThinNoDense { max_mn: 0.20 },
            Rule::O1ThinNoDense { max_mn: 0.22 },
            Rule::O1ThinNoDense { max_mn: 0.25 },
            Rule::ColViaRunsSeg,
        ];

        let mut open_ship = OpenCells::default();
        let mut open_forced = ForcedOpenCells::default();
        let mut seg = Segmented::default();
        let mut r = Runs::default();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let clock = CpuClock::open();

        // Calibrate inner passes using steal-free CPU so each sample >= ~240 ms (tick err <= ~1.5%).
        let c_cal = clock.now_ns();
        let w_cal = Instant::now();
        run_rule_pass(
            blocks,
            meas,
            &oracle_take_open,
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
        let inner = ((240.0 / one_cpu_ms.max(1.0)).ceil() as usize).clamp(1, 5);

        let mut wall_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(reps); rules.len()];
        let mut cpu_ms: Vec<Vec<f64>> = vec![Vec::with_capacity(reps); rules.len()];

        for rep in 0..reps {
            for offset in 0..rules.len() {
                let ri = (rep + offset) % rules.len();
                let rule = rules[ri];
                let c0 = clock.now_ns();
                let w0 = Instant::now();
                for _ in 0..inner {
                    run_rule_pass(
                        blocks,
                        meas,
                        &oracle_take_open,
                        rule,
                        k,
                        &mut open_ship,
                        &mut open_forced,
                        &mut seg,
                        &mut r,
                        &mut ll,
                        &mut counts,
                    );
                }
                let dw = w0.elapsed().as_secs_f64() * 1e3 / inner as f64;
                let dc = clock.now_ns().saturating_sub(c0) as f64 / 1e6 / inner as f64;
                wall_ms[ri].push(dw);
                cpu_ms[ri].push(dc);
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
                "RULE\t{name}\t{}\t{inner}\t{cm:.2}\t[{cmin:.2}..{cmax:.2}]\t{c_pct:+.2}%\t{disj}\t{wm:.2}\t[{wmin:.2}..{wmax:.2}]\t{w_pct:+.2}%",
                rule.name()
            );
        }
    }

    pub fn main() {
        let mut args: Vec<String> = std::env::args().skip(1).collect();
        let mut reps = 5usize;
        let mut dump_blocks = false;
        let mut eval_rules = false;
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
                _ => files.push(std::mem::take(&mut args[i])),
            }
            i += 1;
        }
        if files.is_empty() {
            eprintln!("usage: route_crossover_probe [--reps N] [--dump-blocks] [--eval-rules] FILE.bz2...");
            std::process::exit(2);
        }

        let k = Kernels::detect();
        let feats = {
            let mut v = vec!["moonshot"];
            if cfg!(feature = "a6b-endpoint-redesign") {
                v.push("a6b-endpoint-redesign");
            }
            if cfg!(feature = "forward-anchor-lanes") {
                v.push("forward-anchor-lanes");
            }
            v.join("+")
        };
        println!(
            "# route_crossover_probe features={feats} target_avx2={} reps={reps}",
            cfg!(target_feature = "avx2")
        );
        println!(
            "SUMMARY\tcorpus\tblks\tfull\tcol\topen_lo\topen_dn\tref_dn\tref_hi\tref_sp\t\
             ship_ms\tora_runs_ms\tora_runs_%\tora_all_ms\tora_all_%\t\
             inv_ship_ms\tinv_ora_%\t\
             mis_open_n\tmis_open_ms\tmis_ref_n\tmis_ref_ms\tcol_open_n\tcol_open_ms"
        );
        if dump_blocks {
            eprintln!(
                "BLK\tcorpus\tidx\troute\tn\trn\tmn\tthick\tsyms_r\tfront_us\topen_us\tseg_us\tship_us\topen_over_seg"
            );
        }

        let mut all_corpora: Vec<(String, Vec<Symbols>, Vec<BlockMeas>)> = Vec::new();

        for path in &files {
            let data = std::fs::read(path).expect("read bz2");
            let name = Path::new(path)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let blocks = scan_blocks(&data);
            let meas = verify_and_measure_corpus(&name, &blocks, reps, k, dump_blocks);

            let full: Vec<&BlockMeas> = meas.iter().filter(|b| b.n >= 100_000).collect();
            let cnt = |rt: ShippedRoute| full.iter().filter(|b| b.route == rt).count();

            let ship_ms: f64 = meas.iter().map(BlockMeas::ship_mtf_inv_us).sum::<f64>() / 1e3;
            let ora_runs_ms: f64 = meas
                .iter()
                .map(BlockMeas::oracle_runs_mtf_inv_us)
                .sum::<f64>()
                / 1e3;
            let ora_all_ms: f64 = meas
                .iter()
                .map(BlockMeas::oracle_all_mtf_inv_us)
                .sum::<f64>()
                / 1e3;
            let ora_runs_pct = (ship_ms - ora_runs_ms) / ship_ms.max(1e-9) * 100.0;
            let ora_all_pct = (ship_ms - ora_all_ms) / ship_ms.max(1e-9) * 100.0;

            let inv_ship_ms: f64 = meas.iter().map(|b| b.t_ship_med_us).sum::<f64>() / 1e3;
            let inv_ora_ms: f64 = meas
                .iter()
                .map(|b| {
                    if b.route == ShippedRoute::Column {
                        b.t_ship_med_us
                    } else {
                        b.t_open_med_us.min(b.t_seg_med_us)
                    }
                })
                .sum::<f64>()
                / 1e3;
            let inv_ora_pct = (inv_ship_ms - inv_ora_ms) / inv_ship_ms.max(1e-9) * 100.0;

            let mut mis_open_n = 0usize;
            let mut mis_open_ms = 0.0f64;
            let mut mis_ref_n = 0usize;
            let mut mis_ref_ms = 0.0f64;
            let mut col_open_n = 0usize;
            let mut col_open_ms = 0.0f64;

            for b in &meas {
                if b.route == ShippedRoute::Column {
                    let col_c = b.ship_mtf_inv_us();
                    let run_o = b.t_col_via_runs_front_med_us + b.t_col_via_runs_open_med_us;
                    if run_o < col_c {
                        col_open_n += 1;
                        col_open_ms += (col_c - run_o) / 1e3;
                    }
                } else if b.route.is_open() {
                    if b.t_seg_med_us < b.t_ship_med_us {
                        mis_open_n += 1;
                        mis_open_ms += (b.t_ship_med_us - b.t_seg_med_us) / 1e3;
                    }
                } else if b.t_open_med_us < b.t_ship_med_us {
                    mis_ref_n += 1;
                    mis_ref_ms += (b.t_ship_med_us - b.t_open_med_us) / 1e3;
                }
            }

            println!(
                "SUMMARY\t{name}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\
                 {ship_ms:.2}\t{ora_runs_ms:.2}\t{ora_runs_pct:+.2}%\t{ora_all_ms:.2}\t{ora_all_pct:+.2}%\t\
                 {inv_ship_ms:.2}\t{inv_ora_pct:+.2}%\t\
                 {mis_open_n}\t{mis_open_ms:.2}\t{mis_ref_n}\t{mis_ref_ms:.2}\t{col_open_n}\t{col_open_ms:.2}",
                meas.len(),
                full.len(),
                cnt(ShippedRoute::Column),
                cnt(ShippedRoute::OpenLow),
                cnt(ShippedRoute::OpenDense),
                cnt(ShippedRoute::RefusedDense),
                cnt(ShippedRoute::RefusedHigh),
                cnt(ShippedRoute::RefusedSparse),
            );

            all_corpora.push((name, blocks, meas));
        }

        // Print every full block whose shipped route disagrees with Open vs Seg median timing.
        println!("\n# DISAGREEMENT BLOCKS (full blocks n >= 100,000 where non-shipped kernel won)");
        println!(
            "MIS\tcorpus\tidx\troute\tn\trn\tmn\tthick\tfront_ms\topen_ms\tseg_ms\tship_ms\tsave_ms\topen_over_seg"
        );
        for (name, _, meas) in &all_corpora {
            for b in meas.iter().filter(|b| b.n >= 100_000) {
                if b.route == ShippedRoute::Column {
                    let col_c = b.ship_mtf_inv_us();
                    let run_o = b.t_col_via_runs_front_med_us + b.t_col_via_runs_open_med_us;
                    if run_o < col_c {
                        println!(
                            "MIS\t{name}\t{}\tCol->Open\t{}\t{:.4}\t{:.4}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                            b.idx,
                            b.n,
                            b.rn(),
                            b.mn(),
                            b.thick,
                            b.t_front_med_us / 1e3,
                            run_o / 1e3,
                            col_c / 1e3,
                            col_c / 1e3,
                            (col_c - run_o) / 1e3,
                            run_o / col_c.max(1e-3)
                        );
                    }
                } else if b.route.is_open() && b.t_seg_med_us < b.t_open_med_us {
                    println!(
                        "MIS\t{name}\t{}\t{}->Seg\t{}\t{:.4}\t{:.4}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                        b.idx,
                        b.route.as_str(),
                        b.n,
                        b.rn(),
                        b.mn(),
                        b.thick,
                        b.t_front_med_us / 1e3,
                        b.t_open_med_us / 1e3,
                        b.t_seg_med_us / 1e3,
                        b.t_ship_med_us / 1e3,
                        (b.t_open_med_us - b.t_seg_med_us) / 1e3,
                        b.t_open_med_us / b.t_seg_med_us.max(1e-3)
                    );
                } else if !b.route.is_open() && b.t_open_med_us < b.t_seg_med_us {
                    println!(
                        "MIS\t{name}\t{}\t{}->Open\t{}\t{:.4}\t{:.4}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                        b.idx,
                        b.route.as_str(),
                        b.n,
                        b.rn(),
                        b.mn(),
                        b.thick,
                        b.t_front_med_us / 1e3,
                        b.t_open_med_us / 1e3,
                        b.t_seg_med_us / 1e3,
                        b.t_ship_med_us / 1e3,
                        (b.t_seg_med_us - b.t_open_med_us) / 1e3,
                        b.t_open_med_us / b.t_seg_med_us.max(1e-3)
                    );
                }
            }
        }

        // Print binned crossover profile across ALL full run-route blocks by r/n and by m/n.
        println!("\n# CROSSOVER BY r/n BIN (all full run-route blocks across all corpora)");
        println!(
            "RN_BIN\tbin\tblks\topen_wins\topen_ms\tseg_ms\topen_over_seg\tmn_min\tmn_med\tmn_max"
        );
        let rn_edges = [
            0.0, 0.015625, 0.05, 0.10, 0.15, 0.18, 0.21, 0.24, 0.28, 0.35, 0.50, 1.01,
        ];
        let all_run_full: Vec<&BlockMeas> = all_corpora
            .iter()
            .flat_map(|(_, _, m)| m.iter())
            .filter(|b| b.n >= 100_000 && b.route != ShippedRoute::Column)
            .collect();
        for w in rn_edges.windows(2) {
            let (lo, hi) = (w[0], w[1]);
            let bin: Vec<&BlockMeas> = all_run_full
                .iter()
                .copied()
                .filter(|b| b.rn() >= lo && b.rn() < hi)
                .collect();
            if bin.is_empty() {
                continue;
            }
            let ow = bin
                .iter()
                .filter(|b| b.t_open_med_us < b.t_seg_med_us)
                .count();
            let om: f64 = bin.iter().map(|b| b.t_open_med_us).sum::<f64>() / 1e3;
            let sm: f64 = bin.iter().map(|b| b.t_seg_med_us).sum::<f64>() / 1e3;
            let mut mns: Vec<f64> = bin.iter().map(|b| b.mn()).collect();
            let mn_med = median_f64(&mut mns);
            println!(
                "RN_BIN\t[{lo:.4},{hi:.4})\t{}\t{ow}\t{om:.2}\t{sm:.2}\t{:.3}\t{:.3}\t{mn_med:.3}\t{:.3}",
                bin.len(),
                om / sm.max(1e-9),
                mns[0],
                mns.last().unwrap()
            );
        }

        println!("\n# CROSSOVER BY m/n BIN (all full run-route blocks across all corpora)");
        println!(
            "MN_BIN\tbin\tblks\topen_wins\topen_ms\tseg_ms\topen_over_seg\trn_min\trn_med\trn_max"
        );
        let mn_edges = [
            0.0, 0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40, 0.50, 0.65, 1.01,
        ];
        for w in mn_edges.windows(2) {
            let (lo, hi) = (w[0], w[1]);
            let bin: Vec<&BlockMeas> = all_run_full
                .iter()
                .copied()
                .filter(|b| b.mn() >= lo && b.mn() < hi)
                .collect();
            if bin.is_empty() {
                continue;
            }
            let ow = bin
                .iter()
                .filter(|b| b.t_open_med_us < b.t_seg_med_us)
                .count();
            let om: f64 = bin.iter().map(|b| b.t_open_med_us).sum::<f64>() / 1e3;
            let sm: f64 = bin.iter().map(|b| b.t_seg_med_us).sum::<f64>() / 1e3;
            let mut rns: Vec<f64> = bin.iter().map(|b| b.rn()).collect();
            let rn_med = median_f64(&mut rns);
            println!(
                "MN_BIN\t[{lo:.2},{hi:.2})\t{}\t{ow}\t{om:.2}\t{sm:.2}\t{:.3}\t{:.3}\t{rn_med:.3}\t{:.3}",
                bin.len(),
                om / sm.max(1e-9),
                rns[0],
                rns.last().unwrap()
            );
        }

        if eval_rules {
            println!("\n# WHOLE-CORPUS RULE EVALUATION (MTF/front + InverseBwt, interleaved, schedstat CPU + wall)");
            println!(
                "RULE\tcorpus\trule\tinner\tcpu_med_ms\tcpu_range\tcpu_pct\tcpu_disj\twall_med_ms\twall_range\twall_pct"
            );
            for (name, blocks, meas) in &all_corpora {
                eval_rules_corpus(name, blocks, meas, reps, k);
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
    eprintln!("route_crossover_probe: build with --features moonshot");
    std::process::exit(2);
}
