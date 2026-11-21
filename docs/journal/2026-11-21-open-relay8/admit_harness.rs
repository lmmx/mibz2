//! (measurement only, not for master) the dense-admission harness, after #692's diet harness:
//! every block of a .bz2 file (ADMIT_BZ2) and the six authenticated blocks
//! (MIBZ2_ADMITTED_BWT), decoded with this tree's `OpenCells` and with master's (`admit_ref`:
//! master's src/open.rs, endpoint.rs and outq.rs unchanged) in the same process.
//! ADMIT_MODE = census | parity | heap | t1 | t4.
use super::*;
use crate::bwt::{InverseBwt, Plain};
use crate::stream::{Event, Scanner, SliceSource};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::time::Instant;

// A thread's live and peak heap: a counting pass-through to the system allocator (the test
// binary's own), measurement only
struct Track;
thread_local! {
    static LIVE: Cell<isize> = const { Cell::new(0) };
    static PEAK: Cell<isize> = const { Cell::new(0) };
}
fn add(d: isize) {
    let _ = LIVE.try_with(|l| {
        let v = l.get() + d;
        l.set(v);
        let _ = PEAK.try_with(|p| {
            if v > p.get() {
                p.set(v)
            }
        });
    });
}
unsafe impl GlobalAlloc for Track {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            add(l.size() as isize);
        }
        p
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(l) };
        if !p.is_null() {
            add(l.size() as isize);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        add(-(l.size() as isize));
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let q = unsafe { System.realloc(p, l, new) };
        if !q.is_null() {
            add(new as isize - l.size() as isize);
        }
        q
    }
}
#[global_allocator]
static TRACK: Track = Track;
fn peak_reset() {
    let v = LIVE.with(|l| l.get());
    PEAK.with(|p| p.set(v));
}
fn peak_over(base: isize) -> isize {
    PEAK.with(|p| p.get()) - base
}
fn live() -> isize {
    LIVE.with(|l| l.get())
}

pub(super) enum Blk {
    Run { idx: usize, n: usize, p: u32, runs: Runs },
    Col { idx: usize, ll: Vec<u8>, counts: [u32; 256], p: u32 },
}

fn load_bz2(path: &str) -> Vec<Blk> {
    let data = std::fs::read(path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut v = vec![];
    let mut idx = 0;
    while let Ok(Event::Block { symbols: b, .. }) = sc.next(std::mem::take(&mut syms)) {
        let p = b.orig_ptr;
        if crate::runs::takes_runs(&b) {
            let mut r = Runs::default();
            let n = crate::runs::front(&b, &mut r, k).unwrap();
            v.push(Blk::Run { idx, n, p, runs: r });
        } else {
            let mut ll = Vec::new();
            let mut counts = [0u32; 256];
            crate::mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
            v.push(Blk::Col { idx, ll, counts, p });
        }
        syms = b.syms;
        idx += 1;
    }
    v
}

fn load_six() -> Vec<(String, Blk)> {
    let dir = match std::env::var("MIBZ2_ADMITTED_BWT") {
        Ok(d) => d,
        Err(_) => return vec![],
    };
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
        .enumerate()
        .map(|(i, (f, p))| {
            let l = std::fs::read(format!("{dir}/{f}")).unwrap();
            let runs = Runs::of_column(&l);
            (f, Blk::Run { idx: 10_000 + i, n: l.len(), p, runs })
        })
        .collect()
}

fn column(r: &Runs) -> Vec<u8> {
    let mut l = vec![];
    for &rec in r.records() {
        l.extend(std::iter::repeat((rec >> 8) as u8).take(usize::from(rec & 255)));
    }
    l
}

fn plain(l: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in l {
        counts[c as usize] += 1;
    }
    let mut out = Vec::new();
    Plain::default().invert(l, &counts, p, &mut out);
    out
}

fn dec<I: InverseBwt>(k: &mut I, b: &Blk, out: &mut Vec<u8>) {
    match b {
        Blk::Run { n, p, runs, .. } => {
            assert!(k.invert_runs(runs, *n, *p, out, Kernels::detect()));
        }
        Blk::Col { ll, counts, p, .. } => k.invert(ll, counts, *p, out),
    }
}

type Cand = OpenCells;
type Refk = super::admit_ref::OpenCells;

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// both kernels on `b`, `reps` times each, alternated: the 10th-percentile time of each
fn pair(kr: &mut Refk, k: &mut Cand, b: &Blk, out: &mut Vec<u8>, reps: usize) -> (f64, f64) {
    let (mut tr, mut tc) = (vec![], vec![]);
    for i in 0..2 * reps {
        let t = Instant::now();
        if (i + i / 2) % 2 == 0 {
            dec(kr, b, out);
            tr.push(t.elapsed().as_secs_f64() * 1e6);
        } else {
            dec(k, b, out);
            tc.push(t.elapsed().as_secs_f64() * 1e6);
        }
    }
    tr.sort_by(|a, b| a.partial_cmp(b).unwrap());
    tc.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (tr[tr.len() / 10], tc[tc.len() / 10])
}

fn ref_admits(b: &Blk) -> bool {
    match b {
        Blk::Run { n, p, runs, .. } => super::admit_ref::OpenCells::admits(runs.records().len(), *n, *p),
        _ => false,
    }
}

/// whether this tree's `OpenCells` inverts `b` with the open kernel (its admission)
fn cand_opens(b: &Blk) -> bool {
    match b {
        Blk::Run { .. } => {
            let mut k = Cand::default();
            let mut out = vec![];
            dec(&mut k, b, &mut out);
            k.opened > 0
        }
        _ => false,
    }
}

fn label(b: &Blk) -> String {
    match b {
        Blk::Run { idx, .. } | Blk::Col { idx, .. } => format!("blk{idx}"),
    }
}

#[test]
#[ignore]
fn admit_harness() {
    let mode = std::env::var("ADMIT_MODE").unwrap_or_else(|_| "census".into());
    let reps: usize = std::env::var("ADMIT_REPS").ok().map_or(15, |v| v.parse().unwrap());
    let path = std::env::var("ADMIT_BZ2").expect("ADMIT_BZ2");
    let all = load_bz2(&path);
    let six = load_six();
    match mode.as_str() {
        "census" => {
            let (mut runb, mut colb, mut ra, mut ca, mut full, mut fra, mut fca) = (0, 0, 0, 0, 0, 0, 0);
            let mut spiral_blocks = 0;
            for b in &all {
                match b {
                    Blk::Run { idx, n, p, runs } => {
                        runb += 1;
                        let (r, a, c) = (ref_admits(b), runs.records().len(), cand_opens(b));
                        ra += usize::from(r);
                        ca += usize::from(c);
                        if *n >= 65_536 {
                            full += 1;
                            fra += usize::from(r);
                            fca += usize::from(c);
                        }
                        let s = forced_counts(runs, *n, *p);
                        spiral_blocks += 0usize;
                        if false {
                            println!("SPIRALS {idx} n={n} master={r} steps={} rules={}", s.steps, s.rules);
                        }
                        if *n >= 65_536 && (!r || !c) || std::env::var("ADMIT_ALL").is_ok() {
                            println!(
                                "BLOCK {idx} n={n} r={a} r/n={:.4} master={} cand={} m={} ns={} nodes={} steps={} rules={} words={}",
                                a as f64 / *n as f64, r, c, s.retained, s.thick, s.open_nodes, s.steps, s.rules, s.words
                            );
                        }
                    }
                    Blk::Col { .. } => colb += 1,
                }
            }
            println!("SPIRAL_BLOCKS {spiral_blocks}");
            println!(
                "CENSUS blocks={} run_route={} column_route={} master_open={} cand_open={} full_run={} full_master={} full_cand={}",
                all.len(), runb, colb, ra, ca, full, fra, fca
            );
            for (f, b) in &six {
                println!("SIX {f} master={} cand={}", ref_admits(b), cand_opens(b));
            }
        }
        "parity" => {
            let (mut kc, mut kr) = (Cand::default(), Refk::default());
            let (mut oc, mut or) = (vec![], vec![]);
            let mut checked = 0;
            for (_, b) in six.iter().map(|(f, b)| (f.clone(), b)).chain(all.iter().map(|b| (label(b), b))) {
                let want = match b {
                    Blk::Run { runs, p, .. } => plain(&column(runs), *p),
                    Blk::Col { ll, p, .. } => plain(ll, *p),
                };
                dec(&mut kc, b, &mut oc);
                dec(&mut kr, b, &mut or);
                assert!(oc == want, "{}: candidate not byte-exact", label(b));
                assert!(or == want, "{}: master not byte-exact", label(b));
                // a fresh kernel too (no state from an earlier block)
                let mut fresh = Cand::default();
                let mut of = vec![];
                dec(&mut fresh, b, &mut of);
                assert!(of == want, "{}: fresh candidate not byte-exact", label(b));
                checked += 1;
            }
            println!("PARITY ok blocks={checked}");
        }
        "heap" => {
            // per worker: its share of the file's blocks (round robin, as the decoder deals
            // them), the kernel's peak heap above what it held before (its out buffer and the
            // inputs excluded)
            for workers in [1usize, 4, 10] {
                for arm in ["master", "cand"] {
                    let all = &all;
                    let peaks: Vec<isize> = std::thread::scope(|s| {
                        let hs: Vec<_> = (0..workers)
                            .map(|w| {
                                s.spawn(move || {
                                    let mut out = Vec::with_capacity(900_064);
                                    let base = live();
                                    peak_reset();
                                    if arm == "master" {
                                        let mut k = Refk::default();
                                        for b in all.iter().skip(w).step_by(workers) {
                                            dec(&mut k, b, &mut out);
                                        }
                                        peak_over(base)
                                    } else {
                                        let mut k = Cand::default();
                                        for b in all.iter().skip(w).step_by(workers) {
                                            dec(&mut k, b, &mut out);
                                        }
                                        peak_over(base)
                                    }
                                })
                            })
                            .collect();
                        hs.into_iter().map(|h| h.join().unwrap()).collect()
                    });
                    let max = peaks.iter().max().unwrap();
                    let sum: isize = peaks.iter().sum();
                    println!(
                        "HEAP W={workers} {arm} max_KiB={:.1} sum_KiB={:.1} per={:?}",
                        *max as f64 / 1024.0,
                        sum as f64 / 1024.0,
                        peaks.iter().map(|p| p / 1024).collect::<Vec<_>>()
                    );
                }
            }
            // per block, a fresh kernel's peak (each arm)
            for b in all.iter().filter(|b| matches!(b, Blk::Run { n, .. } if *n >= 65_536)) {
                let mut out = Vec::with_capacity(900_064);
                let base = live();
                peak_reset();
                let mut k = Refk::default();
                dec(&mut k, b, &mut out);
                let pm = peak_over(base);
                drop(k);
                let base = live();
                peak_reset();
                let mut k = Cand::default();
                dec(&mut k, b, &mut out);
                let pc = peak_over(base);
                if !ref_admits(b) || std::env::var("ADMIT_ALL").is_ok() {
                    println!("BHEAP {} master_KiB={} cand_KiB={}", label(b), pm / 1024, pc / 1024);
                }
            }
        }
        "t1" => {
            // the blocks to time: SET=six | rejected | admitted (full blocks master admits) |
            // dense (the cycle's picks 40, 41, 55, 56, 164, 165, 269, 270, 278)
            let set = std::env::var("ADMIT_SET").unwrap_or_else(|_| "six".into());
            let picks: Vec<(String, &Blk)> = match set.as_str() {
                "six" => six.iter().map(|(f, b)| (f.clone(), b)).collect(),
                "rejected" => all
                    .iter()
                    .filter(|b| matches!(b, Blk::Run { n, .. } if *n >= 65_536) && !ref_admits(b))
                    .map(|b| (label(b), b))
                    .collect(),
                "dense" => all
                    .iter()
                    .filter(|b| match b {
                        Blk::Run { idx, .. } => [40, 41, 55, 56, 164, 165, 269, 270, 278].contains(idx),
                        _ => false,
                    })
                    .map(|b| (label(b), b))
                    .collect(),
                "tiny" => vec![(
                    "tiny-all".into(),
                    all.iter().find(|b| matches!(b, Blk::Run { n, .. } if *n < 65_536)).unwrap(),
                )],
                _ => panic!("ADMIT_SET"),
            };
            let mut k = Cand::default();
            let mut kr = Refk::default();
            let mut out = Vec::with_capacity(900_064);
            for (_, b) in &picks {
                dec(&mut k, b, &mut out);
                dec(&mut kr, b, &mut out);
            }
            for (name, b) in &picks {
                let (r, c) = pair(&mut kr, &mut k, b, &mut out, reps);
                println!("T1 {name} {r:.1} {c:.1}");
            }
            if set == "tiny" {
                // every small run-route block, as one pass each, alternated
                let tiny: Vec<&Blk> = all.iter().filter(|b| matches!(b, Blk::Run { n, .. } if *n < 65_536)).collect();
                let (mut tr, mut tc) = (vec![], vec![]);
                for i in 0..2 * reps {
                    let t = Instant::now();
                    if (i + i / 2) % 2 == 0 {
                        for b in &tiny {
                            dec(&mut kr, b, &mut out);
                        }
                        tr.push(t.elapsed().as_secs_f64() * 1e6);
                    } else {
                        for b in &tiny {
                            dec(&mut k, b, &mut out);
                        }
                        tc.push(t.elapsed().as_secs_f64() * 1e6);
                    }
                }
                println!("T1 tinypass {:.1} {:.1}", median(&mut tr), median(&mut tc));
            }
        }
        "t4" => {
            // four concurrent workers, each over every block of the file (the file's own mix of
            // admitted and refused blocks), from a different start; the arms alternate by pass
            let workers: usize = std::env::var("ADMIT_WORKERS").ok().map_or(4, |v| v.parse().unwrap());
            let barrier = std::sync::Barrier::new(workers);
            let all = &all;
            let refused: Vec<usize> = (0..all.len())
                .filter(|&i| matches!(all[i], Blk::Run { n, .. } if n >= 65_536) && !ref_admits(&all[i]))
                .collect();
            let refused = &refused;
            let res: Vec<[Vec<f64>; 4]> = std::thread::scope(|s| {
                let hs: Vec<_> = (0..workers)
                    .map(|w| {
                        let barrier = &barrier;
                        s.spawn(move || {
                            let mut k = Cand::default();
                            let mut kr = Refk::default();
                            let mut out = Vec::with_capacity(900_064);
                            // [pass master, pass cand, refused-block master, refused-block cand]
                            let mut per: [Vec<f64>; 4] = Default::default();
                            for pass in 0..2 * reps + 2 {
                                let which = (pass + pass / 2) % 2;
                                barrier.wait();
                                let t0 = Instant::now();
                                let mut tref = 0.0;
                                for i in 0..all.len() {
                                    let i = (i + w * all.len() / workers) % all.len();
                                    let t = Instant::now();
                                    if which == 0 {
                                        dec(&mut kr, &all[i], &mut out);
                                    } else {
                                        dec(&mut k, &all[i], &mut out);
                                    }
                                    if refused.contains(&i) {
                                        tref += t.elapsed().as_secs_f64() * 1e6;
                                    }
                                }
                                if pass >= 2 {
                                    per[which].push(t0.elapsed().as_secs_f64() * 1e6);
                                    per[2 + which].push(tref);
                                }
                            }
                            per
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for (name, j) in [("pass", 0), ("refused", 2)] {
                let mut a: Vec<f64> = res.iter().flat_map(|r| r[j].clone()).collect();
                let mut c: Vec<f64> = res.iter().flat_map(|r| r[j + 1].clone()).collect();
                println!("T4 {name} {:.1} {:.1}", median(&mut a), median(&mut c));
            }
        }
        "emit" => {
            // the walk alone, serial (the relay off) against the relay, on one kernel's
            // tables for each of the six, alternated: 10th percentiles
            for (name, b) in &six {
                let Blk::Run { n, p, runs, .. } = b else { continue };
                let mut k = Kernel::default();
                let mut out = Vec::with_capacity(900_064);
                let (c, mark, bits) = k.construct::<true, true>(runs.records(), *n as u32, *p, LIMIT, &mut out);
                let m = c.retained;
                k.walk(*n, m, mark, bits, &mut out);
                let want = out.clone();
                let (mut ts, mut tr) = (vec![], vec![]);
                for i in 0..4 * reps {
                    let off = (i + i / 2) % 2 == 0;
                    super::relay::RELAY_OFF.with(|r| r.set(off));
                    let t = Instant::now();
                    emit(&k.edges[..m], &k.extra, &k.spelling, &k.offsets, &k.inline16, &k.lens, *n, mark, bits, &mut out);
                    let dt = t.elapsed().as_secs_f64() * 1e6;
                    assert!(out == want);
                    if off { ts.push(dt) } else { tr.push(dt) }
                }
                super::relay::RELAY_OFF.with(|r| r.set(false));
                ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
                tr.sort_by(|a, b| a.partial_cmp(b).unwrap());
                println!("T1 {name} {:.1} {:.1}", ts[ts.len() / 10], tr[tr.len() / 10]);
            }
        }
        "t4six" => {
            // #692's t4: four concurrent workers, each over the six from a different block,
            // the same kernel in every worker in a pass, alternated by pass
            let workers = 4;
            let barrier = std::sync::Barrier::new(workers);
            let bs = &six;
            let res: Vec<Vec<[Vec<f64>; 2]>> = std::thread::scope(|s| {
                let hs: Vec<_> = (0..workers)
                    .map(|w| {
                        let barrier = &barrier;
                        s.spawn(move || {
                            let mut k = Cand::default();
                            let mut kr = Refk::default();
                            let mut out = Vec::with_capacity(900_064);
                            let mut per = vec![[vec![], vec![]]; bs.len() + 1];
                            for pass in 0..2 * reps + 4 {
                                let which = (pass + pass / 2) % 2;
                                barrier.wait();
                                let t0 = Instant::now();
                                for i in 0..bs.len() {
                                    let i = (i + w) % bs.len();
                                    let t = Instant::now();
                                    if which == 0 {
                                        dec(&mut kr, &bs[i].1, &mut out);
                                    } else {
                                        dec(&mut k, &bs[i].1, &mut out);
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
        _ => panic!("ADMIT_MODE"),
    }
}

fn forced_counts(runs: &Runs, n: usize, p: u32) -> Counts {
    let mut k = Kernel::default();
    let mut out = vec![];
    k.run::<true, true>(runs.records(), n, p, LIMIT, &mut out)
}
