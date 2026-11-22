#![forbid(unsafe_code)]
#![allow(dead_code)]
mod cand_c43_base;
mod cand_c43_s2b;
mod cand_c43_nb42;
mod cand_c43_nb43;
mod cand_c43_nb42_w16;
mod cand_c43_nbw16;

use std::time::Instant;
struct Block { name: String, n: usize, p: u32, records: Vec<u16> }
fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() { if (*rec >> 8) as u8 == b && (*rec & 255) < 127 { *rec += 1; continue; } }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}
fn load_six() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let mut specs = vec![]; let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    specs.into_iter().map(|(f, p)| {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        Block { name: f, n: l.len(), p, records: records_of(&l) }
    }).collect()
}
fn med(v: &mut [f64]) -> f64 { v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v[v.len() / 2] }
fn main() {
    let six = load_six();
    let mut out = Vec::with_capacity(900_064);
    let mut k0 = cand_c43_base::Kernel::default();
    let mut k1 = cand_c43_s2b::Kernel::default();
    let mut k2 = cand_c43_nb42::Kernel::default();
    let mut k3 = cand_c43_nb43::Kernel::default();
    let mut k4 = cand_c43_nb42_w16::Kernel::default();
    let mut k5 = cand_c43_nbw16::Kernel::default();
    let names = ["Head-Base", "Head-S2b", "Nobi-W8", "Nobi-W8+S2b", "Nobi-W16", "Nobi-W16+S2b"];
    let mut csum = [0.0; 6]; let mut wsum = [0.0; 6];
    for b in &six {
        let mut sc: [Vec<f64>; 6] = Default::default();
        let mut sw: [Vec<f64>; 6] = Default::default();
        for i in 0..6 * 40 {
            let m = (i + i / 6) % 6;
            let (c, w) = match m {
                0 => k0.run_stages(&b.records, b.n, b.p, cand_c43_base::Cap::NONE, &mut out),
                1 => k1.run_stages(&b.records, b.n, b.p, cand_c43_s2b::Cap::NONE, &mut out),
                2 => k2.run_stages(&b.records, b.n, b.p, cand_c43_nb42::Cap::NONE, &mut out),
                3 => k3.run_stages(&b.records, b.n, b.p, cand_c43_nb43::Cap::NONE, &mut out),
                4 => k4.run_stages(&b.records, b.n, b.p, cand_c43_nb42_w16::Cap::NONE, &mut out),
                _ => k5.run_stages(&b.records, b.n, b.p, cand_c43_nbw16::Cap::NONE, &mut out),
            };
            sc[m].push(c); sw[m].push(w);
        }
        for m in 0..6 { csum[m] += med(&mut sc[m]); wsum[m] += med(&mut sw[m]); }
    }
    for m in 0..6 {
        println!("{:14} | pre={:6.1} | emit={:6.1}", names[m], csum[m], wsum[m]);
    }
}
