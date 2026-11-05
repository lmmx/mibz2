//! Scratch driver (not for commit): what one segment costs in master's room walk.
//!   segcost sizes FILE
//!   segcost cg FILE BLOCK SEGS WARM      callgrind target: `measured` walks BLOCK at SEGS
//!                                        after walking the WARM blocks before it at SEGS
//!   segcost dump FILE OUTDIR BLOCK...    T-vectors in pw.c's format
//!   segcost lean FILE BLOCK BITS WARM    the lean prototype (lean.rs), `measured_lean`
use mibz2::bwt::{build, walk, walk_segments_room_with_prefetch, SegScratch, TVector, PF_L1};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
use std::io::Write;

mod lean;

fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
    rows.clear();
    let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
    rows.extend((0..m).map(|_| {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        (x % n as u64) as u32
    }));
}

fn ns_of(n: usize, segs: usize, r0: u32) -> usize {
    let mut s = Vec::new();
    random_rows(n, segs - 1, &mut s);
    s.push(r0);
    s.sort_unstable();
    s.dedup();
    s.len()
}

/// build_two_pass's table, as mibz2 builds it (checked by walking it against mibz2's walk)
fn own_build(ll: &[u8], counts: &[u32; 256]) -> Vec<u32> {
    let n = ll.len();
    let mut t: Vec<u32> = ll.iter().map(|&c| c as u32).collect();
    let mut ftab = [0u32; 256];
    let mut cum = 0;
    for i in 0..256 {
        ftab[i] = cum;
        cum += counts[i];
    }
    for i in 0..n {
        let c = ll[i] as usize;
        let s = ftab[c] as usize;
        t[s] = (t[s] & 0xff) | (i as u32) << 8;
        ftab[c] += 1;
    }
    t
}

fn own_walk(t: &[u32], orig: u32, out: &mut Vec<u8>) {
    out.clear();
    let mut p = t[orig as usize];
    for _ in 0..t.len() {
        p = t[(p >> 8) as usize];
        out.push(p as u8);
    }
}

struct Blocks {
    ll: Vec<Vec<u8>>,
    counts: Vec<[u32; 256]>,
    orig: Vec<u32>,
}

fn load(path: &str, upto: usize) -> Blocks {
    let data = std::fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut b = Blocks { ll: vec![], counts: vec![], orig: vec![] };
    while let Event::Block { symbols, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        mtf::mtf(&symbols, &mut ll, &mut counts, k).unwrap();
        b.ll.push(ll);
        b.counts.push(counts);
        b.orig.push(symbols.orig_ptr);
        syms = symbols.syms;
        if b.ll.len() > upto {
            break;
        }
    }
    b
}

#[inline(never)]
fn measured(tt: &mut TVector, orig: u32, out: &mut Vec<u8>, s: &mut SegScratch, segs: usize) {
    walk_segments_room_with_prefetch::<32, PF_L1>(tt, orig, out, s, segs);
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a[1].as_str() {
        "sizes" => {
            let b = load(&a[2], usize::MAX);
            let segl = [512usize, 2048, 16384, 131072];
            let mut tot = [0usize; 4];
            let (mut ntot, mut big, mut nbig) = (0usize, 0usize, 0usize);
            for i in 0..b.ll.len() {
                let n = b.ll[i].len();
                let t = own_build(&b.ll[i], &b.counts[i]);
                let r0 = t[b.orig[i] as usize] >> 8;
                let v: Vec<usize> = segl.iter().map(|&s| ns_of(n, s, r0)).collect();
                for j in 0..4 {
                    tot[j] += v[j];
                }
                ntot += n;
                if n >= 1 << 16 {
                    big += 1;
                    nbig += n;
                }
                println!("block {i} n {n} ns {:?}", v);
            }
            println!(
                "blocks {} n_total {ntot} blocks>=64Ki {big} rows in them {nbig} ns_total 512:{} 2048:{} 16384:{} 131072:{}",
                b.ll.len(),
                tot[0],
                tot[1],
                tot[2],
                tot[3]
            );
        }
        "cg" => {
            let blk: usize = a[3].parse().unwrap();
            let segs: usize = a[4].parse().unwrap();
            let warm: usize = a[5].parse().unwrap();
            let b = load(&a[2], blk);
            let (mut tt, mut s, mut out, mut plain) =
                (TVector::new(), SegScratch::default(), Vec::new(), Vec::new());
            for i in blk.saturating_sub(warm)..blk {
                build(&b.ll[i], &b.counts[i], &mut tt);
                walk_segments_room_with_prefetch::<32, PF_L1>(&mut tt, b.orig[i], &mut out, &mut s, segs);
            }
            build(&b.ll[blk], &b.counts[blk], &mut tt);
            measured(&mut tt, b.orig[blk], &mut out, &mut s, segs);
            walk(&tt, b.orig[blk], &mut plain);
            assert!(out == plain);
            let r0 = own_build(&b.ll[blk], &b.counts[blk])[b.orig[blk] as usize] >> 8;
            println!(
                "block {blk} n {} segs {segs} ns {} warm {warm}: ok",
                plain.len(),
                ns_of(plain.len(), segs, r0)
            );
        }
        "dump" => {
            let want: Vec<usize> = a[4..].iter().map(|s| s.parse().unwrap()).collect();
            let b = load(&a[2], *want.iter().max().unwrap());
            let (mut tt, mut p1, mut p2) = (TVector::new(), Vec::new(), Vec::new());
            for &i in &want {
                let t = own_build(&b.ll[i], &b.counts[i]);
                build(&b.ll[i], &b.counts[i], &mut tt);
                walk(&tt, b.orig[i], &mut p1);
                own_walk(&t, b.orig[i], &mut p2);
                assert!(p1 == p2);
                let mut f = std::fs::File::create(format!("{}/tt-b{i}.bin", a[3])).unwrap();
                f.write_all(&(t.len() as u32).to_le_bytes()).unwrap();
                f.write_all(&b.orig[i].to_le_bytes()).unwrap();
                for &e in &t {
                    f.write_all(&e.to_le_bytes()).unwrap();
                }
                println!("block {i}: n {} orig {}", t.len(), b.orig[i]);
            }
        }
        "lean" => {
            let blk: usize = a[3].parse().unwrap();
            let bits: u32 = a[4].parse().unwrap();
            let warm: usize = a[5].parse().unwrap();
            let b = load(&a[2], blk);
            let (mut s, mut out, mut plain) = (lean::Lean::default(), Vec::new(), Vec::new());
            for i in blk.saturating_sub(warm)..blk {
                let t = own_build(&b.ll[i], &b.counts[i]);
                lean::walk_lean(&t, b.orig[i], bits, &mut s, &mut out);
                own_walk(&t, b.orig[i], &mut plain);
                assert!(out == plain, "warm block {i}");
            }
            let t = own_build(&b.ll[blk], &b.counts[blk]);
            lean::measured_lean(&t, b.orig[blk], bits, &mut s, &mut out);
            own_walk(&t, b.orig[blk], &mut plain);
            assert!(out == plain, "block {blk}");
            println!("lean block {blk} n {} bits {bits} segs {} warm {warm}: ok", plain.len(), s.segs());
        }
        _ => panic!("mode"),
    }
}
