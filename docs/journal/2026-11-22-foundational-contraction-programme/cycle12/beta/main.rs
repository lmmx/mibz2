//! Cycle 12 Same-Binary Benchmark & Exactness Harness (`lto = "fat"`, `cgu = 1`, 21 rounds, 1T)
//! Paired A/B + A/A against `v0.5.12 OpenCells` from `/tmp/moonshot-scorer` on:
//!   (a) the 6 SHA-256 authenticated Wikidata blocks (`/tmp/mibz2-blocks/block-*.L`), AND
//!   (b) all 431 blocks of `/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2`.

pub use mibz2::{bwt, runs, Kernels};

#[path = "/tmp/fcp-b/dt_widest_open_candidate.rs"]
pub mod candidate;

use candidate::{
    BandCoalOpen, BandCoalPlainUni, BandCoalWordOnly, DtWidestOpen, DtWidestPlainUni, EmbOpen,
    StageStats,
};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{self as mruns, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use std::hint::black_box;
use std::time::Instant;

struct BlockItem {
    corpus_idx: usize,
    n: usize,
    p: u32,
    runs: Runs,
}

fn expand_records(records: &[u16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(900_000);
    for &rec in records {
        let c = (rec >> 8) as u8;
        let w = (rec & 255) as usize;
        out.resize(out.len() + w, c);
    }
    out
}

fn load_wikidata_blocks() -> (Vec<BlockItem>, Vec<(String, usize)>) {
    let path = "/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2";
    let data = std::fs::read(path).expect("read wikidata.bz2");
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut blocks = Vec::new();
    let mut idx = 0usize;
    while let Ok(ev) = sc.next(Vec::new()) {
        match ev {
            Event::Block { symbols, .. } => {
                let mut r = Runs::default();
                let n = mruns::front(&symbols, &mut r, k).expect("runs::front");
                blocks.push(BlockItem {
                    corpus_idx: idx,
                    n,
                    p: symbols.orig_ptr,
                    runs: r,
                });
                idx += 1;
            }
            Event::Done => break,
        }
    }

    // Authenticate the 6 blocks byte-for-byte against /tmp/mibz2-blocks/block-*.L
    let auth_specs = [
        ("block-0.L", 264045u32),
        ("block-2.L", 320807u32),
        ("block-85.L", 325088u32),
        ("block-143.L", 802868u32),
        ("block-210.L", 791429u32),
        ("block-207.L", 205180u32),
    ];
    let mut auth_indices = Vec::new();
    for (fname, p_want) in auth_specs {
        let raw = std::fs::read(format!("/tmp/mibz2-blocks/{fname}")).expect(fname);
        let found = blocks
            .iter()
            .position(|b| b.n == raw.len() && b.p == p_want && expand_records(b.runs.records()) == raw)
            .unwrap_or_else(|| panic!("{fname}: no matching block in wikidata.bz2"));
        auth_indices.push((fname.to_string(), found));
    }
    (blocks, auth_indices)
}

fn verify_exact<K: InverseBwt + Default>(
    name: &str,
    blocks: &[BlockItem],
    refs: &[Vec<u8>],
    k: Kernels,
) {
    let mut kern = K::default();
    let mut out = Vec::new();
    for (i, b) in blocks.iter().enumerate() {
        out.clear();
        let ok = kern.invert_runs(&b.runs, b.n, b.p, &mut out, k);
        assert!(ok, "{name}: invert_runs returned false on block {i}");
        assert_eq!(
            out, refs[i],
            "{name}: EXACT BYTE MISMATCH on corpus block {i} (n={}, p={})",
            b.n, b.p
        );
    }
    println!("[verified] {name:18}: 100% byte-for-byte match across all {} blocks", blocks.len());
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
    (v[n / 4], v[n / 2], v[3 * n / 4])
}

/// Paired interleaved A/B benchmark over `rounds` rounds (alternating A-B and B-A order!)
fn paired_bench<A: InverseBwt + Default, B: InverseBwt + Default>(
    blocks: &[&BlockItem],
    rounds: usize,
    k: Kernels,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut a = A::default();
    let mut b = B::default();
    let mut out_a = Vec::with_capacity(900_032);
    let mut out_b = Vec::with_capacity(900_032);

    // 3 warmup rounds so both kernels' buffers reach steady-state capacity
    for _ in 0..3 {
        for blk in blocks {
            a.invert_runs(&blk.runs, blk.n, blk.p, &mut out_a, k);
            b.invert_runs(&blk.runs, blk.n, blk.p, &mut out_b, k);
        }
    }

    let mut ms_a = Vec::with_capacity(rounds);
    let mut ms_b = Vec::with_capacity(rounds);
    let mut ratios = Vec::with_capacity(rounds);

    for r in 0..rounds {
        let (ta, tb) = if r % 2 == 0 {
            let t0 = Instant::now();
            for blk in blocks {
                black_box(a.invert_runs(&blk.runs, blk.n, blk.p, &mut out_a, k));
            }
            let dt_a = t0.elapsed().as_secs_f64() * 1000.0;
            let t1 = Instant::now();
            for blk in blocks {
                black_box(b.invert_runs(&blk.runs, blk.n, blk.p, &mut out_b, k));
            }
            let dt_b = t1.elapsed().as_secs_f64() * 1000.0;
            (dt_a, dt_b)
        } else {
            let t1 = Instant::now();
            for blk in blocks {
                black_box(b.invert_runs(&blk.runs, blk.n, blk.p, &mut out_b, k));
            }
            let dt_b = t1.elapsed().as_secs_f64() * 1000.0;
            let t0 = Instant::now();
            for blk in blocks {
                black_box(a.invert_runs(&blk.runs, blk.n, blk.p, &mut out_a, k));
            }
            let dt_a = t0.elapsed().as_secs_f64() * 1000.0;
            (dt_a, dt_b)
        };
        ms_a.push(ta);
        ms_b.push(tb);
        ratios.push(tb / ta);
    }
    (ms_a, ms_b, ratios)
}

fn profile_arm<const MODE: u8, const UNI: bool>(
    name: &str,
    blocks: &[&BlockItem],
    rounds: usize,
    k: Kernels,
) {
    let mut arm = candidate::OpenArm::<MODE, UNI>::default();
    let mut out = Vec::with_capacity(900_032);
    // warmup
    for blk in blocks {
        arm.invert_runs_timed(&blk.runs, blk.n, blk.p, &mut out, k);
    }

    let mut s1_v = Vec::new();
    let mut s2_v = Vec::new();
    let mut s3a_v = Vec::new();
    let mut s4_v = Vec::new();
    let mut em_v = Vec::new();
    let mut tot_v = Vec::new();
    let mut agg = StageStats::default();
    let mut opened_cnt = 0usize;

    for r in 0..rounds {
        let (mut s1, mut s2, mut s3a, mut s4, mut em) = (0u64, 0u64, 0u64, 0u64, 0u64);
        let mut fallback_ns = 0u64;
        if r == 0 {
            agg = StageStats::default();
            opened_cnt = 0;
        }
        for blk in blocks {
            let t0 = Instant::now();
            let (_, st_opt) = arm.invert_runs_timed(&blk.runs, blk.n, blk.p, &mut out, k);
            let dt = t0.elapsed().as_nanos() as u64;
            if let Some(st) = st_opt {
                s1 += st.t_s1_ns;
                s2 += st.t_s2_ns;
                s3a += st.t_s3a_ns;
                s4 += st.t_s4_ns;
                em += st.t_emit_ns;
                if r == 0 {
                    opened_cnt += 1;
                    agg.retained += st.retained;
                    agg.thick += st.thick;
                    agg.steps += st.steps;
                    agg.splits += st.splits;
                    agg.rules += st.rules;
                    agg.short_rules += st.short_rules;
                    agg.long_rules += st.long_rules;
                    agg.bands += st.bands;
                    agg.dict_words += st.dict_words;
                    agg.spelling_bytes += st.spelling_bytes;
                    agg.peak_scratch_bytes = agg.peak_scratch_bytes.max(st.peak_scratch_bytes);
                    agg.dt_queries += st.dt_queries;
                    agg.dt_scanned += st.dt_scanned;
                    agg.dt_matched += st.dt_matched;
                    agg.dt_pg_prev_skips += st.dt_pg_prev_skips;
                    agg.dt_pg_prev_scans += st.dt_pg_prev_scans;
                }
            } else {
                fallback_ns += dt;
            }
        }
        s1_v.push(s1 as f64 / 1e6);
        s2_v.push(s2 as f64 / 1e6);
        s3a_v.push(s3a as f64 / 1e6);
        s4_v.push(s4 as f64 / 1e6);
        em_v.push(em as f64 / 1e6);
        tot_v.push((s1 + s2 + s3a + s4 + em + fallback_ns) as f64 / 1e6);
    }

    let ms_s1 = median_f64(s1_v);
    let ms_s2 = median_f64(s2_v);
    let ms_s3a = median_f64(s3a_v);
    let ms_s4 = median_f64(s4_v);
    let ms_em = median_f64(em_v);
    let ms_tot = median_f64(tot_v);
    println!(
        "  {name:18} | S1={ms_s1:6.3} S2={ms_s2:6.3} S3a={ms_s3a:6.3} S4={ms_s4:6.3} Emit={ms_em:6.3} Sum={ms_tot:6.3}ms | \
         Steps={:7} Splits={:7} Rules={:7} (L={:6}) Bands={:7} Dict={:6} Spell={:5}KiB PeakM={:5.1}KiB (open={}/{})",
        agg.steps,
        agg.splits,
        agg.rules,
        agg.long_rules,
        agg.bands,
        agg.dict_words,
        agg.spelling_bytes / 1024,
        agg.peak_scratch_bytes as f64 / 1024.0,
        opened_cnt,
        blocks.len(),
    );
    if agg.dt_queries > 0 {
        let ratio = agg.dt_scanned as f64 / agg.dt_matched.max(1) as f64;
        let skip_pct = 100.0 * agg.dt_pg_prev_skips as f64
            / (agg.dt_pg_prev_skips + agg.dt_pg_prev_scans).max(1) as f64;
        println!(
            "    [DT diag] queries={} scanned={} matched={} (scan/match={:.2}x) | pg_prev: skip={} scan={} ({:.1}% skipped)",
            agg.dt_queries, agg.dt_scanned, agg.dt_matched, ratio,
            agg.dt_pg_prev_skips, agg.dt_pg_prev_scans, skip_pct
        );
    }
}

fn print_ab(label: &str, ms_a: Vec<f64>, ms_b: Vec<f64>, ratios: Vec<f64>) {
    let med_a = median_f64(ms_a);
    let med_b = median_f64(ms_b);
    let (q1, med_r, q3) = iqr_f64(ratios);
    let pct = (med_r - 1.0) * 100.0;
    println!(
        "  {label:28}: OpenCells={med_a:7.3}ms | Arm={med_b:7.3}ms | Ratio={med_r:.4}x (IQR [{q1:.4}, {q3:.4}], {pct:+.2}%)"
    );
}

fn main() {
    let k = Kernels::detect();
    let (all_blocks, auth_indices) = load_wikidata_blocks();
    println!(
        "Loaded {} wikidata.bz2 blocks; authenticated {} SHA-256 blocks.",
        all_blocks.len(),
        auth_indices.len()
    );

    // Reference outputs via unmodified v0.5.12 OpenCells
    let mut ref_kern = OpenCells::default();
    let mut refs = Vec::with_capacity(all_blocks.len());
    for b in &all_blocks {
        let mut out = Vec::new();
        assert!(ref_kern.invert_runs(&b.runs, b.n, b.p, &mut out, k));
        refs.push(out);
    }

    // 100% Exactness verification across ALL 431 wikidata blocks
    verify_exact::<EmbOpen>("Arm0 EmbOpen", &all_blocks, &refs, k);
    verify_exact::<BandCoalOpen>("Arm1 BandCoalOpen", &all_blocks, &refs, k);
    verify_exact::<BandCoalWordOnly>("Arm1b BandCoalWord", &all_blocks, &refs, k);
    verify_exact::<DtWidestOpen>("Arm2 DtWidestOpen", &all_blocks, &refs, k);
    verify_exact::<BandCoalPlainUni>("Arm3a BandCoal+Uni", &all_blocks, &refs, k);
    verify_exact::<DtWidestPlainUni>("Arm3b DtWidest+Uni", &all_blocks, &refs, k);

    let auth_refs: Vec<&BlockItem> = auth_indices.iter().map(|&(_, i)| &all_blocks[i]).collect();
    let all_refs: Vec<&BlockItem> = all_blocks.iter().collect();

    // Stabilize CPU turbo clock before timing Part A
    {
        let mut warm = OpenCells::default();
        let mut out = Vec::with_capacity(900_032);
        for _ in 0..25 {
            for blk in &auth_refs {
                black_box(warm.invert_runs(&blk.runs, blk.n, blk.p, &mut out, k));
            }
        }
    }

    const ROUNDS: usize = 21;

    println!("\n=== PART A: 6 AUTHENTICATED WIKIDATA BLOCKS ({ROUNDS} rounds, 1T, paired interleaved A/B + A/A) ===");
    let (a, b, r) = paired_bench::<OpenCells, OpenCells>(&auth_refs, ROUNDS, k);
    print_ab("A/A OpenCells vs OpenCells", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, EmbOpen>(&auth_refs, ROUNDS, k);
    print_ab("Arm 0: EmbOpen", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalOpen>(&auth_refs, ROUNDS, k);
    print_ab("Arm 1: BandCoalOpen (full)", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalWordOnly>(&auth_refs, ROUNDS, k);
    print_ab("Arm 1b: BandCoal (word-only)", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, DtWidestOpen>(&auth_refs, ROUNDS, k);
    print_ab("Arm 2: DtWidestOpen", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalPlainUni>(&auth_refs, ROUNDS, k);
    print_ab("Arm 3a: BandCoal+PlainUni", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, DtWidestPlainUni>(&auth_refs, ROUNDS, k);
    print_ab("Arm 3b: DtWidest+PlainUni", a, b, r);

    println!("\n--- Stage Breakdown & Structural Counters (6 Authenticated Blocks, median of {ROUNDS} rounds) ---");
    profile_arm::<0, false>("Arm0 EmbOpen", &auth_refs, ROUNDS, k);
    profile_arm::<1, false>("Arm1 BandCoalOpen", &auth_refs, ROUNDS, k);
    profile_arm::<3, false>("Arm1b BandCoalWord", &auth_refs, ROUNDS, k);
    profile_arm::<2, false>("Arm2 DtWidestOpen", &auth_refs, ROUNDS, k);
    profile_arm::<1, true>("Arm3a BandCoal+Uni", &auth_refs, ROUNDS, k);
    profile_arm::<2, true>("Arm3b DtWidest+Uni", &auth_refs, ROUNDS, k);

    println!("\n=== PART B: FULL 431-BLOCK WIKIDATA CORPUS ({ROUNDS} rounds, 1T, paired interleaved A/B + A/A) ===");
    let (a, b, r) = paired_bench::<OpenCells, OpenCells>(&all_refs, ROUNDS, k);
    print_ab("A/A OpenCells vs OpenCells", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, EmbOpen>(&all_refs, ROUNDS, k);
    print_ab("Arm 0: EmbOpen", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalOpen>(&all_refs, ROUNDS, k);
    print_ab("Arm 1: BandCoalOpen (full)", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalWordOnly>(&all_refs, ROUNDS, k);
    print_ab("Arm 1b: BandCoal (word-only)", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, DtWidestOpen>(&all_refs, ROUNDS, k);
    print_ab("Arm 2: DtWidestOpen", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, BandCoalPlainUni>(&all_refs, ROUNDS, k);
    print_ab("Arm 3a: BandCoal+PlainUni", a, b, r);
    let (a, b, r) = paired_bench::<OpenCells, DtWidestPlainUni>(&all_refs, ROUNDS, k);
    print_ab("Arm 3b: DtWidest+PlainUni", a, b, r);

    println!("\n--- Stage Breakdown & Structural Counters (All 431 Blocks, median of 7 rounds) ---");
    profile_arm::<0, false>("Arm0 EmbOpen", &all_refs, 7, k);
    profile_arm::<1, false>("Arm1 BandCoalOpen", &all_refs, 7, k);
    profile_arm::<3, false>("Arm1b BandCoalWord", &all_refs, 7, k);
    profile_arm::<2, false>("Arm2 DtWidestOpen", &all_refs, 7, k);
    profile_arm::<1, true>("Arm3a BandCoal+Uni", &all_refs, 7, k);
    profile_arm::<2, true>("Arm3b DtWidest+Uni", &all_refs, 7, k);
}
