//! An independent dense-LF oracle for every retained arrow, including arrows
//! outside the marked component. Allocation here is outside the candidate.
use super::{append_code, each_run, starts, Kernel};
use mibz2::{
    bwt::{InverseBwt, Plain},
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{env, fs};

// Exact LRU reuse distances of the u32 successor-table lines in one marked walk.
// Each live line has one bit at its most recent timestamp in the Fenwick tree.
fn census(trace: &[usize], nlines: usize) -> [usize; 4] {
    let mut tree = vec![0i32; trace.len() + 1];
    let mut last = vec![0; nlines];
    let mut occupied = 0;
    let mut counts = [0; 4];
    for (step, &line) in trace.iter().enumerate() {
        let prev = last[line];
        if prev == 0 {
            for count in &mut counts {
                *count += 1;
            }
            occupied += 1;
        } else {
            let mut k = prev;
            let mut prefix = 0;
            while k != 0 {
                prefix += tree[k];
                k &= k - 1;
            }
            let distance = occupied - prefix;
            for (count, cap) in counts.iter_mut().zip([1, 512, 4096, 32768]) {
                *count += usize::from(distance >= cap);
            }
            let mut k = prev;
            while k < tree.len() {
                tree[k] -= 1;
                k += k & k.wrapping_neg();
            }
        }
        let mut k = step + 1;
        last[line] = k;
        while k < tree.len() {
            tree[k] += 1;
            k += k & k.wrapping_neg();
        }
    }
    counts
}

#[test]
#[ignore = "requires the pinned Wikidata corpus"]
fn audit_wikidata() {
    let path = env::var("MOONSHOT_CORPUS").expect("set MOONSHOT_CORPUS to verified Wikidata");
    let data = fs::read(path).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut r = runs::Runs::default();
    let mut ordinal = 0;
    let mut large = 0;
    let mut kernel = Kernel::default();
    let mut out = Vec::new();
    println!("ordinal,n,records,admitted,retained,bands,thin_runs,writes,single_row_writes,edge_bytes,extra_band_bytes,queue_payload_removed,orbit_area,steps,L_line_switch,L_L1miss,L_L2miss,L_2Mmiss,F_line_switch,F_L1miss,F_L2miss,F_2Mmiss");
    while let Event::Block { symbols, .. } = scanner.next(syms).unwrap() {
        let n = runs::front(&symbols, &mut r, Kernels::detect()).unwrap();
        let p = symbols.orig_ptr;
        if n >= 65536 && large % 24 == 0 {
            let records = r.records();
            kernel.run::<16>(records, n as u32, p, &mut out);
            let mut l = Vec::with_capacity(n);
            let mut counts = [0; 256];
            for &rec in records {
                let c = (rec >> 8) as u8;
                l.resize(l.len() + (rec as usize & 255), c);
                counts[c as usize] += u32::from(rec & 255);
            }
            let mut reference = Vec::new();
            Plain::default().invert(&l, &counts, p, &mut reference);
            assert_eq!(out, reference);
            let mut buckets = starts(counts);
            let lf: Vec<_> = l
                .iter()
                .map(|&v| {
                    let next = &mut buckets[v as usize];
                    *next += 1;
                    (*next - 1) as usize
                })
                .collect();
            let mut keep = vec![false; n];
            let mut thin = [0; 256];
            let mut thin_runs = 0;
            let mut j = 0;
            let mut bi = 0;
            let mut writes = 0;
            let mut one = 0;
            each_run(records, p, 16, |c, x, w, retained| {
                if retained {
                    keep[x as usize..(x + w) as usize].fill(true);
                    thin[c as usize] += w;
                    thin_runs += 1;
                    let end = j + w;
                    while j < end {
                        let until = end.min(kernel.bands[bi].end);
                        one += usize::from(until - j == 1);
                        writes += 1;
                        j = until;
                        if j == kernel.bands[bi].end {
                            bi += 1;
                        }
                    }
                }
            });
            let m = j as usize;
            let bits = (u32::BITS - (j - 1).leading_zeros()).max(1);
            let mask = (1 << bits) - 1;
            let mut thin_bucket = starts(thin);
            let ranks: Vec<_> = l
                .iter()
                .zip(&keep)
                .map(|(&c, &kept)| {
                    if kept {
                        let k = &mut thin_bucket[c as usize];
                        *k += 1;
                        *k - 1
                    } else {
                        u32::MAX
                    }
                })
                .collect();
            let mut forward = Vec::new();
            let mut word = Vec::new();
            let mut area = 0;
            for x in (0..n).filter(|&x| keep[x]) {
                let mut y = x;
                forward.clear();
                loop {
                    forward.push(l[y]);
                    y = lf[y];
                    if keep[y] {
                        break;
                    }
                    assert!(forward.len() < n);
                }
                let index = ranks[y] as usize;
                let edge = kernel.edges[index];
                assert_eq!(edge & mask, ranks[x]);
                let code = if kernel.extra.is_empty() {
                    edge >> bits
                } else {
                    kernel.extra[index]
                };
                word.clear();
                append_code(code, &kernel.spelling, &kernel.offsets, &mut word);
                assert!(word.iter().eq(forward.iter().rev()));
                area += word.len();
            }
            assert!(area <= n);
            let mut f_to_l = vec![0; m];
            for (j, &f) in ranks.iter().filter(|&&s| s != u32::MAX).enumerate() {
                f_to_l[f as usize] = j;
            }
            let start = ranks[p as usize] as usize;
            let mut f = start;
            let mut l_rows = Vec::new();
            let mut f_rows = Vec::new();
            loop {
                l_rows.push(f_to_l[f] / 16);
                f_rows.push(f / 16);
                f = (kernel.edges[f] & mask) as usize;
                if f == start {
                    break;
                }
            }
            let l_cold = census(&l_rows, m.div_ceil(16));
            let f_cold = census(&f_rows, m.div_ceil(16));
            let misses: Vec<_> = l_cold
                .into_iter()
                .chain(f_cold)
                .map(|v| v.to_string())
                .collect();
            let misses = misses.join(",");
            let steps = f_rows.len();
            let admitted = runs::takes_runs(&symbols) && records.len() * 20 < n * 3;
            println!("{ordinal},{n},{},{admitted},{m},{},{thin_runs},{writes},{one},{},{},{},{area},{steps},{misses}", records.len(),kernel.bands.len(),4*(kernel.edges.len()+kernel.extra.len()),12*kernel.top_bands.len(),4*m);
        }
        large += usize::from(n >= 65536);
        ordinal += 1;
        syms = symbols.syms;
    }
    assert_eq!(large, 233);
}
