//! Smooth-zone dissection & redesign probe for `A6c` (`Reader::wire` / `Reader::wire_xor`
//! — Open kernel retained-row wiring), zero `src/` edits.
//!
//! Frozen Stop Test (`docs/journal/2026-11-27-explore/a6c-wire-dissection.md`):
//! - 100% bit-exact `edges[0..m]` (and `predecessor` for bidi) vs shipped `Reader::wire` /
//!   `Reader::wire_xor` across all 233 Open blocks of `wikidata.bz2`, all Open blocks of 3
//!   synthetic guard corpora (`text`, `records`, `runs`), and synthetic stress cases (overflow
//!   chunks `ovf > 0`, exact `EPOCH = 65536` boundaries, `mark` edge positions, self-loop `u==v`).
//! - PASS iff >= 15.0% steal-free CPU reduction (disjoint 7-repeat spread + 95% CI upper <= 0.850)
//!   across BOTH baseline x86-64 AND JCC-32B-aligned native, in BOTH warm and L3-cold regimes,
//!   on Track F (`forward` `Reader::wire`) and/or Track B (`bidi` `Reader::wire_xor`), with
//!   <= 1.02x peak scratch memory and zero > 2.0% disjoint guard regression (plus >= 8.0% joint
//!   `A6a + A6c` for any chunk-size variant).
//!
//! Usage:
//!   cargo run --release --example a6c_wire_dissection -- /path/to/wikidata.bz2 [--repeats 7]

#![allow(clippy::too_many_arguments)]

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("a6c_wire_dissection requires x86_64");
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use bzip2::write::BzEncoder;
    use bzip2::Compression;
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::arch::x86_64::*;
    use std::cell::Cell;
    use std::hint::black_box;
    use std::io::Write;
    use std::time::Instant;

    const LIMIT: u32 = 16;
    const INLINE: usize = 16;
    const LAZY_DENSITY: (usize, usize) = (3, 20);
    const DENSE: (usize, usize) = (7, 25);
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;
    const NONE_CHUNK: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;
    const OVF_BIT: u32 = 1 << 31;
    const PAGE: usize = 64;
    const TERMINAL: u16 = 256;

    fn thread_cpu_ns() -> u64 {
        let s = std::fs::read_to_string("/proc/thread-self/schedstat").unwrap_or_default();
        s.split_whitespace()
            .next()
            .and_then(|x| x.parse::<u64>().ok())
            .unwrap_or(0)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Cap {
        m: u32,
        strips: usize,
    }

    impl Cap {
        const NONE: Cap = Cap {
            m: u32::MAX,
            strips: usize::MAX,
        };
        fn dense(n: usize) -> Cap {
            Cap {
                m: DENSE_M.min((n as u64 * 3 / 10) as u32),
                strips: DENSE_STRIPS,
            }
        }
    }

    fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if n >= 65_536 && records * 64 >= n && records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0
        {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0u32;
        counts.map(|w| {
            let s = total;
            total += w;
            s
        })
    }

    /// Exact production `Band(u64)` layout from `src/open.rs`.
    #[derive(Clone, Copy)]
    struct Band(u64);

    impl Band {
        #[inline(always)]
        fn new(end: u32, dst: u32, code: u32) -> Self {
            Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
        }
        #[inline(always)]
        fn end(self) -> u32 {
            (self.0 & 0xF_FFFF) as u32
        }
        #[inline(always)]
        fn dst(self) -> u32 {
            ((self.0 >> 20) & 0xF_FFFF) as u32
        }
        #[inline(always)]
        fn code(self) -> u32 {
            (self.0 >> 40) as u32
        }
    }

    #[derive(Clone, Copy)]
    struct Tile {
        end: u32,
        delta: u32,
        symbol: u16,
    }

    #[derive(Clone, Copy)]
    struct First {
        thick_rows: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        runs: usize,
    }

    impl Default for First {
        fn default() -> Self {
            First {
                thick_rows: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                m: 0,
                mark: 0,
                mark_byte: 0,
                runs: 0,
            }
        }
    }

    // ---- Generic OutQ16<C, S> (supports both shipped <512,512> and <1008,1024>) ----

    #[derive(Default, Clone)]
    struct Spill<const S: usize> {
        next: Vec<u32>,
        ovf: Vec<Vec<u8>>,
        cuts: Vec<[u32; 256]>,
    }

    #[derive(Clone, Copy)]
    struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    struct Queues<'a, const C: usize, const S: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill<S>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        tails: [u32; 256],
        fill: [u32; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const C: usize, const S: usize> Queues<'a, C, S> {
        const STRIDE_BYTES: usize = 2 * S;

        fn new(out: &'a mut [u8], spill: &'a mut Spill<S>, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            Queues {
                in_out: (out.len() / Self::STRIDE_BYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE_CHUNK; 256],
                tails: [NONE_CHUNK; 256],
                fill: [C as u32; 256],
                limit,
                over: false,
            }
        }

        fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }

        #[inline(always)]
        fn push_shipped(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = self.tails[a];
            if w <= 16
                && f + 16 <= S
                && f + (w as usize) <= C
                && lo + w < EPOCH
                && tail < self.in_out
            {
                let at = tail as usize * Self::STRIDE_BYTES + 2 * f;
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = (f as u32) + w;
            } else {
                self.push_slow(a, m, w, thin);
            }
        }

        #[inline(always)]
        fn push_hoisted(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a];
            let nf = f.wrapping_add(w);
            let lo = m & (EPOCH - 1);
            if nf <= C as u32 && lo + w < EPOCH {
                let at = self.tails[a] as usize * Self::STRIDE_BYTES + 2 * (f as usize);
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = nf;
            } else {
                self.push_slow_tagged(a, m, w, thin);
            }
        }

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                if (self.fill[a] & !OVF_BIT) as usize == C {
                    self.take_chunk(a, false);
                }
                let f = (self.fill[a] & !OVF_BIT) as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((C - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(self.tails[a]);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                self.fill[a] += take;
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let mut cut = *thin;
                    cut[a] += w - left;
                    self.spill.cuts.push(cut);
                }
            }
        }

        #[cold]
        #[inline(never)]
        fn push_slow_tagged(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let raw_f = self.fill[a];
                let f = (raw_f & !OVF_BIT) as usize;
                if f == C {
                    self.take_chunk(a, true);
                }
                let raw_f = self.fill[a];
                let f = (raw_f & !OVF_BIT) as usize;
                let ovf_flag = raw_f & OVF_BIT;
                let lo = m & (EPOCH - 1);
                let take = left.min((C - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(self.tails[a]);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                self.fill[a] = ovf_flag | ((f as u32) + take);
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let mut cut = *thin;
                    cut[a] += w - left;
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk(&mut self, a: usize, tag_ovf: bool) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push(vec![0u8; Self::STRIDE_BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE_CHUNK);
            }
            self.spill.next[id as usize] = NONE_CHUNK;
            if self.heads[a] == NONE_CHUNK {
                self.heads[a] = id;
            } else {
                self.spill.next[self.tails[a] as usize] = id;
            }
            self.tails[a] = id;
            self.fill[a] = if tag_ovf && id >= self.in_out {
                OVF_BIT
            } else {
                0
            };
        }

        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * Self::STRIDE_BYTES;
                &mut self.out[at..at + Self::STRIDE_BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    // ---- Exact shipped Reader (baseline reference) ----

    struct ShippedReader<'a, const C: usize, const S: usize> {
        out: &'a [u8],
        spill: &'a Spill<S>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a, const C: usize, const S: usize> ShippedReader<'a, C, S> {
        const STRIDE_BYTES: usize = 2 * S;

        fn new(out: &'a [u8], spill: &'a Spill<S>, heads: &Heads, a: usize) -> Self {
            ShippedReader {
                out,
                spill,
                in_out: heads.in_out,
                a,
                id: heads.heads[a],
                at: 0,
                read: 0,
                epoch: 0,
                cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }

        #[inline(always)]
        fn wire_xor(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * Self::STRIDE_BYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == C {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(C - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::STRIDE_BYTES;
                    &self.out[at..at + Self::STRIDE_BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)]
                    .chunks_exact(2)
                    .enumerate()
                {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        #[inline(always)]
        fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * Self::STRIDE_BYTES + 2 * self.at;
                for (e, s) in dst
                    .iter_mut()
                    .zip(self.out[at..at + 2 * span].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == C {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = dst
                    .len()
                    .min(C - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::STRIDE_BYTES;
                    &self.out[at..at + Self::STRIDE_BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                for (e, s) in now
                    .iter_mut()
                    .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = rest;
            }
        }
    }

    // ---- Register-resident chunk cursor (ZERO stack spills of Reader!) ----

    /// Holds the current contiguous slice `cur` of byte `a`'s `OutQ16` within one chunk AND
    /// one epoch (`cur.len() == 2 * avail`). Cold refill advances chunk/epoch only once per
    /// `C` ranks (~65k times/233 blks vs 828k bands).
    #[derive(Clone, Copy)]
    struct CursorState<'a> {
        cur: &'a [u8],
        id: u32,
        at_end: usize, // position in current chunk corresponding to end of `cur`
        read_end: u32, // cumulative ranks read at end of `cur`
        epoch: u32,
        cut: u32,
    }

    #[inline(always)]
    fn cursor_init<'a, const C: usize, const S: usize>(
        out: &'a [u8],
        spill: &'a Spill<S>,
        heads: &Heads,
        a: usize,
    ) -> CursorState<'a> {
        let id = heads.heads[a];
        let cut = spill.cuts.first().map_or(u32::MAX, |c| c[a]);
        if id == NONE_CHUNK {
            return CursorState {
                cur: &[],
                id,
                at_end: 0,
                read_end: 0,
                epoch: 0,
                cut,
            };
        }
        let mut epoch = 0u32;
        let mut cur_cut = cut;
        while cur_cut == 0 {
            epoch += 1;
            cur_cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
        }
        let take = C.min(cur_cut as usize);
        let chunk: &'a [u8] = if id < heads.in_out {
            let base = id as usize * (2 * S);
            &out[base..base + 2 * take]
        } else {
            &spill.ovf[(id - heads.in_out) as usize][..2 * take]
        };
        CursorState {
            cur: chunk,
            id,
            at_end: take,
            read_end: take as u32,
            epoch,
            cut: cur_cut,
        }
    }

    /// Cold refill: called only when `cur` is exhausted (once per chunk or epoch cut).
    #[cold]
    #[inline(never)]
    fn cursor_refill<'a, const C: usize, const S: usize>(
        out: &'a [u8],
        spill: &'a Spill<S>,
        in_out: u32,
        a: usize,
        mut id: u32,
        mut at: usize,
        read: u32,
        mut epoch: u32,
        mut cut: u32,
    ) -> CursorState<'a> {
        while read == cut {
            epoch += 1;
            cut = spill.cuts.get(epoch as usize).map_or(u32::MAX, |c| c[a]);
        }
        if at == C {
            id = spill.next[id as usize];
            at = 0;
        }
        let take = (C - at).min((cut - read) as usize);
        let chunk: &'a [u8] = if id < in_out {
            let base = id as usize * (2 * S) + 2 * at;
            &out[base..base + 2 * take]
        } else {
            let ovf = &spill.ovf[(id - in_out) as usize];
            &ovf[2 * at..2 * (at + take)]
        };
        CursorState {
            cur: chunk,
            id,
            at_end: at + take,
            read_end: read + take as u32,
            epoch,
            cut,
        }
    }

    // ---- Safe branch-minimized overlapping slice fillers (zero unsafe!) ----

    #[inline(always)]
    fn fill_fwd_8_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let s: &[u8; 16] = src[..16].try_into().unwrap();
        let d: &mut [u32; 8] = (&mut dst[..8]).try_into().unwrap();
        for i in 0..8 {
            d[i] = hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
        }
    }

    #[inline(always)]
    fn fill_fwd_4_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let s: &[u8; 8] = src[..8].try_into().unwrap();
        let d: &mut [u32; 4] = (&mut dst[..4]).try_into().unwrap();
        for i in 0..4 {
            d[i] = hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
        }
    }

    #[inline(always)]
    fn fill_fwd_overlap_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let n = dst.len();
        if n >= 8 {
            let mut i = 0;
            let limit = n - 8;
            while i < limit {
                fill_fwd_8_safe(&mut dst[i..i + 8], &src[2 * i..2 * i + 16], hi);
                i += 8;
            }
            fill_fwd_8_safe(&mut dst[limit..n], &src[2 * limit..2 * n], hi);
        } else if n >= 4 {
            fill_fwd_4_safe(&mut dst[0..4], &src[0..8], hi);
            let tail = n - 4;
            fill_fwd_4_safe(&mut dst[tail..n], &src[2 * tail..2 * n], hi);
        } else if n >= 2 {
            let v0 = hi | u32::from(u16::from_le_bytes([src[0], src[1]]));
            let v1 = hi | u32::from(u16::from_le_bytes([src[2], src[3]]));
            dst[0] = v0;
            dst[1] = v1;
            if n == 3 {
                dst[2] = hi | u32::from(u16::from_le_bytes([src[4], src[5]]));
            }
        } else if n == 1 {
            dst[0] = hi | u32::from(u16::from_le_bytes([src[0], src[1]]));
        }
    }

    #[inline(always)]
    fn xor_fwd_8_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let s: &[u8; 16] = src[..16].try_into().unwrap();
        let d: &mut [u32; 8] = (&mut dst[..8]).try_into().unwrap();
        for i in 0..8 {
            d[i] ^= hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
        }
    }

    #[inline(always)]
    fn xor_fwd_4_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let s: &[u8; 8] = src[..8].try_into().unwrap();
        let d: &mut [u32; 4] = (&mut dst[..4]).try_into().unwrap();
        for i in 0..4 {
            d[i] ^= hi | u32::from(u16::from_le_bytes([s[2 * i], s[2 * i + 1]]));
        }
    }

    /// Note: XOR is NOT idempotent, so we cannot overlap! Use exact non-overlapping 8/4/2/1.
    #[inline(always)]
    fn xor_fwd_exact_safe(dst: &mut [u32], src: &[u8], hi: u32) {
        let n = dst.len();
        let mut i = 0;
        while i + 8 <= n {
            xor_fwd_8_safe(&mut dst[i..i + 8], &src[2 * i..2 * i + 16], hi);
            i += 8;
        }
        if i + 4 <= n {
            xor_fwd_4_safe(&mut dst[i..i + 4], &src[2 * i..2 * i + 8], hi);
            i += 4;
        }
        while i < n {
            dst[i] ^= hi | u32::from(u16::from_le_bytes([src[2 * i], src[2 * i + 1]]));
            i += 1;
        }
    }

    /// 100% safe zero-bounds-check 4x-unrolled scatter `ep[lo] ^= u` over a 65536-slot epoch window.
    #[inline(always)]
    fn scatter_v_4x_safe(ep: &mut [u32; 65536], src: &[u8], mut u: u32) {
        let mut chunks8 = src.chunks_exact(8);
        for c in &mut chunks8 {
            let w = u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
            ep[(w & 0xFFFF) as usize] ^= u;
            ep[((w >> 16) & 0xFFFF) as usize] ^= u + 1;
            ep[((w >> 32) & 0xFFFF) as usize] ^= u + 2;
            ep[(w >> 48) as usize] ^= u + 3;
            u += 4;
        }
        for c in chunks8.remainder().chunks_exact(2) {
            ep[u16::from_le_bytes([c[0], c[1]]) as usize] ^= u;
            u += 1;
        }
    }

    #[inline(always)]
    fn scatter_v_with_mark_safe(
        ep: &mut [u32; 65536],
        src: &[u8],
        mut u: u32,
        mark_lo: u16,
        pred: &mut u32,
    ) {
        for c in src.chunks_exact(2) {
            let lo = u16::from_le_bytes([c[0], c[1]]);
            if lo == mark_lo {
                *pred = u;
            }
            ep[lo as usize] ^= u;
            u += 1;
        }
    }

    // ---- AVX2 helpers (runtime-dispatched via #[target_feature(enable = "avx2")]) ----

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn fill_fwd_overlap_avx2_8w(dst: &mut [u32], src: &[u8], hi: u32) {
        let n = dst.len();
        debug_assert!(src.len() >= 2 * n);
        let dp = dst.as_mut_ptr();
        let sp = src.as_ptr();
        if n >= 8 {
            let vhi = _mm256_set1_epi32(hi as i32);
            let mut i = 0usize;
            let limit = n - 8;
            while i < limit {
                let raw = _mm_loadu_si128(sp.add(2 * i) as *const __m128i);
                let ext = _mm256_cvtepu16_epi32(raw);
                _mm256_storeu_si256(dp.add(i) as *mut __m256i, _mm256_or_si256(ext, vhi));
                i += 8;
            }
            let raw = _mm_loadu_si128(sp.add(2 * limit) as *const __m128i);
            let ext = _mm256_cvtepu16_epi32(raw);
            _mm256_storeu_si256(dp.add(limit) as *mut __m256i, _mm256_or_si256(ext, vhi));
        } else if n >= 4 {
            let vhi = _mm_set1_epi32(hi as i32);
            let r0 = _mm_loadl_epi64(sp as *const __m128i);
            _mm_storeu_si128(
                dp as *mut __m128i,
                _mm_or_si128(_mm_cvtepu16_epi32(r0), vhi),
            );
            let tail = n - 4;
            let r1 = _mm_loadl_epi64(sp.add(2 * tail) as *const __m128i);
            _mm_storeu_si128(
                dp.add(tail) as *mut __m128i,
                _mm_or_si128(_mm_cvtepu16_epi32(r1), vhi),
            );
        } else if n >= 2 {
            let v0 = hi | u32::from((sp as *const u16).read_unaligned());
            let v1 = hi | u32::from((sp.add(2) as *const u16).read_unaligned());
            *dp = v0;
            *dp.add(1) = v1;
            if n == 3 {
                *dp.add(2) = hi | u32::from((sp.add(4) as *const u16).read_unaligned());
            }
        } else if n == 1 {
            *dp = hi | u32::from((sp as *const u16).read_unaligned());
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn fill_fwd_overlap_avx2_16w(dst: &mut [u32], src: &[u8], hi: u32) {
        let n = dst.len();
        debug_assert!(src.len() >= 2 * n);
        let dp = dst.as_mut_ptr();
        let sp = src.as_ptr();
        if n >= 16 {
            let vhi = _mm256_set1_epi32(hi as i32);
            let mut i = 0usize;
            let limit = n - 16;
            while i < limit {
                let r0 = _mm_loadu_si128(sp.add(2 * i) as *const __m128i);
                let r1 = _mm_loadu_si128(sp.add(2 * i + 16) as *const __m128i);
                _mm256_storeu_si256(
                    dp.add(i) as *mut __m256i,
                    _mm256_or_si256(_mm256_cvtepu16_epi32(r0), vhi),
                );
                _mm256_storeu_si256(
                    dp.add(i + 8) as *mut __m256i,
                    _mm256_or_si256(_mm256_cvtepu16_epi32(r1), vhi),
                );
                i += 16;
            }
            let r0 = _mm_loadu_si128(sp.add(2 * limit) as *const __m128i);
            let r1 = _mm_loadu_si128(sp.add(2 * limit + 16) as *const __m128i);
            _mm256_storeu_si256(
                dp.add(limit) as *mut __m256i,
                _mm256_or_si256(_mm256_cvtepu16_epi32(r0), vhi),
            );
            _mm256_storeu_si256(
                dp.add(limit + 8) as *mut __m256i,
                _mm256_or_si256(_mm256_cvtepu16_epi32(r1), vhi),
            );
        } else if n >= 8 {
            let vhi = _mm256_set1_epi32(hi as i32);
            let r0 = _mm_loadu_si128(sp as *const __m128i);
            _mm256_storeu_si256(
                dp as *mut __m256i,
                _mm256_or_si256(_mm256_cvtepu16_epi32(r0), vhi),
            );
            let tail = n - 8;
            let r1 = _mm_loadu_si128(sp.add(2 * tail) as *const __m128i);
            _mm256_storeu_si256(
                dp.add(tail) as *mut __m256i,
                _mm256_or_si256(_mm256_cvtepu16_epi32(r1), vhi),
            );
        } else if n >= 4 {
            let vhi = _mm_set1_epi32(hi as i32);
            let r0 = _mm_loadl_epi64(sp as *const __m128i);
            _mm_storeu_si128(
                dp as *mut __m128i,
                _mm_or_si128(_mm_cvtepu16_epi32(r0), vhi),
            );
            let tail = n - 4;
            let r1 = _mm_loadl_epi64(sp.add(2 * tail) as *const __m128i);
            _mm_storeu_si128(
                dp.add(tail) as *mut __m128i,
                _mm_or_si128(_mm_cvtepu16_epi32(r1), vhi),
            );
        } else if n >= 2 {
            let v0 = hi | u32::from((sp as *const u16).read_unaligned());
            let v1 = hi | u32::from((sp.add(2) as *const u16).read_unaligned());
            *dp = v0;
            *dp.add(1) = v1;
            if n == 3 {
                *dp.add(2) = hi | u32::from((sp.add(4) as *const u16).read_unaligned());
            }
        } else if n == 1 {
            *dp = hi | u32::from((sp as *const u16).read_unaligned());
        }
    }

    #[target_feature(enable = "avx2")]
    #[inline]
    unsafe fn xor_fwd_exact_avx2(dst: &mut [u32], src: &[u8], hi: u32) {
        let n = dst.len();
        debug_assert!(src.len() >= 2 * n);
        let dp = dst.as_mut_ptr();
        let sp = src.as_ptr();
        let mut i = 0usize;
        if n >= 8 {
            let vhi = _mm256_set1_epi32(hi as i32);
            while i + 8 <= n {
                let raw = _mm_loadu_si128(sp.add(2 * i) as *const __m128i);
                let val = _mm256_or_si256(_mm256_cvtepu16_epi32(raw), vhi);
                let cur = _mm256_loadu_si256(dp.add(i) as *const __m256i);
                _mm256_storeu_si256(dp.add(i) as *mut __m256i, _mm256_xor_si256(cur, val));
                i += 8;
            }
        }
        if i + 4 <= n {
            let vhi = _mm_set1_epi32(hi as i32);
            let raw = _mm_loadl_epi64(sp.add(2 * i) as *const __m128i);
            let val = _mm_or_si128(_mm_cvtepu16_epi32(raw), vhi);
            let cur = _mm_loadu_si128(dp.add(i) as *const __m128i);
            _mm_storeu_si128(dp.add(i) as *mut __m128i, _mm_xor_si128(cur, val));
            i += 4;
        }
        while i < n {
            let v = hi | u32::from((sp.add(2 * i) as *const u16).read_unaligned());
            *dp.add(i) ^= v;
            i += 1;
        }
    }

    // ---- Block data structure (holds BOTH Q512 and Q1008 queues + raw records for joint A6a+A6c) ----

    struct OpenBlock {
        n: u32,
        m: u32,
        mark: u32,
        mark_byte: u8,
        p: u32,
        bits: u32,
        recs: Vec<u16>,
        out512: Vec<u8>,
        spill512: Spill<512>,
        heads512: Heads,
        out1008: Vec<u8>,
        spill1008: Spill<1024>,
        heads1008: Heads,
        bands: Vec<Band>,
        cursors: [usize; 256],
    }

    fn build_real_bands(
        f: &First,
        ns: usize,
        edges_strips: &[u32],
        n: u32,
    ) -> (Vec<Band>, [usize; 256], u32) {
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        let first = starts(all);
        let mut write = starts(f.thick);
        let mut strips = edges_strips[..3 * ns].to_vec();
        let mut image_order = vec![0u32; ns];
        let mut tiles = Vec::with_capacity(2 * ns + 1);
        let (mut last_x, mut retained_x) = (0u32, 0u32);
        for i in 0..ns {
            let sw = &mut strips[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let width = sw[2] >> 8;
            let from = sw[0];
            let a = usize::from(byte);
            sw[1] += first[a];
            let to = sw[1];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if from > last_x {
                tiles.push(Tile {
                    end: from,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
                retained_x += from - last_x;
            }
            last_x = from + width;
            tiles.push(Tile {
                end: last_x,
                delta: to.wrapping_sub(from),
                symbol: u16::from(byte),
            });
        }
        if last_x < n {
            tiles.push(Tile {
                end: n,
                delta: retained_x.wrapping_sub(last_x),
                symbol: TERMINAL,
            });
        }
        let mut directory = Vec::with_capacity(n as usize / PAGE + 1);
        let mut ti = 0usize;
        for x in (0..n).step_by(PAGE) {
            while tiles[ti].end <= x {
                ti += 1;
            }
            directory.push(ti as u32);
        }
        let mut bands = Vec::new();
        let mut cursors = [0usize; 256];
        let mut next_code = 256u32;
        let (mut k, mut si) = (0u32, 0usize);
        let mut trace_range = |mut x0: u32, end: u32, c: u8, k: &mut u32, bands: &mut Vec<Band>| {
            while x0 < end {
                let mut width = end - x0;
                let mut x = x0;
                let mut height = 0usize;
                let exit = loop {
                    let mut q = directory[x as usize / PAGE] as usize;
                    while x >= tiles[q].end {
                        q += 1;
                    }
                    let t = tiles[q];
                    width = width.min(t.end - x);
                    x = x.wrapping_add(t.delta);
                    if t.symbol == TERMINAL {
                        break x;
                    }
                    height += 1;
                };
                let code = if height == 0 {
                    u32::from(c)
                } else {
                    let cd = next_code;
                    next_code += 1;
                    cd
                };
                bands.push(Band::new(*k + width, exit, code));
                *k += width;
                x0 += width;
            }
        };
        for a in 0..256 {
            cursors[a] = bands.len();
            let mut x = first[a];
            while si < write[a] as usize {
                let i = image_order[si] as usize;
                let to = strips[3 * i + 1];
                let width = strips[3 * i + 2] >> 8;
                trace_range(x, to, a as u8, &mut k, &mut bands);
                x = to + width;
                si += 1;
            }
            trace_range(x, first[a] + all[a], a as u8, &mut k, &mut bands);
        }
        assert_eq!(k, f.m);
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        (bands, cursors, bits)
    }

    fn run_pass1<const C: usize, const S: usize, const HOISTED: bool>(
        recs: &[u16],
        p: u32,
        cap: Cap,
        out: &mut [u8],
        spill: &mut Spill<S>,
        edges_strips: &mut Vec<u32>,
    ) -> (First, usize, Heads, bool) {
        let mut q = Queues::<C, S>::new(out, spill, cap.m);
        let mut f = First::default();
        edges_strips.clear();
        let mut ns = 0usize;
        let mut pos = 0u32;
        let mut i = 0usize;
        while i < recs.len() {
            let rec = recs[i];
            let c = (rec >> 8) as u8;
            let mut w = u32::from(rec & 0xFF);
            i += 1;
            while i < recs.len() && (recs[i] >> 8) as u8 == c {
                w += u32::from(recs[i] & 0xFF);
                i += 1;
            }
            f.runs += 1;
            let mut take_piece = |c: u8, x: u32, w: u32, keep: bool| {
                let a = usize::from(c);
                if keep {
                    if p.wrapping_sub(x) < w {
                        f.mark = f.m + (p - x);
                        f.mark_byte = c;
                    }
                    if HOISTED {
                        q.push_hoisted(a, f.m, w, &f.thin);
                    } else {
                        q.push_shipped(a, f.m, w, &f.thin);
                    }
                    f.thin[a] += w;
                    f.m += w;
                } else {
                    if ns < cap.strips {
                        edges_strips.push(x);
                        edges_strips.push(f.thin[a] + f.thick_rows[a]);
                        edges_strips.push((w << 8) | u32::from(c));
                    }
                    ns += 1;
                    f.thick[a] += 1;
                    f.thick_rows[a] += w;
                }
            };
            if w <= LIMIT {
                take_piece(c, pos, w, true);
            } else if p.wrapping_sub(pos) >= w {
                take_piece(c, pos, w, false);
            } else {
                if pos < p {
                    take_piece(c, pos, p - pos, false);
                }
                take_piece(c, p, 1, true);
                if p + 1 < pos + w {
                    take_piece(c, p + 1, pos + w - p - 1, false);
                }
            }
            pos += w;
        }
        let heads = q.heads();
        let over = q.over;
        (f, ns, heads, over)
    }

    fn mtf_to_runs(symbols: &mibz2::block::Symbols) -> (usize, Vec<u16>) {
        let mut mtf: [u8; 256] = std::array::from_fn(|i| symbols.seq_to_unseq[i]);
        let mut recs: Vec<u16> = Vec::with_capacity(symbols.syms.len());
        let mut n = 0usize;
        let mut i = 0usize;
        let syms = &symbols.syms;
        while i < syms.len() {
            let s = syms[i];
            if s == symbols.eob {
                break;
            }
            if s <= 1 {
                let mut run = 0u32;
                let mut bit = 1u32;
                while i < syms.len() && syms[i] <= 1 {
                    run += (u32::from(syms[i]) + 1) * bit;
                    bit <<= 1;
                    i += 1;
                }
                let c = mtf[0];
                n += run as usize;
                let mut rem = run;
                while rem > 127 {
                    recs.push((u16::from(c) << 8) | 127);
                    rem -= 127;
                }
                recs.push((u16::from(c) << 8) | (rem as u16));
            } else {
                let idx = (s - 1) as usize;
                let c = mtf[idx];
                mtf.copy_within(0..idx, 1);
                mtf[0] = c;
                recs.push((u16::from(c) << 8) | 1);
                n += 1;
                i += 1;
            }
        }
        (n, recs)
    }

    fn extract_open_blocks(bz: &[u8]) -> Vec<OpenBlock> {
        let kernels = Kernels::detect();
        let mut sc = Scanner::new(SliceSource(bz));
        let mut syms = Vec::new();
        let mut runs_buf = Runs::default();
        let mut blocks = Vec::new();
        let mut edges_strips = Vec::new();
        while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
            if runs::takes_runs(&symbols) {
                let n = runs::front(&symbols, &mut runs_buf, kernels).unwrap();
                let (n_check, recs) = mtf_to_runs(&symbols);
                assert_eq!(n, n_check);
                let p = symbols.orig_ptr;
                if let Some(cap) = open_cap(recs.len(), n, p) {
                    let mut out512 = vec![0u8; n + INLINE];
                    let mut spill512 = Spill::<512>::default();
                    let (f, ns, heads512, over) = run_pass1::<512, 512, false>(
                        &recs,
                        p,
                        cap,
                        &mut out512,
                        &mut spill512,
                        &mut edges_strips,
                    );
                    if !over && f.m <= cap.m && ns <= cap.strips {
                        let (bands, cursors, bits) =
                            build_real_bands(&f, ns, &edges_strips, n as u32);
                        let mut out1008 = vec![0u8; n + INLINE];
                        let mut spill1008 = Spill::<1024>::default();
                        let mut tmp_strips = Vec::new();
                        let (f2, ns2, heads1008, over2) = run_pass1::<1008, 1024, true>(
                            &recs,
                            p,
                            cap,
                            &mut out1008,
                            &mut spill1008,
                            &mut tmp_strips,
                        );
                        assert!(!over2 && f2.m == f.m && ns2 == ns && f2.mark == f.mark);
                        blocks.push(OpenBlock {
                            n: n as u32,
                            m: f.m,
                            mark: f.mark,
                            mark_byte: f.mark_byte,
                            p,
                            bits,
                            recs,
                            out512,
                            spill512,
                            heads512,
                            out1008,
                            spill1008,
                            heads1008,
                            bands,
                            cursors,
                        });
                    }
                }
            }
            syms = symbols.syms;
        }
        blocks
    }

    // =========================================================================
    // TRACK F: FORWARD WIRING (`Reader::wire`, `#1110` stack)
    // =========================================================================

    /// F0: exact shipped `src/open/outq.rs::Reader::wire`
    #[inline(never)]
    fn f0_shipped<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        bits: u32,
        edges: &mut [u32],
    ) {
        let edges = &mut edges[..m as usize];
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let mut queue = ShippedReader::<C, S>::new(out, spill, heads, a);
            for &b in &bands[cursors[a]..end] {
                let bend = b.end();
                let span = (bend - k) as usize;
                let dst = b.dst() as usize;
                let pack = b.code() << bits;
                queue.wire(&mut edges[dst..dst + span], pack);
                k = bend;
            }
        }
    }

    /// F1: 100% safe register-resident chunk cursor (`cur: &[u8]`, zero `Reader` stack spills,
    /// single `span * 2 <= cur.len()` check), keeping plain `zip(chunks_exact(2))` loop.
    #[inline(never)]
    fn f1_reg_cursor_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        bits: u32,
        edges: &mut [u32],
    ) {
        let edges = &mut edges[..m as usize];
        let in_out = heads.in_out;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for &b in bslice {
                let bend = b.end();
                let span = (bend - k) as usize;
                let dst = b.dst() as usize;
                let pack = b.code() << bits;
                let mut d = &mut edges[dst..dst + span];
                let bytes_need = span * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    let hi = pack | (st.epoch << 16);
                    for (e, s) in d.iter_mut().zip(src.chunks_exact(2)) {
                        *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    }
                } else {
                    while !d.is_empty() {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                        }
                        let take = d.len().min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let (now, drest) = d.split_at_mut(take);
                        let hi = pack | (st.epoch << 16);
                        for (e, s) in now.iter_mut().zip(src.chunks_exact(2)) {
                            *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                        }
                        d = drest;
                    }
                }
                k = bend;
            }
        }
    }

    /// F2: 100% safe register-resident cursor + branch-minimized overlapping 8/4/2/1 SIMD
    /// (zero `>= 32` branch, zero scalar tail loop!).
    #[inline(never)]
    fn f2_reg_overlap_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        bits: u32,
        edges: &mut [u32],
    ) {
        let edges = &mut edges[..m as usize];
        let in_out = heads.in_out;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for &b in bslice {
                let bend = b.end();
                let span = (bend - k) as usize;
                let dst = b.dst() as usize;
                let pack = b.code() << bits;
                let mut d = &mut edges[dst..dst + span];
                let bytes_need = span * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    fill_fwd_overlap_safe(d, src, pack | (st.epoch << 16));
                } else {
                    while !d.is_empty() {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                        }
                        let take = d.len().min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let (now, drest) = d.split_at_mut(take);
                        fill_fwd_overlap_safe(now, src, pack | (st.epoch << 16));
                        d = drest;
                    }
                }
                k = bend;
            }
        }
    }

    /// F3a/F3b/F4: Runtime-dispatched AVX2 (`#[target_feature(enable = "avx2")]`, zero per-band
    /// `vzeroupper`, works on baseline x86-64 builds via `Kernels::avx2`), parameterized by
    /// `WIDE16` (8-wide vs 16-wide overlapping loop) and `PREFETCH` (`_mm_prefetch` 4 bands ahead).
    #[target_feature(enable = "avx2")]
    #[inline(never)]
    unsafe fn f_avx2_impl<
        const C: usize,
        const S: usize,
        const WIDE16: bool,
        const PREFETCH: bool,
    >(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        bits: u32,
        edges: &mut [u32],
    ) {
        let edges = &mut edges[..m as usize];
        let in_out = heads.in_out;
        let ep_base = edges.as_ptr();
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for (bi, &b) in bslice.iter().enumerate() {
                if PREFETCH && bi + 4 < bslice.len() {
                    let pdst = bslice[bi + 4].dst() as usize;
                    if pdst < edges.len() {
                        _mm_prefetch::<_MM_HINT_T0>(ep_base.add(pdst) as *const i8);
                    }
                }
                let bend = b.end();
                let span = (bend - k) as usize;
                let dst = b.dst() as usize;
                let pack = b.code() << bits;
                let mut d = &mut edges[dst..dst + span];
                let bytes_need = span * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    if WIDE16 {
                        fill_fwd_overlap_avx2_16w(d, src, pack | (st.epoch << 16));
                    } else {
                        fill_fwd_overlap_avx2_8w(d, src, pack | (st.epoch << 16));
                    }
                } else {
                    while !d.is_empty() {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                        }
                        let take = d.len().min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let (now, drest) = d.split_at_mut(take);
                        if WIDE16 {
                            fill_fwd_overlap_avx2_16w(now, src, pack | (st.epoch << 16));
                        } else {
                            fill_fwd_overlap_avx2_8w(now, src, pack | (st.epoch << 16));
                        }
                        d = drest;
                    }
                }
                k = bend;
            }
        }
    }

    // =========================================================================
    // TRACK B: BIDIRECTIONAL XOR WIRING (`Reader::wire_xor`, shipped default)
    // =========================================================================

    #[inline(always)]
    fn padded_len(m: u32) -> usize {
        ((m as usize).saturating_add(65535)) & !65535
    }

    /// B0: exact shipped `src/open/outq.rs::Reader::wire_xor`
    #[inline(never)]
    fn b0_shipped<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        let edges = &mut edges[..m as usize];
        edges.fill(0);
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let mut queue = ShippedReader::<C, S>::new(out, spill, heads, a);
            for &band in &bands[cursors[a]..end] {
                queue.wire_xor(
                    edges,
                    band.dst() as usize,
                    (band.end() - k) as usize,
                    band.code() << bits,
                    mark,
                    &mut predecessor,
                );
                k = band.end();
            }
        }
        predecessor
    }

    /// B1: 100% safe register-resident cursor + `&mut [u32; 65536]` zero-bc epoch window +
    /// hoisted `mark_byte`/`mark_epoch` (fused 1-loop per band).
    #[inline(never)]
    fn b1_reg_zerobc_nomark_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        let pad = padded_len(m);
        edges[..m as usize].fill(0);
        let edges = &mut edges[..pad];
        let in_out = heads.in_out;
        let mark_epoch = mark >> 16;
        let mark_lo = (mark & 0xFFFF) as u16;
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let is_mark_byte = a == usize::from(mark_byte);
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for &b in bslice {
                let bend = b.end();
                let mut left = (bend - k) as usize;
                let mut dst = b.dst() as usize;
                let pack = b.code() << bits;
                while left > 0 {
                    if st.cur.is_empty() {
                        st = cursor_refill::<C, S>(
                            out,
                            spill,
                            in_out,
                            a,
                            st.id,
                            st.at_end,
                            st.read_end,
                            st.epoch,
                            st.cut,
                        );
                    }
                    let take = left.min(st.cur.len() / 2);
                    let (src, rest) = st.cur.split_at(take * 2);
                    st.cur = rest;
                    let hi = st.epoch << 16;
                    let hi_pack = pack | hi;
                    let ep_off = hi as usize;
                    if !is_mark_byte || st.epoch != mark_epoch {
                        // Safe zero-bc inner loop: validate dst range once, ep is [u32; 65536].
                        assert!(dst + take <= edges.len() && ep_off + 65536 <= edges.len());
                        for (i, s) in src.chunks_exact(2).enumerate() {
                            let u = dst + i;
                            let lo = u16::from_le_bytes([s[0], s[1]]);
                            edges[u] ^= hi_pack | u32::from(lo);
                            edges[ep_off + usize::from(lo)] ^= u as u32;
                        }
                    } else {
                        for (i, s) in src.chunks_exact(2).enumerate() {
                            let u = dst + i;
                            let lo = u16::from_le_bytes([s[0], s[1]]);
                            if lo == mark_lo {
                                predecessor = u as u32;
                            }
                            edges[u] ^= hi_pack | u32::from(lo);
                            edges[ep_off + usize::from(lo)] ^= u as u32;
                        }
                    }
                    dst += take;
                    left -= take;
                }
                k = bend;
            }
        }
        predecessor
    }

    /// B1b: 100% safe (`#![forbid(unsafe_code)]`) single-pass fused XOR using
    /// `Cell::from_mut(edges).as_slice_of_cells()` so BOTH `dst_cells: &[Cell<u32>]` AND
    /// `ep_cells: &[Cell<u32>; 65536]` coexist simultaneously with ZERO per-element bounds checks,
    /// hoisted `mark_byte`/`mark_epoch`, hoisted `ep_cells` per epoch, and 4x unrolled `u64` loads!
    #[inline(never)]
    fn b1b_cell_zerobc_4x_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        let pad = padded_len(m);
        edges[..m as usize].fill(0);
        let cells: &[Cell<u32>] = Cell::from_mut(&mut edges[..pad]).as_slice_of_cells();
        let in_out = heads.in_out;
        let mark_epoch = mark >> 16;
        let mark_lo = (mark & 0xFFFF) as u16;
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let is_mark_byte = a == usize::from(mark_byte);
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            let mut ep_off = (st.epoch as usize) << 16;
            let mut ep: &[Cell<u32>; 65536] = cells[ep_off..ep_off + 65536].try_into().unwrap();
            for &b in bslice {
                let bend = b.end();
                let mut left = (bend - k) as usize;
                let mut dst = b.dst() as usize;
                let pack = b.code() << bits;
                let bytes_need = left * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    let hi_pack = pack | (st.epoch << 16);
                    let dst_cells = &cells[dst..dst + left];
                    if !is_mark_byte || st.epoch != mark_epoch {
                        cell_xor_4x_safe(dst_cells, ep, src, dst as u32, hi_pack);
                    } else {
                        cell_xor_mark_safe(
                            dst_cells,
                            ep,
                            src,
                            dst as u32,
                            hi_pack,
                            mark_lo,
                            &mut predecessor,
                        );
                    }
                } else {
                    while left > 0 {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                            let new_off = (st.epoch as usize) << 16;
                            if new_off != ep_off {
                                ep_off = new_off;
                                ep = cells[ep_off..ep_off + 65536].try_into().unwrap();
                            }
                        }
                        let take = left.min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let hi_pack = pack | (st.epoch << 16);
                        let dst_cells = &cells[dst..dst + take];
                        if !is_mark_byte || st.epoch != mark_epoch {
                            cell_xor_4x_safe(dst_cells, ep, src, dst as u32, hi_pack);
                        } else {
                            cell_xor_mark_safe(
                                dst_cells,
                                ep,
                                src,
                                dst as u32,
                                hi_pack,
                                mark_lo,
                                &mut predecessor,
                            );
                        }
                        dst += take;
                        left -= take;
                    }
                }
                k = bend;
            }
        }
        predecessor
    }

    #[inline(always)]
    fn cell_xor_4x_safe(
        dst_cells: &[Cell<u32>],
        ep: &[Cell<u32>; 65536],
        src: &[u8],
        mut u: u32,
        hi_pack: u32,
    ) {
        let mut dchunks = dst_cells.chunks_exact(4);
        let mut schunks = src.chunks_exact(8);
        for (dc, sc) in (&mut dchunks).zip(&mut schunks) {
            let w = u64::from_le_bytes([sc[0], sc[1], sc[2], sc[3], sc[4], sc[5], sc[6], sc[7]]);
            let lo0 = (w & 0xFFFF) as u32;
            let lo1 = ((w >> 16) & 0xFFFF) as u32;
            let lo2 = ((w >> 32) & 0xFFFF) as u32;
            let lo3 = (w >> 48) as u32;
            dc[0].set(dc[0].get() ^ (hi_pack | lo0));
            let e0 = &ep[lo0 as usize];
            e0.set(e0.get() ^ u);
            dc[1].set(dc[1].get() ^ (hi_pack | lo1));
            let e1 = &ep[lo1 as usize];
            e1.set(e1.get() ^ (u + 1));
            dc[2].set(dc[2].get() ^ (hi_pack | lo2));
            let e2 = &ep[lo2 as usize];
            e2.set(e2.get() ^ (u + 2));
            dc[3].set(dc[3].get() ^ (hi_pack | lo3));
            let e3 = &ep[lo3 as usize];
            e3.set(e3.get() ^ (u + 3));
            u += 4;
        }
        for (dc, sc) in dchunks
            .remainder()
            .iter()
            .zip(schunks.remainder().chunks_exact(2))
        {
            let lo = u32::from(u16::from_le_bytes([sc[0], sc[1]]));
            dc.set(dc.get() ^ (hi_pack | lo));
            let e = &ep[lo as usize];
            e.set(e.get() ^ u);
            u += 1;
        }
    }

    #[inline(always)]
    fn cell_xor_mark_safe(
        dst_cells: &[Cell<u32>],
        ep: &[Cell<u32>; 65536],
        src: &[u8],
        mut u: u32,
        hi_pack: u32,
        mark_lo: u16,
        pred: &mut u32,
    ) {
        for (dc, sc) in dst_cells.iter().zip(src.chunks_exact(2)) {
            let lo = u16::from_le_bytes([sc[0], sc[1]]);
            if lo == mark_lo {
                *pred = u;
            }
            dc.set(dc.get() ^ (hi_pack | u32::from(lo)));
            let e = &ep[lo as usize];
            e.set(e.get() ^ u);
            u += 1;
        }
    }

    /// B2: 100% safe split-per-band XOR:
    /// `edges.fill(0)` + per-band (A) SIMD `xor_fwd_exact_safe` on `edges[dst..dst+span]` +
    /// (B) 4x-unrolled zero-bc `scatter_v_4x_safe` on `&mut [u32; 65536]` from L1d-hot `src`!
    #[inline(never)]
    fn b2_split_band_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        let pad = padded_len(m);
        edges[..m as usize].fill(0);
        let edges = &mut edges[..pad];
        let in_out = heads.in_out;
        let mark_epoch = mark >> 16;
        let mark_lo = (mark & 0xFFFF) as u16;
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let is_mark_byte = a == usize::from(mark_byte);
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for &b in bslice {
                let bend = b.end();
                let mut left = (bend - k) as usize;
                let mut dst = b.dst() as usize;
                let pack = b.code() << bits;
                let bytes_need = left * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    let hi = st.epoch << 16;
                    xor_fwd_exact_safe(&mut edges[dst..dst + left], src, pack | hi);
                    let ep_off = hi as usize;
                    let ep: &mut [u32; 65536] =
                        (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
                    if !is_mark_byte || st.epoch != mark_epoch {
                        scatter_v_4x_safe(ep, src, dst as u32);
                    } else {
                        scatter_v_with_mark_safe(ep, src, dst as u32, mark_lo, &mut predecessor);
                    }
                } else {
                    while left > 0 {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                        }
                        let take = left.min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let hi = st.epoch << 16;
                        xor_fwd_exact_safe(&mut edges[dst..dst + take], src, pack | hi);
                        let ep_off = hi as usize;
                        let ep: &mut [u32; 65536] =
                            (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
                        if !is_mark_byte || st.epoch != mark_epoch {
                            scatter_v_4x_safe(ep, src, dst as u32);
                        } else {
                            scatter_v_with_mark_safe(
                                ep,
                                src,
                                dst as u32,
                                mark_lo,
                                &mut predecessor,
                            );
                        }
                        dst += take;
                        left -= take;
                    }
                }
                k = bend;
            }
        }
        predecessor
    }

    /// Pass 2 helper shared by B3 (safe) and B5 (AVX2): walks `OutQ16` per byte `a` and runs
    /// 4x-unrolled zero-bounds-check `ep[lo] ^= u` (plus hoisted `mark_byte` check).
    #[inline(never)]
    fn scatter_pass2_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        edges: &mut [u32],
    ) -> u32 {
        let pad = padded_len(m);
        let edges = &mut edges[..pad];
        let in_out = heads.in_out;
        let mark_epoch = mark >> 16;
        let mark_lo = (mark & 0xFFFF) as u16;
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let is_mark_byte = a == usize::from(mark_byte);
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            let mut ep_off = (st.epoch as usize) << 16;
            let mut ep: &mut [u32; 65536] =
                (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
            for &b in bslice {
                let bend = b.end();
                let mut left = (bend - k) as usize;
                let mut dst = b.dst();
                let bytes_need = left * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    if !is_mark_byte || st.epoch != mark_epoch {
                        scatter_v_4x_safe(ep, src, dst);
                    } else {
                        scatter_v_with_mark_safe(ep, src, dst, mark_lo, &mut predecessor);
                    }
                } else {
                    while left > 0 {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                            let new_off = (st.epoch as usize) << 16;
                            if new_off != ep_off {
                                ep_off = new_off;
                                ep = (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
                            }
                        }
                        let take = left.min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        if !is_mark_byte || st.epoch != mark_epoch {
                            scatter_v_4x_safe(ep, src, dst);
                        } else {
                            scatter_v_with_mark_safe(ep, src, dst, mark_lo, &mut predecessor);
                        }
                        dst += take as u32;
                        left -= take;
                    }
                }
                k = bend;
            }
        }
        predecessor
    }

    /// B3: 100% safe 2-pass NO-FILL XOR:
    /// Pass 1 = `f2_reg_overlap_safe` (pure SIMD store, ZERO `edges.fill(0)`, ZERO RMW on `edges[u]`),
    /// Pass 2 = `scatter_pass2_safe` (4x-unrolled zero-bc `ep[lo] ^= u`, `ep` hoisted per epoch!).
    #[inline(never)]
    fn b3_2pass_nofill_safe<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        f2_reg_overlap_safe::<C, S>(out, spill, heads, bands, cursors, m, bits, edges);
        scatter_pass2_safe::<C, S>(out, spill, heads, bands, cursors, m, mark, mark_byte, edges)
    }

    /// B4: runtime-dispatched AVX2 split-per-band XOR (`edges.fill(0)` + AVX2 `xor_fwd_exact_avx2`
    /// + 4x-unrolled zero-bc `scatter_v_4x_safe` from L1d-hot `src`).
    #[target_feature(enable = "avx2")]
    #[inline(never)]
    unsafe fn b4_split_band_avx2<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        let pad = padded_len(m);
        edges[..m as usize].fill(0);
        let edges = &mut edges[..pad];
        let in_out = heads.in_out;
        let mark_epoch = mark >> 16;
        let mark_lo = (mark & 0xFFFF) as u16;
        let mut predecessor = mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(bands.len());
            let bslice = &bands[cursors[a]..end];
            if bslice.is_empty() {
                continue;
            }
            let is_mark_byte = a == usize::from(mark_byte);
            let mut st = cursor_init::<C, S>(out, spill, heads, a);
            for &b in bslice {
                let bend = b.end();
                let mut left = (bend - k) as usize;
                let mut dst = b.dst() as usize;
                let pack = b.code() << bits;
                let bytes_need = left * 2;
                if bytes_need <= st.cur.len() {
                    let (src, rest) = st.cur.split_at(bytes_need);
                    st.cur = rest;
                    let hi = st.epoch << 16;
                    xor_fwd_exact_avx2(&mut edges[dst..dst + left], src, pack | hi);
                    let ep_off = hi as usize;
                    let ep: &mut [u32; 65536] =
                        (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
                    if !is_mark_byte || st.epoch != mark_epoch {
                        scatter_v_4x_safe(ep, src, dst as u32);
                    } else {
                        scatter_v_with_mark_safe(ep, src, dst as u32, mark_lo, &mut predecessor);
                    }
                } else {
                    while left > 0 {
                        if st.cur.is_empty() {
                            st = cursor_refill::<C, S>(
                                out,
                                spill,
                                in_out,
                                a,
                                st.id,
                                st.at_end,
                                st.read_end,
                                st.epoch,
                                st.cut,
                            );
                        }
                        let take = left.min(st.cur.len() / 2);
                        let (src, rest) = st.cur.split_at(take * 2);
                        st.cur = rest;
                        let hi = st.epoch << 16;
                        xor_fwd_exact_avx2(&mut edges[dst..dst + take], src, pack | hi);
                        let ep_off = hi as usize;
                        let ep: &mut [u32; 65536] =
                            (&mut edges[ep_off..ep_off + 65536]).try_into().unwrap();
                        if !is_mark_byte || st.epoch != mark_epoch {
                            scatter_v_4x_safe(ep, src, dst as u32);
                        } else {
                            scatter_v_with_mark_safe(
                                ep,
                                src,
                                dst as u32,
                                mark_lo,
                                &mut predecessor,
                            );
                        }
                        dst += take;
                        left -= take;
                    }
                }
                k = bend;
            }
        }
        predecessor
    }

    /// B5: runtime-dispatched AVX2 2-pass NO-FILL XOR:
    /// Pass 1 = `f_avx2_impl::<16w>` (pure AVX2 store, ZERO `edges.fill(0)`),
    /// Pass 2 = `scatter_pass2_safe` (4x-unrolled zero-bc `ep[lo] ^= u`, `ep` hoisted per epoch).
    #[inline(never)]
    fn b5_2pass_nofill_avx2<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill<S>,
        heads: &Heads,
        bands: &[Band],
        cursors: &[usize; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
        bits: u32,
        edges: &mut [u32],
    ) -> u32 {
        unsafe {
            f_avx2_impl::<C, S, true, false>(out, spill, heads, bands, cursors, m, bits, edges);
        }
        scatter_pass2_safe::<C, S>(out, spill, heads, bands, cursors, m, mark, mark_byte, edges)
    }

    // ---- Guard corpora generators (guaranteed Open-admitted blocks) ----

    fn compress_bz2(raw: &[u8]) -> Vec<u8> {
        let mut enc = BzEncoder::new(Vec::new(), Compression::new(9));
        enc.write_all(raw).unwrap();
        enc.finish().unwrap()
    }

    fn gen_guard_corpora() -> Vec<(&'static str, Vec<OpenBlock>)> {
        // 1. Structured JSON-like text (r/n ~ 0.08..0.14 -> Open-admitted)
        let mut text_raw = Vec::with_capacity(5_400_000);
        let keys = [
            "{\"id\":\"Q",
            "\",\"type\":\"item\",\"labels\":{\"en\":{\"language\":\"en\",\"value\":\"",
            "\"}},\"descriptions\":{\"en\":{\"language\":\"en\",\"value\":\"",
            "\"}}}\n",
        ];
        let vals = [
            "human",
            "village in Poland",
            "asteroid",
            "scientific article",
            "taxon",
            "river",
            "mountain",
            "painting",
            "film",
            "album",
            "book",
            "city",
            "commune in France",
            "municipality of Spain",
            "species of insect",
            "church building",
        ];
        let mut x = 0x1234_5678_9abc_def1u64;
        while text_raw.len() < 5_400_000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            text_raw.extend_from_slice(keys[0].as_bytes());
            text_raw.extend_from_slice(((x % 99999) + 10000).to_string().as_bytes());
            text_raw.extend_from_slice(keys[1].as_bytes());
            text_raw.extend_from_slice(vals[(x as usize) % vals.len()].as_bytes());
            text_raw.extend_from_slice(keys[2].as_bytes());
            text_raw.extend_from_slice(vals[((x >> 8) as usize) % vals.len()].as_bytes());
            text_raw.extend_from_slice(keys[3].as_bytes());
        }
        let text_blocks = extract_open_blocks(&compress_bz2(&text_raw));

        // 2. Dense semi-structured records (r/n ~ 0.14..0.22 -> dense-band + Open-admitted)
        let mut rec_raw = Vec::with_capacity(5_400_000);
        let mut y = 0xfedc_ba98_7654_3211u64;
        while rec_raw.len() < 5_400_000 {
            y ^= y << 13;
            y ^= y >> 7;
            y ^= y << 17;
            let line = format!(
                "2026-11-27T{:02}:{:02}:{:02}Z host=srv-{:02} code={} ms={:03} path=/api/v1/item/{:04}\n",
                y % 24,
                (y >> 5) % 60,
                (y >> 11) % 60,
                (y >> 17) % 32,
                if (y >> 23) % 8 == 0 { 404 } else { 200 },
                (y >> 26) % 250,
                (y >> 34) % 2048
            );
            rec_raw.extend_from_slice(line.as_bytes());
        }
        let rec_blocks = extract_open_blocks(&compress_bz2(&rec_raw));

        // 3. Periodic / run-heavy corpus (r/n ~ 0.02..0.06 -> long bands)
        let mut runs_raw = Vec::with_capacity(5_400_000);
        let mut z = 0x55aa_1122_3344_7789u64;
        let pat = [b"AAAAABBBBB", b"CCCCCDDDDD", b"EEEEEFFFFF", b"0000011111"];
        while runs_raw.len() < 5_400_000 {
            z ^= z << 13;
            z ^= z >> 7;
            z ^= z << 17;
            let p = pat[(z as usize) % pat.len()];
            let reps = 1 + ((z >> 4) % 6) as usize;
            for _ in 0..reps {
                runs_raw.extend_from_slice(p);
            }
            runs_raw.push(b'a' + ((z >> 10) % 12) as u8);
        }
        let runs_blocks = extract_open_blocks(&compress_bz2(&runs_raw));

        vec![
            ("guard_text", text_blocks),
            ("guard_records", rec_blocks),
            ("guard_runs", runs_blocks),
        ]
    }

    // ---- Exactness & Synthetic Stress Verification ----

    fn verify_all_blocks(name: &str, blocks: &[OpenBlock]) -> usize {
        let mut checks = 0usize;
        for (bi, blk) in blocks.iter().enumerate() {
            let pad = padded_len(blk.m);
            let mut ref_fwd = vec![0xDEAD_BEEFu32; pad];
            f0_shipped::<512, 512>(
                &blk.out512,
                &blk.spill512,
                &blk.heads512,
                &blk.bands,
                &blk.cursors,
                blk.m,
                blk.bits,
                &mut ref_fwd,
            );
            let mut ref_bidi = vec![0xDEAD_BEEFu32; pad];
            let ref_pred = b0_shipped::<512, 512>(
                &blk.out512,
                &blk.spill512,
                &blk.heads512,
                &blk.bands,
                &blk.cursors,
                blk.m,
                blk.mark,
                blk.bits,
                &mut ref_bidi,
            );

            let m = blk.m as usize;
            // Forward arms
            for arm in 0..7 {
                let mut got = vec![0xA5A5_A5A5u32; pad];
                match arm {
                    0 => f1_reg_cursor_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.bits,
                        &mut got,
                    ),
                    1 => f2_reg_overlap_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.bits,
                        &mut got,
                    ),
                    2 => unsafe {
                        f_avx2_impl::<512, 512, false, false>(
                            &blk.out512,
                            &blk.spill512,
                            &blk.heads512,
                            &blk.bands,
                            &blk.cursors,
                            blk.m,
                            blk.bits,
                            &mut got,
                        );
                    },
                    3 => unsafe {
                        f_avx2_impl::<512, 512, true, false>(
                            &blk.out512,
                            &blk.spill512,
                            &blk.heads512,
                            &blk.bands,
                            &blk.cursors,
                            blk.m,
                            blk.bits,
                            &mut got,
                        );
                    },
                    4 => unsafe {
                        f_avx2_impl::<512, 512, true, true>(
                            &blk.out512,
                            &blk.spill512,
                            &blk.heads512,
                            &blk.bands,
                            &blk.cursors,
                            blk.m,
                            blk.bits,
                            &mut got,
                        );
                    },
                    5 => f2_reg_overlap_safe::<1008, 1024>(
                        &blk.out1008,
                        &blk.spill1008,
                        &blk.heads1008,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.bits,
                        &mut got,
                    ),
                    6 => unsafe {
                        f_avx2_impl::<1008, 1024, true, false>(
                            &blk.out1008,
                            &blk.spill1008,
                            &blk.heads1008,
                            &blk.bands,
                            &blk.cursors,
                            blk.m,
                            blk.bits,
                            &mut got,
                        );
                    },
                    _ => unreachable!(),
                }
                assert_eq!(
                    &got[..m],
                    &ref_fwd[..m],
                    "{name} blk {bi} fwd arm {arm} mismatch"
                );
                checks += 1;
            }

            // Bidi arms
            for arm in 0..9 {
                let mut got = vec![0xA5A5_A5A5u32; pad];
                let pred = match arm {
                    0 => b1_reg_zerobc_nomark_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    1 => b1b_cell_zerobc_4x_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    2 => b2_split_band_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    3 => b3_2pass_nofill_safe::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    4 => unsafe {
                        b4_split_band_avx2::<512, 512>(
                            &blk.out512,
                            &blk.spill512,
                            &blk.heads512,
                            &blk.bands,
                            &blk.cursors,
                            blk.m,
                            blk.mark,
                            blk.mark_byte,
                            blk.bits,
                            &mut got,
                        )
                    },
                    5 => b5_2pass_nofill_avx2::<512, 512>(
                        &blk.out512,
                        &blk.spill512,
                        &blk.heads512,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    6 => b1b_cell_zerobc_4x_safe::<1008, 1024>(
                        &blk.out1008,
                        &blk.spill1008,
                        &blk.heads1008,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    7 => b3_2pass_nofill_safe::<1008, 1024>(
                        &blk.out1008,
                        &blk.spill1008,
                        &blk.heads1008,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    8 => b5_2pass_nofill_avx2::<1008, 1024>(
                        &blk.out1008,
                        &blk.spill1008,
                        &blk.heads1008,
                        &blk.bands,
                        &blk.cursors,
                        blk.m,
                        blk.mark,
                        blk.mark_byte,
                        blk.bits,
                        &mut got,
                    ),
                    _ => unreachable!(),
                };
                assert_eq!(
                    pred, ref_pred,
                    "{name} blk {bi} bidi arm {arm} pred mismatch"
                );
                assert_eq!(
                    &got[..m],
                    &ref_bidi[..m],
                    "{name} blk {bi} bidi arm {arm} edges mismatch"
                );
                checks += 1;
            }
        }
        checks
    }

    /// Synthetic stress tests: tiny `out` forcing overflow chunks (`ovf > 0`), exact `EPOCH = 65536`
    /// boundaries, `m < 65536`, `m == 65536`, `mark` at `0, 65535, 65536, m-1`, and self-loop `u == v`.
    fn run_stress_tests() -> usize {
        let mut checks = 0usize;
        for &(m, out_bytes) in &[
            (131u32, 1024usize),    // tiny, forces ovf
            (65_535u32, 4096usize), // just below epoch, heavy ovf
            (65_536u32, 4096usize), // exact epoch boundary, heavy ovf
            (70_000u32, 4096usize), // crosses epoch, heavy ovf
            (70_000u32, 300_000usize),
        ] {
            let mut out512 = vec![0u8; out_bytes];
            let mut spill512 = Spill::<512>::default();
            let mut q512 = Queues::<512, 512>::new(&mut out512, &mut spill512, m);
            let mut out1008 = vec![0u8; out_bytes];
            let mut spill1008 = Spill::<1024>::default();
            let mut q1008 = Queues::<1008, 1024>::new(&mut out1008, &mut spill1008, m);
            let mut thin = [0u32; 256];
            let mut pos = 0u32;
            let mut byte_of_rank = vec![0u8; m as usize];
            let mut x = 0x9e37_79b9u32 ^ m;
            while pos < m {
                x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                let a = (x >> 24) as usize % 8; // 8 active bytes -> dense queues + multi-chunk
                let w = (1 + ((x >> 16) % 16)).min(m - pos);
                q512.push_shipped(a, pos, w, &thin);
                q1008.push_hoisted(a, pos, w, &thin);
                for j in pos..pos + w {
                    byte_of_rank[j as usize] = a as u8;
                }
                thin[a] += w;
                pos += w;
            }
            let heads512 = q512.heads();
            let heads1008 = q1008.heads();
            if out_bytes <= 4096 && m >= 1024 {
                assert!(
                    !spill512.ovf.is_empty() && !spill1008.ovf.is_empty(),
                    "stress test must exercise ovf chunks"
                );
            }
            // Build bands that partition [0..m) with arbitrary spans (1,2,3,4,7,8,15,16,33,600)
            // including identity/self-loop positions where u == v!
            let mut bands = Vec::new();
            let mut cursors = [0usize; 256];
            let mut k = 0u32;
            let mut dst = 0u32;
            let span_pat = [1u32, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 129, 600];
            let mut pi = 0usize;
            for a in 0..256 {
                cursors[a] = bands.len();
                let mut rem = thin[a];
                while rem > 0 {
                    let sp = span_pat[pi % span_pat.len()].min(rem);
                    pi += 1;
                    // Alternate identity dst=k (forces u==v self-loops!) and shifted dst
                    let band_dst = dst;
                    bands.push(Band::new(k + sp, band_dst, (a as u32) + (pi as u32 & 63)));
                    k += sp;
                    dst += sp;
                    rem -= sp;
                }
            }
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let marks: Vec<u32> = [0, 1, 65_535, 65_536, m - 1]
                .into_iter()
                .filter(|&mk| mk < m)
                .collect();
            for mk in marks {
                let blk = OpenBlock {
                    n: m,
                    m,
                    mark: mk,
                    mark_byte: byte_of_rank[mk as usize],
                    p: 0,
                    bits,
                    recs: Vec::new(),
                    out512: out512.clone(),
                    spill512: spill512.clone(),
                    heads512,
                    out1008: out1008.clone(),
                    spill1008: spill1008.clone(),
                    heads1008,
                    bands: bands.clone(),
                    cursors,
                };
                checks += verify_all_blocks("stress", &[blk]);
            }
        }
        checks
    }

    // ---- Statistics helpers ----

    fn median(v: &[f64]) -> f64 {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        s[s.len() / 2]
    }

    fn min_max(v: &[f64]) -> (f64, f64) {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[0], s[s.len() - 1])
    }

    /// Paired geometric mean ratio (cand / base) and 95% Student's t CI (df = n-1).
    fn paired_ci(cand: &[f64], base: &[f64]) -> (f64, f64, f64) {
        let n = cand.len();
        let logs: Vec<f64> = cand
            .iter()
            .zip(base)
            .map(|(&c, &b)| (c.max(1e-6) / b.max(1e-6)).ln())
            .collect();
        let mean = logs.iter().sum::<f64>() / n as f64;
        let var = logs.iter().map(|&x| (x - mean).powi(2)).sum::<f64>()
            / (n.saturating_sub(1).max(1) as f64);
        let se = (var / n as f64).sqrt();
        // t_{0.975, df} for df=6 is 2.4469, df=4 is 2.7764, df=8 is 2.3060
        let t = match n.saturating_sub(1) {
            1 => 12.7062,
            2 => 4.3027,
            3 => 3.1824,
            4 => 2.7764,
            5 => 2.5706,
            6 => 2.4469,
            7 => 2.3646,
            8 => 2.3060,
            _ => 2.2622,
        };
        (mean.exp(), (mean - t * se).exp(), (mean + t * se).exp())
    }

    fn bench_track_f(label: &str, blocks: &[OpenBlock], repeats: usize, cold: bool, mult: usize) {
        let max_pad = blocks.iter().map(|b| padded_len(b.m)).max().unwrap_or(0);
        let mut edges_warm = vec![0u32; max_pad];
        let mut cold_edges: Vec<Vec<u32>> = if cold {
            blocks.iter().map(|b| vec![0u32; padded_len(b.m)]).collect()
        } else {
            Vec::new()
        };
        let mut evict = if cold {
            vec![0u8; 64 << 20]
        } else {
            Vec::new()
        };

        let arms = [
            "F0_shipped_q512",
            "F1_reg_cursor_safe",
            "F2_reg_overlap_safe",
            "F3a_avx2_8w",
            "F3b_avx2_16w",
            "F4_avx2_16w_prefetch",
            "F5_reg_q1008_safe",
            "F6_avx2_16w_q1008",
        ];
        let mut cpu: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
        let mut wall: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];

        for r in 0..=repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for ai in order {
                if cold {
                    for b in evict.iter_mut().step_by(64) {
                        *b = b.wrapping_add(1);
                    }
                    black_box(&evict);
                }
                let c0 = thread_cpu_ns();
                let t0 = Instant::now();
                let mut chk = 0u64;
                for _ in 0..mult {
                    for (bi, blk) in blocks.iter().enumerate() {
                        let ebuf: &mut [u32] = if cold {
                            &mut cold_edges[bi]
                        } else {
                            &mut edges_warm
                        };
                        match ai {
                            0 => f0_shipped::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.bits,
                                ebuf,
                            ),
                            1 => f1_reg_cursor_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.bits,
                                ebuf,
                            ),
                            2 => f2_reg_overlap_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.bits,
                                ebuf,
                            ),
                            3 => unsafe {
                                f_avx2_impl::<512, 512, false, false>(
                                    &blk.out512,
                                    &blk.spill512,
                                    &blk.heads512,
                                    &blk.bands,
                                    &blk.cursors,
                                    blk.m,
                                    blk.bits,
                                    ebuf,
                                );
                            },
                            4 => unsafe {
                                f_avx2_impl::<512, 512, true, false>(
                                    &blk.out512,
                                    &blk.spill512,
                                    &blk.heads512,
                                    &blk.bands,
                                    &blk.cursors,
                                    blk.m,
                                    blk.bits,
                                    ebuf,
                                );
                            },
                            5 => unsafe {
                                f_avx2_impl::<512, 512, true, true>(
                                    &blk.out512,
                                    &blk.spill512,
                                    &blk.heads512,
                                    &blk.bands,
                                    &blk.cursors,
                                    blk.m,
                                    blk.bits,
                                    ebuf,
                                );
                            },
                            6 => f2_reg_overlap_safe::<1008, 1024>(
                                &blk.out1008,
                                &blk.spill1008,
                                &blk.heads1008,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.bits,
                                ebuf,
                            ),
                            7 => unsafe {
                                f_avx2_impl::<1008, 1024, true, false>(
                                    &blk.out1008,
                                    &blk.spill1008,
                                    &blk.heads1008,
                                    &blk.bands,
                                    &blk.cursors,
                                    blk.m,
                                    blk.bits,
                                    ebuf,
                                );
                            },
                            _ => unreachable!(),
                        }
                        chk ^= u64::from(ebuf[0]) ^ u64::from(ebuf[blk.m as usize - 1]);
                    }
                }
                let wt = t0.elapsed().as_secs_f64() * 1e3;
                let ct = (thread_cpu_ns().saturating_sub(c0)) as f64 / 1e6;
                black_box(chk);
                if r > 0 {
                    cpu[ai].push(ct);
                    wall[ai].push(wt);
                }
            }
        }
        println!(
            "\n=== TRACK F — FORWARD WIRE ({label}, {}, {repeats} repeats, x{mult}) ===",
            if cold { "L3-COLD" } else { "WARM" }
        );
        let (bmin, bmax) = min_max(&cpu[0]);
        for (ai, name) in arms.iter().enumerate() {
            let cm = median(&cpu[ai]);
            let (lo, hi) = min_max(&cpu[ai]);
            let (gm, ci_lo, ci_hi) = paired_ci(&cpu[ai], &cpu[0]);
            let wm = median(&wall[ai]);
            let disj = if hi < bmin {
                "DISJ_FASTER"
            } else if lo > bmax {
                "DISJ_SLOWER"
            } else {
                "overlap"
            };
            println!(
                "  {:<22} CPU {:6.2} ms [{:6.2}..{:6.2}] | ratio {:.4}x [{:.4}, {:.4}] ({:+6.2}%) {:<11} | wall {:6.2} ms",
                name,
                cm,
                lo,
                hi,
                gm,
                ci_lo,
                ci_hi,
                (gm - 1.0) * 100.0,
                disj,
                wm
            );
        }
    }

    fn bench_track_b(label: &str, blocks: &[OpenBlock], repeats: usize, cold: bool, mult: usize) {
        let max_pad = blocks.iter().map(|b| padded_len(b.m)).max().unwrap_or(0);
        let mut edges_warm = vec![0u32; max_pad];
        let mut cold_edges: Vec<Vec<u32>> = if cold {
            blocks.iter().map(|b| vec![0u32; padded_len(b.m)]).collect()
        } else {
            Vec::new()
        };
        let mut evict = if cold {
            vec![0u8; 64 << 20]
        } else {
            Vec::new()
        };

        let arms = [
            "B0_shipped_q512",
            "B1_reg_zerobc_nomark",
            "B1b_cell_zerobc_4x",
            "B2_split_band_safe",
            "B3_2pass_nofill_safe",
            "B4_split_band_avx2",
            "B5_2pass_nofill_avx2",
            "B6_cell_q1008_safe",
            "B7_2pass_q1008_avx2",
        ];
        let mut cpu: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
        let mut wall: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];

        for r in 0..=repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for ai in order {
                if cold {
                    for b in evict.iter_mut().step_by(64) {
                        *b = b.wrapping_add(1);
                    }
                    black_box(&evict);
                }
                let c0 = thread_cpu_ns();
                let t0 = Instant::now();
                let mut chk = 0u64;
                for _ in 0..mult {
                    for (bi, blk) in blocks.iter().enumerate() {
                        let ebuf: &mut [u32] = if cold {
                            &mut cold_edges[bi]
                        } else {
                            &mut edges_warm
                        };
                        let pred = match ai {
                            0 => b0_shipped::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.bits,
                                ebuf,
                            ),
                            1 => b1_reg_zerobc_nomark_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            2 => b1b_cell_zerobc_4x_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            3 => b2_split_band_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            4 => b3_2pass_nofill_safe::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            5 => unsafe {
                                b4_split_band_avx2::<512, 512>(
                                    &blk.out512,
                                    &blk.spill512,
                                    &blk.heads512,
                                    &blk.bands,
                                    &blk.cursors,
                                    blk.m,
                                    blk.mark,
                                    blk.mark_byte,
                                    blk.bits,
                                    ebuf,
                                )
                            },
                            6 => b5_2pass_nofill_avx2::<512, 512>(
                                &blk.out512,
                                &blk.spill512,
                                &blk.heads512,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            7 => b1b_cell_zerobc_4x_safe::<1008, 1024>(
                                &blk.out1008,
                                &blk.spill1008,
                                &blk.heads1008,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            8 => b5_2pass_nofill_avx2::<1008, 1024>(
                                &blk.out1008,
                                &blk.spill1008,
                                &blk.heads1008,
                                &blk.bands,
                                &blk.cursors,
                                blk.m,
                                blk.mark,
                                blk.mark_byte,
                                blk.bits,
                                ebuf,
                            ),
                            _ => unreachable!(),
                        };
                        chk ^= u64::from(pred) ^ u64::from(ebuf[0]);
                    }
                }
                let wt = t0.elapsed().as_secs_f64() * 1e3;
                let ct = (thread_cpu_ns().saturating_sub(c0)) as f64 / 1e6;
                black_box(chk);
                if r > 0 {
                    cpu[ai].push(ct);
                    wall[ai].push(wt);
                }
            }
        }
        println!(
            "\n=== TRACK B — BIDI XOR WIRE ({label}, {}, {repeats} repeats, x{mult}) ===",
            if cold { "L3-COLD" } else { "WARM" }
        );
        let (bmin, bmax) = min_max(&cpu[0]);
        for (ai, name) in arms.iter().enumerate() {
            let cm = median(&cpu[ai]);
            let (lo, hi) = min_max(&cpu[ai]);
            let (gm, ci_lo, ci_hi) = paired_ci(&cpu[ai], &cpu[0]);
            let wm = median(&wall[ai]);
            let disj = if hi < bmin {
                "DISJ_FASTER"
            } else if lo > bmax {
                "DISJ_SLOWER"
            } else {
                "overlap"
            };
            println!(
                "  {:<22} CPU {:6.2} ms [{:6.2}..{:6.2}] | ratio {:.4}x [{:.4}, {:.4}] ({:+6.2}%) {:<11} | wall {:6.2} ms",
                name,
                cm,
                lo,
                hi,
                gm,
                ci_lo,
                ci_hi,
                (gm - 1.0) * 100.0,
                disj,
                wm
            );
        }
    }

    /// Joint A6a + A6c benchmark pricing `OutQ16<512,512>` vs `OutQ16<1008,1024>` honestly end-to-end.
    fn bench_joint_a6a_a6c(blocks: &[OpenBlock], repeats: usize) {
        let max_n = blocks.iter().map(|b| b.n as usize).max().unwrap_or(0);
        let max_pad = blocks.iter().map(|b| padded_len(b.m)).max().unwrap_or(0);
        let mut out_buf = vec![0u8; max_n + INLINE];
        let mut edges = vec![0u32; max_pad];
        let mut strips = Vec::with_capacity(65_536);
        let mut sp512 = Spill::<512>::default();
        let mut sp1008 = Spill::<1024>::default();

        let arms = [
            "J0_fwd_shipped_q512",
            "J1_fwd_reg_ovl_q512",
            "J2_fwd_avx2_q512",
            "J3_fwd_avx2_q1008_h",
            "J4_bidi_shipped_q512",
            "J5_bidi_cell4x_q512",
            "J6_bidi_cell4x_q1008_h",
            "J7_bidi_2pass_avx2_q1008_h",
        ];
        let mut cpu: Vec<Vec<f64>> = vec![Vec::new(); arms.len()];
        for r in 0..=repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..arms.len()).collect()
            } else {
                (0..arms.len()).rev().collect()
            };
            for ai in order {
                let c0 = thread_cpu_ns();
                let mut chk = 0u64;
                for blk in blocks {
                    let out_slice = &mut out_buf[..blk.n as usize + INLINE];
                    match ai {
                        0 => {
                            let (f, _, heads, _) = run_pass1::<512, 512, false>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp512,
                                &mut strips,
                            );
                            f0_shipped::<512, 512>(
                                out_slice,
                                &sp512,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                blk.bits,
                                &mut edges,
                            );
                        }
                        1 => {
                            let (f, _, heads, _) = run_pass1::<512, 512, false>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp512,
                                &mut strips,
                            );
                            f2_reg_overlap_safe::<512, 512>(
                                out_slice,
                                &sp512,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                blk.bits,
                                &mut edges,
                            );
                        }
                        2 => {
                            let (f, _, heads, _) = run_pass1::<512, 512, false>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp512,
                                &mut strips,
                            );
                            unsafe {
                                f_avx2_impl::<512, 512, true, false>(
                                    out_slice,
                                    &sp512,
                                    &heads,
                                    &blk.bands,
                                    &blk.cursors,
                                    f.m,
                                    blk.bits,
                                    &mut edges,
                                );
                            }
                        }
                        3 => {
                            let (f, _, heads, _) = run_pass1::<1008, 1024, true>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp1008,
                                &mut strips,
                            );
                            unsafe {
                                f_avx2_impl::<1008, 1024, true, false>(
                                    out_slice,
                                    &sp1008,
                                    &heads,
                                    &blk.bands,
                                    &blk.cursors,
                                    f.m,
                                    blk.bits,
                                    &mut edges,
                                );
                            }
                        }
                        4 => {
                            let (f, _, heads, _) = run_pass1::<512, 512, false>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp512,
                                &mut strips,
                            );
                            chk ^= u64::from(b0_shipped::<512, 512>(
                                out_slice,
                                &sp512,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                f.mark,
                                blk.bits,
                                &mut edges,
                            ));
                        }
                        5 => {
                            let (f, _, heads, _) = run_pass1::<512, 512, false>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp512,
                                &mut strips,
                            );
                            chk ^= u64::from(b1b_cell_zerobc_4x_safe::<512, 512>(
                                out_slice,
                                &sp512,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                f.mark,
                                f.mark_byte,
                                blk.bits,
                                &mut edges,
                            ));
                        }
                        6 => {
                            let (f, _, heads, _) = run_pass1::<1008, 1024, true>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp1008,
                                &mut strips,
                            );
                            chk ^= u64::from(b1b_cell_zerobc_4x_safe::<1008, 1024>(
                                out_slice,
                                &sp1008,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                f.mark,
                                f.mark_byte,
                                blk.bits,
                                &mut edges,
                            ));
                        }
                        7 => {
                            let (f, _, heads, _) = run_pass1::<1008, 1024, true>(
                                &blk.recs,
                                blk.p,
                                Cap::NONE,
                                out_slice,
                                &mut sp1008,
                                &mut strips,
                            );
                            chk ^= u64::from(b5_2pass_nofill_avx2::<1008, 1024>(
                                out_slice,
                                &sp1008,
                                &heads,
                                &blk.bands,
                                &blk.cursors,
                                f.m,
                                f.mark,
                                f.mark_byte,
                                blk.bits,
                                &mut edges,
                            ));
                        }
                        _ => unreachable!(),
                    }
                    chk ^= u64::from(edges[0]);
                }
                let ct = (thread_cpu_ns().saturating_sub(c0)) as f64 / 1e6;
                black_box(chk);
                if r > 0 {
                    cpu[ai].push(ct);
                }
            }
        }
        println!("\n=== JOINT A6a + A6c (wikidata 233 Open blocks, WARM, {repeats} repeats) ===");
        for (ai, name) in arms.iter().enumerate() {
            let base_idx = if ai < 4 { 0 } else { 4 };
            let cm = median(&cpu[ai]);
            let (lo, hi) = min_max(&cpu[ai]);
            let (gm, ci_lo, ci_hi) = paired_ci(&cpu[ai], &cpu[base_idx]);
            println!(
                "  {:<26} CPU {:6.2} ms [{:6.2}..{:6.2}] | vs {} {:.4}x [{:.4}, {:.4}] ({:+6.2}%)",
                name,
                cm,
                lo,
                hi,
                if ai < 4 { "J0" } else { "J4" },
                gm,
                ci_lo,
                ci_hi,
                (gm - 1.0) * 100.0
            );
        }
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        let path = args
            .get(1)
            .map(String::as_str)
            .unwrap_or("corpora/wikidata.bz2");
        let repeats: usize = args
            .iter()
            .position(|a| a == "--repeats")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(7);
        let mult: usize = args
            .iter()
            .position(|a| a == "--mult")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let run_guards = !args.iter().any(|a| a == "--no-guards");

        let bz = std::fs::read(path).expect("read wikidata.bz2");
        let wiki_blocks = extract_open_blocks(&bz);
        let guards = gen_guard_corpora();

        println!(
            "Loaded wikidata: {} Open blocks (m={}); guards: {}",
            wiki_blocks.len(),
            wiki_blocks.iter().map(|b| u64::from(b.m)).sum::<u64>(),
            guards
                .iter()
                .map(|(n, bs)| format!(
                    "{n}={} blks (m={})",
                    bs.len(),
                    bs.iter().map(|b| u64::from(b.m)).sum::<u64>()
                ))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut total_checks = 0usize;
        total_checks += run_stress_tests();
        total_checks += verify_all_blocks("wikidata", &wiki_blocks);
        for (gname, gblks) in &guards {
            assert!(!gblks.is_empty(), "guard {gname} produced 0 Open blocks");
            total_checks += verify_all_blocks(gname, gblks);
        }
        println!("EXACTNESS: {total_checks}/{total_checks} checks 100% bit-identical (incl. ovf, EPOCH, self-loop u==v)!");

        // Primary Wikidata benchmarks (warm + L3-cold)
        bench_track_f("wikidata", &wiki_blocks, repeats, false, mult);
        bench_track_f("wikidata", &wiki_blocks, repeats, true, mult);
        bench_track_b("wikidata", &wiki_blocks, repeats, false, mult);
        bench_track_b("wikidata", &wiki_blocks, repeats, true, mult);
        bench_joint_a6a_a6c(&wiki_blocks, repeats);

        // Guard corpora (warm, 5 repeats, scaled so schedstat measures ~25-60 ms cleanly)
        if run_guards {
            for (gname, gblks) in &guards {
                let total_m: u64 = gblks.iter().map(|b| u64::from(b.m)).sum();
                let gmult = ((20_000_000 / total_m.max(1)) as usize).clamp(10, 60);
                bench_track_f(gname, gblks, 5, false, gmult);
                bench_track_b(gname, gblks, 5, false, gmult);
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    x86::run();
}
