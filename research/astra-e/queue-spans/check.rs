//! Independently reconstruct the marked orbit using stable sorting. Force every queue route.
pub use mibz2::{bwt, runs, Kernels};
#[allow(dead_code)]
#[path = "candidate.rs"]
mod spans;
use spans::kernel::Kernel;

fn records(col: &[u8]) -> Vec<u16> {
    let mut records: Vec<u16> = Vec::new();
    for &c in col {
        if let Some(last) = records.last_mut() {
            if (*last >> 8) as u8 == c && (*last as u8) < 127 {
                *last += 1;
                continue;
            }
        }
        records.push((u16::from(c) << 8) | 1);
    }
    records
}

fn check<const MODE: u8>(kernel: &mut Kernel, rec: &[u16], n: usize, p: u32, limit: u32, reference: &[u8]) {
    let mut got = Vec::new();
    kernel.run::<true, MODE, false>(rec, n, p, limit, &mut got);
    assert_eq!(got, reference, "mode={MODE}, primary={p}, limit={limit}");
}

fn verify(col: &[u8], marks: &[usize], limits: &[u32], kernel: &mut Kernel) -> usize {
    let mut psi: Vec<usize> = (0..col.len()).collect();
    psi.sort_by_key(|&x| col[x]);
    let rec = records(col);
    for &p in marks {
        let mut row = p;
        let reference: Vec<u8> = (0..col.len()).map(|_| {
            row = psi[row];
            col[row]
        }).collect();
        for &limit in limits {
            check::<0>(kernel, &rec, col.len(), p as u32, limit, &reference);
            check::<1>(kernel, &rec, col.len(), p as u32, limit, &reference);
            check::<2>(kernel, &rec, col.len(), p as u32, limit, &reference);
            check::<3>(kernel, &rec, col.len(), p as u32, limit, &reference);
        }
    }
    marks.len() * limits.len()
}

fn main() {
    let mut kernel = Kernel::default();
    let mut cases = 0;
    for (alphabet, max_n) in [(2usize, 9), (3, 6)] {
        for n in 1..=max_n {
            for v in 0..alphabet.pow(n as u32) {
                let mut x = v;
                let col: Vec<u8> = (0..n).map(|_| {
                    let c = x % alphabet;
                    x /= alphabet;
                    c as u8
                }).collect();
                cases += verify(&col, &(0..n).collect::<Vec<_>>(), &[1, 2, 4, 16], &mut kernel);
            }
        }
    }
    // Each of 5001 retained zeroes can join its preceding zero after the intervening
    // thick strip disappears. A middle primary inside that strip also adds a singleton.
    let strip: Vec<u8> = [vec![0], vec![1; 17]].concat();
    let cap = strip.repeat(5001);
    cases += verify(&cap, &[0, 3, 4096 * 18, cap.len() - 1], &[16], &mut kernel);
    // A wide retained run is not a production admission; it exercises generic splitting.
    cases += verify(&vec![2; 8195], &[0, 4096, 8194], &[900000], &mut kernel);
    println!("{cases} marked-column/limit cases, each checked in all four modes");
}
