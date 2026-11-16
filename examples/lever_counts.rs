//! scratch (memory-lever): a file's blocks decoded on one thread with one Cells kernel, for
//! callgrind, and the kernel's route counts and lazy pool.
//! lever_counts FILE.bz2 cells|lazy|lazy-buf|nt
use mibz2::bwt::{Cells, InverseBwt, LazyBufCells, LazyCells, StagedCells};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, Kernels, Scratch};

fn run<I: InverseBwt + Default>(data: &[u8], mut after: impl FnMut(&Scratch<I>)) -> Scratch<I> {
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(data));
    let mut s = Scratch::<I>::default();
    let (mut out, mut syms) = (Vec::new(), Vec::new());
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        out.clear();
        decode_block(&b, k, &mut s, &mut out).unwrap();
        after(&s);
        syms = b.syms;
    }
    s
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    match a[2].as_str() {
        "cells" => println!("{:?}", run::<Cells>(&data, |_| ()).cells_counts()),
        "nt" => println!("{:?}", run::<StagedCells>(&data, |_| ()).cells_counts()),
        "lazy-buf" => println!("{:?}", run::<LazyBufCells>(&data, |_| ()).cells_counts()),
        "lazy" => {
            let (mut shares, mut admitted) = (Vec::new(), 0);
            let s = run::<LazyCells>(&data, |s| {
                let c = s.cells_counts();
                if c.admitted > admitted {
                    admitted = c.admitted;
                    let (slots, letters) = s.lazy_pool();
                    shares.push(slots as f64 / letters as f64);
                }
            });
            shares.sort_by(|a, b| a.partial_cmp(b).unwrap());
            println!("{:?}", s.cells_counts());
            println!(
                "pool's peak over letters, min median max: {:.3} {:.3} {:.3}",
                shares[0],
                shares[shares.len() / 2],
                shares[shares.len() - 1]
            );
        }
        m => panic!("the kernel is cells, lazy, lazy-buf or nt, not {m}"),
    }
}
