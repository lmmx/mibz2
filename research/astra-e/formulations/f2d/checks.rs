use super::*;

#[test]
fn marked_paths() {
    let mut single = Solver::<false>::default();
    let mut eight = Solver::<true>::default();
    let mut out = Vec::new();
    let (mut accepted, mut refused) = (0, 0);
    assert_eq!(std::mem::size_of::<Run>(), 12);
    for (alphabet, limit) in [(2usize, 9usize), (3, 6)] {
        for n in 1..=limit {
            for mut seed in 0..alphabet.pow(n as u32) {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        let b = (seed % alphabet) as u8 + QUOTE - 1;
                        seed /= alphabet;
                        b
                    })
                    .collect();
                let mut psi: Vec<_> = (0..n).collect();
                psi.sort_by_key(|&i| l[i]);
                let caps: Vec<_> = l.iter().map(|&b| u16::from(b) << 8 | 1).collect();
                for p in 0..n {
                    let mut x = p;
                    let want: Vec<_> = (0..n)
                        .map(|_| {
                            x = psi[x];
                            l[x]
                        })
                        .collect();
                    let used = single.attempt::<true>(&caps, n, p as u32, &mut out);
                    assert_eq!(used, want.contains(&QUOTE));
                    if used {
                        assert_eq!(want, out);
                        accepted += 1;
                    } else {
                        refused += 1;
                    }
                    assert_eq!(eight.attempt::<true>(&caps, n, p as u32, &mut out), used);
                    if used {
                        assert_eq!(want, out);
                    }
                }
            }
        }
    }
    println!("each implementation: {accepted} exact outputs and {refused} correct refusals");
}
