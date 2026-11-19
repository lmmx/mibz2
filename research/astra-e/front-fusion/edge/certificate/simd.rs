// A 16-record window and its one-record-left neighbor stay within the same input slice.
// At each true run's first record the low byte is min(true_width,127), sufficient at 16.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn unmarked_avx2(records: &[u16]) -> (usize, usize) {
    use std::arch::x86_64::*;
    if records.is_empty() { return (0, 0); }
    let mut lane_rows = _mm256_setzero_si256();
    let mut lane_thick = _mm256_setzero_si256();
    let ones = _mm256_set1_epi16(1);
    let mut i = 1;
    while i + 16 <= records.len() {
        let window = _mm256_loadu_si256(records.as_ptr().add(i).cast());
        let previous = _mm256_loadu_si256(records.as_ptr().add(i - 1).cast());
        let same = _mm256_cmpeq_epi16(_mm256_srli_epi16(window, 8), _mm256_srli_epi16(previous, 8));
        let width = _mm256_and_si256(window, _mm256_set1_epi16(255));
        let thin = _mm256_cmpgt_epi16(_mm256_set1_epi16(17), width);
        let retain = _mm256_andnot_si256(same, _mm256_and_si256(thin, width));
        let heavy = _mm256_andnot_si256(_mm256_or_si256(same, thin), ones);
        lane_rows = _mm256_add_epi32(lane_rows, _mm256_madd_epi16(ones, retain));
        lane_thick = _mm256_add_epi32(lane_thick, _mm256_madd_epi16(ones, heavy));
        i += 16;
    }
    let sum = |v| -> usize {
        let lanes: [u32; 8] = std::mem::transmute(v);
        lanes.into_iter().map(|n| n as usize).sum()
    };
    let mut rows = sum(lane_rows);
    let mut thick = sum(lane_thick);
    let mut add = |record: u16| {
        let width = usize::from(record & 255);
        if width <= 16 { rows += width; } else { thick += 1; }
    };
    add(records[0]);
    for j in i..records.len() {
        if records[j] >> 8 != records[j - 1] >> 8 { add(records[j]); }
    }
    (rows, thick)
}
