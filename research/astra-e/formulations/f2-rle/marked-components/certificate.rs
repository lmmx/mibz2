// Verify sparse parsing of intersecting four-letter cylinders and arbitrary marked cuts.
fn main() {
    let mut marks = 0u64;
    let mut singles = 0u64;
    let mut invalid = 0u64;
    let mut unseen_components = 0u64;
    let mut overlap = 0u64;
    let mut marked = 0u64;
    let mut old = 0u64;
    for n in 1..=15 {
        for mask in 0..1usize << n {
            let last: Vec<_> = (0..n)
                .map(|i| if mask & (1 << i) != 0 { 255 } else { 0u8 })
                .collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| last[i]);
            let first: Vec<_> = psi.iter().map(|&x| last[x]).collect();
            let mut lf = vec![0; n];
            for (i, &next) in psi.iter().enumerate() {
                lf[next] = i;
            }
            let mut dirty = vec![false; n];
            let mut intersects = false;
            for start in 0..n {
                let mut x = start;
                let group: [usize; 5] = std::array::from_fn(|_| {
                    let y = x;
                    x = psi[x];
                    y
                });
                if group[..4].iter().all(|&r| first[r] == first[start]) {
                    for r in group {
                        intersects |= dirty[r];
                        dirty[r] = true;
                    }
                }
            }
            for p in 0..n {
                marks += 1;
                old += u64::from(!intersects && !dirty[p]);
                overlap += u64::from(intersects);
                marked += u64::from(dirty[p]);
                // Production can use a D-sized sorted row table. Dense indexing is just the oracle implementation.
                let mut seen = vec![false; n];
                let mut edits = vec![None; n];
                let mut unfinished = false;
                for start in 0..n {
                    if !dirty[start] || (start != p && dirty[lf[start]]) {
                        continue;
                    }
                    let mut x = start;
                    let mut count = 0;
                    let mut previous = 0;
                    loop {
                        assert!(!seen[x]);
                        seen[x] = true;
                        let b = first[x];
                        if count == 4 {
                            edits[x] = Some((previous, b));
                            count = 0;
                        } else if count > 0 && b == previous {
                            count += 1;
                        } else {
                            count = 1;
                            previous = b;
                        }
                        x = psi[x];
                        if x == p || !dirty[x] {
                            if count == 4 {
                                assert_eq!(x, p);
                                unfinished = true;
                            }
                            break;
                        }
                    }
                }
                unseen_components += u64::from((0..n).any(|x| dirty[x] && !seen[x]));
                let mass = (n as i64
                    + edits
                        .iter()
                        .flatten()
                        .map(|&(_, k)| i64::from(k) - 1)
                        .sum::<i64>()) as usize;
                let mut raw = Vec::new();
                let mut corrected = Vec::new();
                let mut x = p;
                loop {
                    raw.push(first[x]);
                    if let Some((byte, k)) = edits[x] {
                        corrected.extend(std::iter::repeat_n(byte, usize::from(k)));
                    } else {
                        corrected.push(first[x]);
                    }
                    x = psi[x];
                    if x == p {
                        break;
                    }
                }
                let mut oracle = Vec::new();
                let error = mibz2::rle::unrle(&raw, &mut oracle, mibz2::Kernels::scalar()).is_err();
                assert_eq!(error, unfinished);
                assert_eq!(corrected.len() == mass, raw.len() == n);
                if error {
                    invalid += 1;
                    corrected.truncate(corrected.len() - 4);
                }
                assert_eq!(oracle, corrected);
                singles += u64::from(raw.len() == n);
            }
        }
    }
    println!("{marks} marked binary last columns through n=15: {singles} single, {} multiple; {invalid} invalid first-lap terminal quartets. {overlap} with intersecting windows; {marked} marks protected; {unseen_components} with a wholly protected off-mark component. Prior disjoint/unprotected certificate applied to {old}. All corrected bytes, terminal errors and global iff decisions matched the dense orbit and stock RLE1.",marks-singles);
}
