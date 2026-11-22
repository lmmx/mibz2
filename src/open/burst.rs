//! Mechanisms cycle 29, LeanRegBurst6_256. Keep the six cursors in scalar
//! locals so the hot loops can issue independent loads without spilling an
//! array of lane state between steps.
use super::{long_word, repeat, Kernel, INLINE, LONG};

pub(super) const MIN_ROWS: usize = 140_000;

impl Kernel {
    /// Forward packed links only. On an unsuitable dictionary or marked cycle
    /// the caller can still emit from the same links with the serial walk.
    pub(super) fn emit_burst(
        &mut self,
        m: usize,
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) -> bool {
        let edges = &self.edges[..m];
        let (spelling, inline16, lens) = (&self.spelling, &self.inline16, &self.lens);
        // The scout touches only links and exact word lengths. Build its compact
        // table from the interned dictionary once, on eligible blocks only.
        let word_len = &mut self.word_len;
        word_len.clear();
        word_len.reserve_exact(lens.len());
        word_len.extend(lens.iter().zip(inline16).map(|(&len, slot)| {
            if len == LONG {
                u16::try_from(long_word(slot, spelling).len()).unwrap_or(u16::MAX)
            } else {
                u16::from(len)
            }
        }));
        let m32 = m as u32;
        if word_len.contains(&u16::MAX) {
            return false;
        }

        // All cuts share the low bits of the mark. The remaining eight bits
        // index the stack tables directly, including the cut at the mark.
        let shift = bits.saturating_sub(8);
        let cm = (1u32 << shift) - 1;
        let rem_low = mark & cm;
        let stride = cm + 1;
        let mut cuts = [0u32; 256];
        let mut n_cuts = 0usize;
        for row in (rem_low..m32).step_by(stride as usize) {
            cuts[n_cuts] = row;
            n_cuts += 1;
        }

        let mask = (1u32 << bits) - 1;
        let mut seg_len = [0u32; 256];
        let mut seg_next = [u16::MAX; 256];
        let mut next_c = 6usize;
        let (mut c0, mut c1, mut c2, mut c3, mut c4, mut c5) = (0usize, 1, 2, 3, 4, 5);
        let (mut j0, mut j1, mut j2, mut j3, mut j4, mut j5) =
            (cuts[0], cuts[1], cuts[2], cuts[3], cuts[4], cuts[5]);
        let (mut a0, mut a1, mut a2, mut a3, mut a4, mut a5) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
        let mut guard = 0usize;

        // Each scout measures the bytes up to the next cut. Refill finished lanes
        // in the outer loop so all six dependent loads can overlap in the burst.
        while (c0 & c1 & c2 & c3 & c4 & c5) != usize::MAX {
            if (c0 | c1 | c2 | c3 | c4 | c5) < 256 {
                loop {
                    let e0 = edges[j0 as usize];
                    let e1 = edges[j1 as usize];
                    let e2 = edges[j2 as usize];
                    let e3 = edges[j3 as usize];
                    let e4 = edges[j4 as usize];
                    let e5 = edges[j5 as usize];
                    a0 = a0.wrapping_add(u32::from(word_len[(e0 >> bits) as usize]));
                    a1 = a1.wrapping_add(u32::from(word_len[(e1 >> bits) as usize]));
                    a2 = a2.wrapping_add(u32::from(word_len[(e2 >> bits) as usize]));
                    a3 = a3.wrapping_add(u32::from(word_len[(e3 >> bits) as usize]));
                    a4 = a4.wrapping_add(u32::from(word_len[(e4 >> bits) as usize]));
                    a5 = a5.wrapping_add(u32::from(word_len[(e5 >> bits) as usize]));
                    j0 = e0 & mask;
                    j1 = e1 & mask;
                    j2 = e2 & mask;
                    j3 = e3 & mask;
                    j4 = e4 & mask;
                    j5 = e5 & mask;
                    guard += 6;
                    if (((j0 ^ mark) & cm) == 0)
                        | (((j1 ^ mark) & cm) == 0)
                        | (((j2 ^ mark) & cm) == 0)
                        | (((j3 ^ mark) & cm) == 0)
                        | (((j4 ^ mark) & cm) == 0)
                        | (((j5 ^ mark) & cm) == 0)
                        | (guard > 2 * m + 64)
                    {
                        break;
                    }
                }
                if guard > 2 * m + 64 {
                    return false;
                }
                macro_rules! check_scout {
                    ($ck:ident, $jk:ident, $ak:ident) => {
                        if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                            seg_len[$ck] = $ak;
                            seg_next[$ck] = ($jk >> shift) as u16;
                            if next_c < n_cuts {
                                $ck = next_c;
                                $jk = cuts[next_c];
                                $ak = 0;
                                next_c += 1;
                            } else {
                                $ck = usize::MAX;
                            }
                        }
                    };
                }
                check_scout!(c0, j0, a0);
                check_scout!(c1, j1, a1);
                check_scout!(c2, j2, a2);
                check_scout!(c3, j3, a3);
                check_scout!(c4, j4, a4);
                check_scout!(c5, j5, a5);
            } else {
                macro_rules! step_drain {
                    ($ck:ident, $jk:ident, $ak:ident) => {
                        if $ck != usize::MAX {
                            let e = edges[$jk as usize];
                            $ak = $ak.wrapping_add(u32::from(word_len[(e >> bits) as usize]));
                            $jk = e & mask;
                            guard += 1;
                            if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                                seg_len[$ck] = $ak;
                                seg_next[$ck] = ($jk >> shift) as u16;
                                $ck = usize::MAX;
                            }
                        }
                    };
                }
                step_drain!(c0, j0, a0);
                step_drain!(c1, j1, a1);
                step_drain!(c2, j2, a2);
                step_drain!(c3, j3, a3);
                step_drain!(c4, j4, a4);
                step_drain!(c5, j5, a5);
                if guard > 2 * m + 64 {
                    return false;
                }
            }
        }

        // Stitch just the marked cycle in output order. Each complete segment
        // now has a destination, irrespective of when its emitter finishes.
        let mark_cid = (mark >> shift) as usize;
        let mut visited = [false; 256];
        let mut t_row = [0u32; 256];
        let mut t_start = [0u32; 256];
        let mut t_len = [0u32; 256];
        let mut n_tasks = 0usize;
        let mut cur_cid = mark_cid;
        let mut total_at = 0usize;
        let mut tail_row = None;

        while total_at < n {
            if visited[cur_cid] {
                if cur_cid == mark_cid {
                    break;
                }
                return false;
            }
            visited[cur_cid] = true;
            let slen = seg_len[cur_cid] as usize;
            let nxt = seg_next[cur_cid] as usize;
            if nxt >= n_cuts {
                return false;
            }
            if total_at + slen > n {
                tail_row = Some(cuts[cur_cid]);
                break;
            }
            if slen > 0 {
                t_row[n_tasks] = cuts[cur_cid];
                t_start[n_tasks] = total_at as u32;
                t_len[n_tasks] = slen as u32;
                n_tasks += 1;
                total_at += slen;
            }
            cur_cid = nxt;
            if cur_cid == mark_cid {
                break;
            }
        }

        if n_tasks < 6 {
            return false;
        }

        out.resize(n + INLINE, 0);
        let mut next_t = 6usize;
        let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5) =
            (t_row[0], t_row[1], t_row[2], t_row[3], t_row[4], t_row[5]);
        let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5) = (
            t_start[0] as usize,
            t_start[1] as usize,
            t_start[2] as usize,
            t_start[3] as usize,
            t_start[4] as usize,
            t_start[5] as usize,
        );
        let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5) =
            (t_len[0], t_len[1], t_len[2], t_len[3], t_len[4], t_len[5]);

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
                                    let word = long_word(&inline16[$kk], spelling);
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
                            $atk = t_start[next_t] as usize;
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
                                let word = long_word(&inline16[k], spelling);
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

        // An inline store may overwrite the next segment's first 15 bytes.
        // Replay cut starts with exact stores after all lanes have finished.
        for t in 1..n_tasks {
            let mut rj = t_row[t];
            let mut rat = t_start[t] as usize;
            let rend = (rat + INLINE - 1).min(total_at);
            while rat < rend {
                let e = edges[rj as usize];
                let k = (e >> bits) as usize;
                rj = e & mask;
                let l = lens[k];
                let w = if l != LONG {
                    &inline16[k][..usize::from(l)]
                } else {
                    long_word(&inline16[k], spelling)
                };
                let tk = w.len().min(rend - rat);
                out[rat..rat + tk].copy_from_slice(&w[..tk]);
                rat += tk;
            }
        }

        if let Some(mut rj) = tail_row {
            let mut rat = total_at;
            for _ in 0..m {
                if rat >= n {
                    break;
                }
                let e = edges[rj as usize];
                let k = (e >> bits) as usize;
                rj = e & mask;
                let l = lens[k];
                if l != LONG {
                    out[rat..rat + INLINE].copy_from_slice(&inline16[k]);
                    rat += usize::from(l);
                } else {
                    let word = long_word(&inline16[k], spelling);
                    let tk = word.len().min(n - rat);
                    out[rat..rat + tk].copy_from_slice(&word[..tk]);
                    rat += tk;
                }
                total_at = rat;
                if rj == mark {
                    break;
                }
            }
        }

        out.truncate(total_at.min(n));
        repeat(n, out);
        true
    }
}
