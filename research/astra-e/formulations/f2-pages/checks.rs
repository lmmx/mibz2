use super::*;

fn verify<const WIDTH: u32>() -> usize {
    let mut kernel = PageCandidate::<WIDTH>::default();
    let mut out = Vec::new();
    let mut accepted = 0;
    for (alphabet, limit) in [(2usize, 9), (3, 6)] {
        for n in 1..=limit {
            for mut seed in 0..alphabet.pow(n as u32) {
                let l: Vec<_> = (0..n).map(|_| {
                    let byte = QUOTE - 1 + (seed % alphabet) as u8;
                    seed /= alphabet;
                    byte
                }).collect();
                let mut psi: Vec<_> = (0..n).collect();
                psi.sort_by_key(|&i| l[i]);
                let caps: Vec<_> = l.iter().map(|&c| u16::from(c) << 8 | 1).collect();
                for p in 0..n {
                    let mut x = p;
                    let want: Vec<_> = (0..n).map(|_| { x = psi[x]; l[x] }).collect();
                    let took = kernel.attempt::<true>(&caps,n,p as u32,&mut out);
                    assert_eq!(took, want.contains(&QUOTE));
                    if took {
                        assert_eq!(out, want);
                        accepted += 1;
                    }
                }
            }
        }
    }
    accepted
}

#[test]
fn arbitrary_marks() {
    for covered in [verify::<32>(),verify::<40>(),verify::<44>(),verify::<64>()] {
        assert_eq!(covered, 11264);
    }
    println!("all four page encodings matched the marked orbit in 11264 admitted small cases each");
}
