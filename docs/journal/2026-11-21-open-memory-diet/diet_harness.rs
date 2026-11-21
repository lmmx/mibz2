//! (measurement only, not for master) the memory-diet harness: the six admitted blocks
//! (MIBZ2_ADMITTED_BWT, authenticated beforehand), the open kernel's scratch after each, and
//! its time against master's kernel (`diet_ref`, master's src/open.rs and endpoint.rs
//! unchanged) in the same process, decode by decode alternated: at one thread, and with four
//! concurrent workers. DIET_MODE = scratch | t1 | t4.
use super::*;
use std::time::Instant;

type Block = (String, Vec<u8>, u32, Runs);

fn blocks() -> Vec<Block> {
    let dir = std::env::var("MIBZ2_ADMITTED_BWT").expect("MIBZ2_ADMITTED_BWT");
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
            let r = Runs::of_column(&l);
            (f, l, p, r)
        })
        .collect()
}

fn decode(k: &mut Kernel, b: &Block, out: &mut Vec<u8>) -> Counts {
    k.run::<true, true>(b.3.records(), b.1.len(), b.2, LIMIT, out)
}

fn decode_ref(k: &mut super::diet_ref::Kernel, b: &Block, out: &mut Vec<u8>) {
    k.run::<true, true>(b.3.records(), b.1.len(), b.2, LIMIT, out);
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Both kernels on block `b`, `reps` times each, alternated (the order swapped each time):
/// the 10th-percentile time of each
fn pair(
    kr: &mut super::diet_ref::Kernel,
    k: &mut Kernel,
    b: &Block,
    out: &mut Vec<u8>,
    reps: usize,
) -> (f64, f64) {
    let (mut tr, mut tc) = (vec![], vec![]);
    for i in 0..2 * reps {
        let t = Instant::now();
        if (i + i / 2) % 2 == 0 {
            decode_ref(kr, b, out);
            tr.push(t.elapsed().as_secs_f64() * 1e6);
        } else {
            decode(k, b, out);
            tc.push(t.elapsed().as_secs_f64() * 1e6);
        }
    }
    tr.sort_by(|a, b| a.partial_cmp(b).unwrap());
    tc.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (tr[tr.len() / 10], tc[tc.len() / 10])
}

#[test]
#[ignore]
fn diet_harness() {
    let bs = blocks();
    let mode = std::env::var("DIET_MODE").unwrap_or_else(|_| "scratch".into());
    let reps: usize = std::env::var("DIET_REPS").ok().map_or(15, |v| v.parse().unwrap());
    match mode.as_str() {
        "scratch" => {
            let mut k = Kernel::default();
            let mut out = Vec::with_capacity(900_064);
            for b in &bs {
                let mut fresh = Kernel::default();
                let mut o2 = Vec::with_capacity(900_064);
                let c = decode(&mut fresh, b, &mut o2);
                println!(
                    "COUNTS {} n={} m={} thick={} nodes={} rules={} bands={} words={} spelling={}",
                    b.0, c.n, c.retained, c.thick, c.open_nodes, c.rules, c.bands, c.words, c.traced
                );
                decode(&mut k, b, &mut out);
                let want = super::b5_plain(&b.1, b.2);
                assert!(out == want && o2 == want, "{}: not byte-exact", b.0);
                println!(
                    "SCRATCH {} fresh={} cumulative={} {} | cumulative {}",
                    b.0,
                    diet_scratch(&fresh),
                    diet_scratch(&k),
                    diet_parts(&fresh),
                    diet_parts(&k)
                );
            }
        }
        "t1" => {
            let mut k = Kernel::default();
            let mut kr = super::diet_ref::Kernel::default();
            let mut out = Vec::with_capacity(900_064);
            for b in &bs {
                decode(&mut k, b, &mut out);
                decode_ref(&mut kr, b, &mut out);
            }
            for b in &bs {
                let (r, c) = pair(&mut kr, &mut k, b, &mut out, reps);
                println!("T1 {} {:.1} {:.1}", b.0, r, c);
            }
        }
        "t4" => {
            let workers = 4;
            let barrier = std::sync::Barrier::new(workers);
            let bs = &bs;
            // per worker, per block (and the whole pass, last), per kernel: its times
            let res: Vec<Vec<[Vec<f64>; 2]>> = std::thread::scope(|s| {
                let hs: Vec<_> = (0..workers)
                    .map(|w| {
                        let barrier = &barrier;
                        s.spawn(move || {
                            let mut k = Kernel::default();
                            let mut kr = super::diet_ref::Kernel::default();
                            let mut out = Vec::with_capacity(900_064);
                            let mut per = vec![[vec![], vec![]]; bs.len() + 1];
                            // every worker runs the same kernel in a pass, alternated by pass
                            for pass in 0..2 * reps + 4 {
                                let which = (pass + pass / 2) % 2;
                                barrier.wait();
                                let t0 = Instant::now();
                                for i in 0..bs.len() {
                                    // each worker starts at a different block
                                    let i = (i + w) % bs.len();
                                    let t = Instant::now();
                                    if which == 0 {
                                        decode_ref(&mut kr, &bs[i], &mut out);
                                    } else {
                                        decode(&mut k, &bs[i], &mut out);
                                    }
                                    if pass >= 4 {
                                        per[i][which].push(t.elapsed().as_secs_f64() * 1e6);
                                    }
                                }
                                if pass >= 4 {
                                    per[bs.len()][which].push(t0.elapsed().as_secs_f64() * 1e6);
                                }
                            }
                            per
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for i in 0..=bs.len() {
                let mut a: Vec<f64> = res.iter().flat_map(|r| r[i][0].clone()).collect();
                let mut c: Vec<f64> = res.iter().flat_map(|r| r[i][1].clone()).collect();
                let name = if i < bs.len() { bs[i].0.as_str() } else { "pass" };
                println!("T4 {} {:.1} {:.1}", name, median(&mut a), median(&mut c));
            }
        }
        _ => panic!("DIET_MODE"),
    }
}
