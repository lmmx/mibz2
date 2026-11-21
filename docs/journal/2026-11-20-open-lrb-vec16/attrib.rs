//! (temporary, not for master) Per-stage time of a decode on one thread, block by block, with
//! the inverse by each of Cells (the eager kernel, production below 8 workers and the serial
//! reader), LazyCells and OpenCells (production's lazy route from 8 workers).
//!
//! attrib FILE.bz2 [PASSES]
//!
//! (research/open-lrb-vec16) One more column, `fused`: `OpenCells::invert_symbols`, the open
//! kernel with its first pass fused into the front (LRB-Vec16), timed from the symbols, so
//! it is the front and the inverse together; on a block it leaves to the records' path, that
//! path's front and `invert_runs` are timed with it (the route's whole cost).
use std::time::Instant;

use mibz2::bwt::{Cells, InverseBwt, LazyCells, OpenCells};
use mibz2::runs::{self, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, mtf, rle, Kernels};

#[derive(Clone, Default)]
struct Blk {
    n: usize,
    recs: usize,
    runs_route: bool,
    admitted: bool,
    scan: f64,
    front: f64,
    cells: f64,
    lazy: f64,
    open: f64,
    fused: f64,
    rle: f64,
    crc: f64,
}

fn t<F: FnMut()>(f: &mut F) -> f64 {
    let s = Instant::now();
    f();
    s.elapsed().as_secs_f64()
}

fn pass(data: &[u8], k: Kernels, acc: &mut Vec<Blk>) {
    let mut sc = Scanner::new(SliceSource(data));
    let mut syms = Vec::new();
    let mut r = Runs::default();
    let (mut c, mut l, mut o) = (Cells::default(), LazyCells::default(), OpenCells::default());
    let mut of = OpenCells::default();
    let mut rf = Runs::default();
    let (mut inv, mut inv2, mut out) = (Vec::new(), Vec::new(), Vec::new());
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let mut i = 0;
    loop {
        let mut ev = None;
        let scan = t(&mut || ev = Some(sc.next(std::mem::take(&mut syms)).unwrap()));
        let Some(Event::Block { symbols: b, .. }) = ev else {
            break;
        };
        if acc.len() <= i {
            acc.push(Blk {
                scan: f64::INFINITY,
                front: f64::INFINITY,
                cells: f64::INFINITY,
                lazy: f64::INFINITY,
                open: f64::INFINITY,
                fused: f64::INFINITY,
                rle: f64::INFINITY,
                crc: f64::INFINITY,
                ..Default::default()
            });
        }
        let e = &mut acc[i];
        e.scan = e.scan.min(scan);
        let p = b.orig_ptr;
        if runs::takes_runs(&b) {
            e.runs_route = true;
            let mut n = 0;
            e.front = e
                .front
                .min(t(&mut || n = runs::front(&b, &mut r, k).unwrap()));
            e.n = n;
            e.recs = r.records().len();
            e.admitted = e.recs * 20 < n * 3 && e.recs * 64 >= n;
            e.cells = e
                .cells
                .min(t(&mut || assert!(c.invert_runs(&r, n, p, &mut inv, k))));
            e.lazy = e
                .lazy
                .min(t(&mut || assert!(l.invert_runs(&r, n, p, &mut inv2, k))));
            assert_eq!(inv, inv2);
            e.open = e
                .open
                .min(t(&mut || assert!(o.invert_runs(&r, n, p, &mut inv2, k))));
            assert_eq!(inv, inv2);
            inv2.clear();
            e.fused = e.fused.min(t(&mut || {
                if of.invert_symbols(&b, &mut rf, k, &mut inv2).is_none() {
                    let n = runs::front(&b, &mut rf, k).unwrap();
                    assert!(of.invert_runs(&rf, n, p, &mut inv2, k));
                }
            }));
            assert_eq!(inv, inv2);
        } else {
            let mut n = 0;
            e.front = e.front.min(t(&mut || {
                n = mtf::mtf(&b, &mut ll, &mut counts, k).unwrap()
            }));
            e.n = n;
            fn col<I: InverseBwt>(
                x: &mut I,
                ll: &[u8],
                cn: &[u32; 256],
                p: u32,
                out: &mut Vec<u8>,
            ) -> f64 {
                let mut buf = ll.to_vec();
                let s = Instant::now();
                if x.invert_in_place(&mut buf, cn, p) {
                    let d = s.elapsed().as_secs_f64();
                    *out = buf;
                    d
                } else {
                    x.invert(ll, cn, p, out);
                    s.elapsed().as_secs_f64()
                }
            }
            e.cells = e.cells.min(col(&mut c, &ll, &counts, p, &mut inv));
            e.lazy = e.lazy.min(col(&mut l, &ll, &counts, p, &mut inv2));
            assert_eq!(inv, inv2);
            e.open = e.open.min(col(&mut o, &ll, &counts, p, &mut inv2));
            assert_eq!(inv, inv2);
            e.fused = e.open + e.front;
        }
        e.rle = e.rle.min(t(&mut || {
            out.clear();
            rle::unrle(&inv, &mut out, k).unwrap()
        }));
        let mut got = 0;
        e.crc = e.crc.min(t(&mut || got = crc::crc32(&out, k)));
        assert_eq!(got, b.crc);
        syms = b.syms;
        i += 1;
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let passes: usize = a.get(2).map_or(5, |s| s.parse().unwrap());
    let k = Kernels::detect();
    let mut acc = Vec::new();
    for _ in 0..passes {
        pass(&data, k, &mut acc);
    }
    let ms = |f: &dyn Fn(&Blk) -> f64, filt: &dyn Fn(&Blk) -> bool| -> f64 {
        acc.iter().filter(|b| filt(b)).map(f).sum::<f64>() * 1e3
    };
    type G<'a> = (&'a str, &'a dyn Fn(&Blk) -> bool);
    let groups: [G; 4] = [
        ("all", &|_| true),
        ("admitted", &|b| b.admitted),
        ("refused (runs)", &|b| b.runs_route && !b.admitted),
        ("column route", &|b| !b.runs_route),
    ];
    println!(
        "# blocks {}, passes {passes}, best per block per stage, ms",
        acc.len()
    );
    println!("group\tblocks\trows_M\tscan\tfront\tcells\tlazy\topen\tfused\trle1\tcrc");
    for (name, f) in groups {
        let cnt = acc.iter().filter(|b| f(b)).count();
        let rows = acc.iter().filter(|b| f(b)).map(|b| b.n).sum::<usize>() as f64 / 1e6;
        println!(
            "{name}\t{cnt}\t{rows:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}",
            ms(&|b| b.scan, f),
            ms(&|b| b.front, f),
            ms(&|b| b.cells, f),
            ms(&|b| b.lazy, f),
            ms(&|b| b.open, f),
            ms(&|b| b.fused, f),
            ms(&|b| b.rle, f),
            ms(&|b| b.crc, f)
        );
    }
    if a.get(3).is_some() {
        for (i, b) in acc.iter().enumerate() {
            println!(
                "blk\t{i}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                b.n,
                b.recs,
                b.runs_route as u8,
                b.admitted as u8,
                b.scan * 1e3,
                b.front * 1e3,
                b.cells * 1e3,
                b.lazy * 1e3,
                b.open * 1e3,
                b.fused * 1e3,
                b.rle * 1e3,
                b.crc * 1e3
            );
        }
    }
}
