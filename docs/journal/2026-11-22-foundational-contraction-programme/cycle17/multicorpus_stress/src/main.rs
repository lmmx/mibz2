#![forbid(unsafe_code)]
//! Cycle 17 — 6-Corpus Adversarial & Diverse Wall-Time Stress-Test Harness
//! Kernels under test (100% unmodified):
//!   1. Open0512        : mibz2::bwt::OpenCells (shipped 0.5.12 baseline)
//!   2. LeanNoMarkXor   : /tmp/fcp-b/lean_nomark_xor_candidate.rs
//!   3. LeanOpen        : /tmp/fcp-c/lean_open_candidate.rs
//!   4. LeanBandCoalUni : /tmp/fcp-b/lean_bandcoal_uni_candidate.rs
//!
//! Zero unsafe code (`#![forbid(unsafe_code)]`). Thread CPU time is read at nanosecond
//! resolution via `/proc/thread-self/schedstat` field 0 (kernel `se.sum_exec_runtime`,
//! equivalent to `CLOCK_THREAD_CPUTIME_ID`), alongside monotonic wall time (`Instant::now()`).

pub use mibz2::{bwt, runs, Kernels};

mod corpora;
#[allow(clippy::all, dead_code)]
mod probe_bc;
#[allow(clippy::all, dead_code)]
mod probe_open;

#[path = "/tmp/fcp-b/lean_nomark_xor_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod lean_nomark_xor;

#[path = "/tmp/fcp-c/lean_open_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod lean_open;

#[path = "/tmp/fcp-b/lean_bandcoal_uni_candidate.rs"]
#[allow(clippy::all, dead_code)]
mod lean_bandcoal_uni;

use corpora::{BlockItem, CorpusSpec};
use lean_bandcoal_uni::LeanBandCoalUni;
use lean_nomark_xor::LeanNoMarkXor;
use lean_open::LeanOpen;
use mibz2::bwt::{InverseBwt, LazyCells, OpenCells};
use std::hint::black_box;
use std::time::Instant;

const LIMIT: u32 = 16;
const LAZY_DENSITY: (usize, usize) = (3, 20);
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

/// Exact thread CPU runtime in nanoseconds via `/proc/thread-self/schedstat` field 0
/// (`se.sum_exec_runtime`, kernel-maintained equivalent of `CLOCK_THREAD_CPUTIME_ID`).
#[inline]
fn thread_cpu_ns() -> u64 {
    let s = std::fs::read_to_string("/proc/thread-self/schedstat")
        .expect("read /proc/thread-self/schedstat");
    s.split_whitespace()
        .next()
        .expect("schedstat field 0")
        .parse::<u64>()
        .expect("schedstat u64")
}

struct BlockStats {
    n: usize,
    r0_records: usize,
    r_true: usize,
    ns_thick: usize,
    m16_thin: u64,
    admitted_open: bool,
    regime: &'static str,
    bands_open: usize,
    bands_bc: usize,
}

fn analyze_block(b: &BlockItem, k: Kernels) -> BlockStats {
    let recs = b.runs.records();
    let r0 = recs.len();
    let mut r_true = 0usize;
    let mut ns_thick = 0usize;
    let mut m16_thin = 0u64;
    let mut i = 0usize;
    while i < recs.len() {
        let c = (recs[i] >> 8) as u8;
        let mut w = u32::from(recs[i] & 255);
        i += 1;
        while i < recs.len() && (recs[i] >> 8) as u8 == c {
            w += u32::from(recs[i] & 255);
            i += 1;
        }
        r_true += 1;
        if w > LIMIT {
            ns_thick += 1;
        } else {
            m16_thin += u64::from(w);
        }
    }
    let (cap_m, cap_strips, regime) = if b.n == 0 || b.n > 900_000 || b.p as usize >= b.n {
        (0u32, 0usize, "FB_RANGE")
    } else if r0 * LAZY_DENSITY.1 < b.n * LAZY_DENSITY.0 && r0 * 64 >= b.n {
        (u32::MAX, usize::MAX, "OPEN_STD")
    } else if r0 * 64 >= b.n && r0 * DENSE.1 < b.n * DENSE.0 {
        (DENSE_M.min((b.n as u64 * 3 / 10) as u32), DENSE_STRIPS, "OPEN_DENSE")
    } else if r0 * 64 < b.n {
        (0u32, 0usize, "FB_ULTRA_REP")
    } else {
        (0u32, 0usize, "FB_HIGH_ENT")
    };
    let admitted = cap_m > 0 && m16_thin > 0 && m16_thin <= u64::from(cap_m) && ns_thick <= cap_strips;
    let final_regime = if admitted {
        regime
    } else if cap_m > 0 {
        "FB_CAP_EXCEED"
    } else {
        regime
    };

    let (bands_open, bands_bc) = if admitted {
        let mut po = probe_open::LeanOpen::default();
        let mut pb = probe_bc::LeanBandCoalUni::default();
        let mut out = Vec::with_capacity(b.n + 32);
        po.invert_runs(&b.runs, b.n, b.p, &mut out, k);
        let bo = po.last_bands();
        out.clear();
        pb.invert_runs(&b.runs, b.n, b.p, &mut out, k);
        let bb = pb.last_bands();
        (bo, bb)
    } else {
        (0, 0)
    };

    BlockStats {
        n: b.n,
        r0_records: r0,
        r_true,
        ns_thick,
        m16_thin,
        admitted_open: admitted,
        regime: final_regime,
        bands_open,
        bands_bc,
    }
}

fn median_f64(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    let m = v.len();
    if m % 2 == 1 {
        v[m / 2]
    } else {
        0.5 * (v[m / 2 - 1] + v[m / 2])
    }
}

fn iqr_f64(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    (v[n / 4], v[n / 2], v[(3 * n) / 4])
}

/// Latin-square / rotated+reversed order across 4 arms per round
fn arm_order(rnd: usize) -> [usize; 4] {
    let mut ord = [0, 1, 2, 3];
    ord.rotate_left(rnd % 4);
    if (rnd / 4) % 2 == 1 {
        ord.reverse();
    }
    ord
}

fn run_arm_once<K: InverseBwt>(
    kern: &mut K,
    blocks: &[BlockItem],
    out: &mut Vec<u8>,
    k: Kernels,
) -> (f64, f64) {
    let c0 = thread_cpu_ns();
    let w0 = Instant::now();
    for b in blocks {
        out.clear();
        let ok = kern.invert_runs(&b.runs, b.n, b.p, out, k);
        black_box(ok);
        black_box(out.as_slice());
    }
    let wall_ms = w0.elapsed().as_secs_f64() * 1e3;
    let cpu_ms = (thread_cpu_ns().saturating_sub(c0)) as f64 * 1e-6;
    (cpu_ms, wall_ms)
}

fn verify_corpus(spec: &CorpusSpec, k: Kernels) -> (usize, usize) {
    let mut oracle = LazyCells::default();
    let mut k0 = OpenCells::default();
    let mut k1 = LeanNoMarkXor::default();
    let mut k2 = LeanOpen::default();
    let mut k3 = LeanBandCoalUni::default();

    let mut ref_out = Vec::with_capacity(900_032);
    let mut out = Vec::with_capacity(900_032);
    let mut pass = 0usize;
    let total = spec.blocks.len();

    for (bi, b) in spec.blocks.iter().enumerate() {
        ref_out.clear();
        assert!(
            oracle.invert_runs(&b.runs, b.n, b.p, &mut ref_out, k),
            "{}: oracle LazyCells failed on block {bi}",
            spec.id
        );
        for (name, res) in [
            ("Open0512", {
                out.clear();
                let ok = k0.invert_runs(&b.runs, b.n, b.p, &mut out, k);
                ok && out == ref_out
            }),
            ("LeanNoMarkXor", {
                out.clear();
                let ok = k1.invert_runs(&b.runs, b.n, b.p, &mut out, k);
                ok && out == ref_out
            }),
            ("LeanOpen", {
                out.clear();
                let ok = k2.invert_runs(&b.runs, b.n, b.p, &mut out, k);
                ok && out == ref_out
            }),
            ("LeanBandCoalUni", {
                out.clear();
                let ok = k3.invert_runs(&b.runs, b.n, b.p, &mut out, k);
                ok && out == ref_out
            }),
        ] {
            assert!(
                res,
                "BYTE MISMATCH in {} on corpus {} block {} (n={}, p={})",
                name, spec.id, bi, b.n, b.p
            );
        }
        pass += 1;
    }
    (pass, total)
}

fn bench_corpus(spec: &CorpusSpec, rounds: usize, k: Kernels) {
    let mut k0 = OpenCells::default();
    let mut k1 = LeanNoMarkXor::default();
    let mut k2 = LeanOpen::default();
    let mut k3 = LeanBandCoalUni::default();

    let mut o0 = Vec::with_capacity(900_032);
    let mut o1 = Vec::with_capacity(900_032);
    let mut o2 = Vec::with_capacity(900_032);
    let mut o3 = Vec::with_capacity(900_032);

    // 2 warm-up rounds to pre-size all kernel scratch buffers
    for _ in 0..2 {
        let _ = run_arm_once(&mut k0, &spec.blocks, &mut o0, k);
        let _ = run_arm_once(&mut k1, &spec.blocks, &mut o1, k);
        let _ = run_arm_once(&mut k2, &spec.blocks, &mut o2, k);
        let _ = run_arm_once(&mut k3, &spec.blocks, &mut o3, k);
    }

    let mut cpu_ms: [Vec<f64>; 4] = std::array::from_fn(|_| Vec::with_capacity(rounds));
    let mut wall_ms: [Vec<f64>; 4] = std::array::from_fn(|_| Vec::with_capacity(rounds));

    for rnd in 0..rounds {
        for &arm in &arm_order(rnd) {
            let (c, w) = match arm {
                0 => run_arm_once(&mut k0, &spec.blocks, &mut o0, k),
                1 => run_arm_once(&mut k1, &spec.blocks, &mut o1, k),
                2 => run_arm_once(&mut k2, &spec.blocks, &mut o2, k),
                3 => run_arm_once(&mut k3, &spec.blocks, &mut o3, k),
                _ => unreachable!(),
            };
            cpu_ms[arm].push(c);
            wall_ms[arm].push(w);
        }
    }

    let base_cpu_med = median_f64(cpu_ms[0].clone());
    let base_wall_med = median_f64(wall_ms[0].clone());

    let names = ["Open0512", "LeanNoMarkXor", "LeanOpen", "LeanBandCoalUni"];
    println!(
        "  [measured] TIMING ({rounds} interleaved rounds; baseline Open0512: cpu={base_cpu_med:.3} ms, wall={base_wall_med:.3} ms):"
    );
    for arm in 0..4 {
        let c_med = median_f64(cpu_ms[arm].clone());
        let w_med = median_f64(wall_ms[arm].clone());
        let paired_cpu_ratios: Vec<f64> = cpu_ms[arm]
            .iter()
            .zip(&cpu_ms[0])
            .map(|(a, b)| a / b)
            .collect();
        let paired_wall_ratios: Vec<f64> = wall_ms[arm]
            .iter()
            .zip(&wall_ms[0])
            .map(|(a, b)| a / b)
            .collect();
        let (cq1, cmed_r, cq3) = iqr_f64(paired_cpu_ratios);
        let (wq1, wmed_r, wq3) = iqr_f64(paired_wall_ratios);
        println!(
            "    {:<16} | CPU: {:8.3} ms | ratio(med_paired)={:.4}x IQR=[{:.4}, {:.4}] | WALL: {:8.3} ms | wall_ratio={:.4}x IQR=[{:.4}, {:.4}]",
            names[arm], c_med, cmed_r, cq1, cq3, w_med, wmed_r, wq1, wq3
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let stats_only = args.iter().any(|a| a == "--stats-only");
    let rounds_small: usize = args
        .windows(2)
        .find(|w| w[0] == "--rounds")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(21);
    let rounds_431: usize = args
        .windows(2)
        .find(|w| w[0] == "--rounds-431")
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(9);

    let k = Kernels::detect();
    println!("================================================================================");
    println!("Cycle 17 Multi-Corpus Stress-Test Harness (#![forbid(unsafe_code)])");
    println!("Kernels: Open0512 vs LeanNoMarkXor vs LeanOpen vs LeanBandCoalUni");
    println!("Mode: {} | rounds_small={rounds_small}, rounds_431={rounds_431}", if stats_only { "STATS+VERIFY ONLY" } else { "FULL BENCHMARK" });
    println!("================================================================================");

    let (wiki_6, wiki_431) = corpora::load_wikidata(k);
    let specs: Vec<(CorpusSpec, usize)> = vec![
        (wiki_6, rounds_small),
        (wiki_431, rounds_431),
        (corpora::gen_hi_ent_base64_fallback(k), rounds_small),
        (corpora::gen_hi_ent_dense_open(k), rounds_small),
        (corpora::gen_english_prose(k), rounds_small),
        (corpora::gen_dna_open(k), rounds_small),
        (corpora::gen_dna_iid(k), rounds_small),
        (corpora::gen_archive_open(k), rounds_small),
        (corpora::gen_archive_rho85(k), rounds_small),
        (corpora::gen_adv_giant_thin_cycle(k), rounds_small),
        (corpora::gen_adv_fib_altrun(k), rounds_small),
    ];

    for (spec, rnds) in &specs {
        println!("\n--------------------------------------------------------------------------------");
        println!("CORPUS [{}]: {} ({} block(s))", spec.id, spec.label, spec.blocks.len());

        let mut tot_n = 0usize;
        let mut tot_r0 = 0usize;
        let mut tot_rtrue = 0usize;
        let mut tot_ns = 0usize;
        let mut tot_m16 = 0u64;
        let mut tot_bands_open = 0usize;
        let mut tot_bands_bc = 0usize;
        let mut admitted_cnt = 0usize;
        let mut first_regime = "";

        for (i, b) in spec.blocks.iter().enumerate() {
            let st = analyze_block(b, k);
            if i == 0 {
                first_regime = st.regime;
            }
            tot_n += st.n;
            tot_r0 += st.r0_records;
            tot_rtrue += st.r_true;
            tot_ns += st.ns_thick;
            tot_m16 += st.m16_thin;
            tot_bands_open += st.bands_open;
            tot_bands_bc += st.bands_bc;
            if st.admitted_open {
                admitted_cnt += 1;
            }
        }
        let rho = tot_n as f64 / tot_r0.max(1) as f64;
        let rho_true = tot_n as f64 / tot_rtrue.max(1) as f64;
        let avg_m16 = tot_m16 as f64 / spec.blocks.len().max(1) as f64;
        let avg_edges_kib = (avg_m16 * 4.0) / 1024.0;
        let max_edges_kib = spec
            .blocks
            .iter()
            .map(|b| {
                let st = analyze_block(b, k);
                (st.m16_thin as f64 * 4.0) / 1024.0
            })
            .fold(0.0f64, f64::max);

        println!(
            "  [measured] STRUCT: n={tot_n} | r_0={tot_r0} (rho={rho:.2}, r_true={tot_rtrue}, rho_true={rho_true:.2}) | ns_thick={tot_ns} | m_16={tot_m16} (avg_edges={avg_edges_kib:.1} KiB, max_edges={max_edges_kib:.1} KiB)"
        );
        println!(
            "  [measured] OPEN:   admitted={admitted_cnt}/{} (blk0={first_regime}) | bands_open={tot_bands_open} | bands_bc={tot_bands_bc} (delta={:+.1}%)",
            spec.blocks.len(),
            if tot_bands_open > 0 {
                100.0 * (tot_bands_bc as f64 - tot_bands_open as f64) / tot_bands_open as f64
            } else {
                0.0
            }
        );

        let (pass, total) = verify_corpus(spec, k);
        println!("  [verified] EXACT:  {pass}/{total} blocks 100% byte-exact across ALL 4 kernels vs LazyCells oracle");

        if !stats_only {
            bench_corpus(spec, *rnds, k);
        }
    }
    println!("\n================================================================================");
    println!("DONE.");
}
