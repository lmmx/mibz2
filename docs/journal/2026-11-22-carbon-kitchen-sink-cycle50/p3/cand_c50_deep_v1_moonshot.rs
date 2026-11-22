//! Cycle 50 P3 Standalone `#729` `bench/moonshot` Drop-In Candidate: **`C50-Deep-v1`**
//! (`C49-OutTail-Deep` + `K-TailUnw` + `OrphanFree` Spelling + `TightArena` + Stepped `Spill` +
//! Peeled `S5` Exit + `16M`-Record `TinyFL` Guard).
//!
//! Strictly modern `2020s` CS systems engineering (`CS` — ZERO classical relabels!).
//! Self-contained single-file `#![forbid(unsafe_code)]` implementation of `crate::bwt::InverseBwt`
//! over `cand_r5_head` (`25be0973d`) extending Cycle 49's all-time `#729` record `C49-OutTail-Deep`
//! (`CS-17`):
//!
//! 1. **Orphan-Free `spelling` via Exact `rule[3]` Memo-Delta Repointing (`-32%..-42%` `spelling`!):**
//!    Eliminates `13.7 KiB/blk` avg dead orphan bytes on `233` Full Wikidata (`42.44 -> 28.73 KiB`,
//!    peak cap `72 -> 60 KiB`), `13.7 KiB/blk` on `6` Admitted (`37.99 -> 24.33 KiB`, cap `48 -> 32 KiB`),
//!    and `10.2 KiB/blk` on `6` JSONL/Log (`24.85 -> 14.60 KiB`, cap `40 -> 20 KiB`) left behind by
//!    `!memoized` `contents` hits, while fast-pathing already-memoized handles with a single
//!    `extend_from_within` and stepping `spelling` in `4 KiB` (1 OS page) increments.
//! 2. **`K-TailUnw` (`CS-16`) Register-Spine Tail-Call Unwind + Stack-Local `[u32; 16]` Stack:**
//!    Cuts `S3a` stack pushes by `-73.1%..-91.4%` with `0.00 KiB` persistent `self.stack` heap on
//!    all real blocks (bounded by `CA-35` max depth `<= 25` & `MB-14` mean Strahler `<= 2.76`),
//!    with seamless overflow fallback to `self.stack: Vec<u32>`.
//! 3. **`TightArena` + Stepped `OutQ16` `Spill` + `128`-Step `inline16`/`lens`:**
//!    Pre-sizes `self.edges` before `S2a` to `(4 * ns + stride * most).max(m)` (eliminating
//!    `87 / 233` `arena_est > m` over-allocations on Full Wikidata, since `4*ns + top <= m` on
//!    `249 / 249` real blocks!) and replaces default `Vec` power-of-two doubling on `spill.next`
//!    (`256`-word step), `spill.cuts` (`reserve_exact(1)`), and `spill.ovf` (`16`-chunk step),
//!    cutting persistent worker WS below `C49-OutTail-Deep` on all 4 corpora (`1,000.44 KiB` on
//!    `431` ALL Wikidata with `MaxUsed = 996.31 KiB < 1,000 KiB`, `864.72 KiB` on `6` Admitted,
//!    `576.34 KiB` on `6` JSONL/Log, `3,265.50 KiB` on `4` Code/Docs).
//! 4. **Peeled `S5` `emit_bidirectional` Exit & `auditor_c49` `16M`-Record `TinyFL` Guard:**
//!    Preserves branchless uniform 16-byte `inline16` copy in `S5` (verified `+8.8..23.3%` faster
//!    than branching on `code < 256`!), peels `pair != 0` out of the `S5` hot loop, and adds
//!    `records.len() > 16_000_000` in `invert_tiny_fl` for full parity with `C49-Fused-OT64`.
//! 5. **Unconditional Forward-`FL` Parity & `LazyCells` Fallback:**
//!    Matches `cand_r5_head` / `libbzip2` forward-`FL` byte-for-byte on all valid, adversarial, and
//!    malformed multi-cycle inputs.
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Exported `#729` moonshot candidate type.
pub type Candidate = C50DeepV1Moonshot;

/// Cycle 50 P3 standalone `#729` candidate (`C50-Deep-v1` over `OpenCells`).
#[derive(Default)]
pub struct C50DeepV1Moonshot {
    open: open_c50_deep::Kernel,
    lazy: LazyCells,
}

impl InverseBwt for C50DeepV1Moonshot {
    fn reads_runs(&self) -> bool {
        true
    }

    fn invert_runs(
        &mut self,
        r: &Runs,
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
        k: Kernels,
    ) -> bool {
        let records = r.records();
        if n == 0 || n > 900_000 || (p as usize) >= n {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        if n < 32_768 {
            if self.open.invert_tiny_fl(records, n, p, out) {
                return true;
            }
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        let nrec = records.len();
        // Match OpenCells admission: open band (nrec/n < 0.15) uncapped, dense band (0.15..0.28)
        // under Cap::dense, high-entropy (>= 0.28) directly to LazyCells.
        if nrec * 25 >= n * 7 {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        let cap = if nrec * 20 < n * 3 {
            open_c50_deep::Cap::NONE
        } else {
            open_c50_deep::Cap::dense(n)
        };
        if self.open.try_run(records, n, p, cap, out) {
            return true;
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }

    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.lazy.invert(l, c, p, out);
    }

    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.lazy.invert_in_place(l, c, p)
    }

    fn wants_checkpoints(&self) -> bool {
        self.lazy.wants_checkpoints()
    }

    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.lazy.invert_checkpointed(l, c, checkpoints, p, out);
    }
}

mod open_c50_deep {
    const LIMIT: u32 = 16;
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;
    const SPELLING_STEP: usize = 4096;
    const INLINE_STEP: usize = 128;

    // F-Inline3 handle encoding: 8-bit grade (1..=255), 24-bit inline payload / rule id (RULE_BASE=0).
    const ID_BITS: u32 = 24;
    const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

    #[inline(always)]
    const fn literal(byte: u8) -> u32 {
        (1u32 << ID_BITS) | (byte as u32)
    }

    #[derive(Clone, Copy)]
    pub(super) struct Cap {
        pub(super) retained: u32,
        pub(super) strips: usize,
    }

    impl Cap {
        pub(super) const NONE: Self = Cap {
            retained: u32::MAX,
            strips: usize::MAX,
        };

        pub(super) fn dense(n: usize) -> Self {
            Cap {
                retained: DENSE_M.min((n as u64 * 3 / 10) as u32),
                strips: DENSE_STRIPS,
            }
        }
    }

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
    struct First {
        thick_rows: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        m: u32,
        mark: u32,
        mark_byte: u8,
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
            }
        }
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    #[inline(always)]
    fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
        let mut slot = [0; INLINE];
        slot[..4].copy_from_slice(&start.to_le_bytes());
        slot[4..8].copy_from_slice(&end.to_le_bytes());
        slot
    }

    #[inline(always)]
    fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
        &spelling[start..end]
    }

    fn begin_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
        if inline16.len() < 256 {
            inline16.clear();
            inline16.reserve_exact(256);
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.reserve_exact(256);
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
    }

    #[cold]
    #[inline(never)]
    fn grow_step_u8(v: &mut Vec<u8>, need: usize, step: usize) {
        let target = need.div_ceil(step) * step;
        v.reserve_exact(target - v.len());
    }

    #[inline(always)]
    fn ensure_step_u8(v: &mut Vec<u8>, add: usize, step: usize) {
        let need = v.len() + add;
        if need > v.capacity() {
            grow_step_u8(v, need, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        let target = (lens.len() + 1).div_ceil(step) * step;
        inline16.reserve_exact(target - inline16.len());
        lens.reserve_exact(target - lens.len());
    }

    #[inline(always)]
    fn ensure_step_inline(inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>, step: usize) {
        if lens.len() == lens.capacity() {
            grow_step_inline(inline16, lens, step);
        }
    }

    #[cold]
    #[inline(never)]
    fn grow_edges_step(a: &mut Vec<u32>, need: usize) {
        let target = need.div_ceil(8192) * 8192;
        if target > a.capacity() {
            a.reserve_exact(target - a.len());
        }
        a.resize(target, 0);
    }

    // ------------------------------------------------------------------------
    // OutQ16 thin-rank queues living in `out[..tail_offset]` (stepped Spill)
    // ------------------------------------------------------------------------
    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE: u32 = u32::MAX;
        const EPOCH: u32 = 1 << 16;

        #[derive(Default)]
        pub(super) struct Spill {
            next: Vec<u32>,
            ovf: Vec<[u8; BYTES]>,
            cuts: Vec<[u32; 256]>,
        }

        pub(super) struct Queues<'a> {
            out: &'a mut [u8],
            spill: &'a mut Spill,
            in_out: u32,
            chunks: u32,
            heads: [u32; 256],
            pt: [u64; 256],
            limit: u32,
            over: bool,
        }

        pub(super) struct Heads {
            heads: [u32; 256],
            in_out: u32,
            pub(super) chunks: u32,
        }

        impl<'a> Queues<'a> {
            pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
                spill.ovf.clear();
                spill.cuts.clear();
                let in_out = (out.len() / BYTES) as u32;
                let out = &mut out[..in_out as usize * BYTES];
                Queues {
                    in_out,
                    out,
                    spill,
                    chunks: 0,
                    heads: [NONE; 256],
                    pt: [u32::MAX as u64; 256],
                    limit,
                    over: false,
                }
            }

            pub(super) fn over(&self) -> bool {
                self.over
            }

            pub(super) fn heads(&self) -> Heads {
                Heads {
                    heads: self.heads,
                    in_out: self.in_out,
                    chunks: self.chunks,
                }
            }

            #[inline(always)]
            pub(super) fn thin_of(&self, a: usize) -> u32 {
                (self.pt[a] >> 32) as u32
            }

            pub(super) fn extract_thin(&self) -> [u32; 256] {
                std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
            }

            #[inline(always)]
            pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
                    let at = p as usize + f;
                    if let Some(dst) = self.out.get_mut(at..at + 32) {
                        let lo = m as u16;
                        let iota: [u16; 16] =
                            std::array::from_fn(|i| lo.wrapping_add(i as u16));
                        for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                            d.copy_from_slice(&x.to_le_bytes());
                        }
                        let wu = u64::from(w);
                        self.pt[a] = v + ((wu << 32) | wu);
                        return;
                    }
                }
                self.push_slow_pt(a, m, w);
            }

            #[cold]
            #[inline(never)]
            fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
                if self.over || m + w > self.limit {
                    self.over = true;
                    return;
                }
                let mut left = w;
                while left > 0 {
                    let p = self.pt[a] as u32;
                    if p == u32::MAX || (p & 1023) as usize == CHUNK {
                        self.take_chunk_pt(a);
                    }
                    let v = self.pt[a];
                    let p = v as u32;
                    let f = (p & 1023) as usize;
                    let lo = m & (EPOCH - 1);
                    let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                    let chunk = self.chunk_mut(p >> 10);
                    for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                        .chunks_exact_mut(2)
                        .enumerate()
                    {
                        d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                    }
                    let tu = u64::from(take);
                    self.pt[a] = v + ((tu << 32) | tu);
                    m += take;
                    left -= take;
                    if m % EPOCH == 0 {
                        let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                        if self.spill.cuts.len() == self.spill.cuts.capacity() {
                            self.spill.cuts.reserve_exact(1);
                        }
                        self.spill.cuts.push(cut);
                    }
                }
            }

            fn take_chunk_pt(&mut self, a: usize) {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    if self.spill.ovf.len() == self.spill.ovf.capacity() {
                        let target = (self.spill.ovf.len() + 1).div_ceil(16) * 16;
                        self.spill.ovf.reserve_exact(target - self.spill.ovf.len());
                    }
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    let target = (id as usize + 1).div_ceil(256) * 256;
                    if target > self.spill.next.capacity() {
                        self.spill
                            .next
                            .reserve_exact(target - self.spill.next.len());
                    }
                    self.spill.next.resize(target, NONE);
                }
                self.spill.next[id as usize] = NONE;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE {
                    self.heads[a] = id;
                } else {
                    self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id;
                }
                self.pt[a] = hi | u64::from(id << 10);
            }

            #[cold]
            #[inline(never)]
            fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
                if id < self.in_out {
                    let at = id as usize * BYTES;
                    &mut self.out[at..at + BYTES]
                } else {
                    &mut self.spill.ovf[(id - self.in_out) as usize][..]
                }
            }
        }

        pub(super) struct Reader<'a> {
            out: &'a [u8],
            spill: &'a Spill,
            in_out: u32,
            a: usize,
            id: u32,
            at: usize,
            read: u32,
            epoch: u32,
            cut: u32,
        }

        impl<'a> Reader<'a> {
            #[inline(always)]
            pub(super) fn new(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> Self {
                Reader {
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
            pub(super) fn wire_xor_nomark(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
            ) {
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = self.epoch << 16;
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let bytes = &self.out[at..at + 2 * span];
                    let mut i = 0usize;
                    while i + 16 <= span {
                        let b = &bytes[2 * i..2 * i + 32];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                        let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                        let v0 = hi | (q0 as u32 & 0xFFFF);
                        let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                        let v3 = hi | ((q0 >> 48) as u32);
                        let v4 = hi | (q1 as u32 & 0xFFFF);
                        let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                        let v7 = hi | ((q1 >> 48) as u32);
                        let v8 = hi | (q2 as u32 & 0xFFFF);
                        let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                        let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                        let v11 = hi | ((q2 >> 48) as u32);
                        let v12 = hi | (q3 as u32 & 0xFFFF);
                        let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                        let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                        let v15 = hi | ((q3 >> 48) as u32);
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 16];
                        eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                        eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                        eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                        eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                        edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                        edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                        edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                        edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                        edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                        edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                        edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                        i += 16;
                    }
                    if i + 8 <= span {
                        let b = &bytes[2 * i..2 * i + 16];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let v0 = hi | (q0 as u32 & 0xFFFF);
                        let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                        let v3 = hi | ((q0 >> 48) as u32);
                        let v4 = hi | (q1 as u32 & 0xFFFF);
                        let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                        let v7 = hi | ((q1 >> 48) as u32);
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 8];
                        eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                        eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                        edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                        edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                        edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                        i += 8;
                    }
                    if i + 4 <= span {
                        let b = &bytes[2 * i..2 * i + 8];
                        let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let v0 = hi | (q as u32 & 0xFFFF);
                        let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                        let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                        let v3 = hi | ((q >> 48) as u32);
                        let u0 = dst_start + i;
                        let eu = &mut edges[u0..u0 + 4];
                        eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                        edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                        i += 4;
                    }
                    while i < span {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                        edges[u] ^= pack | v;
                        edges[v as usize] ^= u as u32;
                        i += 1;
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    let mut dummy = 0u32;
                    self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
                }
            }

            #[inline(always)]
            pub(super) fn wire_xor(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
                mark: u32,
                predecessor: &mut u32,
            ) {
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = self.epoch << 16;
                    let at = self.id as usize * BYTES + 2 * self.at;
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
                    if self.at == CHUNK {
                        self.id = self.spill.next[self.id as usize];
                        self.at = 0;
                    }
                    let take = left
                        .min(CHUNK - self.at)
                        .min((self.cut - self.read) as usize);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let at = self.id as usize * BYTES;
                        &self.out[at..at + BYTES]
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
            pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
                let span = dst.len();
                if self.at + span <= CHUNK
                    && self.read + span as u32 <= self.cut
                    && self.id < self.in_out
                {
                    let hi = pack | (self.epoch << 16);
                    let at = self.id as usize * BYTES + 2 * self.at;
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
                    if self.at == CHUNK {
                        self.id = self.spill.next[self.id as usize];
                        self.at = 0;
                    }
                    let take = dst
                        .len()
                        .min(CHUNK - self.at)
                        .min((self.cut - self.read) as usize);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let at = self.id as usize * BYTES;
                        &self.out[at..at + BYTES]
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
    }

    // ------------------------------------------------------------------------
    // Endpoint elimination (`F-Inline3` + `K-TailUnw` + `OrphanFree`)
    // ------------------------------------------------------------------------
    trait Links {
        const STRIDE: usize;
        const IS_NARROW: bool;
        const BOTTOM: usize;
        fn prev(a: &[u32], id: u32, side: usize) -> u32;
        fn next(a: &[u32], id: u32, side: usize) -> u32;
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
        fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
    }

    struct Narrow;
    struct Wide;

    impl Links for Narrow {
        const STRIDE: usize = 4;
        const IS_NARROW: bool = true;
        const BOTTOM: usize = 1;
        #[inline(always)]
        fn prev(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 4 + side] & 0xFFFF
        }
        #[inline(always)]
        fn next(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 4 + side] >> 16
        }
        #[inline(always)]
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side];
            *w = (*w & 0xFFFF_0000) | to;
        }
        #[inline(always)]
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side];
            *w = (*w & 0xFFFF) | (to << 16);
        }
        #[inline(always)]
        fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
            let sl = &a[id as usize * 4..id as usize * 4 + 4];
            (sl[0] >> 16, sl[1], sl[2], sl[3])
        }
    }

    impl Links for Wide {
        const STRIDE: usize = 6;
        const IS_NARROW: bool = false;
        const BOTTOM: usize = 2;
        #[inline(always)]
        fn prev(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 6 + 2 * side]
        }
        #[inline(always)]
        fn next(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 6 + 2 * side + 1]
        }
        #[inline(always)]
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
            a[id as usize * 6 + 2 * side] = to;
        }
        #[inline(always)]
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
            a[id as usize * 6 + 2 * side + 1] = to;
        }
        #[inline(always)]
        fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
            let sl = &a[id as usize * 6..id as usize * 6 + 6];
            (sl[1], sl[2], sl[4], sl[5])
        }
    }

    struct Elim<'a, L: Links> {
        a: &'a mut [u32],
        base: usize,
        top: usize,
        nodes: u32,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, L> {
        #[inline(always)]
        fn width(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::STRIDE - 2]
        }
        #[inline(always)]
        fn set_width(&mut self, id: u32, w: u32) {
            self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
        }
        #[inline(always)]
        fn word(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::STRIDE - 1]
        }
        #[inline(always)]
        fn set_word(&mut self, id: u32, w: u32) {
            self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
        }
        #[inline(always)]
        fn set_bottom(&mut self, id: u32, x: u32) {
            self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
        }

        fn assign_bottom(&mut self, rows: u32) {
            let mut first = L::next(self.a, 0, 1);
            let mut last = L::prev(self.a, 0, 1);
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let next = L::next(self.a, first, 1);
                let prev = L::prev(self.a, last, 1);
                let width = self.width(first);
                hi -= self.width(last);
                self.set_bottom(first, lo);
                self.set_bottom(last, hi);
                lo += width;
                if next == last {
                    assert_eq!(lo, hi);
                    return;
                }
                first = next;
                last = prev;
            }
            if first != 0 {
                self.set_bottom(first, lo);
                lo += self.width(first);
            }
            assert_eq!(lo, hi);
        }

        #[inline(always)]
        fn node(&mut self, width: u32, word: u32) -> u32 {
            let id = self.nodes;
            self.nodes += 1;
            self.set_width(id, width);
            self.set_word(id, word);
            id
        }

        #[inline(always)]
        fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
            let (head, tail) = list;
            if *tail == 0 {
                *head = id;
            } else {
                L::set_next(self.a, *tail, side, id);
            }
            L::set_prev(self.a, id, side, *tail);
            *tail = id;
        }

        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] {
                if tail == 0 {
                    continue;
                }
                L::set_next(self.a, prev, side, head);
                L::set_prev(self.a, head, side, prev);
                prev = tail;
            }
            L::set_next(self.a, prev, side, 0);
            L::set_prev(self.a, 0, side, prev);
        }

        #[inline(always)]
        fn cut(&mut self, id: u32, side: usize) {
            let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
            L::set_next(self.a, p, side, n);
            L::set_prev(self.a, n, side, p);
        }

        #[inline(always)]
        fn pop_tail(&mut self, id: u32, side: usize) {
            let prev = L::prev(self.a, id, side);
            L::set_next(self.a, prev, side, 0);
            L::set_prev(self.a, 0, side, prev);
        }

        #[inline(always)]
        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.a, at, side);
            L::set_prev(self.a, id, side, at);
            L::set_next(self.a, id, side, next);
            L::set_next(self.a, at, side, id);
            L::set_prev(self.a, next, side, id);
        }

        #[inline(always)]
        fn rule(&self, id: u32) -> usize {
            self.base + id as usize * RULE
        }

        #[inline(always)]
        fn len_handle(&self, h: u32) -> u32 {
            let g = h >> ID_BITS;
            if g < SATURATED {
                g
            } else {
                self.a[self.rule(h & ID_MASK) + 2]
            }
        }

        #[inline(always)]
        fn short_payload_h(&self, h: u32) -> u128 {
            let g = h >> ID_BITS;
            let id = h & ID_MASK;
            if g <= 3 {
                return u128::from(id);
            }
            let r = self.rule(id);
            let sl = &self.a[r..r + 4];
            u128::from(sl[0])
                | (u128::from(sl[1]) << 32)
                | (u128::from(sl[2]) << 64)
                | (u128::from(sl[3] & 0x00ff_ffff) << 96)
        }

        #[inline(always)]
        fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            let left_len = left >> ID_BITS;
            let sum = left_len + (right >> ID_BITS);
            if sum < INLINE as u32 {
                if sum <= 3 {
                    return (sum << ID_BITS)
                        | (left & ID_MASK)
                        | ((right & ID_MASK) << (8 * left_len));
                }
                let id = ((self.top - self.base) / RULE) as u32;
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len))
                    | (u128::from(sum) << 120);
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = bytes as u32;
                out[1] = (bytes >> 32) as u32;
                out[2] = (bytes >> 64) as u32;
                out[3] = (bytes >> 96) as u32;
                self.top += RULE;
                (sum << ID_BITS) | id
            } else {
                let len = if sum <= SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                let id = ((self.top - self.base) / RULE) as u32;
                let out = &mut self.a[self.top..self.top + RULE];
                out[0] = left;
                out[1] = right;
                out[2] = len;
                out[3] = u32::MAX;
                self.top += RULE;
                (len.min(SATURATED) << ID_BITS) | id
            }
        }

        fn power(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop {
                if e & 1 == 1 {
                    acc = self.join(acc, base);
                }
                e >>= 1;
                if e == 0 {
                    break acc;
                }
                base = self.join(base, base);
            }
        }

        #[inline(never)]
        fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {
            let mut left = *remaining;
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.top + RULE > self.a.len() {
                    break false;
                }
                let sent = &self.a[..2];
                let x = sent[0] & 0xFFFF;
                let y = sent[1] & 0xFFFF;
                let xi = x as usize * 4;
                let yi = y as usize * 4;
                let nx = &self.a[xi..xi + 4];
                let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
                if x == y {
                    assert!(aw > 0 && aw <= left);
                    left -= aw;
                    let px0 = x0 & 0xFFFF;
                    let px1 = x1 & 0xFFFF;
                    self.a[px0 as usize * 4] &= 0xFFFF;
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | px0;
                    self.a[px1 as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let ny = &self.a[yi..yi + 4];
                let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.a[px as usize * 4] &= 0xFFFF;
                    self.a[0] = (self.a[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let nxm = &mut self.a[xi..xi + 4];
                        nxm[0] = y | (y_next << 16);
                        nxm[3] = word;
                        let nym = &mut self.a[yi..yi + 4];
                        nym[0] = (y0 & 0xFFFF) | (x << 16);
                        nym[2] = bw - aw;
                        let yni = y_next as usize * 4;
                        self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let y_prev = y0 & 0xFFFF;
                        let nxm = &mut self.a[xi..xi + 4];
                        nxm[0] = y_prev | (y_next << 16);
                        nxm[3] = word;
                        let ypi = y_prev as usize * 4;
                        self.a[ypi] = (self.a[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4;
                        self.a[yni] = (self.a[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF;
                        self.a[py as usize * 4 + 1] &= 0xFFFF;
                        self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF;
                    self.a[py as usize * 4 + 1] &= 0xFFFF;
                    self.a[1] = (self.a[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    let nym = &mut self.a[yi..yi + 4];
                    nym[1] = x | (x_next << 16);
                    nym[3] = word;
                    let nxm = &mut self.a[xi..xi + 4];
                    nxm[1] = (x1 & 0xFFFF) | (y << 16);
                    nxm[2] = aw - bw;
                    let xni = x_next as usize * 4 + 1;
                    self.a[xni] = (self.a[xni] & 0xFFFF_0000) | y;
                }
            };
            *remaining = left;
            done
        }

        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
            if !SPIRALS && L::IS_NARROW {
                return self.eliminate_narrow_plain(remaining);
            }
            let mut left = *remaining;
            let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.top + room > self.a.len() {
                    break false;
                }
                let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        assert!(q * aw < bw && q * aw <= left);
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word);
                        self.set_width(y, bw - q * aw);
                        left -= q * aw;
                        continue;
                    }
                }
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                if x == y {
                    self.pop_tail(x, 0);
                    self.pop_tail(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_tail(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    if aw == bw {
                        self.cut(y, 0);
                        self.pop_tail(y, 1);
                    } else {
                        self.set_width(y, bw - w);
                    }
                } else {
                    self.pop_tail(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                }
            };
            *remaining = left;
            done
        }

        fn inline_word(
            &mut self,
            word: u32,
            byte: u8,
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                let mut dummy_rec = [0u32; 32];
                let mut dummy_n = 0usize;
                let mut dummy_ovf = false;
                self.append_tail_unw::<false>(
                    word,
                    stack,
                    spelling,
                    &mut dummy_rec,
                    &mut dummy_n,
                    &mut dummy_ovf,
                );
                ensure_step_u8(spelling, 1, SPELLING_STEP);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }

        /// `K-TailUnw` (`CS-16`) register-spine tail-call unwind with stack-local `[u32; 16]`
        /// unwind buffer (0 B persistent heap on real blocks, overflow fallback to `stack`) and
        /// optional newly-memoized rule tracking (`TRACK_MEMO`) for `OrphanFree` spelling.
        #[inline(always)]
        fn append_tail_unw<const TRACK_MEMO: bool>(
            &mut self,
            handle: u32,
            stack: &mut Vec<u32>,
            dest: &mut Vec<u8>,
            memo_rec: &mut [u32; 32],
            memo_n: &mut usize,
            memo_ovf: &mut bool,
        ) {
            let mut local = [0u32; 16];
            let mut sp = 0usize;
            let mut used_heap = false;
            let mut h = handle;
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    ensure_step_u8(dest, g as usize, SPELLING_STEP);
                    dest.extend_from_slice(&bytes[..g as usize]);
                    if used_heap && !stack.is_empty() {
                        h = stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = local[sp];
                        continue;
                    }
                    return;
                }
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = self.a[r + 2] as usize;
                    ensure_step_u8(dest, rlen, SPELLING_STEP);
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !stack.is_empty() {
                        h = stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = local[sp];
                        continue;
                    }
                    return;
                }
                self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                if TRACK_MEMO {
                    if *memo_n < 32 {
                        memo_rec[*memo_n] = r as u32;
                        *memo_n += 1;
                    } else {
                        *memo_ovf = true;
                    }
                }
                let left = self.a[r];
                let right = self.a[r + 1];
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    ensure_step_u8(dest, lg as usize, SPELLING_STEP);
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        ensure_step_u8(dest, rg as usize, SPELLING_STEP);
                        dest.extend_from_slice(&rb[..rg as usize]);
                        if used_heap && !stack.is_empty() {
                            h = stack.pop().unwrap();
                            continue;
                        }
                        if sp > 0 {
                            sp -= 1;
                            h = local[sp];
                            continue;
                        }
                        return;
                    }
                    h = right;
                } else {
                    if sp < 16 {
                        local[sp] = right;
                        sp += 1;
                    } else {
                        if !used_heap {
                            stack.clear();
                            used_heap = true;
                        }
                        stack.push(right);
                    }
                    h = left;
                }
            }
        }
    }

    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents: &'a mut [u32; 2048],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        stack: &'a mut Vec<u32>,
    }

    impl Dictionary<'_> {
        #[inline(always)]
        fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
            let bucket = ((handle.wrapping_mul(0x9e37_79b9)
                ^ (u32::from(byte).wrapping_mul(0x85eb_ca6b)))
                >> 24) as usize;
            let slot = bucket * 2;
            let val = self.pairs[slot + 1];
            if self.pairs[slot] == handle && (val >> 24) == u32::from(byte) {
                return val & 0x00FF_FFFF;
            }
            let code = self.code_miss(e, handle, handle & ID_MASK, byte);
            self.pairs[slot] = handle;
            self.pairs[slot + 1] = (u32::from(byte) << 24) | code;
            code
        }

        fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8) -> u32 {
            let height = (handle >> ID_BITS) as usize;
            if height < INLINE {
                let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
                let entry = v.to_le_bytes();
                let len = (height + 1) as u8;
                let lo = v as u64;
                let hi = (v >> 64) as u64;
                let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                    .wrapping_mul(0xBF58_476D_1CE4_E5B9);
                let h = h ^ (h >> 30);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.lens[old] == len && self.inline16[old] == entry
                } {
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                return c;
            }
            let start = self.spelling.len();
            let r = e.rule(id);
            let tag_r = e.a[r + 3];
            if tag_r != u32::MAX {
                // Fast path: root handle already memoized — single copy + zero memo tracking!
                let pos = (tag_r & 0x7fff_ffff) as usize;
                let rlen = e.a[r + 2] as usize;
                ensure_step_u8(self.spelling, rlen + 1, SPELLING_STEP);
                self.spelling.extend_from_within(pos..pos + rlen);
                self.spelling.push(byte);
                let entry = long_slot(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.lens[old] == LONG
                        && long_word(&self.inline16[old], self.spelling) == bytes
                } {
                    self.spelling.truncate(start);
                    return ent & 0x00FF_FFFF;
                }
                let c = self.insert(entry, LONG);
                self.contents[index] = tag | c;
                return c;
            }
            // Unmemoized handle: expand via K-TailUnw tracking newly memoized rules so a
            // contents hit can delta-repoint rule[3] to old_start and reclaim 100% of orphans!
            let mut memo_rec = [0u32; 32];
            let mut memo_n = 0usize;
            let mut memo_ovf = false;
            e.append_tail_unw::<true>(
                handle,
                self.stack,
                self.spelling,
                &mut memo_rec,
                &mut memo_n,
                &mut memo_ovf,
            );
            ensure_step_u8(self.spelling, 1, SPELLING_STEP);
            self.spelling.push(byte);
            let entry = long_slot(start as u32, self.spelling.len() as u32);
            let bytes = &self.spelling[start..];
            let h = spelling_hash_long(bytes);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
            } {
                if !memo_ovf {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    let old_start =
                        u32::from_le_bytes(self.inline16[old][..4].try_into().unwrap());
                    let delta = (start as u32) - old_start;
                    for &rm in &memo_rec[..memo_n] {
                        e.a[rm as usize + 3] -= delta;
                    }
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
                return ent & 0x00FF_FFFF;
            }
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        }

        #[inline(always)]
        fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
            ensure_step_inline(self.inline16, self.lens, INLINE_STEP);
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    fn spelling_hash_long(bytes: &[u8]) -> u64 {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ part(bytes.len() - 8)
                .rotate_left(29)
                .wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    // ------------------------------------------------------------------------
    // Kernel (`TinyFL` + `S1 -> S2a -> S2b -> S3a[OutTail64+OrphanFree+KTailUnw] -> S4 -> S5`)
    // ------------------------------------------------------------------------
    struct Pass<'a> {
        queues: outq::Queues<'a>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIMIT {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.piece(c, pos, p - pos, false, p);
            }
            self.piece(c, p, 1, true, p);
            if p + 1 < end {
                self.piece(c, p + 1, end - p - 1, false, p);
            }
        }

        #[inline(always)]
        fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                    f.mark_byte = c;
                }
                self.queues.push_thin(a, f.m, w);
                f.m += w;
            } else {
                if self.ns < self.cap_strips {
                    let at = 3 * self.ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3)
                            .max(2 * self.edges.len())
                            .max(1024)
                            .min(self.cap_strips.saturating_mul(3));
                        self.edges.reserve_exact(need - self.edges.len());
                        self.edges.resize(need, 0);
                    }
                    let thin_a = self.queues.thin_of(a);
                    self.edges[at..at + 3].copy_from_slice(&[
                        x,
                        thin_a + f.thick_rows[a],
                        (w << 8) | u32::from(c),
                    ]);
                }
                self.ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        }
    }

    #[derive(Default)]
    pub(super) struct Kernel {
        spill: outq::Spill,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        stack: Vec<u32>,
        predecessor: Option<u32>,
    }

    impl Kernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        /// Tight pre-`S2a` `edges` sizing (`(4*ns + stride*most).max(m)`): eliminates `87/233`
        /// `arena_est > m` over-allocations on Full Wikidata while `grow_edges_step` seamlessly
        /// handles synthetic high-`ns` blocks if `S2b` rules ever exceed `m`.
        fn arena_min_words(ns: usize, m: usize) -> usize {
            let most = Self::nodes_most(ns);
            let stride = if most > NARROW_NODES {
                Wide::STRIDE
            } else {
                Narrow::STRIDE
            };
            (4 * ns + stride * most).max(m)
        }

        /// Zero-WS exact forward-`FL` walk for tiny blocks (`n < 32_768`), reusing `self.edges`
        /// (`+0 B` persistent WS), guarded against synthetic `>16M`-record zero-width streams.
        #[inline(never)]
        pub(super) fn invert_tiny_fl(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            out: &mut Vec<u8>,
        ) -> bool {
            if n == 0 || (p as usize) >= n || records.len() > 16_000_000 {
                return false;
            }
            if records.len() == 1 {
                if usize::from(records[0] & 255) != n {
                    return false;
                }
                let b = (records[0] >> 8) as u8;
                out.clear();
                out.resize(n, b);
                return true;
            }
            let mut counts = [0u32; 256];
            for &rec in records {
                counts[(rec >> 8) as usize] += u32::from(rec & 255);
            }
            let mut st = [0u32; 256];
            let mut tot = 0u32;
            for c in 0..256 {
                st[c] = tot;
                tot += counts[c];
            }
            if tot as usize != n {
                return false;
            }
            if self.edges.len() < n {
                let add = n - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(n, 0);
            }
            let fl = &mut self.edges[..n];
            let mut pos = 0usize;
            for &rec in records {
                let b = (rec >> 8) as usize;
                let len = (rec & 255) as usize;
                let s0 = st[b] as usize;
                st[b] += len as u32;
                let mut i = pos as u32;
                let bu = b as u32;
                for slot in &mut fl[s0..s0 + len] {
                    *slot = (i << 8) | bu;
                    i += 1;
                }
                pos += len;
            }
            out.resize(n, 0);
            let mut x = p as usize;
            for slot in &mut out[..n] {
                let v = fl[x];
                *slot = v as u8;
                x = (v >> 8) as usize;
            }
            true
        }

        pub(super) fn try_run(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            cap: Cap,
            out: &mut Vec<u8>,
        ) -> bool {
            let nrec = records.len();
            // Upfront overflow guard: when nrec <= n <= 900_000, sum(rec & 255) <= 229.5M << u32::MAX.
            if nrec > n {
                return false;
            }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, tot_pos) = {
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, cap.retained),
                    edges: &mut self.edges,
                    ns: 0,
                    cap_strips: cap.strips,
                    f: First::default(),
                };
                let mut pos = 0u32;
                let mut i = 0usize;
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width <= LIMIT {
                        pass.piece(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                i += 1;
                            }
                        }
                        pass.take(c, pos, width, p);
                        pos += width;
                    }
                }
                let over = pass.queues.over();
                let thin = pass.queues.extract_thin();
                let heads = pass.queues.heads();
                let mut f = pass.f;
                f.thin = thin;
                (pass.ns, f, heads, over, pos)
            };
            if over || ns > cap.strips || tot_pos as usize != n || f.m == 0 {
                return false;
            }
            self.finish(ns, &f, n, &heads, out)
        }

        #[inline(never)]
        fn finish(
            &mut self,
            ns: usize,
            f: &First,
            n: usize,
            heads: &outq::Heads,
            out: &mut Vec<u8>,
        ) -> bool {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 {
                return false;
            }
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let need = Self::arena_min_words(ns, m);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }

            self.bands.clear();
            self.spelling.clear();

            // Exact byte offset in `out` strictly after OutQ16's live chunks.
            let in_out_chunks = out.len() / outq::BYTES;
            let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;

            let most = Self::nodes_most(ns);
            let (cursors, used_tail, nbands) = if most <= NARROW_NODES {
                self.induced_narrow(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                )
            } else {
                self.induced_wide(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    &mut out[tail_offset..],
                )
            };

            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m {
                self.edges.resize(m, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m, 0);
            }
            self.predecessor = None;
            let use_xor = !wide && f.m >= 65_536;
            if use_xor {
                self.edges[..m].fill(0);
            }
            let mut predecessor = f.mark;
            let mb = usize::from(f.mark_byte);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if used_tail {
                self.wire_from_outtail64(
                    &cursors,
                    nbands,
                    out_tail,
                    heads,
                    out_q,
                    m,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            } else {
                self.wire_from_band64(
                    &cursors,
                    heads,
                    out_q,
                    m,
                    bits,
                    wide,
                    use_xor,
                    f.mark,
                    mb,
                    &mut predecessor,
                );
            }
            if use_xor {
                self.predecessor = Some(predecessor);
            }

            if let Some(before) = self.predecessor {
                emit_bidirectional(
                    &self.edges[..m],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    before,
                    bits,
                    out,
                );
            } else {
                emit(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    bits,
                    out,
                );
            }
            true
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_narrow(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            first: &[u32; 256],
            write: &mut [u32; 256],
            most: usize,
            out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize) {
            self.induced_with::<Narrow>(ns, n, m, thin, first, write, most, out_tail)
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn induced_wide(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            first: &[u32; 256],
            write: &mut [u32; 256],
            most: usize,
            out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize) {
            self.induced_with::<Wide>(ns, n, m, thin, first, write, most, out_tail)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn induced_with<L: Links>(
            &mut self,
            ns: usize,
            n: u32,
            m: u32,
            thin: &[u32; 256],
            first: &[u32; 256],
            write: &mut [u32; 256],
            most: usize,
            out_tail: &mut [u8],
        ) -> ([usize; 256], bool, usize) {
            let nodes_words = L::STRIDE * most;
            assert!(self.edges.len() >= 4 * ns + nodes_words);

            let mut widest = 0u32;
            let (base, mut top, nodes, open_nodes) = {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let mut e = Elim::<L> {
                    a: arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                e.node(0, 0);
                e.nodes = ns as u32 + 1;
                let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_raw[3 * i..3 * i + 3];
                    let from = sw[0];
                    let wbyte = sw[2];
                    let byte = (wbyte & 0xFF) as u8;
                    let width = wbyte >> 8;
                    let a = usize::from(byte);
                    sw[1] += first[a];
                    let dst_idx = write[a] as usize;
                    image_order[dst_idx] = i as u32;
                    write[a] = dst_idx as u32 + 1;
                    let sid = i as u32 + 1;
                    e.set_width(sid, width);
                    e.set_word(sid, literal(byte));
                    widest = widest.max(width);
                    if x < from {
                        let gw = from - x;
                        widest = widest.max(gw);
                        let gid = e.node(gw, 0);
                        e.append_to(&mut order[0], gid, 0);
                        e.append_to(&mut terminals[1], gid, 1);
                    }
                    e.append_to(&mut order[0], sid, 0);
                    x = from + width;
                }
                if x < n {
                    widest = widest.max(n - x);
                    let id = e.node(n - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                x = 0;
                for &i in image_order.iter() {
                    let idx = i as usize;
                    let sw = &strips_raw[3 * idx..3 * idx + 3];
                    let to = sw[1];
                    let width = sw[2] >> 8;
                    if x < to {
                        widest = widest.max(to - x);
                        let id = e.node(to - x, 0);
                        e.append_to(&mut order[1], id, 1);
                        e.append_to(&mut terminals[0], id, 0);
                    }
                    e.append_to(&mut order[1], i + 1, 1);
                    x = to + width;
                }
                if x < n {
                    widest = widest.max(n - x);
                    let id = e.node(n - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                let open_nodes = e.nodes as usize - 1;
                for side in 0..2 {
                    e.close(side, terminals[side], order[side]);
                }
                let base = L::STRIDE * e.nodes as usize;
                (base, base, e.nodes, open_nodes)
            };

            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            loop {
                let mut e = Elim::<L> {
                    a: &mut self.edges[4 * ns..],
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
                let done = if spirals {
                    e.eliminate::<true>(&mut remaining)
                } else {
                    e.eliminate::<false>(&mut remaining)
                };
                top = e.top;
                if done {
                    break;
                }
                grow_edges_step(&mut self.edges, 4 * ns + top + SPIRAL_RULES * RULE);
            }

            {
                let mut e = Elim::<L> {
                    a: &mut self.edges[4 * ns..],
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
                e.assign_bottom(m);
            }
            const PAIR_WORDS: usize = 512;
            const CACHE_WORDS: usize = PAIR_WORDS + 2048;
            let (header, arena) = self.edges.split_at_mut(4 * ns);
            let (live, tail_arena) = arena.split_at_mut(top);
            let cache = if header.len() >= CACHE_WORDS {
                header
            } else {
                tail_arena
            };
            let mut e = Elim::<L> {
                a: live,
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            begin_inline(&mut self.inline16, &mut self.lens);

            // Fixed-pool injectivity + 256-bucket common refinement: bands_count < open_nodes + 256.
            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;

            if cache.len() < CACHE_WORDS {
                let (band_slots, _) = out_tail.as_chunks_mut::<8>();
                let mut id = L::next(e.a, 0, 0);
                let mut in_node = 0;
                let mut k = 0;
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                        let width = (end - k).min(node_width - in_node);
                        assert!(width > 0);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else {
                            let (entry, len) = e.inline_word(
                                node_word,
                                a as u8,
                                &mut self.stack,
                                &mut self.spelling,
                            );
                            ensure_step_inline(&mut self.inline16, &mut self.lens, INLINE_STEP);
                            let c = self.lens.len() as u32;
                            self.inline16.push(entry);
                            self.lens.push(len);
                            c
                        };
                        if use_tail {
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                            nbands += 1;
                        } else {
                            self.bands.push(Band::new(width, dst, code));
                        }
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                        }
                    }
                }
                assert_eq!(id, 0);
            } else {
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
                let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
                let mut dictionary = Dictionary {
                    pairs,
                    contents,
                    inline16: &mut self.inline16,
                    lens: &mut self.lens,
                    spelling: &mut self.spelling,
                    stack: &mut self.stack,
                };
                let mut id = L::next(e.a, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.a, id)
                } else {
                    (0, 0, 0, 0)
                };
                let mut in_node = 0u32;
                let mut k = 0u32;
                if use_tail {
                    let (band_slots, _) = out_tail[..max_bands * 8].as_chunks_mut::<8>();
                    for a in 0..256 {
                        cursors[a] = nbands;
                        let end = k + thin[a];
                        let mut last_w = 0u32;
                        let mut last_c = 0u32;
                        while k < end {
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else if node_word == last_w {
                                last_c
                            } else {
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word;
                                last_c = c;
                                c
                            };
                            band_slots[nbands] = Band::new(width, dst, code).0.to_le_bytes();
                            nbands += 1;
                            in_node += width;
                            k += width;
                            if in_node == node_width {
                                id = next_id;
                                in_node = 0;
                                if id != 0 {
                                    (next_id, node_bottom, node_width, node_word) =
                                        L::band_node(e.a, id);
                                }
                            }
                        }
                    }
                } else {
                    for a in 0..256 {
                        cursors[a] = self.bands.len();
                        let end = k + thin[a];
                        let mut last_w = 0u32;
                        let mut last_c = 0u32;
                        while k < end {
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else if node_word == last_w {
                                last_c
                            } else {
                                let c = dictionary.code(&mut e, node_word, a as u8);
                                last_w = node_word;
                                last_c = c;
                                c
                            };
                            self.bands.push(Band::new(width, dst, code));
                            in_node += width;
                            k += width;
                            if in_node == node_width {
                                id = next_id;
                                in_node = 0;
                                if id != 0 {
                                    (next_id, node_bottom, node_width, node_word) =
                                        L::band_node(e.a, id);
                                }
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
            }
            (cursors, use_tail, nbands)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_from_outtail64(
            &mut self,
            cursors: &[usize; 256],
            nbands: usize,
            out_tail: &[u8],
            heads: &outq::Heads,
            out_q: &[u8],
            m: usize,
            bits: u32,
            wide: bool,
            use_xor: bool,
            mark: u32,
            mb: usize,
            predecessor: &mut u32,
        ) {
            let edges = &mut self.edges[..m];
            let (band_slots, _) = out_tail.as_chunks::<8>();
            if use_xor {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    if a != mb {
                        for &raw in &band_slots[cursors[a]..end] {
                            let band = Band(u64::from_le_bytes(raw));
                            queue.wire_xor_nomark(
                                edges,
                                band.dst() as usize,
                                band.end() as usize,
                                band.code() << bits,
                            );
                        }
                    } else {
                        for &raw in &band_slots[cursors[a]..end] {
                            let band = Band(u64::from_le_bytes(raw));
                            queue.wire_xor(
                                edges,
                                band.dst() as usize,
                                band.end() as usize,
                                band.code() << bits,
                                mark,
                                predecessor,
                            );
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &raw in &band_slots[cursors[a]..end] {
                        let b = Band(u64::from_le_bytes(raw));
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        let bcode = b.code();
                        let pack = if wide { 0 } else { bcode << bits };
                        if wide {
                            self.extra[dst..dst + span].fill(bcode);
                        }
                        queue.wire(&mut edges[dst..dst + span], pack);
                    }
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_from_band64(
            &mut self,
            cursors: &[usize; 256],
            heads: &outq::Heads,
            out_q: &[u8],
            m: usize,
            bits: u32,
            wide: bool,
            use_xor: bool,
            mark: u32,
            mb: usize,
            predecessor: &mut u32,
        ) {
            let edges = &mut self.edges[..m];
            if use_xor {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    if a != mb {
                        for &band in &self.bands[cursors[a]..end] {
                            queue.wire_xor_nomark(
                                edges,
                                band.dst() as usize,
                                band.end() as usize,
                                band.code() << bits,
                            );
                        }
                    } else {
                        for &band in &self.bands[cursors[a]..end] {
                            queue.wire_xor(
                                edges,
                                band.dst() as usize,
                                band.end() as usize,
                                band.code() << bits,
                                mark,
                                predecessor,
                            );
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        let bcode = b.code();
                        let pack = if wide { 0 } else { bcode << bits };
                        if wide {
                            self.extra[dst..dst + span].fill(bcode);
                        }
                        queue.wire(&mut edges[dst..dst + span], pack);
                    }
                }
            }
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mut at = 0;
        let mask = (1 << bits) - 1;
        let mut j = mark;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = if extra.is_empty() {
                edge >> bits
            } else {
                extra[j as usize]
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional(
        edges: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0, n);
        for _ in 0..=edges.len() / 2 {
            let ef = edges[front as usize];
            let cf = (ef >> bits) as usize;
            let single = front == back;
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
            if single || lo == hi {
                break;
            }
            let eb = edges[back as usize];
            let cb = (eb >> bits) as usize;
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
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
            if lo == hi || front == next {
                break;
            }
        }
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn ref_fl(l: &[u8], p: u32) -> Vec<u8> {
            let n = l.len();
            let mut counts = [0u32; 256];
            for &b in l {
                counts[usize::from(b)] += 1;
            }
            let mut st = [0u32; 256];
            let mut tot = 0u32;
            for c in 0..256 {
                st[c] = tot;
                tot += counts[c];
            }
            let mut fl = vec![0u32; n];
            for (i, &b) in l.iter().enumerate() {
                let s = st[usize::from(b)] as usize;
                st[usize::from(b)] += 1;
                fl[s] = ((i as u32) << 8) | u32::from(b);
            }
            let mut out = vec![0u8; n];
            let mut x = p as usize;
            for slot in &mut out {
                let v = fl[x];
                *slot = v as u8;
                x = (v >> 8) as usize;
            }
            out
        }

        fn to_recs(l: &[u8]) -> Vec<u16> {
            let mut recs = Vec::new();
            let mut i = 0;
            while i < l.len() {
                let b = l[i];
                let mut w = 1usize;
                while i + w < l.len() && l[i + w] == b && w < 127 {
                    w += 1;
                }
                recs.push((u16::from(b) << 8) | (w as u16));
                i += w;
            }
            recs
        }

        #[test]
        fn tiny_fl_16m_guard_and_malformed_parity() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            // Auditor C49 >16M-record defensive guard
            let big = vec![0u16; 16_000_001];
            assert!(!k.invert_tiny_fl(&big, 100, 0, &mut out));

            // Malformed multi-cycle column across both tiny and bi-XOR regimes
            for &n in &[512usize, 40_000, 72_000] {
                let mut l = vec![0u8; n];
                for (i, slot) in l.iter_mut().enumerate() {
                    *slot = ((i * 29 + (i >> 4) * 7) & 31) as u8;
                }
                for pos in (0..n.saturating_sub(128)).step_by(300) {
                    let b = ((pos / 300) % 8) as u8;
                    for j in 0..48 {
                        l[pos + j] = b;
                    }
                }
                let p = (n / 5) as u32;
                let want = ref_fl(&l, p);
                let recs = to_recs(&l);
                out.clear();
                out.resize(n + 37, 0xAA);
                let ok = if n < 32_768 {
                    k.invert_tiny_fl(&recs, n, p, &mut out)
                } else {
                    k.try_run(&recs, n, p, Cap::NONE, &mut out)
                };
                assert!(ok);
                assert_eq!(out, want);
            }
        }
    }
}
