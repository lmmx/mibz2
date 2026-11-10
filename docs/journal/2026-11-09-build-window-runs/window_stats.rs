//! The window census behind docs/journal/2026-11-09-build-window-runs.md (scratch, not built:
//! copy into examples/ to run). Usage: window_stats FILE.bz2
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{mtf, Kernels};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let data = std::fs::read(&a[1]).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let (mut syms, mut ll, mut counts) = (Vec::new(), Vec::new(), [0u32; 256]);
    let (mut n_tot, mut eq_rows, mut mixed_win, mut mixed_runs, mut transitions) =
        (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut hist = [0u64; 17]; // runs per mixed window
    let mut chain_rows = 0u64; // rows in mixed windows equal to the previous row (store-forward chain)
    let mut fits = 0u64;
    let mut runs_total = 0u64;
    loop {
        let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() else {
            break;
        };
        mtf::mtf(&b, &mut ll, &mut counts, k).unwrap();
        syms = b.syms;
        let n = ll.len();
        n_tot += n as u64;
        let mut ftab = [0u32; 256];
        let mut lim = [0u32; 256];
        let mut c = 0;
        for x in 0..256 {
            ftab[x] = c;
            c += counts[x];
            lim[x] = c;
        }
        let mut i = 0;
        let mut prev_eq = None;
        while i + 16 <= n {
            let w = &ll[i..i + 16];
            let eq = w.iter().all(|&x| x == w[0]);
            if let Some(p) = prev_eq {
                if p != eq {
                    transitions += 1;
                }
            }
            prev_eq = Some(eq);
            if eq {
                eq_rows += 16;
                ftab[w[0] as usize] += 16;
            } else {
                mixed_win += 1;
                let mut r = 1;
                for j in 1..16 {
                    if w[j] != w[j - 1] {
                        r += 1;
                    } else {
                        chain_rows += 1;
                    }
                }
                // runs: check s+16 <= lim
                let mut j = 0;
                while j < 16 {
                    let cc = w[j] as usize;
                    let mut e = j + 1;
                    while e < 16 && w[e] == w[j] {
                        e += 1;
                    }
                    runs_total += 1;
                    if ftab[cc] as usize + 16 <= lim[cc] as usize {
                        fits += 1;
                    }
                    ftab[cc] += (e - j) as u32;
                    j = e;
                }
                mixed_runs += r;
                hist[r as usize] += 1;
            }
            i += 16;
        }
    }
    println!(
        "rows {n_tot}, rows in equal windows {:.3}",
        eq_rows as f64 / n_tot as f64
    );
    println!("mixed windows {mixed_win} (rows {:.3}), runs/mixed window {:.2}, transitions {transitions} ({:.4}/window)", (mixed_win*16) as f64/n_tot as f64, mixed_runs as f64/mixed_win as f64, transitions as f64/(n_tot as f64/16.0));
    println!("rows in mixed windows equal to their predecessor (in-window): {chain_rows} = {:.3} of mixed rows", chain_rows as f64/(mixed_win*16) as f64);
    println!(
        "runs with 16 slots of room in bucket: {:.4}",
        fits as f64 / runs_total as f64
    );
    println!("hist {:?}", hist);
}
