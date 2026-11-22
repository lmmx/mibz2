#![forbid(unsafe_code)]

pub use mibz2::{bwt, runs, Kernels};

#[path = "../../b4_grid_flat16_candidate.rs"]
pub mod candidate;

use candidate::{B4GridFlat16, ExecMode};
use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use std::fs;
use std::time::Instant;

struct BlockSpec {
    file: &'static str,
    n: usize,
    p: u32,
}

const BLOCKS: [BlockSpec; 6] = [
    BlockSpec { file: "block-0.L", n: 899_931, p: 264_045 },
    BlockSpec { file: "block-2.L", n: 900_000, p: 320_807 },
    BlockSpec { file: "block-85.L", n: 899_935, p: 325_088 },
    BlockSpec { file: "block-143.L", n: 899_869, p: 802_868 },
    BlockSpec { file: "block-210.L", n: 899_959, p: 791_429 },
    BlockSpec { file: "block-207.L", n: 899_998, p: 205_180 },
];

fn build_runs_from_column(ll: &[u8], p: u32, k: Kernels) -> Runs {
    let mut present = [false; 256];
    for &b in ll {
        present[b as usize] = true;
    }
    let mut seq_to_unseq = [0u8; 256];
    let mut mtf_list = [0u8; 256];
    let mut alpha = 0usize;
    for b in 0..256usize {
        if present[b] {
            seq_to_unseq[alpha] = b as u8;
            mtf_list[alpha] = b as u8;
            alpha += 1;
        }
    }
    let mut syms: Vec<u16> = Vec::with_capacity(ll.len() / 2);
    let mut zrun = 0usize;
    let flush_zrun = |z: &mut usize, out: &mut Vec<u16>| {
        let mut r = *z;
        while r > 0 {
            if (r & 1) == 1 {
                out.push(0);
                r = (r - 1) >> 1;
            } else {
                out.push(1);
                r = (r - 2) >> 1;
            }
        }
        *z = 0;
    };
    for &b in ll {
        if mtf_list[0] == b {
            zrun += 1;
        } else {
            flush_zrun(&mut zrun, &mut syms);
            let mut pos = 1usize;
            while mtf_list[pos] != b {
                pos += 1;
            }
            for j in (1..=pos).rev() {
                mtf_list[j] = mtf_list[j - 1];
            }
            mtf_list[0] = b;
            syms.push((pos + 1) as u16);
        }
    }
    flush_zrun(&mut zrun, &mut syms);
    let eob = (alpha + 1) as u16;
    syms.push(eob);

    let symbols = Symbols {
        crc: 0,
        orig_ptr: p,
        seq_to_unseq,
        syms,
        eob,
        max_len: 900_000,
    };
    let mut runs = Runs::default();
    let n = front(&symbols, &mut runs, k).expect("front failed");
    assert_eq!(n, ll.len());
    runs
}

fn capped_records_from_l(ll: &[u8]) -> Vec<u16> {
    let mut recs = Vec::new();
    let mut i = 0;
    while i < ll.len() {
        let b = ll[i];
        let mut j = i + 1;
        while j < ll.len() && ll[j] == b && j - i < 127 {
            j += 1;
        }
        recs.push(((b as u16) << 8) | ((j - i) as u16));
        i = j;
    }
    recs
}

fn ref_n_step_lf(ll: &[u8], p: u32) -> Vec<u8> {
    let n = ll.len();
    let mut counts = [0u32; 256];
    for &b in ll {
        counts[b as usize] += 1;
    }
    let mut c_off = [0u32; 256];
    let mut acc = 0u32;
    for i in 0..256 {
        c_off[i] = acc;
        acc += counts[i];
    }
    let mut occ = c_off;
    let mut lf = vec![0u32; n];
    for (i, &b) in ll.iter().enumerate() {
        lf[i] = occ[b as usize];
        occ[b as usize] += 1;
    }
    let mut out = vec![0u8; n];
    let mut cur = p as usize;
    for k in (0..n).rev() {
        out[k] = ll[cur];
        cur = lf[cur] as usize;
    }
    out
}

fn bwt_encode_fast(t: &[u8]) -> (Vec<u8>, u32) {
    let n = t.len();
    let mut sa: Vec<usize> = (0..n).collect();
    let mut rk: Vec<u32> = t.iter().map(|&b| b as u32).collect();
    let mut tmp = vec![0u32; n];
    let mut k = 1usize;
    while k < n {
        sa.sort_unstable_by_key(|&i| (rk[i], rk[(i + k) % n]));
        tmp[sa[0]] = 0;
        for j in 1..n {
            let a = sa[j - 1];
            let b = sa[j];
            let diff = (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]);
            tmp[b] = tmp[a] + if diff { 1 } else { 0 };
        }
        std::mem::swap(&mut rk, &mut tmp);
        if rk[sa[n - 1]] as usize == n - 1 {
            break;
        }
        k <<= 1;
    }
    let ll: Vec<u8> = sa.iter().map(|&i| t[(i + n - 1) % n]).collect();
    let p = sa.iter().position(|&i| i == 0).unwrap() as u32;
    (ll, p)
}

fn de_bruijn_2_4() -> Vec<u8> {
    let n = 4usize;
    let mut a = vec![0usize; 2 * n];
    let mut seq = Vec::new();
    fn db(t: usize, p: usize, n: usize, a: &mut [usize], seq: &mut Vec<u8>) {
        if t > n {
            if n % p == 0 {
                for i in 1..=p {
                    seq.push(a[i] as u8);
                }
            }
        } else {
            a[t] = a[t - p];
            db(t + 1, p, n, a, seq);
            for j in (a[t - p] + 1)..2 {
                a[t] = j;
                db(t + 1, t, n, a, seq);
            }
        }
    }
    db(1, 1, n, &mut a, &mut seq);
    seq
}

fn verify_edge_and_adversarial() {
    println!("=== 1. EDGE-CASE & ADVERSARIAL EXACTNESS VERIFICATION (SAFE RUST) ===");
    let mut out = Vec::new();
    for mode in [ExecMode::DirectHubP16, ExecMode::CoreRvP16] {
        let mut kern = B4GridFlat16::with_mode(mode);
        let edge_cases: Vec<(&str, Vec<u8>)> = vec![
            ("n=1 [42]", vec![42]),
            ("unary [0,0,0,0]", vec![0, 0, 0, 0]),
            ("periodic [0,1,0,1]", bwt_encode_fast(&[0, 1, 0, 1]).0),
            ("periodic [0,1,2,0,1,2]", bwt_encode_fast(&[0, 1, 2, 0, 1, 2]).0),
            ("ref2 closed-cycle [1,0,2,2]", vec![1, 0, 2, 2]),
            ("ref2 closed-cycle [1,0,2,1,2,2]", vec![1, 0, 2, 1, 2, 2]),
            ("ref2 delta-q=+1", bwt_encode_fast(&[0, 0, 1, 0, 1, 2, 0, 1]).0),
        ];
        for (name, ll) in &edge_cases {
            let recs = capped_records_from_l(ll);
            for p in 0..ll.len() as u32 {
                let ref_out = ref_n_step_lf(ll, p);
                let ok = kern.invert_runs_raw(&recs, ll.len(), p, &mut out);
                assert!(ok, "invert_runs_raw returned false on {name} p={p} ({mode:?})");
                assert_eq!(out, ref_out, "MISMATCH on {name} p={p} ({mode:?})");
            }
        }

        let db = de_bruijn_2_4();
        let mut padded_ring = Vec::new();
        for &x in &db { padded_ring.extend(std::iter::repeat(if x == 0 { b'a' } else { b'b' }).take(64)); }
        for &x in &db { padded_ring.extend(std::iter::repeat(if x == 0 { b'b' } else { b'c' }).take(64)); }
        for &x in &db { padded_ring.extend(std::iter::repeat(if x == 0 { b'c' } else { b'a' }).take(64)); }

        let mut three_phase = Vec::new();
        for _ in 0..256 { three_phase.extend_from_slice(&[0, 1]); }
        three_phase.push(2);
        for _ in 0..256 { three_phase.extend_from_slice(&[0, 0, 1]); }
        three_phase.push(2);
        for _ in 0..256 { three_phase.extend_from_slice(&[0, 1, 1]); }
        three_phase.push(2);

        let mut alt_ring = Vec::new();
        for _ in 0..256 { alt_ring.extend_from_slice(&[0, 1]); }
        for _ in 0..256 { alt_ring.extend_from_slice(&[1, 2]); }
        for _ in 0..256 { alt_ring.extend_from_slice(&[2, 0]); }

        let mut sym_mid = vec![b'M'; 256];
        sym_mid.push(b'A');
        sym_mid.extend(std::iter::repeat(b'M').take(256));
        sym_mid.push(b'Z');

        let mut fa = vec![0u8];
        let mut fb = vec![0u8, 1u8];
        for _ in 3..=20 {
            let mut nxt = fb.clone();
            nxt.extend_from_slice(&fa);
            fa = fb;
            fb = nxt;
        }
        fb.push(2u8);

        let killers: Vec<(&str, Vec<u8>)> = vec![
            ("3-phase(256)", three_phase),
            ("AltRing(256)", alt_ring),
            ("Low-r Padded-Ring(64)", padded_ring),
            ("SymMidRun(256)", sym_mid),
            ("Fib_20+$", fb),
        ];
        for (name, t) in &killers {
            let (ll, p) = bwt_encode_fast(t);
            let recs = capped_records_from_l(&ll);
            let ok = kern.invert_runs_profiled(&recs, ll.len(), p, &mut out);
            assert!(ok && out == *t, "FAIL killer {name} ({mode:?})");
            let st = kern.last_stats;
            if mode == ExecMode::CoreRvP16 {
                println!(
                    "  [PASS] {:22} n={:5} r0={:2} | s1_1st={:3} s1_slf={:3} subint_s1={} s2_ops={:3} m_core={:4}",
                    name, t.len(), st.r0, st.s1_1st_hop, st.s1_self_lkp, st.s1_subint_buckets, st.s2_ops, st.m_core
                );
            }
        }
    }
    println!("  [PASS] All edge cases + all 5 adversarial killers 100% EXACT in both DirectHubP16 & CoreRvP16!");
}

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 { vals[m / 2] } else { 0.5 * (vals[m / 2 - 1] + vals[m / 2]) }
}

fn main() {
    verify_edge_and_adversarial();

    let k = Kernels::detect();
    let mut loaded: Vec<(&BlockSpec, Runs, Vec<u8>)> = Vec::new();
    for spec in &BLOCKS {
        let path = format!("/tmp/mibz2-blocks/{}", spec.file);
        let ll = fs::read(&path).expect("read block");
        assert_eq!(ll.len(), spec.n);
        let runs = build_runs_from_column(&ll, spec.p, k);
        let mut ref_out = Vec::new();
        let mut open = OpenCells::default();
        assert!(open.invert_runs(&runs, spec.n, spec.p, &mut ref_out, k));
        assert_eq!(ref_out.len(), spec.n);
        loaded.push((spec, runs, ref_out));
    }

    println!("\n=== 2. WIKIDATA 6-BLOCK PARITY & STEADY-STATE PER-STAGE BREAKDOWN (CHUNKPOOL!) ===");
    for mode in [ExecMode::DirectHubP16, ExecMode::CoreRvP16] {
        let mut kern = B4GridFlat16::with_mode(mode);
        let mut out = Vec::with_capacity(900_000);
        // Warmup 2 passes so per-stage timings reflect steady-state zero-alloc!
        for _ in 0..2 {
            for (spec, runs, _) in &loaded {
                kern.invert_runs(runs, spec.n, spec.p, &mut out, k);
            }
        }
        println!("\n--- Mode: {:?} (steady-state warmed) ---", mode);
        println!(
            "{:11} | {:>6} {:>5} {:>5} {:>6} | {:>5} {:>5} {:>5} {:>5} | {:>5} {:>6} | {:>5} {:>5} {:>5} {:>5} {:>5} {:>5}",
            "Block", "r0", "App/r", "Stale", "1stH", "SlfL", "S2Op", "SlbK", "SlpN", "FlatQ", "PeakM",
            "S0", "S1Bin", "S1Op", "S2", "S3Fl", "S3Em"
        );
        let (mut s0, mut sb, mut so, mut s2, mut sf, mut se) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        for (spec, runs, ref_out) in &loaded {
            let ok = kern.invert_runs_profiled(runs.records(), spec.n, spec.p, &mut out);
            assert!(ok && out == *ref_out, "PARITY FAIL on {} ({:?})", spec.file, mode);
            let st = kern.last_stats;
            s0 += st.ns_s0; sb += st.ns_s1_bin; so += st.ns_s1_ops; s2 += st.ns_s2; sf += st.ns_s3_flat; se += st.ns_s3_emit;
            println!(
                "{:11} | {:6} {:5.2} {:5} {:6} | {:5} {:5} {:4}K {:5} | {:4}K {:5}K | {:4.0}u {:4.0}u {:4.0}u {:4.0}u {:4.0}u {:4.0}u",
                spec.file,
                st.r0,
                st.appends as f64 / st.r0 as f64,
                st.stale,
                st.s1_1st_hop,
                st.s1_self_lkp,
                st.s2_ops,
                (st.slab16_len * 16) / 1024,
                st.slp_len,
                st.flat_q_bytes / 1024,
                st.peak_scratch_bytes / 1024,
                st.ns_s0 as f64 / 1e3,
                st.ns_s1_bin as f64 / 1e3,
                st.ns_s1_ops as f64 / 1e3,
                st.ns_s2 as f64 / 1e3,
                st.ns_s3_flat as f64 / 1e3,
                st.ns_s3_emit as f64 / 1e3,
            );
        }
        println!(
            "  6-BLK STAGE SUM (ms): S0={:.2} | S1Bin={:.2} | S1Op={:.2} | S2={:.2} | S3Fl={:.2} | S3Em={:.2} | PeakAlloc={} KiB",
            s0 as f64 / 1e6, sb as f64 / 1e6, so as f64 / 1e6, s2 as f64 / 1e6, sf as f64 / 1e6, se as f64 / 1e6,
            kern.allocated_scratch_bytes() / 1024
        );
    }

    println!("\n=== 3. SAME-BINARY PAIRED INTERLEAVED A/B + A/A BENCHMARK vs v0.5.12 OpenCells (1T, 21 ROUNDS) ===");
    let mut open_a = OpenCells::default();
    let mut open_a2 = OpenCells::default();
    let mut b4_p16 = B4GridFlat16::with_mode(ExecMode::DirectHubP16);
    let mut b4_rv = B4GridFlat16::with_mode(ExecMode::CoreRvP16);
    let mut out = Vec::with_capacity(900_000);

    for _ in 0..3 {
        for (spec, runs, _) in &loaded {
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            b4_p16.invert_runs(runs, spec.n, spec.p, &mut out, k);
            b4_rv.invert_runs(runs, spec.n, spec.p, &mut out, k);
        }
    }

    let rounds = 21usize;
    let mut t_oa = vec![vec![0.0; rounds]; 6];
    let mut t_oa2 = vec![vec![0.0; rounds]; 6];
    let mut t_p16 = vec![vec![0.0; rounds]; 6];
    let mut t_rv = vec![vec![0.0; rounds]; 6];

    for r in 0..rounds {
        for (bi, (spec, runs, _)) in loaded.iter().enumerate() {
            let t0 = Instant::now();
            open_a.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            b4_p16.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_p16[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            b4_rv.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_rv[bi][r] = t0.elapsed().as_secs_f64() * 1e3;

            let t0 = Instant::now();
            open_a2.invert_runs(runs, spec.n, spec.p, &mut out, k);
            t_oa2[bi][r] = t0.elapsed().as_secs_f64() * 1e3;
        }
    }

    println!(
        "{:11} | {:>8} {:>8} {:>6} | {:>8} {:>6} {:>7} | {:>8} {:>6} {:>7}",
        "Block", "Open(A)", "Open(A')", "A/A", "P16-Hub", "xOpen", "ns/row", "P16-RV", "xOpen", "ns/row"
    );
    println!("{}", "-".repeat(90));
    let (mut s_oa, mut s_oa2, mut s_p16, mut s_rv) = (0.0, 0.0, 0.0, 0.0);
    for (bi, (spec, _, _)) in loaded.iter().enumerate() {
        let moa = median(&mut t_oa[bi]);
        let moa2 = median(&mut t_oa2[bi]);
        let mp16 = median(&mut t_p16[bi]);
        let mrv = median(&mut t_rv[bi]);
        s_oa += moa;
        s_oa2 += moa2;
        s_p16 += mp16;
        s_rv += mrv;
        println!(
            "{:11} | {:7.3}m {:7.3}m {:5.3}x | {:7.3}m {:5.2}x {:6.2}n | {:7.3}m {:5.2}x {:6.2}n",
            spec.file, moa, moa2, moa2 / moa,
            mp16, mp16 / moa, mp16 * 1e6 / spec.n as f64,
            mrv, mrv / moa, mrv * 1e6 / spec.n as f64
        );
    }
    println!("{}", "-".repeat(90));
    println!(
        "{:11} | {:7.3}m {:7.3}m {:5.3}x | {:7.3}m {:5.2}x {:6.2}n | {:7.3}m {:5.2}x {:6.2}n",
        "6-BLK SUM", s_oa, s_oa2, s_oa2 / s_oa,
        s_p16, s_p16 / s_oa, s_p16 * 1e6 / 5_399_692.0,
        s_rv, s_rv / s_oa, s_rv * 1e6 / 5_399_692.0
    );
    println!(
        "\nSteady-State Peak Allocated Scratch: P16-Hub = {} KiB | P16-RV = {} KiB",
        b4_p16.allocated_scratch_bytes() / 1024,
        b4_rv.allocated_scratch_bytes() / 1024,
    );
}
