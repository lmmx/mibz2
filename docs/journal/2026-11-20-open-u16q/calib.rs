use super::*;
use std::time::Instant;
#[test]
#[ignore]
fn calib_six_blocks() {
    let dir = "/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/wt-data/admitted-bwt";
    let blocks = [("block-0.L",264045u32),("block-2.L",320807)];
    let man = std::fs::read_to_string(format!("{dir}/manifest.json")).unwrap();
    let _ = blocks;
    // parse file/primary_index pairs crudely
    let mut specs = vec![];
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") { file = v.trim_end_matches("\",").to_string(); }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") { specs.push((file.clone(), v.trim_end_matches(',').parse::<u32>().unwrap())); }
    }
    let mut k = Kernel::default();
    for (f, p) in specs {
        let l = std::fs::read(format!("{dir}/{f}")).unwrap();
        let n = l.len();
        let r = crate::runs::Runs::of_column(&l);
        let mut out = Vec::with_capacity(n + 64);
        let (mut bc, mut be, mut bt) = (f64::MAX, f64::MAX, f64::MAX);
        for _ in 0..25 {
            let t0 = Instant::now();
            let (_s, mark, bits) = k.construct::<true, true>(r.records(), n as u32, p, LIMIT);
            let t1 = Instant::now();
            emit(&k.edges, &k.extra, &k.spelling, &k.offsets, n, mark, bits, &mut out);
            let t2 = Instant::now();
            bc = bc.min((t1 - t0).as_secs_f64() * 1e3);
            be = be.min((t2 - t1).as_secs_f64() * 1e3);
            bt = bt.min((t2 - t0).as_secs_f64() * 1e3);
        }
        let t0 = Instant::now();
        for _ in 0..5 { let _ = crate::runs::Runs::of_column(&l); }
        let rr = (Instant::now() - t0).as_secs_f64() * 1e3 / 5.0;
        println!("CALIB {f:12} n={n} construct={bc:.3} emit={be:.3} total={bt:.3} ms  (runs-of-column {rr:.3} ms)");
    }
}
