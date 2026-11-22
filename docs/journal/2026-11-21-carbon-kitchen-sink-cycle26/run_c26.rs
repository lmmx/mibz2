#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_cks_diet;
mod cand_cks_bi;
mod cand_epoch64;
mod cand_subtile32;
mod cand_noqueue;

use std::sync::Barrier;
use std::time::Instant;

struct Block {
    name: String,
    n: usize,
    p: u32,
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

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 { b' ' } else { b'a' + ((s >> 33) % 26) as u8 });
    }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..64 {
            let ca = text[(ia + k) % n];
            let cb = text[(ib + k) % n];
            if ca != cb { return ca.cmp(&cb); }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 { p = i as u32; }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "dense-300k".into(), n, p, records, expected }
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
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = std::fs::read(format!("{dir}/{f}")).unwrap();
            let expected = plain_invert(&l, p);
            let records = records_of(&l);
            Block { name: f, n: l.len(), p, records, expected }
        })
        .collect()
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let six = load_six();
    let dense = make_dense_300k();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kbi = cand_cks_bi::Kernel::default();
    let mut ke64 = cand_epoch64::Kernel::default();
    let mut ks32 = cand_subtile32::Kernel::default();
    let mut knq = cand_noqueue::Kernel::default();

    println!("=== 1. PARITY, SPELLING CAPACITY ANOMALY AUDIT & SCRATCH ===");
    for b in six.iter().chain(std::iter::once(&dense)) {
        let mut t_m = master_a1::Kernel::default();
        let mut t_b = cand_cks_bi::Kernel::default();
        t_m.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        t_b.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let (ml, mc) = t_m.spelling_stats();
        let (bl, bc) = t_b.spelling_stats();
        println!(
            "SPELLING {:12} | Master len={:6} cap={:6} | BiOperad len={:6} cap={:6} (ratio={:.2}x len)",
            b.name, ml, mc, bl, bc, bl as f64 / ml.max(1) as f64
        );
    }

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        ke64.run_entry(&b.records, b.n, b.p, cand_epoch64::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "EpochTile64 mismatch on {}", b.name);
        ks32.run_entry(&b.records, b.n, b.p, cand_subtile32::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "SubTile32 mismatch on {}", b.name);
        knq.run_entry(&b.records, b.n, b.p, cand_noqueue::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "NoQueue mismatch on {}", b.name);
    }
    // Check dense-300k parity on fresh kernels
    {
        let mut te = cand_epoch64::Kernel::default();
        let mut ts = cand_subtile32::Kernel::default();
        let mut tn = cand_noqueue::Kernel::default();
        te.run_entry(&dense.records, dense.n, dense.p, cand_epoch64::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
        ts.run_entry(&dense.records, dense.n, dense.p, cand_subtile32::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
        tn.run_entry(&dense.records, dense.n, dense.p, cand_noqueue::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
    }
    let (sm, _, _) = km1.scratch_report();
    let (sbi, _, _) = kbi.scratch_report();
    let (se64, _, _) = ke64.scratch_report();
    let (ss32, _, _) = ks32.scratch_report();
    let (snq, _, _) = knq.scratch_report();
    println!("SCRATCH Master 0.5.11      : {:7.1} KiB", sm as f64 / 1024.0);
    println!("SCRATCH CKS-BiOperad (C25) : {:7.1} KiB ({:+.2}%)", sbi as f64 / 1024.0, (sbi as f64 / sm as f64 - 1.0) * 100.0);
    println!("SCRATCH EpochTile64        : {:7.1} KiB ({:+.2}%)", se64 as f64 / 1024.0, (se64 as f64 / sm as f64 - 1.0) * 100.0);
    println!("SCRATCH SubTile32          : {:7.1} KiB ({:+.2}%)", ss32 as f64 / 1024.0, (ss32 as f64 / sm as f64 - 1.0) * 100.0);
    println!("SCRATCH NoQueue-BiOperad   : {:7.1} KiB ({:+.2}%)", snq as f64 / 1024.0, (snq as f64 / sm as f64 - 1.0) * 100.0);

    println!("\n=== 2. STAGE ATTRIBUTION (`construct` vs `walk`, 50 rounds/block) ===");
    let s_reps = 50usize;
    let (mut scm, mut swm, mut scbi, mut swbi, mut sce, mut swe, mut scs, mut sws, mut scn, mut swn) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut bic, mut biw, mut ec, mut ew, mut sc, mut sw, mut nc, mut nw) =
            (vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]);
        for i in 0..5 * s_reps {
            match (i + i / 5) % 5 {
                0 => { let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); mc.push(c); mw.push(w); }
                1 => { let (c, w) = kbi.run_stages(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); bic.push(c); biw.push(w); }
                2 => { let (c, w) = ke64.run_stages(&b.records, b.n, b.p, cand_epoch64::Cap::NONE, &mut out); ec.push(c); ew.push(w); }
                3 => { let (c, w) = ks32.run_stages(&b.records, b.n, b.p, cand_subtile32::Cap::NONE, &mut out); sc.push(c); sw.push(w); }
                _ => { let (c, w) = knq.run_stages(&b.records, b.n, b.p, cand_noqueue::Cap::NONE, &mut out); nc.push(c); nw.push(w); }
            }
        }
        let (mc10, mw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10));
        let (bic10, biw10) = (pct(&mut bic, 1, 10), pct(&mut biw, 1, 10));
        let (ec10, ew10) = (pct(&mut ec, 1, 10), pct(&mut ew, 1, 10));
        let (sc10, sw10) = (pct(&mut sc, 1, 10), pct(&mut sw, 1, 10));
        let (nc10, nw10) = (pct(&mut nc, 1, 10), pct(&mut nw, 1, 10));
        scm += mc10; swm += mw10; scbi += bic10; swbi += biw10;
        sce += ec10; swe += ew10; scs += sc10; sws += sw10; scn += nc10; swn += nw10;
        println!(
            "STAGE {:12} | construct: M={:6.1} Bi={:6.1} Ep64={:6.1} ({:+.1}) Sub32={:6.1} ({:+.1}) NoQ={:6.1} ({:+.1}) | walk: M={:6.1} Bi={:6.1}",
            b.name, mc10, bic10, ec10, ec10 - bic10, sc10, sc10 - bic10, nc10, nc10 - bic10, mw10, biw10
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: M={:6.1} Bi={:6.1} ({:.3}x) Ep64={:6.1} ({:.3}x, {:+.1} vs Bi) Sub32={:6.1} ({:.3}x, {:+.1}) NoQ={:6.1} ({:.3}x, {:+.1}) | walk: M={:6.1} Bi={:6.1} Ep64={:6.1}",
        scm, scbi, scbi / scm, sce, sce / scm, sce - scbi, scs, scs / scm, scs - scbi, scn, scn / scm, scn - scbi, swm, swbi, swe
    );
    let _ = (sws, swn);

    println!("\n=== 3. 1T INTERLEAVED A/A & A/B (60 pairs/block + dense-300k) ===");
    let reps = 60usize;
    let (mut saar, mut saac, mut sbir, mut sbic, mut ser, mut sec, mut ssr, mut ssc, mut snr, mut snc) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(std::iter::once(&dense)) {
        let is_six = b.n >= 890_000;
        let run_pair = |mode: usize, km1: &mut master_a1::Kernel, km2: &mut master_a2::Kernel, kbi: &mut cand_cks_bi::Kernel, ke64: &mut cand_epoch64::Kernel, ks32: &mut cand_subtile32::Kernel, knq: &mut cand_noqueue::Kernel, out: &mut Vec<u8>| -> (f64, f64) {
            let (mut ta, mut tb) = (vec![], vec![]);
            for i in 0..2 * reps {
                let t = Instant::now();
                if (i + i / 2) % 2 == 0 {
                    km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, out);
                    ta.push(t.elapsed().as_secs_f64() * 1e6);
                } else {
                    match mode {
                        0 => { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, out); }
                        1 => { kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, out); }
                        2 => { ke64.run_entry(&b.records, b.n, b.p, cand_epoch64::Cap::NONE, out); }
                        3 => { ks32.run_entry(&b.records, b.n, b.p, cand_subtile32::Cap::NONE, out); }
                        _ => { knq.run_entry(&b.records, b.n, b.p, cand_noqueue::Cap::NONE, out); }
                    }
                    tb.push(t.elapsed().as_secs_f64() * 1e6);
                }
            }
            (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10))
        };
        let (aar, aac) = run_pair(0, &mut km1, &mut km2, &mut kbi, &mut ke64, &mut ks32, &mut knq, &mut out);
        let (bir, bic) = run_pair(1, &mut km1, &mut km2, &mut kbi, &mut ke64, &mut ks32, &mut knq, &mut out);
        let (er, ec) = run_pair(2, &mut km1, &mut km2, &mut kbi, &mut ke64, &mut ks32, &mut knq, &mut out);
        let (sr, sc) = run_pair(3, &mut km1, &mut km2, &mut kbi, &mut ke64, &mut ks32, &mut knq, &mut out);
        let (nr, nc) = run_pair(4, &mut km1, &mut km2, &mut kbi, &mut ke64, &mut ks32, &mut knq, &mut out);
        if is_six {
            saar += aar; saac += aac; sbir += bir; sbic += bic;
            ser += er; sec += ec; ssr += sr; ssc += sc; snr += nr; snc += nc;
        }
        println!(
            "1T {:12} | A/A={:.3} | BiOp={:.3} | Ep64={:.3} | Sub32={:.3} | NoQ={:.3}",
            b.name, aac / aar, bic / bir, ec / er, sc / sr, nc / nr
        );
    }
    println!(
        "1T 6-BLK SUM  | A/A={:.4} | BiOp={:.4} | Ep64={:.4} | Sub32={:.4} | NoQ={:.4}",
        saac / saar, sbic / sbir, sec / ser, ssc / ssr, snc / snr
    );

    println!("\n=== 4. 2T CONCURRENT BARRIER A/A & A/B (40 passes/worker) ===");
    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [
        ("2T A/A (Master1 vs Master2)", 0),
        ("2T A/B (Master1 vs BiOperad)", 1),
        ("2T A/B (Master1 vs EpochTile64)", 2),
        ("2T A/B (Master1 vs SubTile32)", 3),
        ("2T A/B (Master1 vs NoQueue-Bi)", 4),
    ] {
        let barrier = Barrier::new(workers);
        let six_ref = &six;
        let res: Vec<(Vec<f64>, Vec<f64>)> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..workers)
                .map(|w| {
                    let bar = &barrier;
                    s.spawn(move || {
                        let mut km = master_a1::Kernel::default();
                        let mut km2 = master_a2::Kernel::default();
                        let mut kb = cand_cks_bi::Kernel::default();
                        let mut ke = cand_epoch64::Kernel::default();
                        let mut ks = cand_subtile32::Kernel::default();
                        let mut kn = cand_noqueue::Kernel::default();
                        let mut out = Vec::with_capacity(900_064);
                        for b in six_ref {
                            km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out);
                            ke.run_entry(&b.records, b.n, b.p, cand_epoch64::Cap::NONE, &mut out);
                            ks.run_entry(&b.records, b.n, b.p, cand_subtile32::Cap::NONE, &mut out);
                            kn.run_entry(&b.records, b.n, b.p, cand_noqueue::Cap::NONE, &mut out);
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
                                else if mode == 1 { kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); }
                                else if mode == 2 { ke.run_entry(&b.records, b.n, b.p, cand_epoch64::Cap::NONE, &mut out); }
                                else if mode == 3 { ks.run_entry(&b.records, b.n, b.p, cand_subtile32::Cap::NONE, &mut out); }
                                else { kn.run_entry(&b.records, b.n, b.p, cand_noqueue::Cap::NONE, &mut out); }
                            }
                            let dt = t0.elapsed().as_secs_f64() * 1e6;
                            if pass >= 4 {
                                if which == 0 { tr.push(dt); } else { tc.push(dt); }
                            }
                        }
                        (tr, tc)
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut ar: Vec<f64> = res.iter().flat_map(|x| x.0.clone()).collect();
        let mut ac: Vec<f64> = res.iter().flat_map(|x| x.1.clone()).collect();
        println!(
            "{:34} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
            label,
            pct(&mut ac, 1, 10) / pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 10), pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2),
            pct(&mut ac, 1, 2), pct(&mut ar, 1, 2)
        );
    }
}
