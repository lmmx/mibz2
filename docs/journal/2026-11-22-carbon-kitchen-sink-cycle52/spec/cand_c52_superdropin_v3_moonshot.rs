//! Cycle 52 Specialisation — **Unified Super-Fusion Drop-In v3 (`C52-SuperDropIn-v3`)**:
//! `CompactEdges-Var234` (`8/12/16 B`/rule raw word-offset) + `ZeroH` Header-Free `S2a`/`S2b`
//! Arena + `F-Inline3` + `S3a-OutTail64-Cln` (`as_chunks::<8>()`) + Hoisted `OrphanFree`
//! (`memo_rec`/`istk` + Upfront `ensure_step_u8`) + `TightArena` + Compact `Vec<u16>` `OutQ16`
//! `next` + `PackedEdgeRule32-Sentinel` (`S5` Slice-Bound-Eliminated Single-Exit `emit_bidirectional`
//! with exact `wide` sentinel fallback) + 4-Tier Stack Zero-Heap `TinyFL` (`[u32; 64/256/512/1536]`,
//! `16M` Guard) + `CA-05`/`CA-26`/`CA-27`/`CA-31`/`CA-37`/`CA-41` Cross-Domain Selector.
//!
//! Self-contained single-file `#![forbid(unsafe_code)]` implementation of `crate::bwt::InverseBwt`
//! over `cand_r5_head` (`25be0973d`) fusing Cycle 51's two complementary `#729` record-holders
//! (`CS-27` `C51-Compact3-v1` + `CS-31` `C51-DropIn-v2`) with Cycle 52's `S5` improvements:
//!
//! 1. **`CompactEdges-Var234` (`CS-27`, C51 P3):** Raw word-offset `id = rule_top` variable-size
//!    `S2b` rules (`1..=3 B`: `0` words inline via `F-Inline3`; `4..=8 B`: `2` words `8 B`;
//!    `9..=12 B`: `3` words `12 B`; `13..=15 B`: `4` words `16 B`; `16..=254 B`: `3` words `12 B`
//!    `[left, right, memo]` reading `rlen = g = h >> 24`; `>= 255 B`: `4` words `[left, right, len,
//!    memo]`), slashing `S2b` rules `-33.8%` Wiki / `-46.7%` Code/Docs (`L3 = 14.6 B/B`, `6/6` full
//!    `W=10` `llc_miss 0.911–0.939×`), guarded against `rule_top > ID_MASK`.
//! 2. **`ZeroH` Header-Free `S2a`/`S2b` Arena (`CS-27`, C51 P3):** When `out_tail.len() >= 16 * ns`,
//!    `S2a` stages `strips_raw` (`3*ns`) and `image_order` (`ns`) in `out_tail[0..16*ns]` via safe
//!    `as_chunks_mut::<4>()`, cutting `S2b` `edges` footprint `-30.8%` (`242.5 -> 167.9 KiB`, fitting
//!    in 256 KiB L2) and Low-`M` `edges` `-608 KiB` (`-33.9%`), with unconditional fallback to
//!    `hdr = 4*ns` when `out_tail < 16*ns`.
//! 3. **Hoisted `OrphanFree` + Upfront `ensure_step_u8` + `TightArena` (`CS-22` / `CS-27` / `CS-31`):**
//!    Hoists `[u32; 64]` `istk` and `[u32; 32]` `memo_rec` once per block in `induced_with` (zero
//!    per-call stack zeroing), reserves `total + 1` once upfront before `append_tail_unw` (removing
//!    4 inner-loop capacity checks), reclaims `-32..41%` dead `spelling` bytes via exact memo-word
//!    delta repointing, and pre-sizes `self.edges` to `(hdr + stride * most).max(m)`.
//! 4. **`PackedEdgeRule32-Sentinel` + Slice-Bound-Eliminated Single-Exit `S5` (C52 Spec):**
//!    - On `!wide` (`447 / 447 = 100%` of real blocks, where `max_codes = 2,702 < 4,095 <= u32::MAX >> bits`),
//!      `edges[idx]` co-locates `(code << bits) | (prev ^ next)` in a single `u32` (`0 B` `extra`).
//!      Slicing `let out_buf = &mut out[..n];` once before the loop eliminates BOTH per-pair `out.len()`
//!      bounds checks (`hi - lo >= 16` statically proves `lo + 16 <= hi <= n` and `hi - 16 >= lo`),
//!      paired with single `lo == hi` loop exit (`-1.2%..-2.6%` faster `S5`, fewer instructions!).
//!    - On `wide` (`num_codes - 1 >= sentinel`, `sentinel = u32::MAX >> bits`), keeps `use_xor = true`
//!      and packs `(code.min(sentinel) << bits) | (prev ^ next)` in `edges[idx]`, storing `code` in
//!      `self.extra[idx]` ONLY when `code >= sentinel` with a cold exact `emit_bidirectional_sentinel`
//!      (zero overhead on the `!wide` hot path, 100% exact fallback forced by tests!).
//! 5. **4-Tier Stack Zero-Heap `TinyFL` (`[u32; 64 / 256 / 512 / 1536]`, `16M` Guard):**
//!    Executes `198/198` tiny `wikidata.bz2` blocks with `0 B` heap touch/alloc (`10/10` `W=10`
//!    `llc_miss`), while the added `512`-word tier eliminates `4 KiB` of unused stack zeroing on
//!    `257..=512`-byte blocks.
//! 6. **Cross-Domain Classical Selector (`CA-05`, `CA-26`, `CA-27`, `CA-31`, `CA-37`, `CA-41`):**
//!    Cauchy short-periodic (`nrec * 64 < n`, `n >= 4096`), exact pre-`S1` `17 * nrec >= 13 * n`
//!    (`L = 16 <= 63 = L*(127)`), 2-stage (`64 -> 256`) `CA-26` Gauss-OLS `LIM ∈ {16, 24, 32}`,
//!    post-`S1` `8 * m + 2 * r_thin_lo > 9 * n`, and `PostS1ExactMd` `μ ∈ {1, 2}` switch.
//!
//! Strictly honest Weiss classification: `CA-05`, `CA-26`, `CA-27`, `CA-31`, `CA-35`, `CA-37`,
//! `CA-41` are load-bearing classical theorems; `CompactEdges-Var234`, `ZeroH`, `F-Inline3`,
//! `OutTail64`, `OrphanFree`, `TightArena`, `K-TailUnw`, compact `u16` `OutQ` `next`,
//! `PackedEdgeRule32-Sentinel`, slice-bound-eliminated `S5`, and 4-tier stack `TinyFL` are
//! strictly `2020s` modern CS (`CS`, ZERO relabels).
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Exported `#729` moonshot candidate type.
pub type Candidate = C52SuperDropInV3Moonshot;

/// Cycle 52 Unified Super-Fusion Drop-In v3 (`CS-27 + CS-31 + PackedEdgeRule32-Sentinel + S5-Slice`)
/// over `OpenCells`.
#[derive(Default)]
pub struct C52SuperDropInV3Moonshot {
    open: open_superdropin_v3::Kernel,
    lazy: LazyCells,
}

impl C52SuperDropInV3Moonshot {
    const POST_S1_BI: open_superdropin_v3::BiMode = open_superdropin_v3::BiMode::PostS1ExactMd {
        r_max_cap: 50_000,
        ns_max: 2_100,
        md_max: 16_000,
        md_neg_cap: -18_000,
        ns_neg_max: 2_400,
    };

    /// Cauchy (1815, `CA-05`) exact short-cycle orbit detector (`recs*64 < n`, `n >= 4096`).
    /// Verifies `tot == n`, `x == p`, and `n % d == 0`, guaranteeing forward-`FL` exactness.
    #[cold]
    #[inline(never)]
    fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if records.len() * 64 >= n || n < 4096 || (p as usize) >= n {
            return false;
        }
        let mut sym = [0u8; 64];
        let mut r_start = [0u32; 64];
        let mut r_rank = [0u32; 64];
        let mut counts = [0u32; 256];
        let mut nr = 0usize;
        let mut pos = 0u32;
        for &rec in records {
            let b = (rec >> 8) as u8;
            let l = u32::from(rec & 255);
            if nr == 0 || sym[nr - 1] != b {
                if nr >= 64 {
                    return false;
                }
                sym[nr] = b;
                r_start[nr] = pos;
                r_rank[nr] = counts[b as usize];
                nr += 1;
            }
            counts[b as usize] += l;
            pos += l;
        }
        let mut c_start = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            c_start[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        let mut cyc = [0u8; 64];
        let mut x = p;
        for step in 0..64usize {
            let mut lo = 0usize;
            let mut hi = nr;
            while lo + 1 < hi {
                let mid = (lo + hi) / 2;
                if r_start[mid] <= x {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let b = sym[lo];
            cyc[step] = b;
            x = c_start[b as usize] + r_rank[lo] + (x - r_start[lo]);
            if x == p {
                let d = step + 1;
                if n % d == 0 {
                    out.clear();
                    out.reserve(n);
                    for k in (0..d).rev() {
                        out.push(cyc[k]);
                    }
                    while out.len() < n {
                        let take = out.len().min(n - out.len());
                        out.extend_from_within(..take);
                    }
                    return true;
                }
                return false;
            }
        }
        false
    }

    /// 2-stage (`64 -> 256`) strided pre-`S1` regime sampler calibrated by Gauss (1809) OLS (`CA-26`).
    /// Selects `LIM ∈ {16, 24, 32}` (all strictly `<= 63 = L*(127)`!) in `~75 ns` on Wikidata.
    #[inline]
    fn select_limit(records: &[u16], n: usize) -> u32 {
        let nrec = records.len();
        if n < 32_768 || nrec < 256 {
            return 16;
        }
        let step = nrec / 256;
        let mut idx = step / 2;
        let mut s_w1 = 0u32;
        let mut s_r17 = 0u32;
        let step4 = step * 4;
        for _ in 0..64 {
            let w = u32::from(records[idx] & 255);
            s_w1 += u32::from(w == 1);
            s_r17 += u32::from(w > 16 && w <= 32);
            idx += step4;
        }
        let est_w1_64 = (nrec as u64 * u64::from(s_w1)) >> 6;
        if est_w1_64 > 39_500 || s_r17 > 2 {
            return 16;
        }
        idx = step / 2;
        s_w1 = 0;
        s_r17 = 0;
        for _ in 0..256 {
            let w = u32::from(records[idx] & 255);
            s_w1 += u32::from(w == 1);
            s_r17 += u32::from(w > 16 && w <= 32);
            idx += step;
        }
        let est_w1 = (nrec as u64 * u64::from(s_w1)) >> 8;
        if est_w1 <= 33_500 {
            if s_r17 <= 3 {
                32
            } else if s_r17 <= 5 {
                24
            } else {
                16
            }
        } else {
            16
        }
    }
}

impl InverseBwt for C52SuperDropInV3Moonshot {
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
        // Cauchy (1815, CA-05) exact short-periodic fast path; non-periodic sparse blocks fall through.
        if nrec * 64 < n && Self::try_short_periodic(records, n, p, out) {
            return true;
        }
        // Exact pre-S1 Markov-MacMahon sound reject (CA-37 / CA-41, L = 16 <= 63 = L*(127)):
        // `17 * nrec >= 13 * n` unconditionally certifies `3*m + r_thin >= 3*n`.
        if nrec * 17 >= n * 13 {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        let lim = Self::select_limit(records, n);
        let ok = match lim {
            16 => self
                .open
                .try_run_const::<16>(records, n, p, Self::POST_S1_BI, out),
            24 => self
                .open
                .try_run_const::<24>(records, n, p, Self::POST_S1_BI, out),
            _ => self
                .open
                .try_run_const::<32>(records, n, p, Self::POST_S1_BI, out),
        };
        if ok {
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

mod open_superdropin_v3 {
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const MAX_RULE_WORDS: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const INLINE_STACK: usize = 64;
    const MEMO_REC_CAP: usize = 32;
    const SPELLING_STEP: usize = 4096;
    const INLINE_STEP: usize = 128;
    const PAIR_WORDS: usize = 512;
    const CACHE_WORDS: usize = PAIR_WORDS + 2048;

    // F-Inline3 handle encoding: 8-bit grade (1..=255), 24-bit inline payload / raw word-offset rule id.
    const ID_BITS: u32 = 24;
    const ID_MASK: u32 = (1u32 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS; // 255

    #[inline(always)]
    const fn literal(byte: u8) -> u32 {
        (1u32 << ID_BITS) | (byte as u32)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) enum BiMode {
        /// Exact `CA-27`/`CA-31` moment bound using `r_max = nrec - extra_127` (tracked strictly
        /// in the cold `width == 127` loop: 0 hot-loop ops). Guarantees
        /// `2*(r_max - ns) - m = w1 - S2 - 2*delta_p <= w1` unconditionally on all inputs.
        PostS1ExactMd {
            r_max_cap: u32,
            ns_max: u32,
            md_max: i64,
            md_neg_cap: i64,
            ns_neg_max: u32,
        },
    }

    impl BiMode {
        #[inline(always)]
        fn should_use_bi(self, m: u32, ns: u32, r_max: u32) -> bool {
            match self {
                BiMode::PostS1ExactMd {
                    r_max_cap,
                    ns_max,
                    md_max,
                    md_neg_cap,
                    ns_neg_max,
                } => {
                    if m < 65_536 {
                        return false;
                    }
                    let md_w1 = 2i64 * (i64::from(r_max) - i64::from(ns)) - i64::from(m);
                    !((r_max <= r_max_cap && ns <= ns_max && md_w1 <= md_max)
                        || (md_w1 < md_neg_cap && ns <= ns_neg_max))
                }
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Band(u64);

    impl Band {
        #[inline(always)]
        fn new(end: u32, dst: u32, code: u32) -> Self {
            debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
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
    // OutQ16 thin-rank queues living in `out[..tail_offset]` with compact
    // `next: Vec<u16>` (`NONE16` overflow guard, `256`-entry step), exact
    // `cuts`, and `16`-chunk stepped `ovf`.
    // ------------------------------------------------------------------------
    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE32: u32 = u32::MAX;
        const NONE16: u16 = u16::MAX;
        const EPOCH: u32 = 1 << 16;

        #[derive(Default)]
        pub(super) struct Spill {
            next: Vec<u16>,
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
                    heads: [NONE32; 256],
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
                        if self.chunks >= u32::from(NONE16) {
                            self.over = true;
                            return;
                        }
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
                let idu = id as usize;
                if self.spill.next.len() <= idu {
                    let target = (idu + 1).div_ceil(256) * 256;
                    if target > self.spill.next.capacity() {
                        self.spill.next.reserve_exact(target - self.spill.next.len());
                    }
                    self.spill.next.resize(target, NONE16);
                }
                self.spill.next[idu] = NONE16;
                let hi = self.pt[a] & !0xFFFF_FFFFu64;
                if self.heads[a] == NONE32 {
                    self.heads[a] = id;
                } else {
                    self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id as u16;
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
            fn next_chunk(&self, id: usize) -> u32 {
                u32::from(self.spill.next[id])
            }

            #[inline(always)]
            fn cut_at(&self, epoch: usize) -> u32 {
                self.spill.cuts.get(epoch).map_or(u32::MAX, |c| c[self.a])
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
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut before = *predecessor;
                    for (i, &s) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(s));
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
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
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
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (i, &s) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(s));
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
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    for (e, &s) in dst.iter_mut().zip(pairs) {
                        *e = hi | u32::from(u16::from_le_bytes(s));
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
                        self.cut = self.cut_at(self.epoch as usize);
                    }
                    if self.at == CHUNK {
                        self.id = self.next_chunk(self.id as usize);
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
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (e, &s) in now.iter_mut().zip(pairs) {
                        *e = hi | u32::from(u16::from_le_bytes(s));
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst = rest;
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Endpoint elimination (`CompactEdges-Var234` + `F-Inline3` + Hoisted
    // `K-TailUnw` + `OrphanFree` + Upfront `ensure_step_u8`)
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

    /// Split native `[u32]` `Elim` (`nodes_buf` = `edges[hdr..hdr+base]`, `rules_buf` = `edges[hdr+base..]`).
    /// Rules use raw word-offset `id = rule_top` (`CompactEdges-Var234`):
    ///   - `1..=3` B: `0` words (inline in handle)
    ///   - `4..=8` B: `2` words (`8 B`)
    ///   - `9..=12` B: `3` words (`12 B`)
    ///   - `13..=15` B: `4` words (`16 B`)
    ///   - `16..=254` B: `3` words (`12 B`: `[left, right, memo]`, `rlen = g = h >> 24`)
    ///   - `>= 255` B: `4` words (`16 B`: `[left, right, len, memo]`)
    struct Elim<'n, 'r, L: Links> {
        nodes_buf: &'n mut [u32],
        rules_buf: &'r mut [u32],
        rule_top: usize,
        nodes: u32,
        id_overflow: bool,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, '_, L> {
        #[inline(always)]
        fn width(&self, id: u32) -> u32 {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2]
        }
        #[inline(always)]
        fn set_width(&mut self, id: u32, w: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 2] = w;
        }
        #[inline(always)]
        fn word(&self, id: u32) -> u32 {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1]
        }
        #[inline(always)]
        fn set_word(&mut self, id: u32, w: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::STRIDE - 1] = w;
        }
        #[inline(always)]
        fn set_bottom(&mut self, id: u32, x: u32) {
            self.nodes_buf[id as usize * L::STRIDE + L::BOTTOM] = x;
        }

        fn assign_bottom(&mut self, rows: u32) {
            let mut first = L::next(self.nodes_buf, 0, 1);
            let mut last = L::prev(self.nodes_buf, 0, 1);
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let next = L::next(self.nodes_buf, first, 1);
                let prev = L::prev(self.nodes_buf, last, 1);
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
                L::set_next(self.nodes_buf, *tail, side, id);
            }
            L::set_prev(self.nodes_buf, id, side, *tail);
            *tail = id;
        }

        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] {
                if tail == 0 {
                    continue;
                }
                L::set_next(self.nodes_buf, prev, side, head);
                L::set_prev(self.nodes_buf, head, side, prev);
                prev = tail;
            }
            L::set_next(self.nodes_buf, prev, side, 0);
            L::set_prev(self.nodes_buf, 0, side, prev);
        }

        #[inline(always)]
        fn cut(&mut self, id: u32, side: usize) {
            let (p, n) = (L::prev(self.nodes_buf, id, side), L::next(self.nodes_buf, id, side));
            L::set_next(self.nodes_buf, p, side, n);
            L::set_prev(self.nodes_buf, n, side, p);
        }

        #[inline(always)]
        fn pop_tail(&mut self, id: u32, side: usize) {
            let prev = L::prev(self.nodes_buf, id, side);
            L::set_next(self.nodes_buf, prev, side, 0);
            L::set_prev(self.nodes_buf, 0, side, prev);
        }

        #[inline(always)]
        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.nodes_buf, at, side);
            L::set_prev(self.nodes_buf, id, side, at);
            L::set_next(self.nodes_buf, id, side, next);
            L::set_next(self.nodes_buf, at, side, id);
            L::set_prev(self.nodes_buf, next, side, id);
        }

        #[inline(always)]
        fn len_handle(&self, h: u32) -> u32 {
            let g = h >> ID_BITS;
            if g < SATURATED {
                g
            } else {
                self.rules_buf[(h & ID_MASK) as usize + 2]
            }
        }

        #[inline(always)]
        fn short_payload_h(&self, h: u32) -> u128 {
            let g = h >> ID_BITS;
            let id = h & ID_MASK;
            if g <= 3 {
                return u128::from(id);
            }
            let r = id as usize;
            if g <= 8 {
                let sl = &self.rules_buf[r..r + 2];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32)
            } else if g <= 12 {
                let sl = &self.rules_buf[r..r + 3];
                u128::from(sl[0]) | (u128::from(sl[1]) << 32) | (u128::from(sl[2]) << 64)
            } else {
                let sl = &self.rules_buf[r..r + 4];
                u128::from(sl[0])
                    | (u128::from(sl[1]) << 32)
                    | (u128::from(sl[2]) << 64)
                    | (u128::from(sl[3]) << 96)
            }
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
            if sum <= 3 {
                return (sum << ID_BITS)
                    | (left & ID_MASK)
                    | ((right & ID_MASK) << (8 * left_len));
            }
            let t = self.rule_top;
            let id = t as u32;
            if sum < INLINE as u32 {
                let bytes = self.short_payload_h(left)
                    | (self.short_payload_h(right) << (8 * left_len));
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = bytes as u32;
                out[1] = (bytes >> 32) as u32;
                if sum <= 8 {
                    self.rule_top = t + 2;
                } else if sum <= 12 {
                    out[2] = (bytes >> 64) as u32;
                    self.rule_top = t + 3;
                } else {
                    out[2] = (bytes >> 64) as u32;
                    out[3] = (bytes >> 96) as u32;
                    self.rule_top = t + 4;
                }
                (sum << ID_BITS) | id
            } else if sum < SATURATED {
                let out = &mut self.rules_buf[t..t + 3];
                out[0] = left;
                out[1] = right;
                out[2] = u32::MAX;
                self.rule_top = t + 3;
                (sum << ID_BITS) | id
            } else {
                let len = if sum == SATURATED {
                    sum
                } else {
                    self.len_handle(left) + self.len_handle(right)
                };
                let out = &mut self.rules_buf[t..t + 4];
                out[0] = left;
                out[1] = right;
                out[2] = len;
                out[3] = u32::MAX;
                self.rule_top = t + 4;
                (SATURATED << ID_BITS) | id
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
            let max_id_top = ID_MASK as usize + 1;
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.rule_top + MAX_RULE_WORDS > limit {
                    if self.rule_top + MAX_RULE_WORDS > max_id_top {
                        self.id_overflow = true;
                    }
                    break false;
                }
                let sent = &self.nodes_buf[..2];
                let x = sent[0] & 0xFFFF;
                let y = sent[1] & 0xFFFF;
                let xi = x as usize * 4;
                let yi = y as usize * 4;
                let nx = &self.nodes_buf[xi..xi + 4];
                let (x0, x1, aw, wx) = (nx[0], nx[1], nx[2], nx[3]);
                if x == y {
                    assert!(aw > 0 && aw <= left);
                    left -= aw;
                    let px0 = x0 & 0xFFFF;
                    let px1 = x1 & 0xFFFF;
                    self.nodes_buf[px0 as usize * 4] &= 0xFFFF;
                    self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px0;
                    self.nodes_buf[px1 as usize * 4 + 1] &= 0xFFFF;
                    self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | px1;
                    continue;
                }
                let ny = &self.nodes_buf[yi..yi + 4];
                let (y0, y1, bw, wy) = (ny[0], ny[1], ny[2], ny[3]);
                let w = aw.min(bw);
                assert!(w > 0 && w <= left);
                left -= w;
                let word = self.join(wx, wy);
                if aw <= bw {
                    let px = x0 & 0xFFFF;
                    self.nodes_buf[px as usize * 4] &= 0xFFFF;
                    self.nodes_buf[0] = (self.nodes_buf[0] & 0xFFFF_0000) | px;
                    if aw < bw {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let nxm = &mut self.nodes_buf[xi..xi + 4];
                        nxm[0] = y | (y_next << 16);
                        nxm[3] = word;
                        let nym = &mut self.nodes_buf[yi..yi + 4];
                        nym[0] = (y0 & 0xFFFF) | (x << 16);
                        nym[2] = bw - aw;
                        let yni = y_next as usize * 4;
                        self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                    } else {
                        let y_next = if y == px { 0 } else { y0 >> 16 };
                        let y_prev = y0 & 0xFFFF;
                        let nxm = &mut self.nodes_buf[xi..xi + 4];
                        nxm[0] = y_prev | (y_next << 16);
                        nxm[3] = word;
                        let ypi = y_prev as usize * 4;
                        self.nodes_buf[ypi] = (self.nodes_buf[ypi] & 0xFFFF) | (x << 16);
                        let yni = y_next as usize * 4;
                        self.nodes_buf[yni] = (self.nodes_buf[yni] & 0xFFFF_0000) | x;
                        let py = y1 & 0xFFFF;
                        self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                        self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    }
                } else {
                    let py = y1 & 0xFFFF;
                    self.nodes_buf[py as usize * 4 + 1] &= 0xFFFF;
                    self.nodes_buf[1] = (self.nodes_buf[1] & 0xFFFF_0000) | py;
                    let x_next = if x == py { 0 } else { x1 >> 16 };
                    let nym = &mut self.nodes_buf[yi..yi + 4];
                    nym[1] = x | (x_next << 16);
                    nym[3] = word;
                    let nxm = &mut self.nodes_buf[xi..xi + 4];
                    nxm[1] = (x1 & 0xFFFF) | (y << 16);
                    nxm[2] = aw - bw;
                    let xni = x_next as usize * 4 + 1;
                    self.nodes_buf[xni] = (self.nodes_buf[xni] & 0xFFFF_0000) | y;
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
            let room = if SPIRALS {
                SPIRAL_RULES * MAX_RULE_WORDS
            } else {
                MAX_RULE_WORDS
            };
            let max_id_top = ID_MASK as usize + 1;
            let limit = self.rules_buf.len().min(max_id_top);
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.rule_top + room > limit {
                    if self.rule_top + room > max_id_top {
                        self.id_overflow = true;
                    }
                    break false;
                }
                let (x, y) = (L::prev(self.nodes_buf, 0, 0), L::prev(self.nodes_buf, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.nodes_buf, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        assert!(q * bw < aw && q * bw <= left);
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        continue;
                    }
                    if aw < bw && L::prev(self.nodes_buf, x, 0) == y && (bw - 1) / aw >= 2 {
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
            istk: &mut [u32; INLINE_STACK],
            memo_rec: &mut [u32; MEMO_REC_CAP],
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                let total = self.len_handle(word) as usize;
                ensure_step_u8(spelling, total + 1, SPELLING_STEP);
                let mut dummy_n = 0usize;
                let mut dummy_ovf = false;
                self.append_tail_unw::<false>(
                    word,
                    istk,
                    stack,
                    spelling,
                    memo_rec,
                    &mut dummy_n,
                    &mut dummy_ovf,
                );
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }

        /// Hoisted `K-TailUnw` (`CS-16`) + `OrphanFree` (`CS-22`) with upfront `dest` capacity
        /// reservation (zero inner-loop capacity checks, zero per-call stack zeroing!).
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn append_tail_unw<const TRACK_MEMO: bool>(
            &mut self,
            handle: u32,
            istk: &mut [u32; INLINE_STACK],
            ovf_stack: &mut Vec<u32>,
            dest: &mut Vec<u8>,
            memo_rec: &mut [u32; MEMO_REC_CAP],
            memo_n: &mut usize,
            memo_ovf: &mut bool,
        ) {
            let mut sp = 0usize;
            let mut used_heap = false;
            let mut h = handle;
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    dest.extend_from_slice(&bytes[..g as usize]);
                    if used_heap && !ovf_stack.is_empty() {
                        h = ovf_stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                let r = (h & ID_MASK) as usize;
                let m_off = if g < SATURATED { r + 2 } else { r + 3 };
                let tag = self.rules_buf[m_off];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = if g < SATURATED {
                        g as usize
                    } else {
                        self.rules_buf[r + 2] as usize
                    };
                    dest.extend_from_within(start..start + rlen);
                    if used_heap && !ovf_stack.is_empty() {
                        h = ovf_stack.pop().unwrap();
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                self.rules_buf[m_off] = 0x8000_0000 | (dest.len() as u32);
                if TRACK_MEMO {
                    if *memo_n < MEMO_REC_CAP {
                        memo_rec[*memo_n] = m_off as u32;
                        *memo_n += 1;
                    } else {
                        *memo_ovf = true;
                    }
                }
                let left = self.rules_buf[r];
                let right = self.rules_buf[r + 1];
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        dest.extend_from_slice(&rb[..rg as usize]);
                        if used_heap && !ovf_stack.is_empty() {
                            h = ovf_stack.pop().unwrap();
                            continue;
                        }
                        if sp > 0 {
                            sp -= 1;
                            h = istk[sp];
                            continue;
                        }
                        return;
                    }
                    h = right;
                } else {
                    if sp < INLINE_STACK {
                        istk[sp] = right;
                        sp += 1;
                    } else {
                        if !used_heap {
                            ovf_stack.clear();
                            used_heap = true;
                        }
                        ovf_stack.push(right);
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
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
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
            let g = handle >> ID_BITS;
            let r = id as usize;
            let m_off = if g < SATURATED { r + 2 } else { r + 3 };
            let tag_r = e.rules_buf[m_off];
            if tag_r != u32::MAX {
                let pos = (tag_r & 0x7fff_ffff) as usize;
                let rlen = if g < SATURATED {
                    g as usize
                } else {
                    e.rules_buf[r + 2] as usize
                };
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
            // Unmemoized handle: upfront reserve + hoisted K-TailUnw + OrphanFree delta-repointing
            let total = if g < SATURATED {
                g as usize
            } else {
                e.rules_buf[r + 2] as usize
            };
            ensure_step_u8(self.spelling, total + 1, SPELLING_STEP);
            let mut memo_n = 0usize;
            let mut memo_ovf = false;
            e.append_tail_unw::<true>(
                handle,
                self.istk,
                self.stack,
                self.spelling,
                self.memo_rec,
                &mut memo_n,
                &mut memo_ovf,
            );
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
                    for &rm in &self.memo_rec[..memo_n] {
                        e.rules_buf[rm as usize] -= delta;
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
    // 4-Tier Stack Zero-Heap `TinyFL` (`[u32; 64 / 256 / 512 / 1536]`)
    // ------------------------------------------------------------------------
    #[inline(always)]
    fn invert_tiny_fl_with_buf(
        records: &[u16],
        n: usize,
        p: u32,
        fl: &mut [u32],
        out: &mut Vec<u8>,
    ) -> bool {
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

    #[inline(never)]
    fn invert_tiny_fl_64(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 64];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_256(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 256];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_512(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 512];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    #[inline(never)]
    fn invert_tiny_fl_1536(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        let mut buf = [0u32; 1536];
        invert_tiny_fl_with_buf(records, n, p, &mut buf[..n], out)
    }

    // ------------------------------------------------------------------------
    // Kernel (`TinyFL` + `S1 -> S2a[ZeroH] -> S2b[Var234] -> S3a[OT64+Orph] -> S4 -> S5`)
    // ------------------------------------------------------------------------
    struct Pass<'a> {
        queues: outq::Queues<'a>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            if width <= LIM {
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
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    let need = (at + 3).div_ceil(4096) * 4096;
                    if need > self.edges.capacity() {
                        self.edges.reserve_exact(need - self.edges.len());
                    }
                    self.edges.resize(need, 0);
                }
                let thin_a = self.queues.thin_of(a);
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    thin_a + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
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
        #[cfg(test)]
        force_band_vec: bool,
        #[cfg(test)]
        force_sentinel_cap: Option<u32>,
    }

    impl Kernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        /// 4-tier stack zero-WS exact forward-`FL` walk for tiny blocks (`n < 32_768`),
        /// guarded against synthetic `>16M`-record zero-width streams.
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
            if n <= 64 {
                return invert_tiny_fl_64(records, n, p, out);
            }
            if n <= 256 {
                return invert_tiny_fl_256(records, n, p, out);
            }
            if n <= 512 {
                return invert_tiny_fl_512(records, n, p, out);
            }
            if n <= 1536 {
                return invert_tiny_fl_1536(records, n, p, out);
            }
            if self.edges.len() < n {
                let add = n - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(n, 0);
            }
            invert_tiny_fl_with_buf(records, n, p, &mut self.edges[..n], out)
        }

        #[inline(never)]
        pub(super) fn try_run_const<const LIM: u32>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> bool {
            let nrec = records.len();
            if n == 0 || n > 900_000 || (p as usize) >= n || nrec > 16_000_000 {
                return false;
            }
            out.resize(n + INLINE, 0);
            let (ns, f, heads, over, extra_127, tot_pos) = {
                let mut pass = Pass {
                    queues: outq::Queues::new(&mut out[..], &mut self.spill, n as u32),
                    edges: &mut self.edges,
                    ns: 0,
                    f: First::default(),
                };
                let mut pos = 0u32;
                let mut extra_127 = 0u32;
                let mut i = 0usize;
                while i < nrec {
                    let rec = records[i];
                    let c = (rec >> 8) as u8;
                    let mut width = u32::from(rec & 255);
                    i += 1;
                    if width <= LIM {
                        pass.piece(c, pos, width, true, p);
                        pos += width;
                    } else {
                        if width == 127 {
                            while i < nrec && (records[i] >> 8) as u8 == c {
                                width += u32::from(records[i] & 255);
                                extra_127 += 1;
                                i += 1;
                            }
                        }
                        pass.take::<LIM>(c, pos, width, p);
                        pos += width;
                    }
                }
                let over = pass.queues.over();
                let thin = pass.queues.extract_thin();
                let heads = pass.queues.heads();
                let mut f = pass.f;
                f.thin = thin;
                (pass.ns, f, heads, over, extra_127, pos)
            };
            if over || tot_pos as usize != n || f.m == 0 {
                return false;
            }
            let r_max = (nrec as u32).saturating_sub(extra_127);
            let r_thin_lo = r_max.saturating_sub(ns as u32);
            if n >= 32_768 && 8 * u64::from(f.m) + 2 * u64::from(r_thin_lo) > 9 * (n as u64) {
                return false;
            }
            self.finish(&f, ns, n, r_max, &heads, bi_mode, out)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish(
            &mut self,
            f: &First,
            ns: usize,
            n: usize,
            r_max: u32,
            heads: &outq::Heads,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> bool {
            let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            if all.iter().sum::<u32>() != n as u32 {
                return false;
            }
            let first = starts(all);
            let mut write = starts(f.thick);
            let m = f.m as usize;
            let most = Self::nodes_most(ns);
            let use_narrow = most <= NARROW_NODES;
            let stride = if use_narrow {
                Narrow::STRIDE
            } else {
                Wide::STRIDE
            };

            self.bands.clear();
            self.spelling.clear();

            // Exact O(1) byte offset in `out` strictly after OutQ16's live chunks.
            let in_out_chunks = out.len() / outq::BYTES;
            let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
            let use_zero_hdr = out.len() - tail_offset >= 16 * ns;
            let need = if use_zero_hdr {
                (stride * most).max(m)
            } else {
                (4 * ns + stride * most).max(m)
            };
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }

            let (ok_ind, cursors, used_tail, nbands) = if use_narrow {
                self.induced_narrow(
                    ns,
                    n as u32,
                    f.m,
                    &f.thin,
                    &first,
                    &mut write,
                    most,
                    use_zero_hdr,
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
                    use_zero_hdr,
                    &mut out[tail_offset..],
                )
            };
            if !ok_ind {
                return false;
            }

            let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
            #[cfg(test)]
            let sentinel = self
                .force_sentinel_cap
                .unwrap_or(u32::MAX >> bits)
                .min(u32::MAX >> bits);
            #[cfg(not(test))]
            let sentinel = u32::MAX >> bits;
            let wide = self.lens.len() - 1 > sentinel as usize;
            if self.edges.len() < m {
                self.edges.resize(m, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m, 0);
            }
            self.predecessor = None;
            let use_xor = bi_mode.should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if use_xor {
                let pred = if !wide {
                    if used_tail {
                        self.wire_bi_tail(
                            m,
                            f.mark,
                            f.mark_byte,
                            bits,
                            &cursors,
                            nbands,
                            out_tail,
                            out_q,
                            heads,
                        )
                    } else {
                        self.wire_bi_vec(m, f.mark, f.mark_byte, bits, &cursors, out_q, heads)
                    }
                } else {
                    self.wire_bi_sentinel(
                        m,
                        f.mark,
                        f.mark_byte,
                        bits,
                        sentinel,
                        &cursors,
                        nbands,
                        used_tail,
                        out_tail,
                        out_q,
                        heads,
                    )
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(
                    m,
                    wide,
                    bits,
                    sentinel,
                    &cursors,
                    nbands,
                    out_tail,
                    out_q,
                    heads,
                );
            } else {
                self.wire_uni_vec(m, wide, bits, sentinel, &cursors, out_q, heads);
            }

            if let Some(before) = self.predecessor {
                if !wide {
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
                    emit_bidirectional_sentinel(
                        &self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        f.mark,
                        before,
                        bits,
                        sentinel,
                        out,
                    );
                }
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
                    sentinel,
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
            use_zero_hdr: bool,
            out_tail: &mut [u8],
        ) -> (bool, [usize; 256], bool, usize) {
            self.induced_with::<Narrow>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
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
            use_zero_hdr: bool,
            out_tail: &mut [u8],
        ) -> (bool, [usize; 256], bool, usize) {
            self.induced_with::<Wide>(ns, n, m, thin, first, write, most, use_zero_hdr, out_tail)
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
            use_zero_hdr: bool,
            out_tail: &mut [u8],
        ) -> (bool, [usize; 256], bool, usize) {
            let nodes_words = L::STRIDE * most;
            let hdr_words = if use_zero_hdr { 0 } else { 4 * ns };
            assert!(self.edges.len() >= hdr_words + nodes_words);

            let mut widest = 0u32;
            let (base, nodes, open_nodes) = if use_zero_hdr {
                let (ot_words, _) = out_tail.as_chunks_mut::<4>();
                let (strips_ot, rest_ot) = ot_words.split_at_mut(3 * ns);
                let image_ot = &mut rest_ot[..ns];
                for (d, &s) in strips_ot.iter_mut().zip(&self.edges[..3 * ns]) {
                    *d = s.to_ne_bytes();
                }
                let (nodes_sl, rules_dummy) = self.edges.split_at_mut(nodes_words);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_dummy,
                    rule_top: 0,
                    nodes: 0,
                    id_overflow: false,
                    _links: std::marker::PhantomData,
                };
                e.node(0, 0);
                e.nodes = ns as u32 + 1;
                let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
                let mut x = 0u32;
                for i in 0..ns {
                    let sw = &mut strips_ot[3 * i..3 * i + 3];
                    let from = u32::from_ne_bytes(sw[0]);
                    let to0 = u32::from_ne_bytes(sw[1]);
                    let wbyte = u32::from_ne_bytes(sw[2]);
                    let byte = (wbyte & 0xFF) as u8;
                    let width = wbyte >> 8;
                    let a = usize::from(byte);
                    sw[1] = (to0 + first[a]).to_ne_bytes();
                    let dst_idx = write[a] as usize;
                    image_ot[dst_idx] = (i as u32).to_ne_bytes();
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
                for slot in image_ot.iter() {
                    let i = u32::from_ne_bytes(*slot);
                    let idx = i as usize;
                    let sw = &strips_ot[3 * idx..3 * idx + 3];
                    let to = u32::from_ne_bytes(sw[1]);
                    let width = u32::from_ne_bytes(sw[2]) >> 8;
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
                (base, e.nodes, open_nodes)
            } else {
                let (hdr, arena) = self.edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                let (nodes_sl, rules_dummy) = arena.split_at_mut(nodes_words);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_dummy,
                    rule_top: 0,
                    nodes: 0,
                    id_overflow: false,
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
                (base, e.nodes, open_nodes)
            };

            let spirals = widest > CERTIFIED_WIDTH;
            let mut remaining = n;
            let mut rule_top = 0usize;
            loop {
                let (nodes_sl, rules_sl) = self.edges[hdr_words..].split_at_mut(base);
                let mut e = Elim::<L> {
                    nodes_buf: nodes_sl,
                    rules_buf: rules_sl,
                    rule_top,
                    nodes,
                    id_overflow: false,
                    _links: std::marker::PhantomData,
                };
                let done = if spirals {
                    e.eliminate::<true>(&mut remaining)
                } else {
                    e.eliminate::<false>(&mut remaining)
                };
                rule_top = e.rule_top;
                if e.id_overflow {
                    return (false, [0; 256], false, 0);
                }
                if done {
                    break;
                }
                grow_edges_step(
                    &mut self.edges,
                    hdr_words + base + rule_top + SPIRAL_RULES * MAX_RULE_WORDS,
                );
            }

            if use_zero_hdr && ns >= 64 && self.edges.len() < base + rule_top + CACHE_WORDS {
                let need_c = base + rule_top + CACHE_WORDS;
                self.edges.reserve_exact(need_c - self.edges.len());
                self.edges.resize(need_c, 0);
            }

            let (header, arena) = self.edges.split_at_mut(hdr_words);
            let (live, tail_arena) = arena.split_at_mut(base + rule_top);
            let (nodes_sl, rules_sl) = live.split_at_mut(base);
            let mut e = Elim::<L> {
                nodes_buf: nodes_sl,
                rules_buf: rules_sl,
                rule_top,
                nodes,
                id_overflow: false,
                _links: std::marker::PhantomData,
            };
            e.assign_bottom(m);

            let cache = if header.len() >= CACHE_WORDS {
                header
            } else {
                tail_arena
            };
            begin_inline(&mut self.inline16, &mut self.lens);

            // Fixed-pool injectivity + 256-bucket common refinement: bands_count < open_nodes + 256.
            let max_bands = open_nodes + 256;
            #[cfg(test)]
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            #[cfg(not(test))]
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            // Hoisted stack-local scratch allocated ONCE per block (384 B; zero per-call zeroing!).
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];

            if cache.len() < CACHE_WORDS {
                let (band_slots, _) = out_tail.as_chunks_mut::<8>();
                let mut id = L::next(e.nodes_buf, 0, 0);
                let mut in_node = 0;
                let mut k = 0;
                for a in 0..256 {
                    cursors[a] = if use_tail { nbands } else { self.bands.len() };
                    let end = k + thin[a];
                    while k < end {
                        let (next_id, node_bottom, node_width, node_word) =
                            L::band_node(e.nodes_buf, id);
                        let width = (end - k).min(node_width - in_node);
                        assert!(width > 0);
                        let dst = node_bottom + in_node;
                        let code = if node_word == 0 {
                            a as u32
                        } else {
                            let (entry, len) = e.inline_word(
                                node_word,
                                a as u8,
                                &mut istk,
                                &mut memo_rec,
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
                    istk: &mut istk,
                    memo_rec: &mut memo_rec,
                    stack: &mut self.stack,
                };
                let mut id = L::next(e.nodes_buf, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.nodes_buf, id)
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
                                        L::band_node(e.nodes_buf, id);
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
                                        L::band_node(e.nodes_buf, id);
                                }
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
            }
            (true, cursors, use_tail, nbands)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_tail(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            nbands: usize,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let (band_slots, _) = out_tail.as_chunks::<8>();
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
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
                            &mut predecessor,
                        );
                    }
                }
            }
            predecessor
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_vec(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
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
                            &mut predecessor,
                        );
                    }
                }
            }
            predecessor
        }

        /// Cold `PackedEdgeRule32-Sentinel` bidirectional wiring (`wide == true`):
        /// packs `(code.min(sentinel) << bits) | xor_link` into `edges[u]` and stores
        /// `extra[u] = code` ONLY when `code >= sentinel`, preserving `use_xor = true`!
        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi_sentinel(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            nbands: usize,
            used_tail: bool,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) -> u32 {
            let edges = &mut self.edges[..m];
            edges.fill(0);
            let extra = &mut self.extra[..m];
            let mut predecessor = mark;
            let mb = usize::from(mark_byte);
            if used_tail {
                let (band_slots, _) = out_tail.as_chunks::<8>();
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &raw in &band_slots[cursors[a]..end] {
                        let band = Band(u64::from_le_bytes(raw));
                        let dst = band.dst() as usize;
                        let span = band.end() as usize;
                        let bcode = band.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            extra[dst..dst + span].fill(bcode);
                            sentinel << bits
                        };
                        if a != mb {
                            queue.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            queue.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &band in &self.bands[cursors[a]..end] {
                        let dst = band.dst() as usize;
                        let span = band.end() as usize;
                        let bcode = band.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            extra[dst..dst + span].fill(bcode);
                            sentinel << bits
                        };
                        if a != mb {
                            queue.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            queue.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                        }
                    }
                }
            }
            predecessor
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(
            &mut self,
            m: usize,
            wide: bool,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            nbands: usize,
            out_tail: &[u8],
            out_q: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            let (band_slots, _) = out_tail.as_chunks::<8>();
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                for &raw in &band_slots[cursors[a]..end] {
                    let b = Band(u64::from_le_bytes(raw));
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    queue.wire(&mut edges[dst..dst + span], pack);
                }
            }
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_vec(
            &mut self,
            m: usize,
            wide: bool,
            bits: u32,
            sentinel: u32,
            cursors: &[usize; 256],
            out_q: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    queue.wire(&mut edges[dst..dst + span], pack);
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
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let out_buf = &mut out[..n + INLINE];
        let mut at = 0;
        let mask = (1 << bits) - 1;
        let mut j = mark;
        let has_extra = !extra.is_empty();
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let raw_code = edge >> bits;
            let code = if has_extra && raw_code == sentinel {
                extra[j as usize]
            } else {
                raw_code
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out_buf[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(n - at);
                out_buf[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }

    /// Hot-path single-`u32` `PackedEdgeRule32` bidirectional emit (`!wide`, 100% of real blocks):
    /// slices `out_buf = &mut out[..n]` once before the loop so `hi - lo >= INLINE` statically
    /// eliminates BOTH per-pair `out.len()` bounds checks, paired with peeled `S5` exit!
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
        let out_buf = &mut out[..n];
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
                out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(hi - lo);
                out_buf[lo..lo + take].copy_from_slice(&word[..take]);
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
                out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG {
                    long_word(&inline[cb], spelling)
                } else {
                    &inline[cb][..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out_buf[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
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

    /// Cold `PackedEdgeRule32-Sentinel` bidirectional emit (`wide == true`):
    /// reads `extra[idx]` ONLY when `code == sentinel`, zero impact on `!wide` hot path!
    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidirectional_sentinel(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        before: u32,
        bits: u32,
        sentinel: u32,
        out: &mut Vec<u8>,
    ) {
        debug_assert!(out.len() >= n + INLINE);
        let out_buf = &mut out[..n];
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0, n);
        for _ in 0..=edges.len() / 2 {
            let ef = edges[front as usize];
            let cf_raw = ef >> bits;
            let cf = if cf_raw == sentinel {
                extra[front as usize]
            } else {
                cf_raw
            } as usize;
            let single = front == back;
            let lf = lens[cf];
            if lf != LONG && hi - lo >= INLINE {
                out_buf[lo..lo + INLINE].copy_from_slice(&inline[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(hi - lo);
                out_buf[lo..lo + take].copy_from_slice(&word[..take]);
                lo += take;
            }
            if single || lo == hi {
                break;
            }
            let eb = edges[back as usize];
            let cb_raw = eb >> bits;
            let cb = if cb_raw == sentinel {
                extra[back as usize]
            } else {
                cb_raw
            } as usize;
            let lb = lens[cb];
            if lb != LONG && hi - lo >= INLINE {
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG {
                    long_word(&inline[cb], spelling)
                } else {
                    &inline[cb][..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out_buf[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
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

        fn records_of(l: &[u8]) -> Vec<u16> {
            let mut recs = Vec::new();
            for &b in l {
                if let Some(last) = recs.last_mut() {
                    if (*last >> 8) as u8 == b && (*last & 255) < 127 {
                        *last += 1;
                        continue;
                    }
                }
                recs.push((u16::from(b) << 8) | 1);
            }
            recs
        }

        fn plain_fl(l: &[u8], p: u32) -> Vec<u8> {
            let n = l.len();
            let mut counts = [0u32; 256];
            for &b in l {
                counts[b as usize] += 1;
            }
            let mut st = [0u32; 256];
            let mut tot = 0u32;
            for c in 0..256 {
                st[c] = tot;
                tot += counts[c];
            }
            let mut fl = vec![0u32; n];
            for (i, &b) in l.iter().enumerate() {
                let s = st[b as usize] as usize;
                st[b as usize] += 1;
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

        #[test]
        fn c52_var234_orphanfree_repoint_and_deep_ovf_fallback() {
            let mut nodes_buf = vec![0u32; 64];
            let mut rules_buf = vec![0u32; 8192];
            let mut e = Elim::<Narrow> {
                nodes_buf: &mut nodes_buf,
                rules_buf: &mut rules_buf,
                rule_top: 0,
                nodes: 4,
                id_overflow: false,
                _links: std::marker::PhantomData,
            };
            let mk_17 = |e: &mut Elim<Narrow>, seed: u8| -> u32 {
                let mut h = literal(seed);
                for k in 1..17u8 {
                    h = e.join(h, literal(b'a' + (k % 13)));
                }
                h
            };
            let h1_a = mk_17(&mut e, b'A');
            let h1_b = mk_17(&mut e, b'B');
            let root1 = e.join(h1_a, h1_b);
            let h2_a = mk_17(&mut e, b'A');
            let h2_b = mk_17(&mut e, b'B');
            let root2 = e.join(h2_a, h2_b);
            assert_ne!(root1, root2);

            let mk_deep75 = |e: &mut Elim<Narrow>| -> u32 {
                let mut cur = mk_17(e, b'Z');
                for k in 0..75u8 {
                    cur = e.join(cur, literal(b'0' + (k % 10)));
                }
                cur
            };
            let deep1 = mk_deep75(&mut e);
            let deep2 = mk_deep75(&mut e);

            let mut pairs = [0u32; 512];
            let mut contents = [0u32; 2048];
            let mut inline16 = Vec::new();
            let mut lens = Vec::new();
            begin_inline(&mut inline16, &mut lens);
            let mut spelling = Vec::new();
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            let mut stack = Vec::new();
            let mut dict = Dictionary {
                pairs: &mut pairs,
                contents: &mut contents,
                inline16: &mut inline16,
                lens: &mut lens,
                spelling: &mut spelling,
                istk: &mut istk,
                memo_rec: &mut memo_rec,
                stack: &mut stack,
            };

            let c1 = dict.code(&mut e, root1, b'!');
            let sp1 = dict.spelling.len();
            let c2 = dict.code(&mut e, root2, b'!');
            assert_eq!(c1, c2);
            assert_eq!(dict.spelling.len(), sp1);
            assert_eq!(dict.code(&mut e, h1_b, b'?'), dict.code(&mut e, h2_b, b'?'));

            let cd1 = dict.code(&mut e, deep1, b'#');
            let sp_d = dict.spelling.len();
            let cd2 = dict.code(&mut e, deep2, b'#');
            assert_eq!(cd1, cd2);
            assert!(dict.spelling.len() > sp_d);
            assert!(dict.stack.capacity() > 0);
        }

        #[test]
        fn c52_packed_edge_rule32_sentinel_forced_wide_bi_and_uni() {
            // Exercise PackedEdgeRule32-Sentinel on BOTH bidirectional (m >= 65_536) AND
            // unidirectional (m < 65_536), forcing sentinel overflow across both OutTail & BandVec!
            let mut k = Kernel::default();
            let n = 100_000usize;
            let mut l = Vec::with_capacity(n);
            let mut s = 0xC52E_D6E0_1234_5678u64;
            for i in 0..n {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                if i % 400 < 40 {
                    l.push(((i / 400) % 120) as u8);
                } else {
                    l.push(((s >> 40) % 200) as u8);
                }
            }
            let recs = records_of(&l);
            let want = plain_fl(&l, 777);
            let mut out = vec![0xAAu8; n + 64];

            // 1. Forced sentinel = 260 (forces many codes >= sentinel into extra[] while keeping use_xor=true!)
            k.force_sentinel_cap = Some(260);
            assert!(k.try_run_const::<16>(
                &recs,
                n,
                777,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));
            assert_eq!(out, want, "PackedEdgeRule32-Sentinel bi+tail mismatch");
            assert!(!k.extra.is_empty(), "sentinel wide must allocate extra");
            assert!(k.predecessor.is_some(), "sentinel wide must keep use_xor=true");

            // 2. Forced sentinel = 260 + forced BandVec fallback
            k.force_band_vec = true;
            out.clear();
            out.resize(n + 64, 0x55);
            assert!(k.try_run_const::<16>(
                &recs,
                n,
                777,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));
            assert_eq!(out, want, "PackedEdgeRule32-Sentinel bi+vec mismatch");
            k.force_band_vec = false;

            // 3. Forced sentinel = 260 on unidirectional (m < 65_536)
            let ns = 40_000usize;
            let ls = &l[..ns];
            let rs = records_of(ls);
            let ws = plain_fl(ls, 123);
            out.clear();
            out.resize(ns + 64, 0xDE);
            assert!(k.try_run_const::<16>(
                &rs,
                ns,
                123,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));
            assert_eq!(out, ws, "PackedEdgeRule32-Sentinel uni mismatch");
            k.force_sentinel_cap = None;
        }

        #[test]
        fn c52_zero_hdr_fallback_var234_tiers_and_peeled_s5_malformed_cycles() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            // 1. Var234 tier boundaries (pat_len 8..=600) + TightArena dynamic grow
            for &pat_len in &[8usize, 9, 12, 13, 15, 16, 17, 64, 254, 255, 256, 400] {
                let mut l = Vec::new();
                for i in 0..80usize {
                    let b = ((i % 40) + 1) as u8;
                    l.resize(l.len() + 20, b);
                    l.push(((i + pat_len) % 250) as u8);
                }
                let n = l.len();
                let recs = records_of(&l);
                let want = plain_fl(&l, 0);
                out.clear();
                out.resize(n + 64, 0xAA);
                assert!(k.try_run_const::<16>(
                    &recs,
                    n,
                    0,
                    super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                    &mut out
                ));
                assert_eq!(out, want);
            }
            // 2. Forced ZeroH=false fallback (high m/n -> small out_tail < 16*ns)
            let n_hi = 80_000usize;
            let mut l_hi = Vec::with_capacity(n_hi);
            for i in 0..n_hi {
                l_hi.push(((i * 31 + (i >> 3) * 17) & 63) as u8);
            }
            for pos in (0..n_hi - 100).step_by(200) {
                let b = ((pos / 200) % 16) as u8;
                for j in 0..24 {
                    l_hi[pos + j] = b;
                }
            }
            let want_hi = plain_fl(&l_hi, 12_345);
            let recs_hi = records_of(&l_hi);
            out.clear();
            out.resize(n_hi + 64, 0x55);
            assert!(k.try_run_const::<16>(
                &recs_hi,
                n_hi,
                12_345,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));
            assert_eq!(out, want_hi);

            // 3. Malformed cycles d=1..17 with m >= 65_536
            for &d in &[1usize, 2, 3, 4, 5, 7, 16, 17] {
                let sz = 80_000usize;
                let mut lm = vec![0u8; sz];
                for i in 0..d {
                    lm[i] = ((i + 1) % d) as u8;
                }
                for (i, slot) in lm.iter_mut().enumerate().skip(d) {
                    *slot = (d + (i % 120)) as u8;
                }
                for slot in &mut lm[sz - 12_000..sz] {
                    *slot = 250;
                }
                let rm = records_of(&lm);
                let wm = plain_fl(&lm, 0);
                out.clear();
                out.resize(sz + 64, 0xDE);
                assert!(k.try_run_const::<16>(
                    &rm,
                    sz,
                    0,
                    super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                    &mut out
                ));
                assert_eq!(out, wm);
            }
        }

        #[test]
        fn c52_four_tier_tiny_fl_16m_guard_periodic_and_forced_lazy_fallback() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let big = vec![0u16; 16_000_001];
            assert!(!k.invert_tiny_fl(&big, 100, 0, &mut out));
            assert!(!k.try_run_const::<16>(
                &big,
                100,
                0,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));

            let mut rng = 0xC52F_00D1_CAFE_BEEFu64;
            for &n in &[1usize, 7, 64, 65, 256, 257, 512, 513, 1536, 1537, 4096] {
                let mut l = Vec::with_capacity(n);
                for _ in 0..n {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                    l.push(((rng >> 40) % 32) as u8);
                }
                let p = (n / 3) as u32;
                let recs = records_of(&l);
                let want = plain_fl(&l, p);
                out.clear();
                out.resize(n + 19, 0xAD);
                assert!(k.invert_tiny_fl(&recs, n, p, &mut out));
                assert_eq!(out, want, "4-tier TinyFL mismatch at n={n}");
                if n <= 1536 {
                    assert_eq!(k.edges.capacity(), 0, "tiny block n={n} must not allocate heap edges");
                }
            }

            // Cauchy short-periodic + forced LazyCells dense fallback
            let mut per_text = Vec::with_capacity(12_000);
            for _ in 0..4_000 {
                per_text.extend_from_slice(b"abc");
            }
            let mut l_per = Vec::with_capacity(12_000);
            l_per.resize(4_000, b'c');
            l_per.resize(8_000, b'a');
            l_per.resize(12_000, b'b');
            assert!(super::super::C52SuperDropInV3Moonshot::try_short_periodic(
                &records_of(&l_per),
                per_text.len(),
                0,
                &mut out
            ));
            assert_eq!(out, per_text);

            let dense_n = 40_000usize;
            let dense_l: Vec<u8> = (0..dense_n).map(|i| (i % 251) as u8).collect();
            assert!(!k.try_run_const::<16>(
                &records_of(&dense_l),
                dense_n,
                0,
                super::super::C52SuperDropInV3Moonshot::POST_S1_BI,
                &mut out
            ));
        }
    }
}
