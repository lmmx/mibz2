use super::*;

#[test]
fn selected_cycles_and_marks() {
    let mut decoder = Candidate::default();
    let mut out = Vec::new();
    let mut takes = 0;
    let mut misses = 0;
    for size in 1..=9 {
        for mask in 0..1usize << size {
            let last: Vec<u8> = (0..size).map(|i| 34 + ((mask >> i) & 1) as u8).collect();
            let caps: Vec<u16> = last.iter().map(|b| (u16::from(*b) << 8) | 1).collect();
            let mut psi: Vec<usize> = (0..size).collect();
            psi.sort_by_key(|i| last[*i]);
            for mark in 0..size {
                let mut row = mark;
                let want: Vec<u8> = (0..size)
                    .map(|_| {
                        row = psi[row];
                        last[row]
                    })
                    .collect();
                for depth in 0..=3 {
                    if decoder.for_depth::<true>(&caps, size, mark as u32, &mut out, b"\"", depth) {
                        assert_eq!(out, want);
                        takes += 1;
                    } else {
                        misses += 1;
                    }
                }
            }
        }
    }
    assert!(takes > 0 && misses > 0);
    eprintln!("{takes} exact nested marked outputs, {misses} selected-section refusals");
}
