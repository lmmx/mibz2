use super::*;

fn check(column: &[u8], marks: &[usize], words: usize, visits: usize) {
    let mut candidate = Candidate::default();
    let mut psi: Vec<_> = (0..column.len()).collect();
    psi.sort_by_key(|&i| column[i]);
    let caps: Vec<_> = column.iter().map(|&b| u16::from(b) << 8 | 1).collect();
    for &mark in marks {
        let mut row = mark;
        let expected: Vec<_> = (0..column.len())
            .map(|_| {
                row = psi[row];
                column[row]
            })
            .collect();
        let mut out = Vec::new();
        assert!(candidate.attempt::<true>(&caps, column.len(), mark as u32, &mut out));
        assert_eq!(out, expected);
        assert_eq!(candidate.words.len(), words);
        assert_eq!(candidate.stats[1], visits as u64);
        assert_eq!(candidate.merged.len(), visits);
    }
}

#[test]
fn pure_root_and_reopened_deep_leaf() {
    check(&[QUOTE; 127], &[0, 108, 126], 1, 1);
    // This exact cycle-lifted column is the D=2, M=4 member of audit/lift.py.
    // Each of the four deep-token runs formerly visited three cursor nodes.
    let bits = [1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
    let column: Vec<_> = bits.iter().map(|&b| b + QUOTE).collect();
    check(&column, &[0, 4, 11, 17], 3, if ABSORB { 11 } else { 12 });
}
