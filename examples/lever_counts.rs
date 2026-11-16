//! scratch (memory-lever): a file's blocks decoded on one thread with one Cells kernel, for
//! callgrind, and the kernel's route counts and lazy pool. lever_counts FILE.bz2 cells|lazy|lazy-hint|lazy-ring|nt
use mibz2::bwt::{Cells, CellsVariant, InverseBwt, StagedCells};
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

/// A lazy kernel's blocks: its route counts, its pool's peak over its letters, and the bytes
/// of words it kept, a letter (summed over the admitted blocks).
fn lazy<const V: u8>(data: &[u8]) {
    let (mut shares, mut admitted) = (Vec::new(), 0);
    let (mut letters, mut bytes, mut rules) = (0usize, 0usize, 0usize);
    let s = run::<CellsVariant<V>>(data, |s| {
        let c = s.cells_counts();
        if c.admitted > admitted {
            admitted = c.admitted;
            let (slots, n) = s.lazy_pool();
            shares.push(slots as f64 / n as f64);
            let (b, r) = s.lazy_words();
            (letters, bytes, rules) = (letters + n, bytes + b, rules + r);
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
    println!(
        "words kept, bytes a letter: as bytes {:.2}, as rules {:.2}, both {:.2} ({} letters)",
        bytes as f64 / letters as f64,
        rules as f64 / letters as f64,
        (bytes + rules) as f64 / letters as f64,
        letters
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    match a[2].as_str() {
        "cells" => println!("{:?}", run::<Cells>(&data, |_| ()).cells_counts()),
        "nt" => println!("{:?}", run::<StagedCells>(&data, |_| ()).cells_counts()),
        // (the variants' numbers: `cells_lazy.rs`)
        "lazy" => lazy::<1>(&data),
        "lazy-hint" => lazy::<3>(&data),
        "lazy-ring" => lazy::<4>(&data),
        m => panic!("the kernel is cells, lazy, lazy-hint, lazy-ring or nt, not {m}"),
    }
}
