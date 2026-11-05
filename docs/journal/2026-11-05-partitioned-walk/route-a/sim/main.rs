//! Scratch prototype and simulator for a radix-partitioned inverse-BWT walk (Route A).
//! Not for merge. No timing: counts, traces (callgrind) and simulation only.
//!
//! pwsim locality FILE...                 psi's region locality on real T-vectors
//! pwsim stats FILE K B CAP               the partitioned walk's queue dynamics (counters)
//! pwsim master FILE                      mibz2's room walk once (for callgrind)
//! pwsim part FILE K B CAP                the partitioned prototype once, checked (for callgrind)
//! pwsim lines FILE CAP...                cold-region model check: distinct lines per visit

use mibz2::bwt::{build, walk_segments_room_parts, SegScratch, TVector};

const START: u32 = 1 << 31;
const LINK: u32 = !START;
const SEGBITS: u32 = 28;
const SEGMASK: u32 = (1 << SEGBITS) - 1;

struct Block {
    n: usize,
    orig_ptr: u32,
    t: Vec<u32>,
}

fn load(path: &str) -> Block {
    let d = std::fs::read(path).unwrap();
    let n = u32::from_le_bytes(d[0..4].try_into().unwrap()) as usize;
    let orig_ptr = u32::from_le_bytes(d[4..8].try_into().unwrap());
    let t = d[8..]
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(t.len(), n);
    Block { n, orig_ptr, t }
}

fn plain(t: &[u32], orig_ptr: u32) -> Vec<u8> {
    let n = t.len();
    let mut out = Vec::with_capacity(n);
    let mut p = t[orig_ptr as usize];
    for _ in 0..n {
        p = t[(p >> 8) as usize];
        out.push(p as u8);
    }
    out
}

// ---------------------------------------------------------------- master, mibz2's own code
#[inline(never)]
fn master_walk(tt: &mut TVector, orig_ptr: u32, out: &mut Vec<u8>, s: &mut SegScratch) -> bool {
    walk_segments_room_parts(tt, orig_ptr, out, s)
}
#[inline(never)]
fn master_assemble(s: &SegScratch, out: &mut Vec<u8>) {
    s.assemble(out)
}

fn master(b: &Block) {
    let ll: Vec<u8> = b.t.iter().map(|&e| e as u8).collect();
    let mut counts = [0u32; 256];
    for &c in &ll {
        counts[c as usize] += 1;
    }
    let mut tt = TVector::new();
    build(&ll, &counts, &mut tt);
    let (mut s, mut out) = (SegScratch::default(), Vec::new());
    assert!(master_walk(&mut tt, b.orig_ptr, &mut out, &mut s));
    master_assemble(&s, &mut out);
    assert_eq!(out, plain(&b.t, b.orig_ptr), "master's output");
    println!("master: ok, n={}", b.n);
}

// ---------------------------------------------------------------- the partitioned walk
/// A lane in a region's queue: its next row, its segment (low 28 bits) and the count of
/// bytes pending in `acc` (top 4 bits), and up to 7 pending bytes (the newest at the top)
#[derive(Clone, Copy, Default)]
#[repr(C, align(16))]
struct Ent {
    row: u32,
    seg: u32,
    acc: u64,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Ovf {
    seg: u32,
    _pad: u32,
    acc: u64,
}

#[derive(Default, Debug, Clone)]
struct Stats {
    sweeps: u64,
    visits: u64,
    empty_visits: u64,
    tloads: u64,
    ends: u64,
    pushes: u64,
    chase: u64,
    takes_inline: u64,
    steps_full: u64,
    visit_q_sum: u64,
    visit_q_sq: u128,
    visit_lines_sum: u64,
    ovf: u64,
}

struct Part {
    k: u32,
    b: u32,
    cap: usize,
    starts: Vec<u32>,
    firsts: Vec<u32>,
    q: Vec<Vec<Ent>>,
    qlen: Vec<usize>,
    ovf: Vec<Ovf>,
    tail_acc: Vec<u64>,
    tail_cnt: Vec<u8>,
    next: Vec<u32>,
    st: Stats,
    stamp: Vec<u32>,
}

fn hash(j: u32) -> u32 {
    let mut x = j.wrapping_mul(0x9E37_79B9) ^ 0x5bd1_e995;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= x >> 12;
    x
}

impl Part {
    fn new(n: usize, k: u32, b: u32, cap: usize) -> Part {
        let s = (n + (1 << k) - 1) >> k;
        let p = (n + (1 << b) - 1) >> b;
        Part {
            k,
            b,
            cap,
            starts: Vec::with_capacity(s),
            firsts: vec![0; s],
            q: (0..p).map(|_| vec![Ent::default(); cap.min(s)]).collect(),
            qlen: vec![0; p],
            ovf: Vec::with_capacity(n / 8 + s + 8),
            tail_acc: vec![0; s],
            tail_cnt: vec![0; s],
            next: vec![u32::MAX; s],
            st: Stats::default(),
            stamp: vec![0; (n + 15) / 16],
        }
    }
}

/// The starts, one a 2^k-row interval at a hashed row, r0 in its own interval's place; each
/// flagged in its entry, its first entry kept (the segment's first byte and second row)
#[inline(never)]
fn pw_flag(t: &mut [u32], r0: u32, w: &mut Part) {
    let n = t.len();
    let k = w.k;
    let s = (n + (1 << k) - 1) >> k;
    w.starts.clear();
    for j in 0..s {
        let lo = j << k;
        let hi = n.min((j + 1) << k);
        let r = if (r0 as usize) >> k == j {
            r0
        } else {
            (lo + hash(j as u32) as usize % (hi - lo)) as u32
        };
        w.starts.push(r);
    }
    for j in 0..s {
        let r = w.starts[j] as usize;
        let x = t[r];
        w.firsts[j] = x;
        t[r] = x | START;
    }
}

#[inline(never)]
fn pw_unflag(t: &mut [u32], w: &Part) {
    for &r in &w.starts {
        t[r as usize] &= LINK;
    }
}

#[inline(always)]
fn fresh(w: &Part, j: usize) -> Ent {
    let x = w.firsts[j];
    Ent {
        row: x >> 8,
        seg: j as u32 | 1 << SEGBITS,
        acc: ((x & 0xff) as u64) << 56,
    }
}

/// The walk: regions served in order, every queued lane stepped while its rows stay in the
/// region, then appended to its next row's region's queue
#[inline(never)]
fn pw_walk<const ST: bool>(t: &[u32], w: &mut Part) {
    let n = t.len();
    let (k, b) = (w.k, w.b);
    let s = w.starts.len();
    let p = w.q.len();
    let tp = t.as_ptr();
    w.ovf.clear();
    let mut taken = 0usize;
    let mut active = 0usize;
    while taken < s && active < w.cap {
        let e = fresh(w, taken);
        let d = (e.row >> b) as usize;
        let l = w.qlen[d];
        w.q[d][l] = e;
        w.qlen[d] = l + 1;
        taken += 1;
        active += 1;
    }
    let mut gen = 0u32;
    while active > 0 {
        if ST {
            w.st.sweeps += 1;
        }
        for r in 0..p {
            let len = w.qlen[r];
            if ST {
                if len == 0 {
                    w.st.empty_visits += 1;
                } else {
                    w.st.visits += 1;
                }
            }
            if len == 0 {
                continue;
            }
            gen += 1;
            let mut vq = 0u64;
            let mut vlines = 0u64;
            // the queue being served gets no appends during its own visit (its own
            // region's rows are chased in place, a take into it is walked in place), so
            // it is read through a raw pointer while the other queues are appended to
            let qr = w.q[r].as_ptr();
            for i in 0..len {
                // SAFETY: i < len <= the queue's length
                let mut e = unsafe { *qr.add(i) };
                loop {
                    // SAFETY: every row is below n (links from the build, starts are rows)
                    let x = unsafe { *tp.add(e.row as usize) };
                    if ST {
                        w.st.tloads += 1;
                        vq += 1;
                        if active == w.cap {
                            w.st.steps_full += 1;
                        }
                        let line = (e.row >> 4) as usize;
                        if w.stamp[line] != gen {
                            w.stamp[line] = gen;
                            vlines += 1;
                        }
                    }
                    if x & START != 0 {
                        let seg = (e.seg & SEGMASK) as usize;
                        w.tail_acc[seg] = e.acc;
                        w.tail_cnt[seg] = (e.seg >> SEGBITS) as u8;
                        w.next[seg] = e.row >> k;
                        if ST {
                            w.st.ends += 1;
                        }
                        if taken < s {
                            e = fresh(w, taken);
                            taken += 1;
                            let d = (e.row >> b) as usize;
                            if d == r {
                                if ST {
                                    w.st.takes_inline += 1;
                                }
                                continue;
                            }
                            let l = w.qlen[d];
                            w.q[d][l] = e;
                            w.qlen[d] = l + 1;
                            if ST {
                                w.st.pushes += 1;
                            }
                        } else {
                            active -= 1;
                        }
                        break;
                    }
                    let cnt = (e.seg >> SEGBITS) + 1;
                    e.acc = (e.acc >> 8) | ((x as u64) << 56);
                    if cnt == 8 {
                        w.ovf.push(Ovf {
                            seg: e.seg & SEGMASK,
                            _pad: 0,
                            acc: e.acc,
                        });
                        e.seg &= SEGMASK;
                        if ST {
                            w.st.ovf += 1;
                        }
                    } else {
                        e.seg += 1 << SEGBITS;
                    }
                    e.row = x >> 8;
                    let d = (e.row >> b) as usize;
                    if d != r {
                        let l = w.qlen[d];
                        w.q[d][l] = e;
                        w.qlen[d] = l + 1;
                        if ST {
                            w.st.pushes += 1;
                        }
                        break;
                    }
                    if ST {
                        w.st.chase += 1;
                    }
                }
            }
            w.qlen[r] = 0;
            if ST {
                w.st.visit_q_sum += vq;
                w.st.visit_q_sq += (vq as u128) * (vq as u128);
                w.st.visit_lines_sum += vlines;
            }
        }
    }
    let _ = n;
}

/// `pw_walk` with its bookkeeping in locals and raw pointers: each queue's end pointer in a
/// stack array (a push is a load, a 16-byte store and a store), the overflow log a bump
/// pointer. The same order of service, so the same output.
#[inline(never)]
fn pw_walk2(t: &[u32], w: &mut Part) {
    const MAXP: usize = 64;
    let (k, b) = (w.k, w.b);
    let s = w.starts.len();
    let p = w.q.len();
    assert!(p <= MAXP);
    let tp = t.as_ptr();
    let mut base = [std::ptr::null_mut::<Ent>(); MAXP];
    let mut end = [std::ptr::null_mut::<Ent>(); MAXP];
    for d in 0..p {
        base[d] = w.q[d].as_mut_ptr();
        end[d] = base[d];
    }
    w.ovf.clear();
    let ovf0 = w.ovf.as_mut_ptr();
    let mut ovf = ovf0;
    let firsts = w.firsts.as_ptr();
    let (ta, tc, nx) = (
        w.tail_acc.as_mut_ptr(),
        w.tail_cnt.as_mut_ptr(),
        w.next.as_mut_ptr(),
    );
    let fresh = |j: usize| -> Ent {
        // SAFETY: j < s
        let x = unsafe { *firsts.add(j) };
        Ent {
            row: x >> 8,
            seg: j as u32 | 1 << SEGBITS,
            acc: ((x & 0xff) as u64) << 56,
        }
    };
    let mut taken = 0usize;
    let mut active = 0usize;
    while taken < s && active < w.cap {
        let e = fresh(taken);
        let d = (e.row >> b) as usize;
        unsafe {
            *end[d] = e;
            end[d] = end[d].add(1);
        }
        taken += 1;
        active += 1;
    }
    while active > 0 {
        for r in 0..p {
            let (mut i, stop) = (base[r], end[r]);
            if i == stop {
                continue;
            }
            // the served queue gets no appends during its own visit
            while i < stop {
                let mut e = unsafe { *i };
                i = unsafe { i.add(1) };
                loop {
                    // SAFETY: every row is below n
                    let x = unsafe { *tp.add(e.row as usize) };
                    if x & START != 0 {
                        let seg = (e.seg & SEGMASK) as usize;
                        unsafe {
                            *ta.add(seg) = e.acc;
                            *tc.add(seg) = (e.seg >> SEGBITS) as u8;
                            *nx.add(seg) = e.row >> k;
                        }
                        if taken < s {
                            e = fresh(taken);
                            taken += 1;
                            let d = (e.row >> b) as usize;
                            if d == r {
                                continue;
                            }
                            unsafe {
                                *end[d] = e;
                                end[d] = end[d].add(1);
                            }
                        } else {
                            active -= 1;
                        }
                        break;
                    }
                    let cnt = e.seg >> SEGBITS;
                    e.acc = (e.acc >> 8) | ((x as u64) << 56);
                    e.seg += 1 << SEGBITS;
                    if cnt == 7 {
                        unsafe {
                            *ovf = Ovf {
                                seg: e.seg & SEGMASK,
                                _pad: 0,
                                acc: e.acc,
                            };
                            ovf = ovf.add(1);
                        }
                        e.seg &= SEGMASK;
                    }
                    e.row = x >> 8;
                    let d = (e.row >> b) as usize;
                    if d != r {
                        unsafe {
                            *end[d] = e;
                            end[d] = end[d].add(1);
                        }
                        break;
                    }
                }
            }
            end[r] = base[r];
        }
    }
    // SAFETY: every slot below ovf was written
    unsafe { w.ovf.set_len(ovf.offset_from(ovf0) as usize) };
}

/// The segments joined from r0's: each one's 8-byte chunks (the overflow log, sorted by
/// segment, stably) then its tail, followed by its successor, until n bytes
#[inline(never)]
fn pw_assemble(n: usize, r0: u32, w: &Part, out: &mut Vec<u8>) {
    let s = w.starts.len();
    let mut first = vec![0u32; s + 1];
    for o in &w.ovf {
        first[o.seg as usize + 1] += 1;
    }
    for j in 0..s {
        first[j + 1] += first[j];
    }
    let mut cur = first.clone();
    let mut chunks = vec![0u64; w.ovf.len()];
    for o in &w.ovf {
        let c = &mut cur[o.seg as usize];
        chunks[*c as usize] = o.acc;
        *c += 1;
    }
    out.clear();
    out.reserve(n + 64);
    let mut seg = (r0 >> w.k) as usize;
    while out.len() < n {
        for c in &chunks[first[seg] as usize..first[seg + 1] as usize] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        let c = w.tail_cnt[seg] as usize;
        if c > 0 {
            let v = w.tail_acc[seg] >> (64 - 8 * c);
            out.extend_from_slice(&v.to_le_bytes()[..c]);
        }
        seg = w.next[seg] as usize;
    }
    out.truncate(n);
}

fn part_run<const ST: bool>(b: &mut Block, k: u32, rb: u32, cap: usize, v2: bool) -> (Part, Vec<u8>) {
    let r0 = b.t[b.orig_ptr as usize] >> 8;
    let mut w = Part::new(b.n, k, rb, cap);
    pw_flag(&mut b.t, r0, &mut w);
    if v2 {
        pw_walk2(&b.t, &mut w);
    } else {
        pw_walk::<ST>(&b.t, &mut w);
    }
    pw_unflag(&mut b.t, &w);
    let mut out = Vec::new();
    pw_assemble(b.n, r0, &w, &mut out);
    (w, out)
}

// ---------------------------------------------------------------- locality
fn locality(b: &Block) {
    let n = b.n;
    let mut p = b.t[b.orig_ptr as usize] >> 8;
    let bits: Vec<u32> = vec![20, 19, 18, 17, 16, 15, 14, 13, 12];
    let mut stay = vec![0u64; bits.len()];
    let mut same_line = 0u64;
    let mut near = 0u64; // within +-1024 rows
    for _ in 0..n {
        let q = b.t[p as usize] >> 8;
        for (i, &bb) in bits.iter().enumerate() {
            if q >> bb == p >> bb {
                stay[i] += 1;
            }
        }
        if q >> 4 == p >> 4 {
            same_line += 1;
        }
        if (q as i64 - p as i64).abs() < 1024 {
            near += 1;
        }
        p = q;
    }
    for (i, &bb) in bits.iter().enumerate() {
        let regions = (n + (1 << bb) - 1) >> bb;
        println!(
            "  region 2^{bb} rows ({} KB), P={regions}: stay {:.4} (uniform 1/P = {:.4})",
            (4usize << bb) >> 10,
            stay[i] as f64 / n as f64,
            1.0 / regions as f64
        );
    }
    println!(
        "  same 64-B line {:.5}, within 1024 rows {:.5}",
        same_line as f64 / n as f64,
        near as f64 / n as f64
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "locality" => {
            for f in &a[2..] {
                println!("{f}:");
                locality(&load(f));
            }
        }
        "master" => master(&load(&a[2])),
        "part" | "stats" | "part2" => {
            let mut b = load(&a[2]);
            let k: u32 = a[3].parse().unwrap();
            let rb: u32 = a[4].parse().unwrap();
            let cap: usize = a[5].parse().unwrap();
            let want = plain(&b.t, b.orig_ptr);
            let (w, out) = if a[1] == "stats" {
                part_run::<true>(&mut b, k, rb, cap, false)
            } else {
                part_run::<false>(&mut b, k, rb, cap, a[1] == "part2")
            };
            assert!(out == want, "partitioned output differs from the plain walk");
            let s = w.starts.len();
            let p = w.q.len();
            let n = b.n as f64;
            let lines = ((b.n + 15) / 16) as f64;
            print!(
                "ok n={} k={k} S={s} mean_seg={:.2} b={rb} P={p} region={}KB cap={cap}",
                b.n,
                n / s as f64,
                (4usize << rb) >> 10
            );
            if a[1] == "stats" {
                let st = &w.st;
                let qm = st.visit_q_sum as f64 / st.visits as f64;
                let qv = st.visit_q_sq as f64 / st.visits as f64 - qm * qm;
                println!();
                println!(
                    "  sweeps={} visits={} empty={} tloads/n={:.4} ends={} pushes/n={:.4} chase/n={:.4} \
                     takes_inline={} ovf/n={:.4} steps_at_full_cap={:.4}",
                    st.sweeps,
                    st.visits,
                    st.empty_visits,
                    st.tloads as f64 / n,
                    st.ends,
                    st.pushes as f64 / n,
                    st.chase as f64 / n,
                    st.takes_inline,
                    st.ovf as f64 / n,
                    st.steps_full as f64 / st.tloads as f64
                );
                let region_lines = lines / p as f64;
                println!(
                    "  Q/visit mean {:.1} sd {:.1}; region lines {:.0}; lambda=Q/lines {:.3}; \
                     distinct lines/visit {:.1} -> touched-line fills per step {:.4} (cold model)",
                    qm,
                    qv.max(0.0).sqrt(),
                    region_lines,
                    qm / region_lines,
                    st.visit_lines_sum as f64 / st.visits as f64,
                    st.visit_lines_sum as f64 / st.tloads as f64
                );
                println!(
                    "  steps per lane per sweep (tloads/sweeps/cap) {:.3}",
                    st.tloads as f64 / st.sweeps as f64 / cap.min(s) as f64
                );
            } else {
                println!();
            }
        }
        _ => panic!("usage"),
    }
}
