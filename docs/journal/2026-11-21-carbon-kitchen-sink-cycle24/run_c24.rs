#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_xor;
mod cand_xor_u8;
mod cand_xor4;

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

fn bwt_encode(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &s) in sa.iter().enumerate() {
        if s == 0 {
            p = i as u32;
        }
        l.push(text[(s + n - 1) % n]);
    }
    (l, p)
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
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "dense-300k".into(), n, p, records, expected }
}

fn make_periodic_large() -> Block {
    // d = 3 copies of a 35_000-byte dense period -> n = 105_000, m > 75_000 (>= 65_536 L2 gate!)
    // Explicitly exercises Obstruction B (disjoint cycles on periodic blocks when m >= 65_536)!
    let p_len = 35_000usize;
    let d = 3usize;
    let mut base = Vec::with_capacity(p_len);
    let mut s = 0x1234_5678_9ABC_DEF0u64;
    for i in 0..p_len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        base.push(if i % 11 == 0 { b'_' } else { b'a' + ((s >> 40) % 26) as u8 });
    }
    let text = base.repeat(d);
    let n = text.len();
    // Sort cyclic shifts of base, then replicate each shift d times (exact BWT of base^d!)
    let mut sa_base: Vec<usize> = (0..p_len).collect();
    sa_base.sort_by(|&a, &b| {
        for k in 0..p_len {
            let ca = base[(a + k) % p_len];
            let cb = base[(b + k) % p_len];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for &sb in &sa_base {
        for copy in 0..d {
            let full_pos = copy * p_len + sb;
            if full_pos == 0 {
                p = l.len() as u32;
            }
            l.push(base[(sb + p_len - 1) % p_len]);
        }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "periodic-105k-d3".into(), n, p, records, expected }
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
    let per_large = make_periodic_large();
    let mut out = Vec::with_capacity(900_064);

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kfx = cand_xor::Kernel::default();
    let mut ku8 = cand_xor_u8::Kernel::default();
    let mut kx4 = cand_xor4::Kernel::default();

    println!("=== 1. EXHAUSTIVE SMALL + PERIODIC-LARGE + DENSE + 6-BLOCK PARITY & SCRATCH AUDIT ===");
    // Small exhaustive binary 1..=10
    let mut small_checks = 0usize;
    for len in 1..=10 {
        for mask in 0..(1usize << len) {
            let text: Vec<u8> = (0..len).map(|i| b'a' + ((mask >> i) & 1) as u8).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            ku8.run_entry(&recs, len, p, cand_xor_u8::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text);
            kx4.run_entry(&recs, len, p, cand_xor4::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text);
            small_checks += 1;
        }
    }
    // Periodic large (m >= 65_536, d = 3)
    let cp = kx4.run_entry(&per_large.records, per_large.n, per_large.p, cand_xor4::Cap::NONE, &mut out).unwrap();
    assert_eq!(out, per_large.expected, "XOR-BiEmit4 failed periodic-105k-d3!");
    ku8.run_entry(&per_large.records, per_large.n, per_large.p, cand_xor_u8::Cap::NONE, &mut out).unwrap();
    assert_eq!(out, per_large.expected);
    println!("Verified {small_checks} small BWTs + {} (m={}, d=3 periodic >= 65_536 gate)!", per_large.name, cp.retained);

    // Reset fresh kernels for exact cumulative 6-block Wikidata scratch audit
    km1 = master_a1::Kernel::default();
    km2 = master_a2::Kernel::default();
    kfx = cand_xor::Kernel::default();
    ku8 = cand_xor_u8::Kernel::default();
    kx4 = cand_xor4::Kernel::default();

    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kfx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        ku8.run_entry(&b.records, b.n, b.p, cand_xor_u8::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "Unrolled4-Epoch-XOR mismatch on {}", b.name);
        kx4.run_entry(&b.records, b.n, b.p, cand_xor4::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected, "XOR-BiEmit4 mismatch on {}", b.name);
    }

    let (sm, _, _) = km1.scratch_report();
    let (sfx, _, _) = kfx.scratch_report();
    let (su8, _, _) = ku8.scratch_report();
    let (sx4, _, _) = kx4.scratch_report();
    println!("SCRATCH Master 0.5.11       : {:9} B ({:6.1} KiB)", sm, sm as f64 / 1024.0);
    println!("SCRATCH Fused-XOR (mu=2)    : {:9} B ({:6.1} KiB, {:+} B)", sfx, sfx as f64 / 1024.0, sfx as isize - sm as isize);
    println!("SCRATCH Unrolled4-XOR (mu=2): {:9} B ({:6.1} KiB, {:+} B)", su8, su8 as f64 / 1024.0, su8 as isize - sm as isize);
    println!("SCRATCH XOR-BiEmit4 (mu=4)  : {:9} B ({:6.1} KiB, {:+} B = {:+.1}%)", sx4, sx4 as f64 / 1024.0, sx4 as isize - sm as isize, (sx4 as f64 / sm as f64 - 1.0) * 100.0);

    // Verify dense-300k parity too (on separate kernels so cumulative 6-block scratch isn't inflated before 2T)
    {
        let mut t_u8 = cand_xor_u8::Kernel::default();
        let mut t_x4 = cand_xor4::Kernel::default();
        t_u8.run_entry(&dense.records, dense.n, dense.p, cand_xor_u8::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
        t_x4.run_entry(&dense.records, dense.n, dense.p, cand_xor4::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, dense.expected);
    }

    println!("\n=== 2. STAGE ATTRIBUTION (50 interleaved pairs/block) ===");
    let reps = 50usize;
    let (mut scm, mut swm, mut scfx, mut swfx, mut scu8, mut swu8, mut scx4, mut swx4) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut fxc, mut fxw, mut u8c, mut u8w, mut x4c, mut x4w) =
            (vec![], vec![], vec![], vec![], vec![], vec![], vec![], vec![]);
        for i in 0..4 * reps {
            match (i + i / 4) % 4 {
                0 => { let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); mc.push(c); mw.push(w); }
                1 => { let (c, w) = kfx.run_stages(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out); fxc.push(c); fxw.push(w); }
                2 => { let (c, w) = ku8.run_stages(&b.records, b.n, b.p, cand_xor_u8::Cap::NONE, &mut out); u8c.push(c); u8w.push(w); }
                _ => { let (c, w) = kx4.run_stages(&b.records, b.n, b.p, cand_xor4::Cap::NONE, &mut out); x4c.push(c); x4w.push(w); }
            }
        }
        let (mc10, mw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10));
        let (fxc10, fxw10) = (pct(&mut fxc, 1, 10), pct(&mut fxw, 1, 10));
        let (u8c10, u8w10) = (pct(&mut u8c, 1, 10), pct(&mut u8w, 1, 10));
        let (x4c10, x4w10) = (pct(&mut x4c, 1, 10), pct(&mut x4w, 1, 10));
        scm += mc10; swm += mw10;
        scfx += fxc10; swfx += fxw10;
        scu8 += u8c10; swu8 += u8w10;
        scx4 += x4c10; swx4 += x4w10;
        println!(
            "STAGE {:12} | construct: M={:6.1} FX={:6.1} U8={:6.1} X4={:6.1} | walk: M={:6.1} FX={:6.1} ({:.3}x) U8={:6.1} ({:.3}x) X4={:6.1} ({:.3}x)",
            b.name, mc10, fxc10, u8c10, x4c10, mw10, fxw10, fxw10 / mw10, u8w10, u8w10 / mw10, x4w10, x4w10 / mw10
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: M={:6.1} FX={:6.1} ({:+.1}) U8={:6.1} ({:+.1}) X4={:6.1} ({:+.1}) | walk: M={:6.1} FX={:6.1} ({:.4}x) U8={:6.1} ({:.4}x) X4={:6.1} ({:.4}x)",
        scm, scfx, scfx - scm, scu8, scu8 - scm, scx4, scx4 - scm,
        swm, swfx, swfx / swm, swu8, swu8 / swm, swx4, swx4 / swm
    );

    println!("\n=== 3. END-TO-END 1T PAIRWISE INTERLEAVED A/A & A/B (60 pairs/block + dense-300k) ===");
    let reps = 60usize;
    let (mut saar, mut saac, mut sfxr, mut sfxc, mut su8r, mut su8c, mut sx4r, mut sx4c) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain(std::iter::once(&dense)) {
        let is_six = b.name != "dense-300k";
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1000000.0); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kfx.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (fxr, fxc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { ku8.run_entry(&b.records, b.n, b.p, cand_xor_u8::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (u8r, u8c) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kx4.run_entry(&b.records, b.n, b.p, cand_xor4::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (x4r, x4c) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        if is_six {
            saar += aar; saac += aac;
            sfxr += fxr; sfxc += fxc;
            su8r += u8r; su8c += u8c;
            sx4r += x4r; sx4c += x4c;
        }
        println!(
            "1T {:12} | A/A={:.3} | Fused-XOR={:.3} ({:.1}/{:.1}) | Unrolled4-XOR={:.3} ({:.1}/{:.1}) | XOR-BiEmit4={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, fxc / fxr, fxc, fxr, u8c / u8r, u8c, u8r, x4c / x4r, x4c, x4r
        );
    }
    println!(
        "1T 6-BLOCK SUM  | A/A={:.4} | Fused-XOR={:.4} | Unrolled4-XOR={:.4} | XOR-BiEmit4={:.4}",
        saac / saar, sfxc / sfxr, su8c / su8r, sx4c / sx4r
    );

    println!("\n=== 4. 2T CONCURRENT BARRIER A/A & A/B (40 passes/worker) ===");
    let workers = 2usize;
    let passes = 40usize;
    for (label, mode) in [
        ("2T A/A (Master1 vs Master2)", 0),
        ("2T A/B (Master1 vs Fused-XOR)", 1),
        ("2T A/B (Master1 vs Unrolled4-XOR)", 2),
        ("2T A/B (Master1 vs XOR-BiEmit4)", 3),
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
                        let mut kf = cand_xor::Kernel::default();
                        let mut ku = cand_xor_u8::Kernel::default();
                        let mut kx = cand_xor4::Kernel::default();
                        let mut out = Vec::with_capacity(900_064);
                        for b in six_ref {
                            km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            kf.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
                            ku.run_entry(&b.records, b.n, b.p, cand_xor_u8::Cap::NONE, &mut out);
                            kx.run_entry(&b.records, b.n, b.p, cand_xor4::Cap::NONE, &mut out);
                        }
                        let (mut tr, mut tc) = (vec![], vec![]);
                        for pass in 0..(2 * passes + 4) {
                            let which = (pass + pass / 2) % 2;
                            bar.wait();
                            let t0 = Instant::now();
                            for i in 0..six_ref.len() {
                                let b = &six_ref[(i + w * 3) % six_ref.len()];
                                if which == 0 {
                                    km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                                } else if mode == 0 {
                                    km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                                } else if mode == 1 {
                                    kf.run_entry(&b.records, b.n, b.p, cand_xor::Cap::NONE, &mut out);
                                } else if mode == 2 {
                                    ku.run_entry(&b.records, b.n, b.p, cand_xor_u8::Cap::NONE, &mut out);
                                } else {
                                    kx.run_entry(&b.records, b.n, b.p, cand_xor4::Cap::NONE, &mut out);
                                }
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
            pct(&mut ac, 1, 10),
            pct(&mut ar, 1, 10),
            pct(&mut ac, 1, 2) / pct(&mut ar, 1, 2),
            pct(&mut ac, 1, 2),
            pct(&mut ar, 1, 2)
        );
    }
}
