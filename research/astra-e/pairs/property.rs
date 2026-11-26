//! Exhaust the residual permutation, independent of the constructor and scorer.
include!("contract.rs");

fn test_permutation(permutation: &mut [usize], fixed: usize, cases: &mut usize) {
    if fixed != permutation.len() {
        for i in fixed..permutation.len() {
            permutation.swap(i, fixed);
            test_permutation(permutation, fixed + 1, cases);
            permutation.swap(i, fixed);
        }
        return;
    }
    let n = permutation.len();
    let bits = (usize::BITS - (n - 1).leading_zeros()).max(1);
    let mask = (1 << bits) - 1;
    let first_pair = 301;
    let alphabet: [u32; 4] = [0, 127, 255, 300];
    let mut original = vec![0; n];
    let mut contracted = vec![0; n];
    let (mut expected, mut actual) = (Vec::new(), Vec::new());
    for mut selector in 0..4usize.pow(n as u32) {
        for (i, &next) in permutation.iter().enumerate() {
            original[i] = (alphabet[selector % 4] << bits) | next as u32;
            selector /= 4;
        }
        for mark in 0..n {
            expected.clear();
            let mut row = mark;
            loop {
                let e = original[row];
                expected.push(e >> bits);
                row = (e & mask) as usize;
                if row == mark {
                    break;
                }
            }
            actual.clear();
            contracted.copy_from_slice(&original);
            contract::<false>(&mut contracted, mark as u32, bits, first_pair);
            let mut row = mark;
            loop {
                let e = contracted[row];
                let code = e >> bits;
                if code >= first_pair {
                    let pair = code - first_pair;
                    actual.extend([pair / ALPHABET, pair % ALPHABET]);
                } else {
                    actual.push(code)
                }
                assert!(actual.len() <= expected.len());
                row = (e & mask) as usize;
                if row == mark {
                    break;
                }
            }
            assert_eq!(actual, expected);
            *cases += 1;
        }
    }
}

fn main() {
    let mut cases = 0;
    for n in 1..=5 {
        let mut permutation = (0..n).collect::<Vec<_>>();
        test_permutation(&mut permutation, 0, &mut cases);
    }
    println!("{cases} marked permutations and label assignments passed");
}
