#![forbid(unsafe_code)]

pub use mibz2::{bwt, runs, Kernels};

#[path = "../../plain_uni_open_candidate.rs"]
pub mod candidate;

use candidate::{under_density, TubeMode, TubeOpen};
use mibz2::bwt::{InverseBwt, OpenCells};
use mibz2::runs::{front, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use std::fs;
use std::time::Instant;

fn median(vals: &mut [f64]) -> f64 {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = vals.len();
    if m % 2 == 1 { vals[m / 2] } else { 0.5 * (vals[m / 2 - 1] + vals[m / 2]) }
}

fn main() {
    let k = Kernels::detect();
    let bz2 = fs::read("/home/antigravity/lmmx/mibz2/corpora/wikidata.bz2").expect("read");
    let mut scanner = Scanner::new(SliceSource(&bz2));
    let mut syms = Vec::new();
    let mut all_blks: Vec<(usize, Runs, usize, u32, bool)> = Vec::new();
    loop {
        match scanner.next(syms).expect("scan") {
            Event::Done => break,
            Event::Block { index, symbols, .. } => {
                let mut runs = Runs::default();
                let n = front(&symbols, &mut runs, k).expect("front");
                let adm = n > 0 && n <= 900_000 && under_density(runs.records().len(), n);
                all_blks.push((index as usize, runs, n, symbols.orig_ptr, adm));
                syms = symbols.syms;
            }
        }
    }

    let mut out = Vec::with_capacity(900_064);
    let mut oa = OpenCells::default();
    let mut oa2 = OpenCells::default();
    let mut emb = TubeOpen::with_config(TubeMode::BaselineOpen, 2);
    let mut puni = TubeOpen::with_config(TubeMode::PlainUniOpen, 2);
    let mut tube = TubeOpen::with_config(TubeMode::TubeUniV6, 2);

    // 1. Clean 25-round A/B + A/A on Referee #2's 4 full chaingrind blocks (52, 162, 272, 380)!
    let targets = [52usize, 162, 272, 380];
    for _ in 0..3 {
        for &idx in &targets {
            let (_, ref runs, n, p, _) = all_blks[idx];
            oa.invert_runs(runs, n, p, &mut out, k);
            emb.invert_runs(runs, n, p, &mut out, k);
            puni.invert_runs(runs, n, p, &mut out, k);
            tube.invert_runs(runs, n, p, &mut out, k);
            oa2.invert_runs(runs, n, p, &mut out, k);
        }
    }
    let rounds = 25;
    println!("=== CLEAN 25-ROUND PAIRED A/B + A/A ON REFEREE #2's 4 FULL CHAINGRIND BLOCKS ===");
    println!("{:6} {:>7} | {:>7} {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5} | {:>7} {:>5}",
        "Blk#", "m_open", "Open(A)", "OpenA'", "A/A", "EmbOp", "xOp", "PlnUni", "xOp", "TubeV6", "xOp");
    let (mut soa, mut soa2, mut semb, mut sp, mut st) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for &idx in &targets {
        let (_, ref runs, n, p, _) = all_blks[idx];
        let (mut va, mut va2, mut ve, mut vp, mut vt) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for _ in 0..rounds {
            let t0 = Instant::now(); oa.invert_runs(runs, n, p, &mut out, k); va.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now(); emb.invert_runs(runs, n, p, &mut out, k); ve.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now(); puni.invert_runs(runs, n, p, &mut out, k); vp.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now(); tube.invert_runs(runs, n, p, &mut out, k); vt.push(t0.elapsed().as_secs_f64() * 1e3);
            let t0 = Instant::now(); oa2.invert_runs(runs, n, p, &mut out, k); va2.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        emb.run_profiled(runs.records(), n, p, &mut out);
        let m_op = emb.last_stats.m_open;
        let (ma, ma2, me, mp, mt) = (median(&mut va), median(&mut va2), median(&mut ve), median(&mut vp), median(&mut vt));
        soa += ma; soa2 += ma2; semb += me; sp += mp; st += mt;
        println!("{:6} {:7} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3}",
            idx, m_op, ma, ma2, ma2/ma, me, me/ma, mp, mp/ma, mt, mt/ma);
    }
    println!("{:6} {:>7} | {:6.3}m {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3} | {:6.3}m {:4.3}",
        "4-SUM", "", soa, soa2, soa2/soa, semb, semb/soa, sp, sp/soa, st, st/soa);

    // 2. Per-stage clean median (15 rounds) on the 4 chaingrind blocks: EmbOpen vs PlainUni vs TubeV6!
    println!("\n=== CLEAN 15-ROUND PER-STAGE MEDIAN ON BLOCKS 52, 162, 272, 380 ===");
    for &(name, mode) in &[("EmbOpen", TubeMode::BaselineOpen), ("PlainUni", TubeMode::PlainUniOpen), ("TubeUniV6", TubeMode::TubeUniV6)] {
        let mut kern = TubeOpen::with_config(mode, 2);
        let (mut s1, mut s23, mut s4, mut stb, mut sem) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for &idx in &targets {
            let (_, ref runs, n, p, _) = all_blks[idx];
            let (mut v1, mut v23, mut v4, mut vtb, mut vem) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for _ in 0..15 {
                kern.run_profiled(runs.records(), n, p, &mut out);
                let st = kern.last_stats;
                v1.push(st.ns_s1 as f64 / 1e6);
                v23.push(st.ns_s23 as f64 / 1e6);
                v4.push(st.ns_s4 as f64 / 1e6);
                vtb.push(st.ns_tube as f64 / 1e6);
                vem.push(st.ns_emit as f64 / 1e6);
            }
            s1 += median(&mut v1); s23 += median(&mut v23); s4 += median(&mut v4); stb += median(&mut vtb); sem += median(&mut vem);
        }
        println!("  {:10}: S1={:.2}m | S23={:.2}m | S4={:.2}m | Tube={:.2}m | Emit={:.2}m => Sum={:.2}m",
            name, s1, s23, s4, stb, sem, s1 + s23 + s4 + stb + sem);
    }

    // 3. Whole-corpus paired 9-round total time (A/A included!) across ALL 431 blocks!
    let mut ca = Vec::new(); let mut ca2 = Vec::new(); let mut ce = Vec::new(); let mut cp = Vec::new(); let mut ct = Vec::new();
    for _ in 0..9 {
        let t0 = Instant::now();
        for (_, runs, n, p, _) in &all_blks { oa.invert_runs(runs, *n, *p, &mut out, k); }
        ca.push(t0.elapsed().as_secs_f64() * 1e3);

        let t0 = Instant::now();
        for (_, runs, n, p, _) in &all_blks { emb.invert_runs(runs, *n, *p, &mut out, k); }
        ce.push(t0.elapsed().as_secs_f64() * 1e3);

        let t0 = Instant::now();
        for (_, runs, n, p, _) in &all_blks { puni.invert_runs(runs, *n, *p, &mut out, k); }
        cp.push(t0.elapsed().as_secs_f64() * 1e3);

        let t0 = Instant::now();
        for (_, runs, n, p, _) in &all_blks { tube.invert_runs(runs, *n, *p, &mut out, k); }
        ct.push(t0.elapsed().as_secs_f64() * 1e3);

        let t0 = Instant::now();
        for (_, runs, n, p, _) in &all_blks { oa2.invert_runs(runs, *n, *p, &mut out, k); }
        ca2.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    let (ma, ma2, me, mp, mt) = (median(&mut ca), median(&mut ca2), median(&mut ce), median(&mut cp), median(&mut ct));
    println!("\n=== CLEAN 9-ROUND WHOLE-CORPUS (ALL 431 BLOCKS) PAIRED A/B + A/A ===");
    println!("  Open(A)={:.1}m | Open(A')={:.1}m (A/A={:.3}x) | EmbOp={:.1}m ({:.3}x) | PlainUni={:.1}m ({:.3}x) | TubeV6={:.1}m ({:.3}x)",
        ma, ma2, ma2/ma, me, me/ma, mp, mp/ma, mt, mt/ma);
}
