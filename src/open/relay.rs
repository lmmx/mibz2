//! Cycle 26's direct emitter with five length scouts, then six emitters.
//! Each cut is claimed at most once, yielding at most one deferred task.
use super::{repeat, INLINE, LONG};
pub(super) const MIN_ROWS: usize = 135_000;
const CUTS: usize = 256;
#[allow(clippy::too_many_arguments)]
pub(super) fn emit(
    edges: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    word_len: &[u16],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    let m = edges.len();
    let m32 = m as u32;
    let shift = bits.saturating_sub(8);
    let cm = (1u32 << shift) - 1;
    let rem_low = mark & cm;
    let stride = cm + 1;
    let n_cuts = ((m32 - 1 - rem_low) / stride + 1) as usize;
    let cut = |id: usize| (id as u32) * stride + rem_low;

    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut state = [0u8; CUTS];
    let mut seg_len = [0u32; CUTS];
    let mut seg_next = [0u16; CUTS];

    let mark_cid = (mark >> shift) as usize;
    state[mark_cid] = 3;

    let mut next_c = 0usize;
    let mut sc_cid = [usize::MAX; 5];
    let mut active = 0usize;
    let (mut j1, mut j2, mut j3, mut j4, mut j5) = (0u32, 0, 0, 0, 0);
    let (mut a1, mut a2, mut a3, mut a4, mut a5) = (0u32, 0, 0, 0, 0);

    macro_rules! claim_cut {
        ($idx:expr, $jk:ident, $ak:ident) => {
            while next_c < n_cuts {
                let c = next_c;
                next_c += 1;
                if state[c] == 0 {
                    state[c] = 1;
                    sc_cid[$idx] = c;
                    $jk = cut(c);
                    $ak = 0;
                    active += 1;
                    break;
                }
            }
        };
    }
    claim_cut!(0, j1, a1);
    claim_cut!(1, j2, a2);
    claim_cut!(2, j3, a3);
    claim_cut!(3, j4, a4);
    claim_cut!(4, j5, a5);

    let mut t_row = [0u32; CUTS];
    let mut t_start = [0usize; CUTS];
    let mut t_len = [0u32; CUTS];
    let mut n_tasks = 0usize;

    let mut j0 = mark;
    let mut at0 = 0usize;
    let mut steps0 = 0usize;

    'stage1: loop {
        if active == 5 && at0 < n && steps0 < m {
            // Fast 6-lane scalar-register burst: Lane 0 emits directly, Lanes 1..5 scout!
            loop {
                let e0 = edges[j0 as usize];
                let e1 = edges[j1 as usize];
                let e2 = edges[j2 as usize];
                let e3 = edges[j3 as usize];
                let e4 = edges[j4 as usize];
                let e5 = edges[j5 as usize];

                let k0 = (e0 >> bits) as usize;
                j0 = e0 & mask;
                let l0 = lens[k0];
                if l0 != LONG {
                    out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                    at0 += usize::from(l0);
                } else {
                    let w = k0 - 256;
                    let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                    let tk = word.len().min(n - at0);
                    out[at0..at0 + tk].copy_from_slice(&word[..tk]);
                    at0 += tk;
                }

                a1 += u32::from(word_len[(e1 >> bits) as usize]);
                a2 += u32::from(word_len[(e2 >> bits) as usize]);
                a3 += u32::from(word_len[(e3 >> bits) as usize]);
                a4 += u32::from(word_len[(e4 >> bits) as usize]);
                a5 += u32::from(word_len[(e5 >> bits) as usize]);
                j1 = e1 & mask;
                j2 = e2 & mask;
                j3 = e3 & mask;
                j4 = e4 & mask;
                j5 = e5 & mask;
                steps0 += 1;

                if (((j0 ^ mark) & cm) == 0)
                    | (((j1 ^ mark) & cm) == 0)
                    | (((j2 ^ mark) & cm) == 0)
                    | (((j3 ^ mark) & cm) == 0)
                    | (((j4 ^ mark) & cm) == 0)
                    | (((j5 ^ mark) & cm) == 0)
                    | (at0 >= n)
                    | (steps0 >= m)
                {
                    break;
                }
            }
        } else {
            // Drain / pure-Lane-0 tail when < 5 scouts remain (zero dummy loads!)
            if steps0 >= m || at0 >= n {
                break 'stage1;
            }
            let e0 = edges[j0 as usize];
            let k0 = (e0 >> bits) as usize;
            j0 = e0 & mask;
            let l0 = lens[k0];
            if l0 != LONG {
                out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                at0 += usize::from(l0);
            } else {
                let w = k0 - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let tk = word.len().min(n - at0);
                out[at0..at0 + tk].copy_from_slice(&word[..tk]);
                at0 += tk;
            }
            steps0 += 1;

            if active > 0 {
                macro_rules! step_one_scout {
                    ($idx:expr, $jk:ident, $ak:ident) => {
                        if sc_cid[$idx] != usize::MAX {
                            let es = edges[$jk as usize];
                            $ak += u32::from(word_len[(es >> bits) as usize]);
                            $jk = es & mask;
                        }
                    };
                }
                step_one_scout!(0, j1, a1);
                step_one_scout!(1, j2, a2);
                step_one_scout!(2, j3, a3);
                step_one_scout!(3, j4, a4);
                step_one_scout!(4, j5, a5);
            }
        }

        // Retire any scouts that hit a cut
        if active > 0 {
            macro_rules! retire_scout {
                ($idx:expr, $jk:ident, $ak:ident) => {
                    if sc_cid[$idx] != usize::MAX && ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                        let cid = sc_cid[$idx];
                        state[cid] = 2;
                        seg_len[cid] = $ak;
                        seg_next[cid] = ($jk >> shift) as u16;
                        sc_cid[$idx] = usize::MAX;
                        active -= 1;
                        claim_cut!($idx, $jk, $ak);
                    }
                };
            }
            retire_scout!(0, j1, a1);
            retire_scout!(1, j2, a2);
            retire_scout!(2, j3, a3);
            retire_scout!(3, j4, a4);
            retire_scout!(4, j5, a5);
        }

        if j0 == mark || at0 >= n || steps0 >= m {
            break 'stage1;
        }

        // Check if Lane 0 hit a cut: hop all SCOUTED cuts or split an active scout!
        if ((j0 ^ mark) & cm) == 0 && j0 < m32 {
            loop {
                let cid = (j0 >> shift) as usize;
                let st = state[cid];
                if st == 2 && at0 + (seg_len[cid] as usize) <= n {
                    if seg_len[cid] > 0 {
                        t_row[n_tasks] = cut(cid);
                        t_start[n_tasks] = at0;
                        t_len[n_tasks] = seg_len[cid];
                        n_tasks += 1;
                        at0 += seg_len[cid] as usize;
                    }
                    state[cid] = 3;
                    j0 = cut(seg_next[cid] as usize);
                    if j0 == mark || at0 >= n {
                        break 'stage1;
                    }
                } else if st == 1 {
                    macro_rules! try_split {
                        ($idx:expr, $jk:ident, $ak:ident) => {
                            if sc_cid[$idx] == cid {
                                if $ak > 0 && at0 + ($ak as usize) <= n {
                                    t_row[n_tasks] = cut(cid);
                                    t_start[n_tasks] = at0;
                                    t_len[n_tasks] = $ak;
                                    n_tasks += 1;
                                    at0 += $ak as usize;
                                    j0 = $jk;
                                }
                                state[cid] = 3;
                                sc_cid[$idx] = usize::MAX;
                                active -= 1;
                                claim_cut!($idx, $jk, $ak);
                            }
                        };
                    }
                    try_split!(0, j1, a1);
                    try_split!(1, j2, a2);
                    try_split!(2, j3, a3);
                    try_split!(3, j4, a4);
                    try_split!(4, j5, a5);
                    if j0 == mark || at0 >= n {
                        break 'stage1;
                    }
                    break;
                } else {
                    state[cid] = 3;
                    break;
                }
            }
        }
    }

    let total_at = at0.min(n);

    // === Stage 2: 6-Lane Scalar-Register Sloppy Emit Burst over Hopped Tasks ===
    if n_tasks > 0 {
        let mut saved = [[0u8; INLINE]; CUTS];
        for t in 0..n_tasks {
            let ea = t_start[t] + t_len[t] as usize;
            if ea < total_at {
                saved[t].copy_from_slice(&out[ea..ea + INLINE]);
            }
        }

        let init = n_tasks.min(6);
        let mut next_t = init;
        let mut rj = [0u32; 6];
        let mut at = [0usize; 6];
        let mut rem = [0u32; 6];
        rj[..init].copy_from_slice(&t_row[..init]);
        at[..init].copy_from_slice(&t_start[..init]);
        rem[..init].copy_from_slice(&t_len[..init]);
        let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5) =
            (rj[0], rj[1], rj[2], rj[3], rj[4], rj[5]);
        let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5) =
            (at[0], at[1], at[2], at[3], at[4], at[5]);
        let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5) =
            (rem[0], rem[1], rem[2], rem[3], rem[4], rem[5]);

        while (rem0 | rem1 | rem2 | rem3 | rem4 | rem5) != 0 {
            if (rem0 != 0) & (rem1 != 0) & (rem2 != 0) & (rem3 != 0) & (rem4 != 0) & (rem5 != 0) {
                loop {
                    let e0 = edges[rj0 as usize];
                    let e1 = edges[rj1 as usize];
                    let e2 = edges[rj2 as usize];
                    let e3 = edges[rj3 as usize];
                    let e4 = edges[rj4 as usize];
                    let e5 = edges[rj5 as usize];
                    let k0 = (e0 >> bits) as usize;
                    let k1 = (e1 >> bits) as usize;
                    let k2 = (e2 >> bits) as usize;
                    let k3 = (e3 >> bits) as usize;
                    let k4 = (e4 >> bits) as usize;
                    let k5 = (e5 >> bits) as usize;
                    let (l0, l1, l2, l3, l4, l5) =
                        (lens[k0], lens[k1], lens[k2], lens[k3], lens[k4], lens[k5]);
                    if ((l0 | l1 | l2 | l3 | l4 | l5) & 0x80) != 0 {
                        macro_rules! step_cold {
                            ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                                $rjk = $ek & mask;
                                if $lk != LONG {
                                    out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
                                    $atk += usize::from($lk);
                                    $remk = $remk.saturating_sub(u32::from($lk));
                                } else {
                                    let w = $kk - 256;
                                    let word =
                                        &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                                    let tk = word.len().min($remk as usize);
                                    out[$atk..$atk + tk].copy_from_slice(&word[..tk]);
                                    $atk += tk;
                                    $remk = $remk.saturating_sub(tk as u32);
                                }
                            };
                        }
                        step_cold!(e0, k0, l0, rj0, at0, rem0);
                        step_cold!(e1, k1, l1, rj1, at1, rem1);
                        step_cold!(e2, k2, l2, rj2, at2, rem2);
                        step_cold!(e3, k3, l3, rj3, at3, rem3);
                        step_cold!(e4, k4, l4, rj4, at4, rem4);
                        step_cold!(e5, k5, l5, rj5, at5, rem5);
                        break;
                    }
                    rj0 = e0 & mask;
                    out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                    at0 += usize::from(l0);
                    rem0 -= u32::from(l0);

                    rj1 = e1 & mask;
                    out[at1..at1 + INLINE].copy_from_slice(&inline16[k1]);
                    at1 += usize::from(l1);
                    rem1 -= u32::from(l1);

                    rj2 = e2 & mask;
                    out[at2..at2 + INLINE].copy_from_slice(&inline16[k2]);
                    at2 += usize::from(l2);
                    rem2 -= u32::from(l2);

                    rj3 = e3 & mask;
                    out[at3..at3 + INLINE].copy_from_slice(&inline16[k3]);
                    at3 += usize::from(l3);
                    rem3 -= u32::from(l3);

                    rj4 = e4 & mask;
                    out[at4..at4 + INLINE].copy_from_slice(&inline16[k4]);
                    at4 += usize::from(l4);
                    rem4 -= u32::from(l4);

                    rj5 = e5 & mask;
                    out[at5..at5 + INLINE].copy_from_slice(&inline16[k5]);
                    at5 += usize::from(l5);
                    rem5 -= u32::from(l5);

                    if (rem0 == 0)
                        | (rem1 == 0)
                        | (rem2 == 0)
                        | (rem3 == 0)
                        | (rem4 == 0)
                        | (rem5 == 0)
                    {
                        break;
                    }
                }
                macro_rules! refill_emit {
                    ($rjk:ident, $atk:ident, $remk:ident) => {
                        if $remk == 0 && next_t < n_tasks {
                            $rjk = t_row[next_t];
                            $atk = t_start[next_t];
                            $remk = t_len[next_t];
                            next_t += 1;
                        }
                    };
                }
                refill_emit!(rj0, at0, rem0);
                refill_emit!(rj1, at1, rem1);
                refill_emit!(rj2, at2, rem2);
                refill_emit!(rj3, at3, rem3);
                refill_emit!(rj4, at4, rem4);
                refill_emit!(rj5, at5, rem5);
            } else {
                macro_rules! drain_emit {
                    ($rjk:ident, $atk:ident, $remk:ident) => {
                        if $remk != 0 {
                            let e = edges[$rjk as usize];
                            let k = (e >> bits) as usize;
                            $rjk = e & mask;
                            let l = lens[k];
                            if l != LONG {
                                out[$atk..$atk + INLINE].copy_from_slice(&inline16[k]);
                                $atk += usize::from(l);
                                $remk = $remk.saturating_sub(u32::from(l));
                            } else {
                                let w = k - 256;
                                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                                let tk = word.len().min($remk as usize);
                                out[$atk..$atk + tk].copy_from_slice(&word[..tk]);
                                $atk += tk;
                                $remk = $remk.saturating_sub(tk as u32);
                            }
                        }
                    };
                }
                drain_emit!(rj0, at0, rem0);
                drain_emit!(rj1, at1, rem1);
                drain_emit!(rj2, at2, rem2);
                drain_emit!(rj3, at3, rem3);
                drain_emit!(rj4, at4, rem4);
                drain_emit!(rj5, at5, rem5);
            }
        }

        // Restore Lane-0 regions after task ends, then task starts
        for t in 0..n_tasks {
            let ea = t_start[t] + t_len[t] as usize;
            if ea < total_at {
                let limit_at = if t + 1 < n_tasks {
                    t_start[t + 1]
                } else {
                    total_at
                };
                let keep = (limit_at - ea).min(INLINE);
                out[ea..ea + keep].copy_from_slice(&saved[t][..keep]);
            }
        }
        for t in 0..n_tasks {
            let t_end = t_start[t] + t_len[t] as usize;
            let mut rj = t_row[t];
            let mut rat = t_start[t];
            let rend = t_end.min(rat + INLINE);
            while rat < rend {
                let e = edges[rj as usize];
                let k = (e >> bits) as usize;
                rj = e & mask;
                let l = lens[k];
                let w = if l != LONG {
                    &inline16[k][..usize::from(l)]
                } else {
                    let wi = k - 256;
                    &spelling[offsets[wi] as usize..offsets[wi + 1] as usize]
                };
                let tk = w.len().min(t_end - rat);
                out[rat..rat + tk].copy_from_slice(&w[..tk]);
                rat += tk;
            }
        }
    }

    out.truncate(total_at.min(n));
    repeat(n, out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open::{emit_reference, inline_words};

    #[test]
    fn segment_boundaries_and_meetings() {
        // Nonuniform words make a misplaced boundary visible, including
        // unconditional 16-byte copies next to a 1-byte or a long word.
        let spelling: Vec<u8> = (0..53).collect();
        let offsets = [0, 3, 19, 53];
        let (mut inline, mut lens) = (vec![], vec![]);
        inline_words(&spelling, &offsets, &mut inline, &mut lens);
        for (m, stride) in [
            (1u32, 1),
            (2, 1),
            (7, 3),
            (135_001, 1024),
            (135_168, 1024),
            (135_000, 1),
        ] {
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            for mark in [0, m / 2, m - 1] {
                let ids = [90, 256, 257, 258, 65, 66, 67];
                let ordinary: Vec<_> = (0..m)
                    .map(|u| (ids[u as usize % ids.len()] << bits) | ((u + stride) % m))
                    .collect();
                let cycle = m / gcd(m, stride);
                let bytes: usize = (0..cycle)
                    .map(|i| {
                        let row =
                            (u64::from(mark) + u64::from(i) * u64::from(stride)) % u64::from(m);
                        match ids[row as usize % ids.len()] {
                            256 => 3,
                            257 => 16,
                            258 => 34,
                            _ => 1,
                        }
                    })
                    .sum();
                let mut lengths = vec![1u16; 256];
                lengths.extend([3, 16, 34]);
                for n in [
                    1,
                    15,
                    16,
                    17,
                    31,
                    32,
                    33,
                    bytes.saturating_sub(1).max(1),
                    bytes,
                    bytes + 1,
                    2 * bytes + 3,
                ] {
                    let mut want = Vec::new();
                    emit_reference(
                        &ordinary,
                        &[],
                        &spelling,
                        &offsets,
                        n,
                        mark,
                        bits,
                        &mut want,
                    );
                    let mut out = vec![0xa5; n + INLINE];
                    emit(
                        &ordinary, &spelling, &offsets, &inline, &lens, &lengths, n, mark, bits,
                        &mut out,
                    );
                    assert!(out == want, "m={m} stride={stride} mark={mark} n={n}");
                }
            }
        }
    }

    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            gcd(b, a % b)
        }
    }
}
