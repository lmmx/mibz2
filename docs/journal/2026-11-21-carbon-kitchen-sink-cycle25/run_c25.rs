#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_cks_diet;
mod cand_cks_bi;

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

fn make_periodic_150k() -> Block {
    let p_len = 30_000usize;
    let d = 5usize;
    let mut base = Vec::with_capacity(p_len);
    let mut s = 0xCAFE_BABE_DEAD_BEEFu64;
    for i in 0..p_len {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        let run = ((s >> 58) % 24 + 1) as usize;
        let ch = b'a' + ((i + (s >> 40) as usize) % 8) as u8;
        for _ in 0..run.min(p_len - base.len()) {
            base.push(ch);
        }
        if base.len() >= p_len {
            break;
        }
    }
    base.truncate(p_len);
    let text = base.repeat(d);
    let n = text.len();
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
            if copy * p_len + sb == 0 {
                p = l.len() as u32;
            }
            l.push(base[(sb + p_len - 1) % p_len]);
        }
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, text);
    let records = records_of(&l);
    Block { name: "periodic-150k".into(), n, p, records, expected }
}

fn make_periodic_105k_d3() -> Block {
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
            if copy * p_len + sb == 0 {
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

fn make_spiral_200k() -> Block {
    // Fibonacci / Sturmian word of length ~200k triggers deep Rauzy-Veech spirals!
    let mut a = vec![b'a'];
    let mut b = vec![b'a', b'b'];
    while b.len() < 200_000 {
        let mut next = Vec::with_capacity(b.len() + a.len());
        next.extend_from_slice(&b);
        next.extend_from_slice(&a);
        a = b;
        b = next;
    }
    b.truncate(200_000);
    let n = b.len();
    // Sturmian BWT: only 2 runs on exact Fibonacci, truncated has small run count & huge spirals!
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&x, &y| {
        let (ix, iy) = (x as usize, y as usize);
        let mut k = 0;
        while k < n {
            let cx = b[(ix + k) % n];
            let cy = b[(iy + k) % n];
            if cx != cy {
                return cx.cmp(&cy);
            }
            k += 1;
        }
        ix.cmp(&iy)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(b[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert(&l, p);
    assert_eq!(expected, b);
    let records = records_of(&l);
    Block { name: "spiral-200k".into(), n, p, records, expected }
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
    let per150 = make_periodic_150k();
    let per105 = make_periodic_105k_d3();
    let spiral = make_spiral_200k();
    let mut out = Vec::with_capacity(900_064);

    println!("=== 1. EXHAUSTIVE PARITY, MALFORMED-COLUMN SAFETY & #703 CAP-REFUSAL GATE ===");
    let mut kd_chk = cand_cks_diet::Kernel::default();
    let mut kb_chk = cand_cks_bi::Kernel::default();
    let mut exhaustive_cnt = 0usize;

    // Binary 1..=11 across INDUCE=true/false, QUEUE=true/false, LIMIT in {1, 4, 16}
    for len in 1..=11 {
        for mask in 0..(1usize << len) {
            let text: Vec<u8> = (0..len).map(|i| b'a' + ((mask >> i) & 1) as u8).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            for &lim in &[1u32, 4, 16] {
                kd_chk.try_run::<true, true>(&recs, len, p, lim, cand_cks_diet::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kd_chk.try_run::<true, false>(&recs, len, p, lim, cand_cks_diet::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kd_chk.try_run::<false, true>(&recs, len, p, lim, cand_cks_diet::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kb_chk.try_run::<true, true>(&recs, len, p, lim, cand_cks_bi::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                kb_chk.try_run::<true, false>(&recs, len, p, lim, cand_cks_bi::Cap::NONE, &mut out).unwrap();
                assert_eq!(out, text);
                exhaustive_cnt += 5;
            }
        }
    }
    // Ternary 1..=7
    for len in 1..=7 {
        let mut pow = 1usize;
        for _ in 0..len { pow *= 3; }
        for code in 0..pow {
            let mut x = code;
            let text: Vec<u8> = (0..len).map(|_| { let r = (x % 3) as u8; x /= 3; b'a' + r }).collect();
            let (l, p) = bwt_encode(&text);
            let recs = records_of(&l);
            kd_chk.run_entry(&recs, len, p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text);
            kb_chk.run_entry(&recs, len, p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
            assert_eq!(out, text);
            exhaustive_cnt += 2;
        }
    }
    // Malformed / corrupted non-BWT inputs (including n = 70_000 >= 65_536 XOR gate!)
    let mut rng = 0xDEAD_BEEF_CAFE_1234u64;
    let mut malformed_cnt = 0usize;
    for case in 0..500usize {
        let n = if case < 490 { (case % 250) + 1 } else { 70_000 };
        let mut col = Vec::with_capacity(n);
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            col.push(((rng >> 56) & 7) as u8);
        }
        let p = ((rng >> 32) as usize % n) as u32;
        let recs = records_of(&col);
        kd_chk.run_entry(&recs, n, p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
        kb_chk.run_entry(&recs, n, p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out.len(), n);
        malformed_cnt += 2;
    }

    // #703 Cap::dense refusal scratch test on high-entropy uncompressible 200k block
    {
        let n_ref = 200_000usize;
        let mut col = Vec::with_capacity(n_ref);
        let mut s = 1u64;
        for _ in 0..n_ref {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            col.push((s >> 56) as u8);
        }
        let recs = records_of(&col);
        let mut km_r = master_a1::Kernel::default();
        let mut kd_r = cand_cks_diet::Kernel::default();
        let mut kb_r = cand_cks_bi::Kernel::default();
        assert!(km_r.run_entry(&recs, n_ref, 0, master_a1::Cap::dense(n_ref), &mut out).is_none());
        assert!(kd_r.run_entry(&recs, n_ref, 0, cand_cks_diet::Cap::dense(n_ref), &mut out).is_none());
        assert!(kb_r.run_entry(&recs, n_ref, 0, cand_cks_bi::Cap::dense(n_ref), &mut out).is_none());
        let sm_r = km_r.scratch_report().0;
        let sd_r = kd_r.scratch_report().0;
        let sb_r = kb_r.scratch_report().0;
        assert!(sd_r <= sm_r && sb_r <= sm_r);
        println!(
            "Verified {exhaustive_cnt} exhaustive BWTs + {malformed_cnt} malformed non-BWT runs + #703 Cap::dense refusal (Master={sm_r} B, DietOperad={sd_r} B, BiOperad={sb_r} B)!"
        );
    }

    // Verify stress blocks (dense-300k, periodic-150k, periodic-105k-d3, spiral-200k)
    for sb in [&dense, &per150, &per105, &spiral] {
        kd_chk.run_entry(&sb.records, sb.n, sb.p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, sb.expected, "CKS-DietOperad mismatch on {}", sb.name);
        kb_chk.run_entry(&sb.records, sb.n, sb.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, sb.expected, "CKS-BiOperad mismatch on {}", sb.name);
    }

    println!("\n=== 2. PER-BLOCK FRESH & CUMULATIVE 6-BLOCK WORKER SCRATCH AUDIT (#704 GATE) ===");
    for b in six.iter().chain(std::iter::once(&dense)) {
        let mut km_f = master_a1::Kernel::default();
        let mut kd_f = cand_cks_diet::Kernel::default();
        let mut kb_f = cand_cks_bi::Kernel::default();
        km_f.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kd_f.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kb_f.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        let (sm, dm, _) = km_f.scratch_report();
        let (sd, dd, _) = kd_f.scratch_report();
        let (sb, db, _) = kb_f.scratch_report();
        println!(
            "FRESH {:12} | Master={:6.1} KiB (Dict={:4.1}K) | DietOperad={:6.1} KiB ({:+.1}%, Dict={:4.1}K) | BiOperad={:6.1} KiB ({:+.1}%, Dict={:4.1}K)",
            b.name,
            sm as f64 / 1024.0, dm as f64 / 1024.0,
            sd as f64 / 1024.0, (sd as f64 / sm as f64 - 1.0) * 100.0, dd as f64 / 1024.0,
            sb as f64 / 1024.0, (sb as f64 / sm as f64 - 1.0) * 100.0, db as f64 / 1024.0,
        );
    }

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut kdiet = cand_cks_diet::Kernel::default();
    let mut kbi = cand_cks_bi::Kernel::default();
    for b in &six {
        km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kdiet.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
        kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out).unwrap();
        assert_eq!(out, b.expected);
    }
    let (sm, _, bm) = km1.scratch_report();
    let (sd, _, bd) = kdiet.scratch_report();
    let (sb, _, bbi) = kbi.scratch_report();
    println!("\nCUMULATIVE 6-BLK Master 0.5.11  : {:9} B ({:6.1} KiB) | {}", sm, sm as f64 / 1024.0, bm);
    println!("CUMULATIVE 6-BLK CKS-DietOperad : {:9} B ({:6.1} KiB, {:+.2}%) | {}", sd, sd as f64 / 1024.0, (sd as f64 / sm as f64 - 1.0) * 100.0, bd);
    println!("CUMULATIVE 6-BLK CKS-BiOperad   : {:9} B ({:6.1} KiB, {:+.2}%) | {}", sb, sb as f64 / 1024.0, (sb as f64 / sm as f64 - 1.0) * 100.0, bbi);

    println!("\n=== 3. STAGE ATTRIBUTION (`construct` vs `walk`, 60 rounds/block) ===");
    let s_reps = 60usize;
    let (mut scm, mut swm, mut scd, mut swd, mut scb, mut swb) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in &six {
        let (mut mc, mut mw, mut dc, mut dw, mut bc, mut bw) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        for i in 0..3 * s_reps {
            match (i + i / 3) % 3 {
                0 => { let (c, w) = km1.run_stages(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); mc.push(c); mw.push(w); }
                1 => { let (c, w) = kdiet.run_stages(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out); dc.push(c); dw.push(w); }
                _ => { let (c, w) = kbi.run_stages(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); bc.push(c); bw.push(w); }
            }
        }
        let (mc10, mw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10));
        let (dc10, dw10) = (pct(&mut dc, 1, 10), pct(&mut dw, 1, 10));
        let (bc10, bw10) = (pct(&mut bc, 1, 10), pct(&mut bw, 1, 10));
        scm += mc10; swm += mw10; scd += dc10; swd += dw10; scb += bc10; swb += bw10;
        println!(
            "STAGE {:12} | construct: M={:6.1} Diet={:6.1} ({:.3}x) Bi={:6.1} ({:.3}x) | walk: M={:6.1} Diet={:6.1} ({:.3}x) Bi={:6.1} ({:.3}x)",
            b.name, mc10, dc10, dc10 / mc10, bc10, bc10 / mc10, mw10, dw10, dw10 / mw10, bw10, bw10 / mw10
        );
    }
    println!(
        "STAGE 6-BLK SUM    | construct: M={:6.1} Diet={:6.1} ({:.4}x) Bi={:6.1} ({:.4}x) | walk: M={:6.1} Diet={:6.1} ({:.4}x) Bi={:6.1} ({:.4}x)",
        scm, scd, scd / scm, scb, scb / scm, swm, swd, swd / swm, swb, swb / swm
    );

    println!("\n=== 4. HIGH-PRECISION 1T INTERLEAVED A/A & A/B (80 pairs/block + stress suite) ===");
    let reps = 80usize;
    let (mut saar, mut saac, mut sdr, mut sdc, mut sbr, mut sbc) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let (mut saar50, mut saac50, mut sdr50, mut sdc50, mut sbr50, mut sbc50) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for b in six.iter().chain([&dense, &per150, &spiral]) {
        let is_six = b.n >= 890_000;
        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac, aar5, aac5) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10), pct(&mut ta, 1, 2), pct(&mut tb, 1, 2));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kdiet.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (dr, dc, dr5, dc5) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10), pct(&mut ta, 1, 2), pct(&mut tb, 1, 2));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { kbi.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (br, bc, br5, bc5) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10), pct(&mut ta, 1, 2), pct(&mut tb, 1, 2));

        if is_six {
            saar += aar; saac += aac; sdr += dr; sdc += dc; sbr += br; sbc += bc;
            saar50 += aar5; saac50 += aac5; sdr50 += dr5; sdc50 += dc5; sbr50 += br5; sbc50 += bc5;
        }
        println!(
            "1T {:14} | A/A p10={:.3} | CKS-DietOperad p10={:.3} p50={:.3} ({:.1}/{:.1}) | CKS-BiOperad p10={:.3} p50={:.3} ({:.1}/{:.1})",
            b.name, aac / aar, dc / dr, dc5 / dr5, dc, dr, bc / br, bc5 / br5, bc, br
        );
    }
    println!(
        "1T 6-BLK SUM p10  | A/A={:.4} | CKS-DietOperad={:.4} ({:.1}/{:.1} us) | CKS-BiOperad={:.4} ({:.1}/{:.1} us)",
        saac / saar, sdc / sdr, sdc, sdr, sbc / sbr, sbc, sbr
    );
    println!(
        "1T 6-BLK SUM p50  | A/A={:.4} | CKS-DietOperad={:.4} ({:.1}/{:.1} us) | CKS-BiOperad={:.4} ({:.1}/{:.1} us)",
        saac50 / saar50, sdc50 / sdr50, sdc50, sdr50, sbc50 / sbr50, sbc50, sbr50
    );

    println!("\n=== 5. 2T CONCURRENT BARRIER A/A & A/B (50 passes/worker) ===");
    let workers = 2usize;
    let passes = 50usize;
    for (label, mode) in [
        ("2T A/A (Master1 vs Master2)", 0),
        ("2T A/B (Master1 vs CKS-DietOperad)", 1),
        ("2T A/B (Master1 vs CKS-BiOperad)", 2),
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
                        let mut kd = cand_cks_diet::Kernel::default();
                        let mut kb = cand_cks_bi::Kernel::default();
                        let mut out = Vec::with_capacity(900_064);
                        for b in six_ref {
                            km.run_entry(&b.records, b.n, b.p, master_a1::Cap::NONE, &mut out);
                            km2.run_entry(&b.records, b.n, b.p, master_a2::Cap::NONE, &mut out);
                            kd.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out);
                            kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out);
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
                                    kd.run_entry(&b.records, b.n, b.p, cand_cks_diet::Cap::NONE, &mut out);
                                } else {
                                    kb.run_entry(&b.records, b.n, b.p, cand_cks_bi::Cap::NONE, &mut out);
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
            "{:36} | p10={:.4} ({:.1}/{:.1}) | p50={:.4} ({:.1}/{:.1})",
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
