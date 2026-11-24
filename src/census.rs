//! Research only (`census`): the block-structure census of the open kernel's blocks
//! (docs/journal/2026-11-24-wikidata-block-census.md; #145 LOG 7126445058, C1a). Each block the
//! open kernel takes (`OpenCells`' `Kernel::try_run` returning `Some`) gets one record of
//! properties of its BWT column, computed from its run records and marked row alone, apart from
//! the kernel's own construction (whose retained count is recorded beside the census's, as a
//! check): the retained rows as the kernel classifies them (`open::split` at `open::LIMIT`), the
//! maximal runs' length histogram, the thick strips, the cycle structure of the retained map T
//! (the first return of the walk to a retained row) and the locality |T(s) - s| in retained
//! ranks (the index the emit reads `edges` at). It adds an O(n) pass a block on the open path
//! and nothing else; without the feature none of this is compiled.
use std::sync::Mutex;

/// One block's census record
#[derive(Clone, Debug, PartialEq)]
pub struct Rec {
    /// a hash of the block's run records, `n` and `p`: equal for equal BWT blocks (and so for
    /// equal decoded blocks)
    pub id: u64,
    pub n: u32,
    pub p: u32,
    /// run records (as the decoder holds them, cut at 127 rows)
    pub records: u32,
    /// the kernel's own retained count (`Counts::retained`)
    pub m_kernel: u32,
    /// the census's retained count (thin rows of maximal runs of at most `LIMIT` rows, and the
    /// marked row when it lies in a thick run)
    pub m: u32,
    /// maximal runs (a byte's consecutive records joined)
    pub runs: u32,
    /// maximal runs of each length class: 1, 2-3, 4-16, 17-127, 128+
    pub runs_by_len: [u32; 5],
    /// rows in maximal runs of each length class (as `runs_by_len`)
    pub rows_by_len: [u32; 5],
    /// thick strips (thick pieces, the marked run's fragments counted apart)
    pub strips: u32,
    /// the kernel's construction counts: dictionary words, bands, longest word
    pub words: u32,
    pub bands: u32,
    pub longest: u32,
    /// cycles of T (cycles of LF holding a retained row)
    pub cycles: u32,
    /// retained rows on the marked row's cycle
    pub marked_cycle: u32,
    /// |T(s) - s| over the m retained ranks: counts below 16, 1024, 8192 and 65536 (a 64-byte
    /// line, a 4 KiB page, 32 KiB and 256 KiB of 4-byte `edges` entries), the sum of
    /// log2(1 + d), the sum of d, and the median and 90th percentile
    pub d_lt: [u32; 4],
    pub d_log2_sum: f64,
    pub d_sum: u64,
    pub d_p50: u32,
    pub d_p90: u32,
}

static RECS: Mutex<Vec<Rec>> = Mutex::new(Vec::new());

/// The records since the last `take`, in the order the workers finished them
pub fn take() -> Vec<Rec> {
    std::mem::take(&mut *RECS.lock().unwrap_or_else(|e| e.into_inner()))
}

fn class(w: u32) -> usize {
    match w {
        1 => 0,
        2..=3 => 1,
        4..=16 => 2,
        17..=127 => 3,
        _ => 4,
    }
}

/// Census one block the open kernel took: `records` (`byte << 8 | rows`, L order), `n` rows,
/// marked row `p`, at `limit`, with the kernel's counts
pub(crate) fn record(records: &[u16], n: usize, p: u32, limit: u32, c: &crate::open::Counts) {
    let r = census(records, n, p, limit, c);
    RECS.lock().unwrap_or_else(|e| e.into_inner()).push(r);
}

pub(crate) fn census(
    records: &[u16],
    n: usize,
    p: u32,
    limit: u32,
    c: &crate::open::Counts,
) -> Rec {
    // the identity: FNV-1a over n, p and the records
    let mut id: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |x: u64| {
        for b in x.to_le_bytes() {
            id ^= u64::from(b);
            id = id.wrapping_mul(0x0100_0000_01b3);
        }
    };
    mix(n as u64);
    mix(u64::from(p));
    for &r in records {
        mix(u64::from(r));
    }
    // L, and each row's retained rank (u32::MAX for a thick row)
    let mut l = Vec::with_capacity(n);
    let mut rank = vec![u32::MAX; n];
    let (mut m, mut runs, mut strips) = (0u32, 0u32, 0u32);
    let (mut runs_by_len, mut rows_by_len) = ([0u32; 5], [0u32; 5]);
    let mut i = 0;
    let mut pos = 0u32;
    while i < records.len() {
        let byte = (records[i] >> 8) as u8;
        let mut w = u32::from(records[i] & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == byte {
            w += u32::from(records[i] & 255);
            i += 1;
        }
        runs += 1;
        runs_by_len[class(w)] += 1;
        rows_by_len[class(w)] += w;
        l.resize(l.len() + w as usize, byte);
        let end = pos + w;
        if w <= limit {
            for x in pos..end {
                rank[x as usize] = m;
                m += 1;
            }
        } else if (pos..end).contains(&p) {
            strips += u32::from(pos < p) + u32::from(p + 1 < end);
            rank[p as usize] = m;
            m += 1;
        } else {
            strips += 1;
        }
        pos = end;
    }
    assert_eq!(pos as usize, n);
    // LF
    let mut start = [0u32; 256];
    {
        let mut counts = [0u32; 256];
        for &b in &l {
            counts[usize::from(b)] += 1;
        }
        let mut t = 0;
        for a in 0..256 {
            start[a] = t;
            t += counts[a];
        }
    }
    let lf: Vec<u32> = l
        .iter()
        .map(|&b| {
            let a = usize::from(b);
            let v = start[a];
            start[a] += 1;
            v
        })
        .collect();
    drop(l);
    // the cycles of LF, and on each the retained ranks in walk order: T's cycles, and the
    // distances between successive retained ranks (cyclically; the same multiset in either
    // direction of the walk)
    let mut seen = vec![false; n];
    let mut d = Vec::with_capacity(m as usize);
    let (mut cycles, mut marked_cycle) = (0u32, 0u32);
    let mut on = Vec::new();
    for s in 0..n {
        if seen[s] {
            continue;
        }
        on.clear();
        let mut has_mark = false;
        let mut x = s;
        while !seen[x] {
            seen[x] = true;
            has_mark |= x == p as usize;
            if rank[x] != u32::MAX {
                on.push(rank[x]);
            }
            x = lf[x] as usize;
        }
        if on.is_empty() {
            continue;
        }
        cycles += 1;
        if has_mark {
            marked_cycle = on.len() as u32;
        }
        for k in 0..on.len() {
            let (a, b) = (on[k], on[(k + 1) % on.len()]);
            d.push(a.abs_diff(b));
        }
    }
    assert_eq!(d.len(), m as usize);
    let mut d_lt = [0u32; 4];
    let (mut d_log2_sum, mut d_sum) = (0f64, 0u64);
    for &x in &d {
        for (k, lim) in [16u32, 1024, 8192, 65536].into_iter().enumerate() {
            d_lt[k] += u32::from(x < lim);
        }
        d_log2_sum += (1.0 + f64::from(x)).log2();
        d_sum += u64::from(x);
    }
    d.sort_unstable();
    let q = |f: f64| d[((d.len() - 1) as f64 * f).round() as usize];
    Rec {
        id,
        n: n as u32,
        p,
        records: records.len() as u32,
        m_kernel: c.retained as u32,
        m,
        runs,
        runs_by_len,
        rows_by_len,
        strips,
        words: c.words as u32,
        bands: c.bands as u32,
        longest: c.longest as u32,
        cycles,
        marked_cycle,
        d_lt,
        d_log2_sum,
        d_sum,
        d_p50: q(0.5),
        d_p90: q(0.9),
    }
}

#[cfg(test)]
mod tests {
    use crate::bwt::{ForcedOpenCells, InverseBwt};
    use crate::runs::Runs;

    /// The census's retained count is the kernel's, and T is one cycle through all m retained
    /// rows on a primitive block
    #[test]
    fn census_matches_the_kernel() {
        let mut text = Vec::new();
        let mut s = 12345u32;
        for _ in 0..200_000 {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let w = 1 + (s >> 16) % 30;
            text.resize(text.len() + w as usize, b'a' + ((s >> 8) % 5) as u8);
        }
        text.truncate(200_000);
        // the BWT of `text` by sorting rotations (small enough here)
        let n = text.len();
        let mut rot: Vec<usize> = (0..n).collect();
        let doubled: Vec<u8> = text.iter().chain(text.iter()).copied().collect();
        rot.sort_by(|&a, &b| doubled[a..a + n].cmp(&doubled[b..b + n]));
        let l: Vec<u8> = rot.iter().map(|&r| doubled[r + n - 1]).collect();
        let p = rot.iter().position(|&r| r == 0).unwrap() as u32;
        let mut out = Vec::new();
        let k = crate::Kernels::detect();
        let _ = super::take();
        assert!(ForcedOpenCells::default().invert_runs(&Runs::of_column(&l), n, p, &mut out, k));
        assert_eq!(out, text);
        let recs: Vec<_> = super::take()
            .into_iter()
            .filter(|r| r.n as usize == n)
            .collect();
        assert_eq!(recs.len(), 1);
        let r = &recs[0];
        assert_eq!(r.m, r.m_kernel);
        assert_eq!(r.cycles, 1);
        assert_eq!(r.marked_cycle, r.m);
        assert_eq!(r.rows_by_len.iter().sum::<u32>() as usize, n);
    }
}
