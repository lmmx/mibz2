use super::*;
#[test]
fn arbitrary_columns() {
    let mut k = kernel::Kernel::default();
    let mut out = Vec::new();
    let mut cases = 0;
    for n in 1..=9 {
        for seed in 0..1u32 << n {
            let l: Vec<_> = (0..n).map(|i| ((seed >> i) & 1) as u8).collect();
            let caps: Vec<_> = l.iter().map(|&c| u16::from(c) << 8 | 1).collect();
            let mut psi: Vec<_> = (0..n).collect();
            psi.sort_by_key(|&i| l[i]);
            for p in 0..n {
                let mut x = p;
                let want: Vec<_> = (0..n)
                    .map(|_| {
                        x = psi[x];
                        l[x]
                    })
                    .collect();
                for w in 1..=3 {
                    for word in 0..1usize << w {
                        let pattern: Vec<_> = (0..w).map(|j| ((word >> j) & 1) as u8).collect();
                        if let Some(s) = kernel::Kernel::section(&caps, &pattern) {
                            k.run::<true, true, false>(&caps, n, p as u32, s, &mut out);
                            assert_eq!(out, want);
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    println!("{cases} marked endpoint constructions matched dense Ψ");
}
