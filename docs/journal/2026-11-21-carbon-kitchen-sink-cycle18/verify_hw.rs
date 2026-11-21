#![forbid(unsafe_code)]
#![allow(dead_code)]

mod master_a1;
mod master_a2;
mod cand_hotwire;

use std::time::Instant;

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 { *rec += 1; continue; }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn main() {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    let bs: Vec<_> = specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let r = records_of(&l);
        (f, l.len(), p, r)
    }).collect();

    let mut km1 = master_a1::Kernel::default();
    let mut km2 = master_a2::Kernel::default();
    let mut khw = cand_hotwire::Kernel::default();
    let mut out = Vec::with_capacity(900_064);
    for b in &bs {
        km1.run_entry(&b.3, b.1, b.2, master_a1::Cap::NONE, &mut out);
        km2.run_entry(&b.3, b.1, b.2, master_a2::Cap::NONE, &mut out);
        khw.run_entry(&b.3, b.1, b.2, cand_hotwire::Cap::NONE, &mut out);
    }

    let reps = 80usize;
    let (mut scm, mut swm, mut sch, mut swh) = (0.0, 0.0, 0.0, 0.0);
    let (mut saar, mut saac, mut shwr, mut shwc) = (0.0, 0.0, 0.0, 0.0);
    for b in &bs {
        let (mut mc, mut mw, mut hc, mut hw) = (vec![], vec![], vec![], vec![]);
        for i in 0..2 * reps {
            if (i + i / 2) % 2 == 0 {
                let (c, w) = km1.run_stages(&b.3, b.1, b.2, master_a1::Cap::NONE, &mut out);
                mc.push(c); mw.push(w);
            } else {
                let (c, w) = khw.run_stages(&b.3, b.1, b.2, cand_hotwire::Cap::NONE, &mut out);
                hc.push(c); hw.push(w);
            }
        }
        let (mc10, mw10, hc10, hw10) = (pct(&mut mc, 1, 10), pct(&mut mw, 1, 10), pct(&mut hc, 1, 10), pct(&mut hw, 1, 10));
        scm += mc10; swm += mw10; sch += hc10; swh += hw10;

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.3, b.1, b.2, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { km2.run_entry(&b.3, b.1, b.2, master_a2::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (aar, aac) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));

        let (mut ta, mut tb) = (vec![], vec![]);
        for i in 0..2 * reps {
            let t = Instant::now();
            if (i + i / 2) % 2 == 0 { km1.run_entry(&b.3, b.1, b.2, master_a1::Cap::NONE, &mut out); ta.push(t.elapsed().as_secs_f64() * 1e6); }
            else { khw.run_entry(&b.3, b.1, b.2, cand_hotwire::Cap::NONE, &mut out); tb.push(t.elapsed().as_secs_f64() * 1e6); }
        }
        let (hwr, hwc) = (pct(&mut ta, 1, 10), pct(&mut tb, 1, 10));
        saar += aar; saac += aac; shwr += hwr; shwc += hwc;

        println!("{:14} | construct: {:6.1}->{:6.1} ({:.3}x) | walk: {:6.1}->{:6.1} ({:.3}x) | A/A={:.3} | HotWire={:.3}",
            b.0, mc10, hc10, hc10 / mc10, mw10, hw10, hw10 / mw10, aac / aar, hwc / hwr);
    }
    println!("6-BLOCK SUM    | construct: {:6.1}->{:6.1} ({:.4}x) | walk: {:6.1}->{:6.1} ({:.4}x) | A/A={:.4} | HotWire={:.4}",
        scm, sch, sch / scm, swm, swh, swh / swm, saac / saar, shwc / shwr);
}
