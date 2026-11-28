//! (research only, `moonshot` + `census`) Which inverse-BWT route each block of a file takes in
//! production's kernel (`OpenCells`), and the time each route gets, on one thread:
//!
//!     cargo run --release --features moonshot,census --example route_census -- FILE.bz2... [REPEATS]
//!     cargo run --release --features moonshot --example route_census -- FILE.bz2... [REPEATS]
//!
//! The first build routes exactly but its timings carry the census's own O(n) pass; the
//! second (`moonshot` alone) times production's kernel as shipped and routes by Open's count
//! gates, so a dense-band block Open refused at its cap shows as `open` there.
//!
//! For every block: `column` (the MTF stage kept the column: `runs::takes_runs` refused it),
//! `open` (the open kernel took it: `census::take` holds its record) or `runs-not-open` (the run
//! route, refused by Open's gates or its dense cap: `LazyCells`, then `Segmented`). The run
//! route's r/n (records per row) is printed in Open's bands: below `LazyCells`' line 0.15, the
//! dense band to 0.28, above. Then the whole file is decoded `REPEATS` times (default 5) by
//! `decode_with` with the probe (production's `OpenCells`), with `Segmented` alone and with `LazyCells` alone, best kept,
//! and checked byte-equal. The inverse's own time per route is summed from the probe's timers.
//! A characterisation, not a benchmark (docs/journal/2026-11-27-explore/route-census.md).
//!
//! `ROUTE_FULL=N` counts blocks of at least N rows as full (default 100,000; a bzip2 -1 block has
//! at most 99,981 rows). `ROUTE_SHADOW=1` also times `Segmented`'s inverse on each run-route
//! block just after the probe's, `ROUTE_SHADOW=first` just before it (the order control), one
//! line a block on stderr (docs/notebook/2026-11-28-explore-block-size-route.md).
//!
//! Without `moonshot` (the run records are crate-private) it only says so.
#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::bwt::{InverseBwt, LazyCells, OpenCells, Segmented};
    use mibz2::runs::Runs;
    use mibz2::Kernels;
    use std::sync::Mutex;
    use std::time::Instant;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Route {
        Column,
        Open,
        RunsNotOpen,
    }

    #[derive(Clone, Copy, Debug)]
    struct Blk {
        route: Route,
        n: usize,
        records: usize,
        ns: u128,
        /// with ROUTE_SHADOW=1: `Segmented`'s own inverse on the same block, timed just after
        shadow_ns: u128,
        /// from the records alone: rows in maximal runs of at most LIMIT (16) rows, and the count
        /// of longer runs (the census's `m` less the marked row; thick runs, an upper bound on
        /// strips)
        m_est: usize,
        thick: usize,
    }

    /// `m_est` and `thick` of a block's records (`byte << 8 | length`, a run cut at 127 rows
    /// continuing in the next record of the same byte)
    fn thin_rows(rec: &[u16]) -> (usize, usize) {
        let (mut m, mut thick, mut cur, mut len) = (0usize, 0usize, u16::MAX, 0usize);
        let mut flush = |len: usize| {
            if len > 0 && len <= 16 {
                m += len
            } else if len > 16 {
                thick += 1
            }
        };
        for &x in rec {
            let (b, l) = (x >> 8, (x & 0xFF) as usize);
            if b == cur {
                len += l;
            } else {
                flush(len);
                cur = b;
                len = l;
            }
        }
        flush(len);
        (m, thick)
    }

    static LOG: Mutex<Vec<Blk>> = Mutex::new(Vec::new());

    #[derive(Default)]
    struct Probe(OpenCells, Segmented, Vec<u8>);

    fn shadow_first() -> bool {
        std::env::var_os("ROUTE_SHADOW").is_some_and(|v| v == "first")
    }

    fn shadow() -> bool {
        std::env::var_os("ROUTE_SHADOW").is_some_and(|v| v == "1" || v == "first")
    }

    impl InverseBwt for Probe {
        fn invert(&mut self, ll: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
            let t = Instant::now();
            self.0.invert(ll, c, p, out);
            let ns = t.elapsed().as_nanos();
            LOG.lock().unwrap().push(Blk {
                route: Route::Column,
                n: ll.len(),
                records: 0,
                ns,
                shadow_ns: 0,
                m_est: 0,
                thick: 0,
            });
        }
        fn invert_in_place(&mut self, ll: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
            let n = ll.len();
            let t = Instant::now();
            let ok = self.0.invert_in_place(ll, c, p);
            if ok {
                let ns = t.elapsed().as_nanos();
                LOG.lock().unwrap().push(Blk {
                    route: Route::Column,
                    n,
                    records: 0,
                    ns,
                    shadow_ns: 0,
                    m_est: 0,
                    thick: 0,
                });
            }
            ok
        }
        fn wants_checkpoints(&self) -> bool {
            self.0.wants_checkpoints()
        }
        fn invert_checkpointed(
            &mut self,
            ll: &[u8],
            c: &[u32; 256],
            cp: &[[u32; 256]],
            p: u32,
            out: &mut Vec<u8>,
        ) {
            let t = Instant::now();
            self.0.invert_checkpointed(ll, c, cp, p, out);
            let ns = t.elapsed().as_nanos();
            LOG.lock().unwrap().push(Blk {
                route: Route::Column,
                n: ll.len(),
                records: 0,
                ns,
                shadow_ns: 0,
                m_est: 0,
                thick: 0,
            });
        }
        fn reads_runs(&self) -> bool {
            true
        }
        fn invert_runs(
            &mut self,
            r: &Runs,
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
            k: Kernels,
        ) -> bool {
            #[cfg(feature = "census")]
            let _ = mibz2::census::take();
            // ROUTE_SHADOW=first: `Segmented`'s shadow timed before the probe instead of after
            // (the order bias: the second inverse meets the block's records warm)
            let mut first_ns = None;
            if shadow_first() {
                let t = Instant::now();
                assert!(self.1.invert_runs(r, n, p, &mut self.2, k));
                first_ns = Some(t.elapsed().as_nanos());
            }
            let t = Instant::now();
            let ok = self.0.invert_runs(r, n, p, out, k);
            let ns = t.elapsed().as_nanos();
            // With `census`, exactly whether Open took it. Without (the timing build: the census's
            // own O(n) pass would be timed too), Open's count gates (`OpenCellsOf::cap`), which
            // count a dense-band block Open refused at its cap as open.
            #[cfg(feature = "census")]
            let opened = !mibz2::census::take().is_empty();
            #[cfg(not(feature = "census"))]
            let opened = {
                let rec = r.records().len();
                n <= 900_000 && (p as usize) < n && rec * 64 >= n && rec * 25 < n * 7
            };
            let mut shadow_ns = 0;
            if let Some(f) = first_ns {
                shadow_ns = f;
                if ok {
                    assert!(self.2 == *out, "shadow differs");
                }
            } else if ok && shadow() {
                let t = Instant::now();
                assert!(self.1.invert_runs(r, n, p, &mut self.2, k));
                shadow_ns = t.elapsed().as_nanos();
                assert!(self.2 == *out, "shadow differs");
            }
            if ok {
                let (m_est, thick) = thin_rows(r.records());
                LOG.lock().unwrap().push(Blk {
                    route: if opened {
                        Route::Open
                    } else {
                        Route::RunsNotOpen
                    },
                    n,
                    records: r.records().len(),
                    ns,
                    shadow_ns,
                    m_est,
                    thick,
                });
            }
            ok
        }
    }

    fn best<F: FnMut() -> Vec<u8>>(reps: usize, mut f: F) -> (f64, Vec<u8>) {
        let mut b = f64::MAX;
        let mut out = Vec::new();
        for _ in 0..reps {
            let t = Instant::now();
            out = f();
            b = b.min(t.elapsed().as_secs_f64());
        }
        (b, out)
    }

    pub fn main() {
        let mut a: Vec<String> = std::env::args().skip(1).collect();
        let reps = match a.last().and_then(|s| s.parse::<usize>().ok()) {
            Some(r) => {
                a.pop();
                r
            }
            None => 5,
        };
        let k = Kernels::detect();
        println!(
            "file\tblocks\tfull\tcolumn\topen\truns_not_open\trn_lt015\trn_015_028\trn_ge028\
             \trn_p10\trn_p50\trn_p90\tbytes\tinv_ms_open\tinv_ms_runs_not_open\tinv_ms_column\
             \tprobe_ms\tsegmented_ms\tlazy_ms\tprobe_over_segmented"
        );
        for f in &a {
            let data = std::fs::read(f).expect("read");
            LOG.lock().unwrap().clear();
            let (tp, out_p) = best(reps, || {
                LOG.lock().unwrap().clear();
                mibz2::decode_with::<Probe>(&data, k).expect("decode")
            });
            let log = LOG.lock().unwrap().clone();
            let (ts, out_s) = best(reps, || {
                mibz2::decode_with::<Segmented>(&data, k).expect("seg")
            });
            let (tl, out_l) = best(reps, || {
                mibz2::decode_with::<LazyCells>(&data, k).expect("lazy")
            });
            assert!(out_p == out_s && out_p == out_l, "{f}: outputs differ");
            // "full": at least ROUTE_FULL rows (default 100,000, Wikidata's -9 blocks; a
            // bzip2 -1 block is at most 99,981 rows, so ROUTE_FULL=90000 counts those)
            let min_full: usize = std::env::var("ROUTE_FULL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000);
            let full: Vec<&Blk> = log.iter().filter(|b| b.n >= min_full).collect();
            let cnt = |r: Route| full.iter().filter(|b| b.route == r).count();
            let mut rn: Vec<f64> = full
                .iter()
                .filter(|b| b.route != Route::Column)
                .map(|b| b.records as f64 / b.n as f64)
                .collect();
            rn.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let q = |p: f64| {
                if rn.is_empty() {
                    f64::NAN
                } else {
                    rn[((rn.len() - 1) as f64 * p).round() as usize]
                }
            };
            let band = |lo: f64, hi: f64| rn.iter().filter(|&&x| x >= lo && x < hi).count();
            let ms = |r: Route| {
                log.iter()
                    .filter(|b| b.route == r)
                    .map(|b| b.ns)
                    .sum::<u128>() as f64
                    / 1e6
            };
            let name = std::path::Path::new(f)
                .file_name()
                .unwrap()
                .to_string_lossy();
            if shadow() {
                for b in log
                    .iter()
                    .filter(|b| b.route != Route::Column && b.n >= min_full)
                {
                    eprintln!(
                        "{name}\t{:?}\t{}\t{:.4}\t{:.3}\t{:.3}\t{}\t{}\t{}",
                        b.route,
                        b.n,
                        b.records as f64 / b.n as f64,
                        b.ns as f64 / 1e6,
                        b.shadow_ns as f64 / 1e6,
                        b.m_est,
                        b.thick,
                        256_000.min(b.n * 3 / 10)
                    );
                }
            }
            println!(
                "{name}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.3}",
                log.len(),
                full.len(),
                cnt(Route::Column),
                cnt(Route::Open),
                cnt(Route::RunsNotOpen),
                band(0.0, 0.15),
                band(0.15, 0.28),
                band(0.28, f64::MAX),
                q(0.1),
                q(0.5),
                q(0.9),
                out_p.len(),
                ms(Route::Open),
                ms(Route::RunsNotOpen),
                ms(Route::Column),
                tp * 1e3,
                ts * 1e3,
                tl * 1e3,
                tp / ts
            );
        }
    }
}

#[cfg(feature = "moonshot")]
fn main() {
    imp::main()
}

#[cfg(not(feature = "moonshot"))]
fn main() {
    eprintln!("route_census: build with --features moonshot (and census for exact routes)");
    std::process::exit(2);
}
