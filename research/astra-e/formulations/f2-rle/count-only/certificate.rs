// Verify the cyclic decoded-mass certificate independently on arbitrary last columns.
fn main() {
    let mut eligible = 0;
    let mut single = 0;
    let mut multi = 0;
    let mut with_events = 0;
    let mut nontrivial_multi = 0;
    let mut nontrivial_single = 0;
    for n in 1..=15 {
        for bits in 0..1usize << n {
            let l: Vec<_> = (0..n)
                .map(|i| if bits & (1 << i) == 0 { 0u8 } else { 255u8 })
                .collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| l[i]);
            let first: Vec<_> = psi.iter().map(|&i| l[i]).collect();
            let mut seen = vec![false; n];
            let mut edit = vec![None; n];
            let mut disjoint = true;
            let mut delta = 0i64;
            let mut num = 0;
            for start in 0..n {
                let mut x = start;
                let mut window = [0usize; 5];
                let b = first[x];
                for i in 0..5 {
                    window[i] = x;
                    x = psi[x];
                }
                if !window[..4].iter().all(|&y| first[y] == b) {
                    continue;
                }
                for &y in &window {
                    disjoint &= !seen[y];
                    seen[y] = true;
                }
                let y = window[4];
                let count = first[y];
                edit[y] = Some((b, count));
                delta += i64::from(count) - 1;
                num += 1;
            }
            if !disjoint {
                continue;
            }
            let target = (n as i64 + delta) as usize;
            for p in 0..n {
                if seen[p] {
                    continue;
                }
                let mut x = p;
                let mut raw = Vec::new();
                let mut output = Vec::new();
                loop {
                    raw.push(first[x]);
                    if let Some((b, count)) = edit[x] {
                        output.extend(std::iter::repeat_n(b, count as usize));
                    } else {
                        output.push(first[x]);
                    }
                    x = psi[x];
                    if x == p {
                        break;
                    }
                }
                let mut oracle = Vec::new();
                mibz2::rle::unrle(&raw, &mut oracle, mibz2::Kernels::scalar()).unwrap();
                assert_eq!(oracle, output);
                assert_eq!(raw.len() == n, output.len() == target);
                eligible += 1;
                with_events += usize::from(num != 0);
                if raw.len() == n {
                    single += 1;
                    nontrivial_single += usize::from(num != 0);
                } else {
                    multi += 1;
                    nontrivial_multi += usize::from(num != 0);
                }
            }
        }
    }
    println!("{eligible} certified mark/windows: {single} single cycle, {multi} multi-cycle. {with_events} have at least one true edit ({nontrivial_single} single, {nontrivial_multi} multi). All count-only outputs and iff mass decisions matched dense Psi and stock RLE1.");
}
