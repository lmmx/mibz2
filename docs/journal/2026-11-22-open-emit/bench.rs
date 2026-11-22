use super::*;
use crate::bwt::{InverseBwt, Plain};
use crate::stream::{Event, Scanner, SliceSource};
use std::{hint::black_box, sync::Barrier, time::Instant};
pub(super) enum Blk {
    Run {
        idx: usize,
        n: usize,
        p: u32,
        runs: Runs,
    },
    Col {
        idx: usize,
        ll: Vec<u8>,
        counts: [u32; 256],
        p: u32,
    },
}

fn load_bz2(path: &str) -> Vec<Blk> {
    let data = std::fs::read(path).unwrap();
    let mut sc = Scanner::new(SliceSource(&data));
    let k = Kernels::detect();
    let mut syms = Vec::new();
    let mut v = vec![];
    let mut idx = 0;
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
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
    assert_eq!(v.len(), 431);
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
            specs.push((
                file.clone(),
                v.trim_end_matches(',').parse::<u32>().unwrap(),
            ));
        }
    }
    specs
        .into_iter()
        .enumerate()
        .map(|(i, (f, p))| {
            let l = std::fs::read(format!("{dir}/{f}")).unwrap();
            let runs = Runs::of_column(&l);
            (
                f,
                Blk::Run {
                    idx: 10_000 + i,
                    n: l.len(),
                    p,
                    runs,
                },
            )
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

const DENSE: [usize; 5] = [41, 56, 165, 270, 278];

fn ns(start: Instant) -> f64 {
    start.elapsed().as_nanos() as f64
}

fn pass<I: InverseBwt>(
    k: &mut I,
    blocks: &[&Blk],
    offset: usize,
    out: &mut Vec<u8>,
    per: &mut [f64],
) -> f64 {
    let start = Instant::now();
    for i in 0..blocks.len() {
        let j = (i + offset) % blocks.len();
        let t = Instant::now();
        dec(k, blocks[j], out);
        black_box(&*out);
        per[j] = ns(t);
    }
    ns(start)
}

fn pairs<K: InverseBwt + Default>(tag: &str, bs: &[&Blk], reverse: bool, set: &str) {
    let (mut a, mut b) = (OpenCells::default(), K::default());
    let mut out = Vec::with_capacity(900_064);
    let mut total = [0.; 2];
    for block in bs {
        dec(&mut a, block, &mut out);
        dec(&mut b, block, &mut out);
        let mut times = [vec![], vec![]];
        let reps = if set == "wiki" { 2 } else { 12 };
        for i in 0..2 * reps {
            let arm = (i + i / 2) % 2 ^ usize::from(reverse);
            let t = Instant::now();
            if arm == 0 {
                dec(&mut a, block, &mut out);
            } else {
                dec(&mut b, block, &mut out);
            }
            black_box(&out);
            times[arm].push(ns(t));
        }
        let pair = times.map(|mut v| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 10]
        });
        println!(
            "ROW {tag} W1 {set} {} {} {}",
            label(block),
            pair[0],
            pair[1]
        );
        for arm in 0..2 {
            total[arm] += pair[arm];
        }
    }
    println!("ROW {tag} W1 {set} pass {} {}", total[0], total[1]);
}

fn passes<K: InverseBwt + Default>(
    tag: &str,
    bs: &[&Blk],
    workers: usize,
    reverse: bool,
    set: &str,
) {
    let barrier = Barrier::new(workers);
    let reps = if set == "wiki" { 1 } else { 8 };
    let rows = std::thread::scope(|s| {
        (0..workers)
            .map(|w| {
                let barrier = &barrier;
                s.spawn(move || {
                    let (mut a, mut b) = (OpenCells::default(), K::default());
                    let mut out = Vec::with_capacity(900_064);
                    let mut per = vec![0.; bs.len()];
                    let offset = if set == "wiki" {
                        w * bs.len() / workers
                    } else {
                        w
                    };
                    // Warm both worker-local kernels. Start each timed pass together.
                    for i in 0..2 {
                        barrier.wait();
                        if (i == 0) ^ reverse {
                            pass(&mut a, bs, offset, &mut out, &mut per);
                        } else {
                            pass(&mut b, bs, offset, &mut out, &mut per);
                        }
                    }
                    let mut rows = vec![[vec![], vec![]]; bs.len() + 1];
                    for i in 0..2 * reps {
                        let arm = ((i + i / 2) % 2) ^ usize::from(reverse);
                        barrier.wait();
                        let total = if arm == 0 {
                            pass(&mut a, bs, offset, &mut out, &mut per)
                        } else {
                            pass(&mut b, bs, offset, &mut out, &mut per)
                        };
                        for (r, dt) in rows.iter_mut().zip(&per) {
                            r[arm].push(*dt);
                        }
                        rows[bs.len()][arm].push(total);
                    }
                    rows
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    let slowest = [0, 1].map(|arm| {
        let mut times: Vec<_> = (0..reps)
            .map(|i| {
                rows.iter()
                    .map(|r| r[bs.len()][arm][i])
                    .max_by(f64::total_cmp)
                    .unwrap()
            })
            .collect();
        times.sort_by(f64::total_cmp);
        times[times.len() / 2]
    });
    println!(
        "ROW {tag} W{workers} {set} slowest {} {}",
        slowest[0], slowest[1]
    );
    for j in 0..=bs.len() {
        let name = if j == bs.len() {
            "pass".into()
        } else {
            label(bs[j])
        };
        let mut sample = [0.; 2];
        for arm in 0..2 {
            let mut t: Vec<_> = rows
                .iter()
                .flat_map(|r| r[j][arm].iter().copied())
                .collect();
            t.sort_by(f64::total_cmp);
            sample[arm] = t[if workers == 1 {
                t.len() / 10
            } else {
                t.len() / 2
            }];
        }
        println!(
            "ROW {tag} W{workers} {set} {name} {} {}",
            sample[0], sample[1]
        );
    }
}

fn label(b: &Blk) -> String {
    match b {
        Blk::Run { idx, .. } | Blk::Col { idx, .. } => idx.to_string(),
    }
}

pub(super) trait Stage: Default {
    fn prepare(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Option<(usize, u32, u32, Option<u32>)>;
    fn emit_only(&self, meta: (usize, u32, u32, Option<u32>), n: usize, out: &mut Vec<u8>);
}

fn stages<K: Stage>(tag: &str, bs: &[&Blk], reverse: bool, set: &str) {
    let (mut a, mut b) = (Kernel::default(), K::default());
    let (mut oa, mut ob) = (Vec::with_capacity(900_064), Vec::with_capacity(900_064));
    let mut total = [0.; 2];
    for block in bs {
        let Blk::Run { n, p, runs, .. } = block else {
            continue;
        };
        let ma = a.prepare(runs, *n, *p, &mut oa);
        let mb = b.prepare(runs, *n, *p, &mut ob);
        assert_eq!(ma.is_some(), mb.is_some());
        let (Some(ma), Some(mb)) = (ma, mb) else {
            continue;
        };
        a.emit_only(ma, *n, &mut oa);
        b.emit_only(mb, *n, &mut ob);
        assert!(oa == ob);
        let reps = if set == "wiki" { 1 } else { 8 };
        let mut samples = [vec![], vec![]];
        for i in 0..2 * reps {
            let arm = (i + i / 2) % 2 ^ usize::from(reverse);
            let t = Instant::now();
            if arm == 0 {
                a.emit_only(ma, *n, &mut oa);
                black_box(&oa);
            } else {
                b.emit_only(mb, *n, &mut ob);
                black_box(&ob);
            }
            samples[arm].push(ns(t));
        }
        let dt = samples.map(|mut t| {
            t.sort_by(f64::total_cmp);
            t[t.len() / 10]
        });
        println!("ROW {tag} emit {set} {} {} {}", label(block), dt[0], dt[1]);
        for arm in 0..2 {
            total[arm] += dt[arm];
        }
    }
    println!("ROW {tag} emit {set} pass {} {}", total[0], total[1]);
}

fn suite<K: InverseBwt + Default, S: Stage>(
    tag: &str,
    all: &[Blk],
    six: &[(String, Blk)],
    mode: &str,
    reverse: bool,
) {
    if mode == "parity" {
        let mut k = K::default();
        let mut out = vec![];
        for b in all.iter().chain(six.iter().map(|(_, b)| b)) {
            let want = match b {
                Blk::Run { runs, p, .. } => plain(&column(runs), *p),
                Blk::Col { ll, p, .. } => plain(ll, *p),
            };
            dec(&mut k, b, &mut out);
            assert!(out == want, "warm {}", label(b));
            dec(&mut K::default(), b, &mut out);
            assert!(out == want, "fresh {}", label(b));
        }
        println!("PARITY {tag} {} warm + fresh", all.len() + six.len());
        return;
    }
    let sets = [
        ("six", six.iter().map(|(_, b)| b).collect()),
        ("dense", DENSE.map(|i| &all[i]).to_vec()),
        ("wiki", all.iter().collect()),
    ];
    for (name, bs) in sets {
        if mode == "one" {
            stages::<S>(tag, &bs, reverse, name);
            pairs::<K>(tag, &bs, reverse, name);
        } else {
            passes::<K>(tag, &bs, 4, reverse, name);
        }
    }
}

#[test]
#[ignore]
fn measure() {
    let all = load_bz2(&std::env::var("ADMIT_BZ2").unwrap());
    let six = load_six();
    assert_eq!(six.len(), 6);
    let mode = std::env::var("MODE").unwrap();
    let round: usize = std::env::var("ROUND").unwrap().parse().unwrap();
    // Six permutations counterbalance arm position independently of pair order.
    for &id in &[
        [0, 1, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
        [0, 2, 1],
        [1, 0, 2],
    ][round % 6]
    {
        let rev = round % 2 != 0;
        match id {
            0 => suite::<control::OpenCells, control::Kernel>("AA", &all, &six, &mode, rev),
            1 => suite::<xor::OpenCells, xor::Kernel>("A", &all, &six, &mode, rev),
            _ => suite::<relay6::OpenCells, relay6::Kernel>("B", &all, &six, &mode, rev),
        }
    }
}

#[test]
#[ignore]
fn memory() {
    let bs = load_bz2(&std::env::var("ADMIT_BZ2").unwrap());
    for workers in [1, 4, 10] {
        let max = (0..workers)
            .map(|w| {
                let mut k = relay6::OpenCells::default();
                let mut out = Vec::with_capacity(900_064);
                for b in bs.iter().skip(w).step_by(workers) {
                    dec(&mut k, b, &mut out);
                }
                k.length_bytes()
            })
            .max()
            .unwrap();
        println!("MEM workers={workers} relay_length_heap={max}");
    }
    println!(
        "sizeof Kernel master={} xor={} relay={}",
        size_of::<Kernel>(),
        size_of::<xor::Kernel>(),
        size_of::<relay6::Kernel>()
    );
}
