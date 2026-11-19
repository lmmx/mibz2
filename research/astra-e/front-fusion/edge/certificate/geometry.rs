// Recover unmarked thin-row mass and thick-run count using only symbol bitmaps.
pub fn unmarked(literal: &[u64], one: &[u64], end: usize) -> Option<(usize, usize)> {
    let words = end / 64 + 1;
    let first = literal[..words].iter().enumerate().find_map(|(j, &word)| {
        (word != 0).then(|| j * 64 + word.trailing_zeros() as usize)
    }).unwrap_or(end);
    if first > 21 { return None; }
    let opening_mask = (1 << first) - 1;
    let opening = opening_mask + (one[0] & opening_mask);
    let mut rows = if opening <= 16 { opening as usize } else { 0 };
    let mut thick = usize::from(opening > 16);
    for j in 0..words {
        let start = j * 64;
        let marks = literal[j];
        let mut next = u128::from(marks) | (u128::from(literal[j + 1]) << 64);
        if end - start < 128 { next |= 1 << (end - start); }
        let ones = u128::from(one[j]) | (u128::from(one[j + 1]) << 64);
        let mut live = marks;
        for t in 0..4 {
            let boundary = (next >> (t + 1)) as u64;
            let short = live & boundary;
            rows += (short.count_ones() as usize) << t;
            for k in 0..t {
                let bit = (ones >> (k + 1)) as u64;
                rows += ((short & bit).count_ones() as usize) << k;
            }
            live &= !boundary;
        }
        // Four RUNAs followed by a literal have width 16. All other surviving literals
        // produce at least 17 rows. The EOB sentinel has the same boundary role.
        let has_one = ((ones >> 1) | (ones >> 2) | (ones >> 3) | (ones >> 4)) as u64;
        let exactly_16 = live & (next >> 5) as u64 & !has_one;
        rows += 16 * exactly_16.count_ones() as usize;
        thick += (live & !exactly_16).count_ones() as usize;
    }
    Some((rows, thick))
}

/// Guarded envelope, even when the primary's unmarked run is unknown.
pub fn charge(rows: usize, thick: usize) -> usize {
    let bits = usize::BITS - rows.max(1).saturating_sub(1).leading_zeros();
    let width = if (511 + 3 * thick) <= (u32::MAX >> bits.max(1)) as usize { 4 } else { 8 };
    rows * width
}

/// The first cap record of every run has its whole width or 127, whichever is smaller.
/// Its width is sufficient for classification at a cutoff below 127.
pub fn unmarked_caps(records: &[u16]) -> (usize, usize) {
    let mut previous = 256;
    let mut rows = 0;
    let mut thick = 0;
    for &record in records {
        let byte = record >> 8;
        let length = usize::from(record & 255);
        let first = usize::from(previous != byte);
        rows += first * usize::from(length <= 16) * length;
        thick += first * usize::from(length > 16);
        previous = byte;
    }
    (rows, thick)
}
