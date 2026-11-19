#![allow(dead_code)]
pub use mibz2::{bwt, runs, Kernels};
fn heap<T>(v: &Vec<T>) -> usize {
    v.capacity() * std::mem::size_of::<T>()
}
mod combo;
fn check(l: &[u8], marks: &[usize], c: &mut combo::Kernel, counts: &mut [usize; 3]) {
    let n = l.len();
    let mut psi: Vec<_> = (0..n).collect();
    psi.sort_by_key(|&i| l[i]);
    let mut caps = Vec::<u16>::new();
    for &b in l {
        if let Some(v) = caps
            .last_mut()
            .filter(|v| (**v >> 8) as u8 == b && (**v & 255) < 127)
        {
            *v += 1;
        } else {
            caps.push((u16::from(b) << 8) | 1);
        }
    }
    let (mut ref_out, mut direct) = (Vec::new(), Vec::new());
    for &p in marks {
        let mut x = p;
        let raw: Vec<_> = (0..n)
            .map(|_| {
                x = psi[x];
                l[x]
            })
            .collect();
        for k in [Kernels::scalar(), Kernels::detect()] {
            ref_out.clear();
            let want = mibz2::rle::unrle(&raw, &mut ref_out, k);
            direct.clear();
            direct.push(73);
            let got = c.expanded(&caps, n, p as u32, &mut direct, k);
            assert_eq!(got, want);
            assert_eq!(direct[0], 73);
            if want.is_ok() {
                assert_eq!(&direct[1..], ref_out);
                counts[0] += 1;
            } else {
                counts[1] += 1;
            }
            counts[2] += usize::from(c.counts().4 != 0);
        }
    }
}
fn main() {
    let mut c = combo::Kernel::default();
    let mut total = [0; 3];
    for n in 1..=10 {
        let marks: Vec<_> = (0..n).collect();
        for mask in 0..1 << n {
            let l: Vec<_> = (0..n)
                .map(|j| if mask & (1 << j) == 0 { 0 } else { 255 })
                .collect();
            check(&l, &marks, &mut c, &mut total);
        }
    }
    for n in [37, 47, 2016, 8194, 16385, 21850 * 17] {
        let marks: Vec<_> = [0, 1, n / 7, n / 2, n - 1].into();
        for l in [
            vec![65u8; n],
            (0..n)
                .map(|i| if i / 17 % 2 == 0 { 34 } else { 58 })
                .collect(),
            (0..n)
                .map(|i| [7u8, 191, 0, 255, 17][((i / 21) % 5 + (i / 97)) % 5])
                .collect(),
        ] {
            check(&l, &marks, &mut c, &mut total);
        }
    }
    println!("{} valid and {} exact unfinished-run outcomes; {} capped graph finishes. Scalar/detected, n through {}, arbitrary primary and a prefixed caller output.",total[0],total[1],total[2],21850*17);
}
