#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_hotwire;
mod cand_bipartite;

use std::sync::Barrier;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
    l_col: Vec<u8>,
    records: Vec<u16>,
    expected: Vec<u8>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn plain_invert(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    let mut rank_in_l = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        rank_in_l[i] = counts[b as usize];
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut out = vec![0u8; n];
    let mut x = p as usize;
    for k in (0..n).rev() {
        let b = l[x];
        out[k] = b;
        x = (starts[b as usize] + rank_in_l[x]) as usize;
    }
    out
}

/// Exact Fenwick-tree Mattson LRU simulator for `cap_lines` 64-byte cache lines (16 u32s per line).
struct Fenwick {
    bit: Vec<i32>,
}
impl Fenwick {
    fn new(n: usize) -> Self { Self { bit: vec![0; n + 2] } }
    fn add(&mut self, mut i: usize, d: i32) {
        i += 1;
        while i < self.bit.len() {
            self.bit[i] += d;
            i += i & (!i + 1);
        }
    }
    fn sum(&self, mut i: usize) -> i32 {
        i += 1;
        let mut s = 0;
        while i > 0 {
            s += self.bit[i];
            i &= i - 1;
        }
        s
    }
}

/// Returns (compulsory_or_capacity_misses_cold, misses_after_seq_warm) for a sequence of `u32` indices
/// on a cache of `cap_lines` 64-byte lines (`16` u32s/line).
fn lru_misses(accesses: &[u32], num_lines: usize, cap_lines: usize) -> usize {
    let total_t = accesses.len();
    let mut last_seen = vec![usize::MAX; num_lines];
    let mut fw = Fenwick::new(total_t + 2);
    let mut misses = 0usize;
    for (t, &idx) in accesses.iter().enumerate() {
        let line = (idx as usize) >> 4;
        let prev = last_seen[line];
        if prev == usize::MAX {
            misses += 1;
        } else {
            let dist = fw.sum(t) - fw.sum(prev);
            if dist as usize >= cap_lines {
                misses += 1;
            }
            fw.add(prev, -1);
        }
        fw.add(t, 1);
        last_seen[line] = t;
    }
    misses
}

/// Extract the exact retained permutation `pi[0..m)` and L-domain thin bytes for gauge analysis
fn gauge_census(b: &Block) {
    // Reconstruct retained L-domain ranks and F-bucket bijection `gamma_F`
    let mut thin_counts = [0u32; 256];
    let mut m = 0u32;
    let mut pos = 0u32;
    let mut i = 0usize;
    let recs = &b.records;
    let mut runs_meta: Vec<(u8, u32, bool)> = Vec::new();
    while i < recs.len() {
        let c = (recs[i] >> 8) as u8;
        let mut w = u32::from(recs[i] & 255);
        i += 1;
        while i < recs.len() && (recs[i] >> 8) as u8 == c {
            w += u32::from(recs[i] & 255);
            i += 1;
        }
        let end = pos + w;
        if w > 16 && (pos..end).contains(&b.p) {
            if pos < b.p { runs_meta.push((c, b.p - pos, false)); }
            runs_meta.push((c, 1, true));
            thin_counts[c as usize] += 1;
            m += 1;
            if b.p + 1 < end { runs_meta.push((c, end - b.p - 1, false)); }
        } else {
            let keep = w <= 16;
            runs_meta.push((c, w, keep));
            if keep { thin_counts[c as usize] += w; m += w; }
        }
        pos = end;
    }
    let mut f_starts = [0u32; 256];
    let mut acc = 0u32;
    for a in 0..256 { f_starts[a] = acc; acc += thin_counts[a]; }
    let mut gamma_f = vec![0u32; m as usize];
    let mut cur_f = f_starts;
    let mut j = 0usize;
    for &(c, w, keep) in &runs_meta {
        if keep {
            let a = c as usize;
            for _ in 0..w {
                gamma_f[j] = cur_f[a];
                cur_f[a] += 1;
                j += 1;
            }
        }
    }
    // Extract exact `emit` orbit `v_0, v_1, ..., v_{m-1}` from `master_a1::Kernel`!
    // Wait: we can read `km` by running exact LF first-return on `b.l_col`!
    let n = b.n;
    let mut counts = [0u32; 256];
    let mut rank_l = vec![0u32; n];
    for (x, &ch) in b.l_col.iter().enumerate() {
        rank_l[x] = counts[ch as usize];
        counts[ch as usize] += 1;
    }
    let mut c_starts = [0u32; 256];
    let mut tot = 0u32;
    for a in 0..256 { c_starts[a] = tot; tot += counts[a]; }

    let mut ret_id = vec![u32::MAX; n];
    let mut rpos = 0usize;
    let mut rj = 0u32;
    for &(_c, w, keep) in &runs_meta {
        let w = w as usize;
        if keep {
            for off in 0..w {
                ret_id[rpos + off] = rj;
                rj += 1;
            }
        }
        rpos += w;
    }
    assert_eq!(rj, m);
    // Trace exact `emit` orbit in L-gauge (`master` walks LF from `p`? Wait: `Plain` walks `x = LF(x)` filling `out` from `n-1` down to `0`, whereas `Open` walks forward in text = `\Psi`! Either way, the sequence of visited retained coordinates in `Open` is exact reverse of `LF` retained hits, and Mattson LRU stack-distance histogram of a sequence and its reverse are IDENTICAL by Pin's Reversal Duality Theorem!! Let's check forward-in-emit order explicitly by reversing LF hits!)
    let mut lf_hits = Vec::with_capacity(m as usize);
    let mut x = b.p as usize;
    for _ in 0..n {
        if ret_id[x] != u32::MAX {
            lf_hits.push(ret_id[x]);
        }
        let ch = b.l_col[x] as usize;
        x = (c_starts[ch] + rank_l[x]) as usize;
    }
    assert_eq!(lf_hits.len(), m as usize);
    // Emit order is reverse of backward LF walk (rotated so mark is first):
    lf_hits.reverse();

    // Also build F-2gram gauge `gamma_lf`: sort retained rows `j` by `(L[x_j], F[x_j], j)`!
    let mut x_of_j = vec![0usize; m as usize];
    for (row, &rid) in ret_id.iter().enumerate() {
        if rid != u32::MAX { x_of_j[rid as usize] = row; }
    }
    // Determine F[row] from `c_starts`
    let mut f_col_byte = vec![0u8; m as usize];
    for jj in 0..m as usize {
        let row = x_of_j[jj] as u32;
        let fb = c_starts.partition_point(|&s| s <= row) - 1;
        f_col_byte[jj] = fb as u8;
    }
    let mut order_2g: Vec<u32> = (0..m).collect();
    order_2g.sort_by_key(|&jj| (b.l_col[x_of_j[jj as usize]], f_col_byte[jj as usize], jj));
    let mut gamma_2g = vec![0u32; m as usize];
    for (new_id, &old_j) in order_2g.iter().enumerate() {
        gamma_2g[old_j as usize] = new_id as u32;
    }

    let num_lines = (m as usize + 15) / 16;
    let cap_i9_l2 = 4096usize; // 256 KiB / 64 B = 4,096 cache lines!

    let seq_l: Vec<u32> = lf_hits.clone();
    let seq_f: Vec<u32> = lf_hits.iter().map(|&jj| gamma_f[jj as usize]).collect();
    let seq_2g: Vec<u32> = lf_hits.iter().map(|&jj| gamma_2g[jj as usize]).collect();
    let seq_oracle: Vec<u32> = (0..m).collect();

    let miss_l = lru_misses(&seq_l, num_lines, cap_i9_l2);
    let miss_f = lru_misses(&seq_f, num_lines, cap_i9_l2);
    let miss_2g = lru_misses(&seq_2g, num_lines, cap_i9_l2);
    let miss_or = lru_misses(&seq_oracle, num_lines, cap_i9_l2);

    // Count Bipartite E_{0->1} contractions on `seq_l`:
    let mut bp_skips = 0usize;
    for w in seq_l.windows(2) {
        if (w[0] & 1) == 0 && (w[1] & 1) == 1 {
            bp_skips += 1;
        }
    }

    println!(
        "GAUGE {:14} m={:6} lines={:5} | 256KiB L2 misses: gamma_L={:6} ({:4.1}%) | gamma_F={:6} ({:4.1}%) | gamma_2gram={:6} ({:4.1}%) | oracle={:5} ({:4.1}%) | E01_pairs={:5} ({:4.1}%)",
        b.name, m, num_lines,
        miss_l, 100.0 * miss_l as f64 / m as f64,
        miss_f, 100.0 * miss_f as f64 / m as f64,
        miss_2g, 100.0 * miss_2g as f64 / m as f64,
        miss_or, 100.0 * miss_or as f64 / m as f64,
        bp_skips, 100.0 * bp_skips as f64 / m as f64,
    );
}

fn load_six() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap()));
        }
    }
    specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let expected = plain_invert(&l, p);
        let records = records_of(&l);
        Block { name: f, n: l.len(), p, l_col: l, records, expected }
    }).collect()
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let six = load_six();
    println!("=== 1. EXACT MATTSON 256 KiB (i9 L2) MISS CENSUS ACROSS S_m COORDINATE GAUGES ===");
    for b in &six {
        gauge_census(b);
    }

    println!("\n=== 2. PARITY & SCRATCH AUDIT (HotWire & Bipartite-Bigram vs Master 0.5.11) ===");
    let mut out = Vec::with_capacity(900_064);
    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut khw = cand_hotwire::Kernel::default();
    let mut kbp = cand_bipartite::Kernel::default();

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        khw.run_entry(&b.records, b.n, b.p, cand_hotwire::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "HotWire mismatch on {}", b.name);
        kbp.run_entry(&b.records, b.n, b.p, cand_bipartite::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Bipartite mismatch on {}", b.name);
    }
    let (sm, _, _) = km1.scratch_report();
    let (shw, _, _) = khw.scratch_report();
    let (sbp, _, _) = kbp.scratch_report();
    println!("SCRATCH (6-block cumulative): master={} B ({:.1} KiB) | hotwire={} B ({:.1} KiB) | bipartite={} B ({:.1} KiB, +4.0 KiB bigram L1)",
        sm, sm as f64 / 1024.0, shw, shw as f64 / 1024.0, sbp, sbp as f64 / 1024.0);

    println!("\n=== 3. STAGE ATTRIBUTION (Bipartite-Bigram vs Master, 45 pairs/block) ===");
    let reps = 45usize;
    let (mut scm, mut swm, mut scb, mut swb) = (0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut bc, mut bw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = kbp.run_stages(&b.records, b.n, b.p, cand_bipartite::Cap::NONE, &mut out);
                bc.push(c); bw.push(w);
            }
        }
        let (mc10, mw10, bc10, bw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut bc, 1, 10), pct(&mut bw, 1, 10));
        scm += mc10; swm += mw10; scb += bc10; swb += bw10;
        println!("STAGE {:14} | construct: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.3}x, {:+.1} us)",
            b.name, mc10, bc10, bc10 / mc10, bc10 - mc10, mw10, bw10, bw10 / mw10, bw10 - mw10);
    }
    println!("STAGE 6-BLOCK SUM    | construct: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us) | walk/emit: {:6.1} -> {:6.1} ({:.4}x, {:+.1} us)",
        scm, scb, scb / scm, scb - scm, swm, swb, swb / swm, swb - swm);

    println!("\n=== 4. END-TO-END 1T & 2T A/A vs A/B (50 pairs/block) ===");
    let reps = 50usize;
    let (mut saa_r, mut saa_c, mut shw_r, mut shw_c, mut sbp_r, mut sbp_c) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { khw.run_entry(&b.records, b.n, b.p, cand_hotwire::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (hwr, hwc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kbp.run_entry(&b.records, b.n, b.p, cand_bipartite::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (bpr, bpc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        saa_r += aar; saa_c += aac; shw_r += hwr; shw_c += hwc; sbp_r += bpr; sbp_c += bpc;
        println!("1T {:14} | A/A={:.3} | HotWire={:.3} ({:.1}/{:.1}) | Bipartite={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, hwc / hwr, hwc, hwr, bpc / bpr, bpc, bpr);
    }
    println!("1T 6-BLOCK TOTAL  | A/A={:.4} | HotWire={:.4} | Bipartite={:.4}",
        saa_c / saa_r, shw_c / shw_r, sbp_c / sbp_r);

    let workers = 2usize;
    let passes = 35usize;
    for (label, mode) in [("2T A/A (Master1 vs Master2)", 0), ("2T A/B (Master1 vs HotWire)", 1), ("2T A/B (Master1 vs Bipartite)", 2)] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers).map(|w| {
                let bar = &barrier;
                s.spawn(move || {
                    let mut km = master_a1::Kernel::default();
                    let mut km2 = master_a2::Kernel::default();
                    let mut kh = cand_hotwire::Kernel::default();
                    let mut kb = cand_bipartite::Kernel::default();
                    let mut out = Vec::with_capacity(900_064);
                    for b in six_ref {
                        km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                        kh.run_entry(&b.records, b.n, b.p, cand_hotwire::Cap::NONE, &mut out);
                        kb.run_entry(&b.records, b.n, b.p, cand_bipartite::Cap::NONE, &mut out);
                    }
                    let (mut tr, mut tc) = (vec![], vec![]);
                    for pass in 0..(2 * passes + 4) {
                        let which = (pass + pass / 2) % 2;
                        bar.wait();
                        let t0 = Instant::now();
                        for i in 0..six_ref.len() {
                            let b = &six_ref[(i + w * 3) % six_ref.len()];
                            if which == 0 { km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); }
                            else if mode == 0 { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); }
                            else if mode == 1 { kh.run_entry(&b.records, b.n, b.p, cand_hotwire::Cap::NONE, &mut out); }
                            else { kb.run_entry(&b.records, b.n, b.p, cand_bipartite::Cap::NONE, &mut out); }
                        }
                        let dt = t0.elapsed().as_secs_f64() * 1e6;
                        if pass >= 4 { if which == 0 { tr.push(dt); } else { tc.push(dt); } }
                    }
                    (tr, tc)
                })
            }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!("{:32} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label, pct(&mut ac, 1, 10) / pct(&mut ar, 1, 10), pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2), pct(&mut ac, 1, 2), pct(&mut ar, 1, 2));
    }
}
