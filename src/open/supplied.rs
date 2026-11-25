//! (research only, `supplied-codes`) The open kernel's emit stage on the block's own word codes
//! supplied in marked-cycle output order, for the bench-supplied-codes diagnostic
//! (docs/journal/2026-11-24-supplied-codes-registration.md; the switch, the store and the
//! records are `src/supplied_codes.rs`').
//!
//! Arm B keeps everything of arm A's but the walk: the construction as shipped, then the codes
//! q_0..q_{L-1} of the marked cycle (q_i the dictionary code of the i-th retained row the walk
//! from the marked row visits, L the cycle's length) copied into `edges[..L]`, the packed-edge
//! buffer the walk would have read, and consumed in index order: from the front alone where
//! arm A runs `emit`, from both ends where it runs `emit_bidirectional`. The appends (the
//! 16-byte inline store, its shifted twin at the back, the long words' slices from `spelling`,
//! the clamp to `n`, the short cycle's `copy_within` and `repeat`) are those two functions'
//! own, unchanged; only where the next code comes from differs.
use super::*;
use crate::supplied_codes::{self as sup, Key, Mode, Rec};
use std::time::Instant;

impl Kernel {
    /// `try_run` under the mode now set (`supplied_codes::mode`), its block recorded
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn supplied_run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<Counts> {
        self.staged::<INDUCE, QUEUE>(
            sup::mode(),
            sup::condition(),
            records,
            n,
            p,
            limit,
            cap,
            out,
        )
    }

    /// `try_run` under `mode`, conditioned or not: construct | deliver | condition | consume,
    /// five clock reads in every mode
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn staged<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        mode: Mode,
        cond: bool,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<Counts> {
        let t0 = Instant::now();
        let Some((stats, mark, bits)) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)
        else {
            sup::record(Rec {
                n: n as u32,
                m: 0,
                cycle: 0,
                mode,
                cond,
                bidi: false,
                refused: true,
                supplied: false,
                construct_ns: sup::ns(t0.elapsed()),
                deliver_ns: 0,
                cond_ns: 0,
                consume_ns: 0,
                scratch: 0,
                id: 0,
            });
            return None;
        };
        let t1 = Instant::now();
        let m = stats.retained;
        let key = Key {
            n: n as u32,
            p,
            m: m as u32,
            mark,
            records: records.len() as u32,
            first: records[0],
            last: records[records.len() - 1],
        };
        // arm B's delivery: the lookup and the copy into `edges[..L]`, charged
        let mut cycle = 0;
        if mode == Mode::Supplied {
            cycle = sup::deliver(&key, &mut self.edges[..m]).unwrap_or(0);
        }
        let t2 = Instant::now();
        if cond {
            std::hint::black_box(self.condition(m));
        }
        let t3 = Instant::now();
        let mut supplied = false;
        match mode {
            Mode::Open => self.walk(n, m, mark, bits, out),
            Mode::Supplied => {
                if cycle > 0 {
                    self.consume(n, cycle, out);
                    supplied = true;
                } else {
                    // nothing held: the walk as shipped (`edges` untouched), the record says so
                    self.walk(n, m, mark, bits, out);
                }
            }
            Mode::Record => {
                let codes = self.cycle_codes(m, mark, bits);
                cycle = codes.len();
                sup::store(key, &codes);
                self.walk(n, m, mark, bits, out);
            }
            Mode::Check => {
                let own = self.cycle_codes(m, mark, bits);
                cycle = own.len();
                self.walk(n, m, mark, bits, out);
                let want = out.clone();
                match (sup::held(&key), sup::deliver(&key, &mut self.edges[..m])) {
                    (Some(held), Some(l)) => {
                        supplied = true;
                        self.consume(n, l, out);
                        if held != own || *out != want {
                            sup::mismatch();
                        }
                    }
                    _ => sup::mismatch(),
                }
            }
        }
        let t4 = Instant::now();
        sup::record(Rec {
            n: n as u32,
            m: m as u32,
            cycle: cycle as u32,
            mode,
            cond,
            bidi: self.predecessor.is_some(),
            refused: false,
            supplied,
            construct_ns: sup::ns(t1 - t0),
            deliver_ns: sup::ns(t2 - t1),
            cond_ns: sup::ns(t3 - t2),
            consume_ns: sup::ns(t4 - t3),
            scratch: self.scratch_bytes() as u64,
            id: key.id(),
        });
        Some(stats)
    }

    /// The marked cycle's codes in output order, walked once from the marked row (the oracle,
    /// outside the clock): at most `m` steps, as the emitters bound theirs
    fn cycle_codes(&self, m: usize, mark: u32, bits: u32) -> Vec<u32> {
        let edges = &self.edges[..m];
        let mask = (1u32 << bits) - 1;
        let mut codes = Vec::new();
        if let Some(before) = self.predecessor {
            let (mut prev, mut j) = (before, mark);
            for _ in 0..m {
                let e = edges[j as usize];
                codes.push(e >> bits);
                (prev, j) = (j, (e & mask) ^ prev);
                if j == mark {
                    break;
                }
            }
        } else {
            let mut j = mark;
            for _ in 0..m {
                let e = edges[j as usize];
                codes.push(if self.extra.is_empty() {
                    e >> bits
                } else {
                    self.extra[j as usize]
                });
                j = e & mask;
                if j == mark {
                    break;
                }
            }
        }
        codes
    }

    /// Arm B's consumer on the `l` codes delivered into `edges`: from both ends where arm A's
    /// walk is two-ended, else from the front
    fn consume(&self, n: usize, l: usize, out: &mut Vec<u8>) {
        let codes = &self.edges[..l];
        if self.predecessor.is_some() {
            emit_bidirectional_codes(codes, &self.spelling, &self.inline16, &self.lens, n, out);
        } else {
            emit_codes(codes, &self.spelling, &self.inline16, &self.lens, n, out);
        }
    }

    /// The conditioning read (both arms, outside the consumer's clock): every buffer either
    /// consumer may read, sequentially, one load a 64-byte line
    fn condition(&self, m: usize) -> u64 {
        let mut s = 0u64;
        for &e in self.edges[..m].iter().step_by(16) {
            s = s.wrapping_add(u64::from(e));
        }
        for &e in self.extra.iter().step_by(16) {
            s = s.wrapping_add(u64::from(e));
        }
        for w in self.inline16.iter().step_by(4) {
            s = s.wrapping_add(u64::from(w[0]));
        }
        for &b in self.lens.iter().step_by(64) {
            s = s.wrapping_add(u64::from(b));
        }
        for &b in self.spelling.iter().step_by(64) {
            s = s.wrapping_add(u64::from(b));
        }
        s
    }

    /// The scratch the kernel keeps, in bytes, across every buffer it owns (b5's `scratch`)
    fn scratch_bytes(&self) -> usize {
        use std::mem::size_of;
        self.spill.scratch()
            + 4 * self.directory.capacity()
            + size_of::<Tile>() * self.tiles.capacity()
            + size_of::<Band>() * self.bands.capacity()
            + self.spelling.capacity()
            + 4 * (self.offsets.capacity() + self.edges.capacity() + self.extra.capacity())
            + INLINE * self.inline16.capacity()
            + self.lens.capacity()
            + self.endpoint.scratch()
    }
}

/// `emit` with the next code read from `codes` in order instead of walked from the marked row
fn emit_codes(
    codes: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0;
    for &c in codes {
        let code = c as usize;
        let len = lens[code];
        if len != LONG {
            // `at < n` here, so the copy ends within `n + 16`
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let word = long_word(&inline16[code], spelling);
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        if at >= n {
            break;
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

/// `emit_bidirectional` with the front's next code read from `codes` upward from 0 and the
/// back's downward from `L - 1`, instead of each walked through the XOR links. The front and
/// the back meet where the walk's would: `f == b` is its `front == back`, `f > b` its
/// `front == next`
fn emit_bidirectional_codes(
    codes: &[u32],
    spelling: &[u8],
    inline: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let (mut lo, mut hi) = (0, n);
    let (mut f, mut b) = (0usize, codes.len() - 1);
    for _ in 0..=codes.len() / 2 {
        if lo == hi || f > b {
            break;
        }
        let cf = codes[f] as usize;
        let single = f == b;
        let lf = lens[cf];
        if lf != LONG && hi - lo >= INLINE {
            out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
            lo += usize::from(lf);
        } else {
            let word = if lf == LONG {
                long_word(&inline[cf], spelling)
            } else {
                &inline[cf][..usize::from(lf)]
            };
            let take = word.len().min(hi - lo);
            out[lo..lo + take].copy_from_slice(&word[..take]);
            lo += take;
        }
        if single {
            break;
        }
        let cb = codes[b] as usize;
        let lb = lens[cb];
        if lb != LONG && hi - lo >= INLINE {
            let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
            out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
            hi -= usize::from(lb);
        } else {
            let word = if lb == LONG {
                long_word(&inline[cb], spelling)
            } else {
                &inline[cb][..usize::from(lb)]
            };
            let take = word.len().min(hi - lo);
            out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
            hi -= take;
        }
        f += 1;
        b -= 1;
    }
    if lo < hi {
        out.copy_within(hi..n, lo);
        out.truncate(lo + (n - hi));
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

#[cfg(test)]
mod tests {
    use super::super::b5::{plain, runs_column, Lcg};
    use super::*;

    /// Column `l` at row `p` under `mode`, through the shipped instance, into an output
    /// poisoned first: (bytes, two-ended)
    fn under(k: &mut Kernel, mode: Mode, cond: bool, l: &[u8], p: u32) -> (Vec<u8>, bool) {
        let r = Runs::of_column(l);
        let mut out: Vec<u8> = plain(l, p).iter().map(|b| !b).collect();
        k.staged::<true, true>(
            mode,
            cond,
            r.records(),
            l.len(),
            p,
            LIMIT,
            Cap::NONE,
            &mut out,
        )
        .expect("no cap");
        (out, k.predecessor.is_some())
    }

    /// Arm A, the oracle pass, arm B (plain and conditioned) and arm B again on a kernel that
    /// last ran another block: every one the plain inverse's bytes. Returns (two-ended, the
    /// cycle shorter than the retained rows)
    fn check(k: &mut Kernel, l: &[u8], p: u32, what: &str) -> (bool, bool) {
        let want = plain(l, p);
        let (a, bidi) = under(k, Mode::Open, false, l, p);
        assert!(a == want, "{what}: arm A");
        let (r, bidi_r) = under(k, Mode::Record, false, l, p);
        assert!(r == want, "{what}: the oracle pass");
        for cond in [false, true] {
            let (b, bidi_b) = under(k, Mode::Supplied, cond, l, p);
            assert!(b == want, "{what}: arm B (conditioned {cond})");
            assert_eq!((bidi, bidi), (bidi_r, bidi_b), "{what}: two-ended");
        }
        // the cycle's length against the retained rows, from the store
        let runs = Runs::of_column(l);
        let recs = runs.records();
        let mut probe: Vec<u8> = Vec::new();
        let (stats, mark, bits) = k
            .construct_capped::<true, true>(recs, l.len() as u32, p, LIMIT, Cap::NONE, &mut probe)
            .expect("no cap");
        let codes = k.cycle_codes(stats.retained, mark, bits);
        let key = Key {
            n: l.len() as u32,
            p,
            m: stats.retained as u32,
            mark,
            records: recs.len() as u32,
            first: recs[0],
            last: recs[recs.len() - 1],
        };
        assert_eq!(sup::held(&key).as_deref(), Some(&codes[..]), "{what}: held");
        (bidi, codes.len() < stats.retained)
    }

    /// Run-heavy columns past and below 65,536 retained rows, at many rows `p`: arbitrary
    /// columns, so the marked cycle is anything from 1 row to all of them (the short cycle's
    /// repeat and clamp), arm B the plain inverse's bytes
    #[test]
    fn arm_b_is_the_plain_inverse_on_arbitrary_columns() {
        let mut k = Kernel::default();
        let mut g = Lcg(0x5c0d_e5c0);
        let (mut bidi, mut short) = (0, 0);
        for case in 0..32 {
            let n = [70_000, 150_000, 300_000, 900_000][case % 4];
            let mean = [1, 2, 3, 8][(case / 4) % 4];
            let alphabet = 2 + g.below(60);
            let l = runs_column(&mut g, n, alphabet, mean);
            let p = g.below(n as u32);
            let (b, s) = check(&mut k, &l, p, &format!("case {case} n {n} p {p}"));
            bidi += usize::from(b);
            short += usize::from(s);
        }
        assert!(
            bidi >= 10 && short >= 5,
            "{bidi} two-ended, {short} short cycles"
        );
    }

    /// Small columns (forward blocks), one byte, two bytes alternating, every row of a few tiny
    /// columns: the shortest cycles and the edge rows
    #[test]
    fn short_cycles_and_edge_inputs() {
        let mut k = Kernel::default();
        let mut g = Lcg(0x5107_0001);
        let mut cases: Vec<(Vec<u8>, String)> = vec![
            (vec![7u8; 1], "one row".into()),
            (vec![7u8; 1000], "one byte".into()),
            (b"ab".repeat(500), "ab".into()),
            (b"ba".repeat(5000), "ba".into()),
            (b"abc".repeat(3000), "abc".into()),
        ];
        for n in [2usize, 3, 5, 17, 64, 1000] {
            let l: Vec<u8> = (0..n).map(|_| g.below(3) as u8).collect();
            cases.push((l, format!("random ternary {n}")));
        }
        for (l, what) in &cases {
            let n = l.len() as u32;
            for p in [0, n / 2, n - 1] {
                check(&mut k, l, p, &format!("{what} p {p}"));
            }
        }
    }

    /// A real BWT (one long cycle, two-ended) and its periodic powers (each byte of the column
    /// repeated: the cycle a k-th of the retained rows, so `copy_within` and `repeat`)
    #[test]
    fn long_cycles_and_periodic_blocks() {
        let mut k = Kernel::default();
        let mut g = Lcg(0x000b_10c5);
        let mut short = 0;
        for (i, n) in [3_000usize, 66_000, 131_072].into_iter().enumerate() {
            let text: Vec<u8> = (0..n).map(|_| b'a' + g.below(26) as u8).collect();
            let (l, p) = bwt(&text);
            assert_eq!(plain(&l, p), text);
            check(&mut k, &l, p, &format!("bwt n {n}"));
            let reps = 2 + i;
            let lp: Vec<u8> = l
                .iter()
                .flat_map(|&c| std::iter::repeat_n(c, reps))
                .collect();
            let (_, s) = check(&mut k, &lp, p * reps as u32, &format!("bwt n {n} x{reps}"));
            short += usize::from(s);
        }
        assert!(short > 0);
    }

    /// More words than the packed links hold beside the ranks (`extra`'s codes, a forward
    /// block even past 65,536 retained rows): full blocks of runs around 20 over 250 bytes
    #[test]
    fn wide_codes() {
        let n = 900_000usize;
        let l = runs_column(&mut Lcg(9), n, 250, 20);
        let mut k = Kernel::default();
        let mut wide = 0;
        for p in [0, n as u32 / 3, n as u32 - 1] {
            let (bidi, _) = check(&mut k, &l, p, &format!("wide p {p}"));
            wide += usize::from(!bidi && !k.extra.is_empty());
        }
        assert_eq!(wide, 3);
    }

    /// Synthetic tables at every marked-cycle length 1..=70 and some long ones, words of every
    /// length class, repeated to `n`: the consumers write exactly the walks' bytes
    #[test]
    fn consumers_match_the_walks_at_every_cycle_length() {
        let mut g = Lcg(0x0dd5);
        let mut spelling = Vec::new();
        let mut offsets = vec![0u32];
        for w in 0..320 {
            let len = if w < 300 { 2 + w % 15 } else { 17 + w % 24 };
            spelling.extend((0..len).map(|_| b'a' + g.below(26) as u8));
            offsets.push(spelling.len() as u32);
        }
        let (mut inline, mut lens) = (Vec::new(), Vec::new());
        inline_words(&spelling, &offsets, &mut inline, &mut lens);
        let ncodes = lens.len() as u32;
        let mut lengths: Vec<usize> = (1..=70).collect();
        lengths.extend([127, 128, 129, 1001, 4096]);
        for (case, &cyc) in lengths.iter().enumerate() {
            let m = 5_000.max(cyc + g.below(300) as usize);
            let mut order: Vec<u32> = (0..m as u32).collect();
            for i in (1..m).rev() {
                order.swap(i, g.below(i as u32 + 1) as usize);
            }
            let mut tnext = vec![0u32; m];
            for i in 0..cyc {
                tnext[order[i] as usize] = order[(i + 1) % cyc];
            }
            for i in cyc..m {
                let next = if i + 1 < m { i + 1 } else { cyc };
                tnext[order[i] as usize] = order[next];
            }
            let mut tprev = vec![0u32; m];
            for (u, &v) in tnext.iter().enumerate() {
                tprev[v as usize] = u as u32;
            }
            let bits = (u32::BITS - (m as u32 - 1).leading_zeros()).max(1);
            let code: Vec<u32> = (0..m)
                .map(|_| match g.below(10) {
                    0..=4 => g.below(256),
                    5..=8 => 256 + g.below(300),
                    _ => 556 + g.below(ncodes - 556),
                })
                .collect();
            let mark = order[0];
            let before = tprev[mark as usize];
            let period: usize = (0..cyc)
                .map(|i| {
                    let c = code[order[i] as usize] as usize;
                    match lens[c] {
                        LONG => (offsets[c - 255] - offsets[c - 256]) as usize,
                        x => usize::from(x),
                    }
                })
                .sum();
            let supplied: Vec<u32> = (0..cyc).map(|i| code[order[i] as usize]).collect();
            for reps in [1, 2 + case % 3] {
                // a clamp inside the cycle too: n short of the period
                for n in [period * reps, (period * reps).saturating_sub(5).max(1)] {
                    let xor: Vec<u32> = (0..m)
                        .map(|j| code[j] << bits | (tnext[j] ^ tprev[j]))
                        .collect();
                    let mut want = vec![0xEE; n + INLINE];
                    emit_bidirectional(
                        &xor, &spelling, &inline, &lens, n, mark, before, bits, &mut want,
                    );
                    let mut got = vec![0x11; n + INLINE];
                    emit_bidirectional_codes(&supplied, &spelling, &inline, &lens, n, &mut got);
                    assert!(got == want, "two-ended: cycle {cyc} m {m} n {n}");
                    let fwd: Vec<u32> = (0..m).map(|j| code[j] << bits | tnext[j]).collect();
                    let mut want = vec![0xEE; n + INLINE];
                    emit(
                        &fwd,
                        &[],
                        &spelling,
                        &inline,
                        &lens,
                        n,
                        mark,
                        bits,
                        &mut want,
                    );
                    let mut got = vec![0x11; n + INLINE];
                    emit_codes(&supplied, &spelling, &inline, &lens, n, &mut got);
                    assert!(got == want, "forward: cycle {cyc} m {m} n {n}");
                }
            }
        }
    }

    /// The BWT of `text` by prefix doubling: (the column, the row of the text's rotation 0)
    fn bwt(text: &[u8]) -> (Vec<u8>, u32) {
        let n = text.len();
        let mut rank: Vec<u32> = text.iter().map(|&c| u32::from(c)).collect();
        let mut sa: Vec<u32> = (0..n as u32).collect();
        let mut tmp = vec![0u32; n];
        let mut w = 1;
        loop {
            let key = |i: u32| (rank[i as usize], rank[(i as usize + w) % n]);
            sa.sort_unstable_by_key(|&i| key(i));
            tmp[sa[0] as usize] = 0;
            for x in 1..n {
                tmp[sa[x] as usize] =
                    tmp[sa[x - 1] as usize] + u32::from(key(sa[x - 1]) != key(sa[x]));
            }
            std::mem::swap(&mut rank, &mut tmp);
            if rank[sa[n - 1] as usize] as usize == n - 1 || w >= n {
                break;
            }
            w *= 2;
        }
        let l: Vec<u8> = sa.iter().map(|&i| text[(i as usize + n - 1) % n]).collect();
        let p = sa.iter().position(|&i| i == 0).unwrap() as u32;
        (l, p)
    }
}
