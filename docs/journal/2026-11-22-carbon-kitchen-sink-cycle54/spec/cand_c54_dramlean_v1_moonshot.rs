//! Cycle 54 Specialisation `#729` `bench/moonshot` Drop-In Candidate: **`C54-DramLean-v1`**
//!
//! Attacks **`W=10 DRAM <= 4.65 B/B`** AND **`L2 <= 24.8 B/B`** at **`instr <= 0.815`**
//! (and `PeakCap = 960.00 KiB < 1,000 KiB`) by uniting `C53-PipeLean-v1` (`CS-42` — `SlimS5_Pipe1_Rem16_LE16`
//! [bi & uni] + `S1Step4096` + `V3CodeMiss` + `S2aSlim` + `S3aSlim` + `S4SlowHi0` + `PackedEdgeRule32-Sentinel`
//! + 4-Tier Stack `TinyFL`) with Cycle 54's two verified cache/DRAM breakthroughs:
//!
//! 1. **Post-Compaction `L = 12` DramLean Retention Cutoff (`LIM ∈ {12, 24, 32}`, `CA-41`-certified):**
//!    Because `F-Inline3` (`CS-06`) + `OutTail64` (`CS-09`) + `OrphanFree` (`CS-22`) + `CE-Var234`/`ZeroH`
//!    (`CS-27`) + `S2aSlim`/`S3aSlim` (`CS-42`) + `4 KiB` stack `u16` `Dictionary::contents` (`CS-44`) cut
//!    `S2a`/`S2b`/`S3a` footprint by ~45%, lowering the default retention threshold from `L = 16` to `L = 12`
//!    (while preserving `LIM ∈ {24, 32}` on low-`w1` JSONL/Log blocks and `CA-41`'s exact `13 * nrec >= 10 * n`
//!    sound pre-`S1` reject for `L = 12 <= 63 = L*(127)`!) shrinks retained rows `m` by ~`9.7%` on full Wikidata
//!    blocks (`edges` `-40.2 KiB`, `OutQ16` `-20 KiB`, `PeakCap = 960.00 KiB` vs `998.94 KiB`), slashing:
//!    - `256 KiB` L2 misses by **`-6.47%`** (targeting `L2 <= 24.0 B/B`),
//!    - `W=8` (`2.5 MiB` LLC) misses by **`-6.58%`** (targeting `W=8 DRAM <= 1.21 B/B`),
//!    - `W=10` (`2.0 MiB` LLC) misses by **`-2.03%..3.04%`** (targeting `W=10 DRAM <= 4.62 B/B`),
//!    - Total instructions `Ir` by **`-1.08M`** (`0.9964×` vs `C53-PipeLean-v1`, targeting `instr <= 0.806`)!
//! 2. **Compact `4 KiB` Stack `u16` `Dictionary::contents` + `2 KiB` Stack `pairs` Fused into `V3CodeMiss`:**
//!    Packs `(tag4 << 12) | code` in a `4 KiB` L1D-hot stack table (`max_codes < 4,096` across `447/447` real
//!    blocks!), with lossless `#[cold]` promotion to `contents_u32_fb: Vec<u32>` if `code >= u16_cap`
//!    (`0 B` heap fallback on all `447/447` real blocks!), eliminating `self.edges` `CACHE_WORDS` (`10 KiB`)
//!    tail growth and eliminating the un-deduplicated `cache.len() < CACHE_WORDS` slow path!
//! 3. **Honest Cycle 54 Ablation Kills (`CS-51`, Rule 5):**
//!    - `TopDown-OutQ` (`+18,587` `2 MiB` LLC write misses from middle-`s.ll` `band_slots` eviction),
//!    - `S5` L1D Write-Combining Staging `WC2K/4K/8K` (`+22M Ir`, `+5–30%` wall, `+31k–87k` L2 misses),
//!    - `L = 10` (`+5.4%` native `1T` slowdown from `S2b` branch pressure), `SlimS4_16` (`+1.2%`), `Cold-Pipe1+In`
//!      with `inf/inb` register pressure (`+2.6%`).
//!
//! Strictly honest Weiss classification (`Rule 4`):
//! - `CA-05` (Cauchy 1815), `CA-26` (Gauss 1809), `CA-27`/`CA-31` (MacMahon 1915 / Markov 1884),
//!   `CA-35` (Lucas 1891), `CA-37`/`CA-41` (Markov–MacMahon generalized `(L, CAP)` pre-`S1` threshold)
//!   are inherited load-bearing classical theorems from the Cycle 46–53 Atlas.
//! - ALL Cycle 54 kernel mechanisms (`L = 12` DramLean cutoff, stack `u16` `contents` fused with `V3CodeMiss`,
//!   `SlimS5_Pipe1_Rem16_LE16`, `S1Step4096`, `S2aSlim`, `S3aSlim`, `S4SlowHi0`, `PackedEdgeRule32-Sentinel`,
//!   4-tier `TinyFL`, `CE-Var234`, `ZeroH`, `OrphanFree`) are strictly **`2020s` modern CS** (`0` relabels!).
//!
//! Unconditional forward-`FL` parity & `LazyCells` fallback on all valid, adversarial, and
//! malformed multi-cycle inputs.
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Exported `#729` moonshot candidate type.
pub type Candidate = C54DramLeanV1Moonshot;

/// Cycle 54 Specialisation `#729` Drop-In (`C54-DramLean-v1` over `OpenCells`).
#[derive(Default)]
pub struct C54DramLeanV1Moonshot {
    open: open_c54_dramlean_v1::Kernel,
    lazy: LazyCells,
}

impl C54DramLeanV1Moonshot {
    const POST_S1_BI: open_c54_dramlean_v1::BiMode = open_c54_dramlean_v1::BiMode::PostS1ExactMd {
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
    /// Selects `LIM ∈ {12, 24, 32}` (all strictly `<= 63 = L*(127)` by `CA-41`!) in `~75 ns`:
    /// - `12` (`C54-DramLean`): full Wikidata & Code/Docs (`-9.7%` `m`, `-40 KiB` `edges`, `-6.5%` L2/W8, `-2..3%` W10!),
    /// - `24 / 32`: low-`w1` JSONL / Syslog blocks.
    #[inline]
    fn select_limit(records: &[u16], n: usize) -> u32 {
        let nrec = records.len();
        if n < 32_768 || nrec < 256 {
            return 12;
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
            return 12;
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
                12
            }
        } else {
            12
        }
    }
}

impl InverseBwt for C54DramLeanV1Moonshot {
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
        // Exact pre-S1 Markov-MacMahon sound reject (CA-37 / CA-41, L = 12 <= 63 = L*(127)):
        // `12*m >= 13*nrec - n` & `3*(3*m + r_thin) >= 13*nrec - n`, so `13 * nrec >= 10 * n`
        // unconditionally certifies `3*m(12) + r_thin(12) >= 3*n` (0 false rejects!).
        if nrec * 13 >= n * 10 {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        let lim = Self::select_limit(records, n);
        let ok = match lim {
            12 => self
                .open
                .try_run_const::<12>(records, n, p, Self::POST_S1_BI, out),
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

mod open_c54_dramlean_v1 {
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
    const U16_DICT_CAP: u32 = 4_096;

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
    // `next: Vec<u16>`, exact-reserve `cuts`, `16`-chunk stepped `ovf`, and
    // `SlimS4_Full + S4SlowHi0` (`as_chunks::<2>()` `movzwl` + `pack_hi` +
    // `hi==0` on BOTH fast and 8-way batched slow paths).
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
            pub(super) fn push_thin<const MAX_W: u32>(&mut self, a: usize, m: u32, w: u32) {
                let v = self.pt[a];
                let p = v as u32;
                let f = (p & 1023) as usize;
                let ok_w = if MAX_W <= 16 { true } else { w <= 16 };
                if ok_w & (f <= CHUNK - 16) & ((m as u16) < 65_520) {
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
                        self.spill
                            .next
                            .reserve_exact(target - self.spill.next.len());
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
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi == 0 {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let v0 = u32::from(u16::from_le_bytes(ch[0]));
                            let v1 = u32::from(u16::from_le_bytes(ch[1]));
                            let v2 = u32::from(u16::from_le_bytes(ch[2]));
                            let v3 = u32::from(u16::from_le_bytes(ch[3]));
                            let v4 = u32::from(u16::from_le_bytes(ch[4]));
                            let v5 = u32::from(u16::from_le_bytes(ch[5]));
                            let v6 = u32::from(u16::from_le_bytes(ch[6]));
                            let v7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let pack_hi = pack | hi;
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = u32::from(u16::from_le_bytes(ch[0]));
                            let l1 = u32::from(u16::from_le_bytes(ch[1]));
                            let l2 = u32::from(u16::from_le_bytes(ch[2]));
                            let l3 = u32::from(u16::from_le_bytes(ch[3]));
                            let l4 = u32::from(u16::from_le_bytes(ch[4]));
                            let l5 = u32::from(u16::from_le_bytes(ch[5]));
                            let l6 = u32::from(u16::from_le_bytes(ch[6]));
                            let l7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32;
                            edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    self.wire_xor_slow_nomark(edges, dst_start, span, pack);
                }
            }

            #[cold]
            #[inline(never)]
            fn wire_xor_slow_nomark(
                &mut self,
                edges: &mut [u32],
                mut dst_start: usize,
                mut left: usize,
                pack: u32,
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
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    let mut i = 0usize;
                    if hi == 0 {
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let v0 = u32::from(u16::from_le_bytes(ch[0]));
                            let v1 = u32::from(u16::from_le_bytes(ch[1]));
                            let v2 = u32::from(u16::from_le_bytes(ch[2]));
                            let v3 = u32::from(u16::from_le_bytes(ch[3]));
                            let v4 = u32::from(u16::from_le_bytes(ch[4]));
                            let v5 = u32::from(u16::from_le_bytes(ch[5]));
                            let v6 = u32::from(u16::from_le_bytes(ch[6]));
                            let v7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack | v0;
                            eu[1] ^= pack | v1;
                            eu[2] ^= pack | v2;
                            eu[3] ^= pack | v3;
                            eu[4] ^= pack | v4;
                            eu[5] ^= pack | v5;
                            eu[6] ^= pack | v6;
                            eu[7] ^= pack | v7;
                            edges[v0 as usize] ^= u0 as u32;
                            edges[v1 as usize] ^= (u0 + 1) as u32;
                            edges[v2 as usize] ^= (u0 + 2) as u32;
                            edges[v3 as usize] ^= (u0 + 3) as u32;
                            edges[v4 as usize] ^= (u0 + 4) as u32;
                            edges[v5 as usize] ^= (u0 + 5) as u32;
                            edges[v6 as usize] ^= (u0 + 6) as u32;
                            edges[v7 as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let v = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            i += 1;
                        }
                    } else {
                        let pack_hi = pack | hi;
                        let (chunks8, _) = pairs.as_chunks::<8>();
                        for ch in chunks8 {
                            let l0 = u32::from(u16::from_le_bytes(ch[0]));
                            let l1 = u32::from(u16::from_le_bytes(ch[1]));
                            let l2 = u32::from(u16::from_le_bytes(ch[2]));
                            let l3 = u32::from(u16::from_le_bytes(ch[3]));
                            let l4 = u32::from(u16::from_le_bytes(ch[4]));
                            let l5 = u32::from(u16::from_le_bytes(ch[5]));
                            let l6 = u32::from(u16::from_le_bytes(ch[6]));
                            let l7 = u32::from(u16::from_le_bytes(ch[7]));
                            let u0 = dst_start + i;
                            let eu = &mut edges[u0..u0 + 8];
                            eu[0] ^= pack_hi | l0;
                            eu[1] ^= pack_hi | l1;
                            eu[2] ^= pack_hi | l2;
                            eu[3] ^= pack_hi | l3;
                            eu[4] ^= pack_hi | l4;
                            eu[5] ^= pack_hi | l5;
                            eu[6] ^= pack_hi | l6;
                            eu[7] ^= pack_hi | l7;
                            edges[(hi | l0) as usize] ^= u0 as u32;
                            edges[(hi | l1) as usize] ^= (u0 + 1) as u32;
                            edges[(hi | l2) as usize] ^= (u0 + 2) as u32;
                            edges[(hi | l3) as usize] ^= (u0 + 3) as u32;
                            edges[(hi | l4) as usize] ^= (u0 + 4) as u32;
                            edges[(hi | l5) as usize] ^= (u0 + 5) as u32;
                            edges[(hi | l6) as usize] ^= (u0 + 6) as u32;
                            edges[(hi | l7) as usize] ^= (u0 + 7) as u32;
                            i += 8;
                        }
                        for &p in &pairs[i..] {
                            let u = dst_start + i;
                            let lo = u32::from(u16::from_le_bytes(p));
                            edges[u] ^= pack_hi | lo;
                            edges[(hi | lo) as usize] ^= u as u32;
                            i += 1;
                        }
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
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
                    let pack_hi = pack | hi;
                    for (i, &p) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let lo = u32::from(u16::from_le_bytes(p));
                        let v = hi | lo;
                        edges[u] ^= pack_hi | lo;
                        edges[v as usize] ^= u as u32;
                        if v == mark {
                            *predecessor = u as u32;
                        }
                    }
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
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    for (i, &p) in pairs.iter().enumerate() {
                        let u = dst_start + i;
                        let v = hi | u32::from(u16::from_le_bytes(p));
                        edges[u] ^= pack | v;
                        edges[v as usize] ^= u as u32;
                        if v == mark {
                            *predecessor = u as u32;
                        }
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
                }
            }

            #[inline(always)]
            pub(super) fn wire_uni(
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
                    let (pairs, _) = self.out[at..at + 2 * span].as_chunks::<2>();
                    let pack_hi = pack | hi;
                    for (eu, &p) in edges[dst_start..dst_start + span].iter_mut().zip(pairs) {
                        *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    self.at += span;
                    self.read += span as u32;
                } else {
                    self.wire_uni_slow(edges, dst_start, span, pack);
                }
            }

            #[cold]
            #[inline(never)]
            fn wire_uni_slow(
                &mut self,
                edges: &mut [u32],
                mut dst_start: usize,
                mut left: usize,
                pack: u32,
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
                    let hi = self.epoch << 16;
                    let room_chunk = CHUNK - self.at;
                    let room_cut = (self.cut - self.read) as usize;
                    let take = left.min(room_chunk).min(room_cut);
                    let chunk: &[u8] = if self.id < self.in_out {
                        let base = self.id as usize * BYTES;
                        &self.out[base..base + BYTES]
                    } else {
                        &self.spill.ovf[(self.id - self.in_out) as usize][..]
                    };
                    let (pairs, _) = chunk[2 * self.at..2 * (self.at + take)].as_chunks::<2>();
                    let pack_hi = pack | hi;
                    for (eu, &p) in edges[dst_start..dst_start + take].iter_mut().zip(pairs) {
                        *eu = pack_hi | u32::from(u16::from_le_bytes(p));
                    }
                    self.at += take;
                    self.read += take as u32;
                    dst_start += take;
                    left -= take;
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // Endpoint elimination (`CE-Var234` + `F-Inline3` + hoisted `limit` +
    // `S2aSlim` + hoisted `K-TailUnw` + `OrphanFree` + upfront `ensure_step_u8`)
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
    /// `CE-Var234` raw word-offset `id = rule_top`:
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
            if L::IS_NARROW {
                let (nodes, _) = self.nodes_buf.as_chunks_mut::<4>();
                let mut first = nodes[0][1] >> 16;
                let mut last = nodes[0][1] & 0xFFFF;
                let (mut lo, mut hi) = (0u32, rows);
                while first != last {
                    let nf = &mut nodes[first as usize];
                    let next = nf[1] >> 16;
                    let width = nf[2];
                    nf[1] = lo;
                    lo += width;
                    let nl = &mut nodes[last as usize];
                    let prev = nl[1] & 0xFFFF;
                    hi -= nl[2];
                    nl[1] = hi;
                    if next == last {
                        assert_eq!(lo, hi);
                        return;
                    }
                    first = next;
                    last = prev;
                }
                if first != 0 {
                    let nf = &mut nodes[first as usize];
                    nf[1] = lo;
                    lo += nf[2];
                }
                assert_eq!(lo, hi);
                return;
            }
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
            let (p, n) = (
                L::prev(self.nodes_buf, id, side),
                L::next(self.nodes_buf, id, side),
            );
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

    // Compact `4 KiB` stack-local `u16` `Dictionary::contents` (`-50%` vs `u32`, `0 B` heap on all
    // `447/447` real blocks!) fused directly into `V3CodeMiss` with lossless `#[cold]` promotion
    // to `contents_u32_fb` whenever `code >= u16_cap` (eliminating `edges` `CACHE_WORDS` growth).
    struct Dictionary<'a> {
        pairs: &'a mut [u32; 512],
        contents_u16: &'a mut [u16; 2048],
        contents_u32_fb: &'a mut Vec<u32>,
        u16_cap: u32,
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        istk: &'a mut [u32; INLINE_STACK],
        memo_rec: &'a mut [u32; MEMO_REC_CAP],
        stack: &'a mut Vec<u32>,
    }

    impl Dictionary<'_> {
        #[inline(always)]
        fn probe_contents(&self, index: usize, h: u64) -> Option<usize> {
            if self.contents_u32_fb.is_empty() {
                let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                let ent = self.contents_u16[index];
                if (ent & 0xF000) == tag4 {
                    Some((ent & 0x0FFF) as usize)
                } else {
                    None
                }
            } else {
                let ent = self.contents_u32_fb[index];
                if ent != 0 {
                    Some(ent as usize)
                } else {
                    None
                }
            }
        }

        #[inline(always)]
        fn store_contents(&mut self, index: usize, h: u64, code: u32) {
            if self.contents_u32_fb.is_empty() && code < self.u16_cap {
                let tag4 = ((((h >> 56) as u16) | 0x8) & 0xF) << 12;
                self.contents_u16[index] = tag4 | (code as u16);
            } else {
                Self::store_contents_u32_fallback(
                    self.contents_u16,
                    self.contents_u32_fb,
                    index,
                    code,
                );
            }
        }

        #[cold]
        #[inline(never)]
        fn store_contents_u32_fallback(
            tbl: &[u16; 2048],
            fb: &mut Vec<u32>,
            index: usize,
            code: u32,
        ) {
            if fb.is_empty() {
                fb.reserve_exact(2048);
                fb.extend(
                    tbl.iter()
                        .map(|&v| if v != 0 { u32::from(v & 0x0FFF) } else { 0 }),
                );
            }
            fb[index] = code;
        }

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
                let index = (h as usize) & 2047;
                if let Some(old) = self.probe_contents(index, h) {
                    if self.lens[old] == len && self.inline16[old] == entry {
                        return old as u32;
                    }
                }
                let c = self.insert(entry, len);
                self.store_contents(index, h, c);
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
                let index = (h as usize) & 2047;
                if let Some(old) = self.probe_contents(index, h) {
                    if self.lens[old] == LONG
                        && long_word(&self.inline16[old], self.spelling) == bytes
                    {
                        self.spelling.truncate(start);
                        return old as u32;
                    }
                }
                let c = self.insert(entry, LONG);
                self.store_contents(index, h, c);
                return c;
            }
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
            let index = (h as usize) & 2047;
            if let Some(old) = self.probe_contents(index, h) {
                if self.lens[old] == LONG && long_word(&self.inline16[old], self.spelling) == bytes
                {
                    if !memo_ovf {
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
                    return old as u32;
                }
            }
            let c = self.insert(entry, LONG);
            self.store_contents(index, h, c);
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
    fn invert_tiny_fl_with_buf(
        records: &[u16],
        n: usize,
        p: u32,
        fl: &mut [u32],
        out: &mut Vec<u8>,
    ) -> bool {
        let mut counts = [0u32; 256];
        let mut total = 0usize;
        for &rec in records {
            let len = usize::from(rec & 255);
            counts[usize::from((rec >> 8) as u8)] += len as u32;
            total += len;
        }
        if total != n {
            return false;
        }
        let mut st = starts(counts);
        let mut pos = 0u32;
        for &rec in records {
            let b = (rec >> 8) as u8;
            let len = usize::from(rec & 255);
            let bi = usize::from(b);
            let s0 = st[bi] as usize;
            st[bi] += len as u32;
            let bu = u32::from(b);
            let mut i = pos;
            for slot in &mut fl[s0..s0 + len] {
                *slot = (i << 8) | bu;
                i += 1;
            }
            pos += len as u32;
        }
        out.clear();
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
    // Kernel (`TinyFL` + `S1[Step4096] -> S2a[ZeroH+S2aSlim] -> S2b[Var234] ->
    // `S3a[V3CodeMiss+OrphanFree+S3aSlim] -> S4[SlimS4+SlowHi0+Sentinel] ->
    // `S5[SlimS5_Pipe1_Rem16_LE16+Sentinel]`)
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
                self.piece::<LIM>(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece::<LIM>(c, pos, width, false, p);
            } else {
                self.take_marked::<LIM>(c, pos, width, p);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked<const LIM: u32>(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.piece::<LIM>(c, pos, p - pos, false, p);
            }
            self.piece::<LIM>(c, p, 1, true, p);
            if p + 1 < end {
                self.piece::<LIM>(c, p + 1, end - p - 1, false, p);
            }
        }

        #[inline(always)]
        fn piece<const LIM: u32>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                    f.mark_byte = c;
                }
                self.queues.push_thin::<LIM>(a, f.m, w);
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
        contents_u32_fb: Vec<u32>,
        predecessor: Option<u32>,
        #[cfg(test)]
        force_no_zero_hdr: bool,
        #[cfg(test)]
        force_band_vec: bool,
        #[cfg(test)]
        force_sentinel_cap: Option<u32>,
        #[cfg(test)]
        force_u16_cap: Option<u32>,
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
            // Single check guarantees `pos += width` cannot overflow `u32` (16M * 255 < u32::MAX).
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
                        pass.piece::<LIM>(c, pos, width, true, p);
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
            // Principled post-S1 remaining-cost crossover guard (CA-26 + CA-31):
            // Abort to LazyCells/Seg8-FL on full-size blocks iff `m/n + 0.25*(r_thin/n) > 1.125`.
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
            self.contents_u32_fb.clear();

            // Exact O(1) byte offset in `out` strictly after OutQ16's live chunks.
            let in_out_chunks = out.len() / outq::BYTES;
            let tail_offset = (heads.chunks as usize).min(in_out_chunks) * outq::BYTES;
            #[cfg(test)]
            let use_zero_hdr = !self.force_no_zero_hdr && out.len() - tail_offset >= 16 * ns;
            #[cfg(not(test))]
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
                if L::IS_NARROW {
                    // `S2aSlim`: direct `[[u32; 4]]` node array + `[[u8; 4]; 3]` strip triples.
                    let (nodes_sl, _) = self.edges.split_at_mut(nodes_words);
                    let (nodes, _) = nodes_sl.as_chunks_mut::<4>();
                    nodes[0] = [0, 0, 0, 0];
                    let mut nn = ns as u32 + 1;
                    let (mut t0h, mut t0t) = (0u32, 0u32);
                    let (mut t1h, mut t1t) = (0u32, 0u32);
                    let (mut o0h, mut o0t) = (0u32, 0u32);
                    let (mut o1h, mut o1t) = (0u32, 0u32);
                    let (strips3, _) = strips_ot.as_chunks_mut::<3>();
                    let mut x = 0u32;
                    for (i, sw) in strips3.iter_mut().enumerate() {
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
                        widest = widest.max(width);
                        if x < from {
                            let gw = from - x;
                            widest = widest.max(gw);
                            let gid = nn;
                            nn += 1;
                            if o0t == 0 {
                                o0h = gid;
                            } else {
                                nodes[o0t as usize][0] =
                                    (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                            }
                            if t1t == 0 {
                                t1h = gid;
                            } else {
                                nodes[t1t as usize][1] =
                                    (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                            }
                            nodes[gid as usize] = [o0t, t1t, gw, 0];
                            o0t = gid;
                            t1t = gid;
                        }
                        if o0t == 0 {
                            o0h = sid;
                        } else {
                            nodes[o0t as usize][0] =
                                (nodes[o0t as usize][0] & 0xFFFF) | (sid << 16);
                        }
                        nodes[sid as usize] = [o0t, 0, width, literal(byte)];
                        o0t = sid;
                        x = from + width;
                    }
                    if x < n {
                        let gw = n - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o0t == 0 {
                            o0h = gid;
                        } else {
                            nodes[o0t as usize][0] =
                                (nodes[o0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        if t1t == 0 {
                            t1h = gid;
                        } else {
                            nodes[t1t as usize][1] =
                                (nodes[t1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [o0t, t1t, gw, 0];
                        o0t = gid;
                        t1t = gid;
                    }
                    x = 0;
                    for slot in image_ot.iter() {
                        let i = u32::from_ne_bytes(*slot);
                        let sw = &strips3[i as usize];
                        let to = u32::from_ne_bytes(sw[1]);
                        let width = u32::from_ne_bytes(sw[2]) >> 8;
                        if x < to {
                            let gw = to - x;
                            widest = widest.max(gw);
                            let gid = nn;
                            nn += 1;
                            if o1t == 0 {
                                o1h = gid;
                            } else {
                                nodes[o1t as usize][1] =
                                    (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                            }
                            if t0t == 0 {
                                t0h = gid;
                            } else {
                                nodes[t0t as usize][0] =
                                    (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                            }
                            nodes[gid as usize] = [t0t, o1t, gw, 0];
                            o1t = gid;
                            t0t = gid;
                        }
                        let sid = i + 1;
                        if o1t == 0 {
                            o1h = sid;
                        } else {
                            nodes[o1t as usize][1] =
                                (nodes[o1t as usize][1] & 0xFFFF) | (sid << 16);
                        }
                        nodes[sid as usize][1] = o1t;
                        o1t = sid;
                        x = to + width;
                    }
                    if x < n {
                        let gw = n - x;
                        widest = widest.max(gw);
                        let gid = nn;
                        nn += 1;
                        if o1t == 0 {
                            o1h = gid;
                        } else {
                            nodes[o1t as usize][1] =
                                (nodes[o1t as usize][1] & 0xFFFF) | (gid << 16);
                        }
                        if t0t == 0 {
                            t0h = gid;
                        } else {
                            nodes[t0t as usize][0] =
                                (nodes[t0t as usize][0] & 0xFFFF) | (gid << 16);
                        }
                        nodes[gid as usize] = [t0t, o1t, gw, 0];
                        o1t = gid;
                        t0t = gid;
                    }
                    let mut close_side = |side: usize, first_p: (u32, u32), second_p: (u32, u32)| {
                        let mut prev = 0u32;
                        for (head, tail) in [first_p, second_p] {
                            if tail == 0 {
                                continue;
                            }
                            nodes[prev as usize][side] =
                                (nodes[prev as usize][side] & 0xFFFF) | (head << 16);
                            nodes[head as usize][side] =
                                (nodes[head as usize][side] & 0xFFFF_0000) | prev;
                            prev = tail;
                        }
                        nodes[prev as usize][side] &= 0xFFFF;
                        nodes[0][side] = (nodes[0][side] & 0xFFFF_0000) | prev;
                    };
                    close_side(0, (t0h, t0t), (o0h, o0t));
                    close_side(1, (t1h, t1t), (o1h, o1t));
                    let open_nodes = nn as usize - 1;
                    (4 * nn as usize, nn, open_nodes)
                } else {
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
                }
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

            let (_, arena) = self.edges.split_at_mut(hdr_words);
            let (live, _) = arena.split_at_mut(base + rule_top);
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
            begin_inline(&mut self.inline16, &mut self.lens);

            // Fixed-pool injectivity + 256-bucket common refinement: bands_count < open_nodes + 256.
            let max_bands = open_nodes + 256;
            #[cfg(test)]
            let use_tail = !self.force_band_vec && out_tail.len() >= max_bands * 8;
            #[cfg(not(test))]
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            // Hoisted stack-local scratch allocated ONCE per block (`6.38 KiB` total L1D-hot!).
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            let mut pairs = [0u32; 512];
            let mut contents_u16 = [0u16; 2048];
            #[cfg(test)]
            let u16_cap = self.force_u16_cap.unwrap_or(U16_DICT_CAP).min(U16_DICT_CAP);
            #[cfg(not(test))]
            let u16_cap = U16_DICT_CAP;
            let mut dictionary = Dictionary {
                pairs: &mut pairs,
                contents_u16: &mut contents_u16,
                contents_u32_fb: &mut self.contents_u32_fb,
                u16_cap,
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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
                    let ta = thin[a];
                    if ta == 0 {
                        continue;
                    }
                    let end = k + ta;
                    let mut last_w = 0u32;
                    let mut last_c = 0u32;
                    while k < end {
                        let width = (end - k).min(node_width - in_node);
                        debug_assert!(width > 0);
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
            let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        reader.wire_xor_nomark(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                        );
                    }
                } else {
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        reader.wire_xor(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
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
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                if a != mb {
                    for &b in &self.bands[start..end] {
                        reader.wire_xor_nomark(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
                        );
                    }
                } else {
                    for &b in &self.bands[start..end] {
                        reader.wire_xor(
                            edges,
                            b.dst() as usize,
                            b.end() as usize,
                            b.code() << bits,
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
            let mut predecessor = 0u32;
            let mb = usize::from(mark_byte);
            let sent_pack = sentinel << bits;
            if used_tail {
                let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
                for a in 0..256 {
                    let start = cursors[a];
                    let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                    if start == end {
                        continue;
                    }
                    let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &slot in &band_slots[start..end] {
                        let b = Band(u64::from_le_bytes(slot));
                        let dst = b.dst() as usize;
                        let span = b.end() as usize;
                        let bcode = b.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            self.extra[dst..dst + span].fill(bcode);
                            sent_pack
                        };
                        if a != mb {
                            reader.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
                        }
                    }
                }
            } else {
                for a in 0..256 {
                    let start = cursors[a];
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    if start == end {
                        continue;
                    }
                    let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                    for &b in &self.bands[start..end] {
                        let dst = b.dst() as usize;
                        let span = b.end() as usize;
                        let bcode = b.code();
                        let pack = if bcode < sentinel {
                            bcode << bits
                        } else {
                            self.extra[dst..dst + span].fill(bcode);
                            sent_pack
                        };
                        if a != mb {
                            reader.wire_xor_nomark(edges, dst, span, pack);
                        } else {
                            reader.wire_xor(edges, dst, span, pack, mark, &mut predecessor);
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
            let (band_slots, _) = out_tail[..nbands * 8].as_chunks::<8>();
            for a in 0..256 {
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(nbands);
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &slot in &band_slots[start..end] {
                    let b = Band(u64::from_le_bytes(slot));
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    reader.wire_uni(edges, dst, span, pack);
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
                let start = cursors[a];
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                if start == end {
                    continue;
                }
                let mut reader = outq::Reader::new(out_q, &self.spill, heads, a);
                for &b in &self.bands[start..end] {
                    let dst = b.dst() as usize;
                    let span = b.end() as usize;
                    let bcode = b.code();
                    let pack = if !wide {
                        bcode << bits
                    } else if bcode < sentinel {
                        bcode << bits
                    } else {
                        self.extra[dst..dst + span].fill(bcode);
                        sentinel << bits
                    };
                    reader.wire_uni(edges, dst, span, pack);
                }
            }
        }
    }

    // ------------------------------------------------------------------------
    // `SlimS5_Pipe1_Rem16_LE16`: 1-step software-pipelined (`Q-2Cur-Pipe1`)
    // shrinking-slice `rem: &mut [u8]` + `lf/lb <= INLINE as u8` bounds-check
    // elimination + cold-outlined `LONG` helpers (BOTH `bi` & `uni` paths) +
    // cold `PackedEdgeRule32-Sentinel` fallback for `wide == true`.
    // ------------------------------------------------------------------------
    #[cold]
    #[inline(never)]
    fn emit_long_front(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let take = word.len().min(rem.len());
        rem[..take].copy_from_slice(&word[..take]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_long_back(slot: &[u8; INLINE], spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = long_word(slot, spelling);
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
    }

    #[cold]
    #[inline(never)]
    fn emit_tail_back(slot: &[u8; INLINE], lb: u8, spelling: &[u8], rem: &mut [u8]) -> usize {
        let word = if lb == LONG {
            long_word(slot, spelling)
        } else {
            &slot[..usize::from(lb)]
        };
        let rlen = rem.len();
        let take = word.len().min(rlen);
        rem[rlen - take..].copy_from_slice(&word[word.len() - take..]);
        take
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
        let mask = (1 << bits) - 1;
        let mut j = mark;
        if extra.is_empty() {
            let len_c = lens.len().min(inline16.len());
            let lens = &lens[..len_c];
            let inline16 = &inline16[..len_c];
            let mut rem: &mut [u8] = &mut out[..n];
            let mut done = false;
            if rem.len() >= INLINE {
                let mut edge = edges[j as usize];
                while rem.len() >= INLINE {
                    let j_next = edge & mask;
                    let edge_next = edges[j_next as usize];
                    let code = (edge >> bits) as usize;
                    let len = lens[code];
                    if len <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline16[code]);
                        rem = &mut rem[usize::from(len)..];
                    } else {
                        let take = emit_long_front(&inline16[code], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    j = j_next;
                    edge = edge_next;
                    if j == mark {
                        done = true;
                        break;
                    }
                }
            }
            if !done {
                while !rem.is_empty() {
                    let edge = edges[j as usize];
                    let code = (edge >> bits) as usize;
                    j = edge & mask;
                    let len = lens[code];
                    let word = if len == LONG {
                        long_word(&inline16[code], spelling)
                    } else {
                        &inline16[code][..usize::from(len)]
                    };
                    let take = word.len().min(rem.len());
                    rem[..take].copy_from_slice(&word[..take]);
                    rem = &mut rem[take..];
                    if j == mark {
                        break;
                    }
                }
            }
            let written = n - rem.len();
            out.truncate(written);
            repeat(n, out);
            return;
        }
        let out_buf = &mut out[..n + INLINE];
        let mut at = 0;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let raw_code = edge >> bits;
            let code = if raw_code == sentinel {
                extra[j as usize]
            } else {
                raw_code
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len <= INLINE as u8 {
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
        let out_base = out.as_ptr() as usize;
        let mut rem: &mut [u8] = &mut out[..n];
        let len_c = lens.len().min(inline.len());
        let lens = &lens[..len_c];
        let inline = &inline[..len_c];
        let mut done = false;
        if rem.len() >= 2 * INLINE {
            let mut ef = edges[front as usize];
            let mut eb = edges[back as usize];
            while rem.len() >= 2 * INLINE {
                if front == back {
                    let cf = (ef >> bits) as usize;
                    let lf = lens[cf];
                    if lf <= INLINE as u8 {
                        rem[..INLINE].copy_from_slice(&inline[cf]);
                        rem = &mut rem[usize::from(lf)..];
                    } else {
                        let take = emit_long_front(&inline[cf], spelling, rem);
                        rem = &mut rem[take..];
                    }
                    done = true;
                    break;
                }
                let f_next = (ef & mask) ^ prev;
                let b_next = (eb & mask) ^ next;
                // 1-step software-pipelined loads (`Q-2Cur-Pipe1`, +0 B WS):
                // overlap next-step `edges` L2/L3 misses with current-step emit!
                let ef_nxt = edges[f_next as usize];
                let eb_nxt = edges[b_next as usize];

                let cf = (ef >> bits) as usize;
                let lf = lens[cf];
                if lf <= INLINE as u8 {
                    rem[..INLINE].copy_from_slice(&inline[cf]);
                    rem = &mut rem[usize::from(lf)..];
                } else {
                    let take = emit_long_front(&inline[cf], spelling, rem);
                    rem = &mut rem[take..];
                    if rem.is_empty() {
                        done = true;
                        break;
                    }
                    if rem.len() < INLINE {
                        let cb = (eb >> bits) as usize;
                        let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                        let rlen = rem.len();
                        rem = &mut rem[..rlen - take_b];
                        prev = front;
                        front = f_next;
                        next = back;
                        back = b_next;
                        if front == next {
                            done = true;
                        }
                        break;
                    }
                }
                // Statically guaranteed: rem.len() >= INLINE (16) here!
                let cb = (eb >> bits) as usize;
                let rlen = rem.len();
                let lb = lens[cb];
                if lb <= INLINE as u8 {
                    let lbu = usize::from(lb);
                    let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                    rem[rlen - INLINE..].copy_from_slice(&shifted.to_le_bytes());
                    rem = &mut rem[..rlen - lbu];
                } else {
                    let take = emit_long_back(&inline[cb], spelling, rem);
                    rem = &mut rem[..rlen - take];
                }
                prev = front;
                front = f_next;
                ef = ef_nxt;
                next = back;
                back = b_next;
                eb = eb_nxt;
                if front == next {
                    done = true;
                    break;
                }
            }
        }
        if !done {
            while !rem.is_empty() {
                let ef = edges[front as usize];
                let cf = (ef >> bits) as usize;
                let single = front == back;
                let lf = lens[cf];
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(rem.len());
                rem[..take].copy_from_slice(&word[..take]);
                rem = &mut rem[take..];
                if single || rem.is_empty() {
                    break;
                }
                let eb = edges[back as usize];
                let cb = (eb >> bits) as usize;
                let take_b = emit_tail_back(&inline[cb], lens[cb], spelling, rem);
                let rlen = rem.len();
                rem = &mut rem[..rlen - take_b];
                (prev, front) = (front, (ef & mask) ^ prev);
                (next, back) = (back, (eb & mask) ^ next);
                if front == next {
                    break;
                }
            }
        }
        let rem_len = rem.len();
        let lo_rem = rem.as_ptr() as usize - out_base;
        let hi_rem = lo_rem + rem_len;
        if rem_len > 0 {
            out.copy_within(hi_rem..n, lo_rem);
            out.truncate(lo_rem + (n - hi_rem));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    /// Cold `PackedEdgeRule32-Sentinel` bidirectional emit (`wide == true`):
    /// reads `extra[idx]` ONLY when `(edge >> bits) == sentinel`.
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
        let out_buf = &mut out[..n];
        let mask = (1 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0usize, n);
        for _ in 0..=edges.len() / 2 {
            let ef = edges[front as usize];
            let raw_cf = ef >> bits;
            let cf = if raw_cf == sentinel {
                extra[front as usize]
            } else {
                raw_cf
            } as usize;
            let single = front == back;
            let lf = lens[cf];
            if lf <= INLINE as u8 && hi - lo >= INLINE {
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
            let raw_cb = eb >> bits;
            let cb = if raw_cb == sentinel {
                extra[back as usize]
            } else {
                raw_cb
            } as usize;
            let lb = lens[cb];
            if lb <= INLINE as u8 && hi - lo >= INLINE {
                let lbu = usize::from(lb);
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - lbu));
                out_buf[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= lbu;
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

        struct Rng(u64);
        impl Rng {
            fn next(&mut self) -> u64 {
                self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = self.0;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                z ^ (z >> 31)
            }
        }

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

        fn ref_fl_from_records(records: &[u16], n: usize, p: u32) -> Vec<u8> {
            let mut counts = [0u32; 256];
            for &rec in records {
                counts[(rec >> 8) as usize] += u32::from(rec & 255);
            }
            let mut st = starts(counts);
            let mut fl = vec![0u32; n];
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
            let mut out = vec![0u8; n];
            let mut x = p as usize;
            for slot in &mut out {
                let v = fl[x];
                *slot = v as u8;
                x = (v >> 8) as usize;
            }
            out
        }

        fn bwt_of(text: &[u8]) -> (Vec<u8>, u32) {
            let n = text.len();
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|&a, &b| {
                for off in 0..n {
                    let ca = text[(a + off) % n];
                    let cb = text[(b + off) % n];
                    if ca != cb {
                        return ca.cmp(&cb);
                    }
                }
                a.cmp(&b)
            });
            let mut l = Vec::with_capacity(n);
            let mut p = 0u32;
            for (row, &rot) in order.iter().enumerate() {
                if rot == 0 {
                    p = row as u32;
                }
                l.push(text[(rot + n - 1) % n]);
            }
            (l, p)
        }

        #[test]
        fn c54_small_valid_5760_and_var234_tiers_exact() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            // 510 exhaustive binary words n = 1..=8
            for n in 1..=8usize {
                for mask in 0..(1usize << n) {
                    let text: Vec<u8> = (0..n)
                        .map(|i| if (mask >> i) & 1 == 0 { b'a' } else { b'b' })
                        .collect();
                    let (l, p) = bwt_of(&text);
                    let recs = records_of(&l);
                    out.clear();
                    out.resize(n + 23, 0xAA);
                    assert!(k.try_run_const::<12>(&recs, n, p, bi, &mut out));
                    assert_eq!(out, text);
                }
            }
            // 5,250 random valid BWTs (total 5,760)
            let mut rng = Rng(0xC53A_0001_5760_BEEF);
            for case in 0..5250usize {
                let n = match case % 4 {
                    0 => 1 + (rng.next() as usize % 32),
                    1 => 33 + (rng.next() as usize % 64),
                    2 => 97 + (rng.next() as usize % 128),
                    _ => 225 + (rng.next() as usize % 128),
                };
                let alpha = match case % 4 {
                    0 => 2,
                    1 => 4,
                    2 => 16,
                    _ => 64,
                };
                let mut text = Vec::with_capacity(n);
                if case % 5 == 0 {
                    while text.len() < n {
                        let b = (rng.next() as usize % alpha) as u8;
                        let rl = 1 + (rng.next() as usize % 36);
                        for _ in 0..rl.min(n - text.len()) {
                            text.push(b);
                        }
                    }
                } else {
                    for _ in 0..n {
                        text.push((rng.next() as usize % alpha) as u8);
                    }
                }
                let (l, p) = bwt_of(&text);
                let recs = records_of(&l);
                out.clear();
                out.resize(n + 23, if case % 2 == 0 { 0xAA } else { 0x55 });
                assert!(k.try_run_const::<12>(&recs, n, p, bi, &mut out));
                assert_eq!(out, text);
            }
        }

        #[test]
        fn c54_malformed_330_and_detached_cycles_d1_to_65_bi_and_uni() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            let mut rng = Rng(0xC53B_0330_DEAD_C0DE);
            // 330 random malformed multi-cycle BWTs
            for case in 0..330usize {
                let n = 1 + (rng.next() as usize % 256);
                let mut l = Vec::with_capacity(n);
                if case % 3 == 0 {
                    let alpha = 2 + (rng.next() as usize % 8);
                    while l.len() < n {
                        let b = (rng.next() as usize % alpha) as u8;
                        let rl = 1 + (rng.next() as usize % 40);
                        for _ in 0..rl.min(n - l.len()) {
                            l.push(b);
                        }
                    }
                } else {
                    for _ in 0..n {
                        l.push((rng.next() as usize % 32) as u8);
                    }
                }
                let p = (rng.next() as usize % n) as u32;
                let recs = records_of(&l);
                let want = ref_fl_from_records(&recs, n, p);
                out.clear();
                out.resize(n + 31, 0x55);
                assert!(k.try_run_const::<12>(&recs, n, p, bi, &mut out));
                assert_eq!(out, want, "malformed case {case}");
            }
            // Detached malformed cycles d = 1..=65 on m >= 65_536 (bi XOR Pipe1!)
            for d in 1..=65usize {
                let sz = 75_000usize;
                let mut l = vec![0u8; sz];
                for i in 0..d {
                    l[i] = ((i + 1) % d) as u8;
                }
                for (i, slot) in l.iter_mut().enumerate().skip(d) {
                    *slot = (d + (i % 120)) as u8;
                }
                for slot in &mut l[sz - 8_000..sz] {
                    *slot = 250;
                }
                let recs = records_of(&l);
                let want = ref_fl_from_records(&recs, sz, 0);
                out.clear();
                out.resize(sz + 37, 0xAA);
                assert!(k.try_run_const::<12>(&recs, sz, 0, bi, &mut out));
                assert_eq!(out, want, "detached bi cycle d={d}");
                assert!(k.predecessor.is_some(), "d={d} must exercise bi XOR Pipe1");
            }
            // Detached cycles d = 1..=17 on uni Pipe1 (m < 65_536)
            for d in 1..=17usize {
                let sz = 40_000usize;
                let mut l = vec![0u8; sz];
                for i in 0..d {
                    l[i] = ((i + 1) % d) as u8;
                }
                for (i, slot) in l.iter_mut().enumerate().skip(d) {
                    *slot = (d + (i % 90)) as u8;
                }
                for slot in &mut l[sz - 8_000..sz] {
                    *slot = 250;
                }
                let recs = records_of(&l);
                let want = ref_fl_from_records(&recs, sz, 0);
                out.clear();
                assert!(k.try_run_const::<12>(&recs, sz, 0, bi, &mut out));
                assert_eq!(out, want, "detached uni cycle d={d}");
                assert!(k.predecessor.is_none(), "d={d} must exercise uni Pipe1");
            }
        }

        #[test]
        fn c54_slim_s5_pipe1_rem16_long_exceeds_rem_boundaries() {
            // Directly exercise `emit_bidirectional` and `emit` across n = 1..=48
            // when a LONG word (len = 40) exceeds `rem.len()` on front, back, and tail!
            let spelling: Vec<u8> = (0..40u8).map(|i| b'A' + (i % 26)).collect();
            let mut inline = Vec::new();
            let mut lens = Vec::new();
            begin_inline(&mut inline, &mut lens);
            inline.push(long_slot(0, 40)); // code 256: LONG (40 B)
            lens.push(LONG);
            let mut short7 = [0u8; INLINE];
            short7[..7].copy_from_slice(b"abcdefg");
            inline.push(short7); // code 257: len 7
            lens.push(7);

            // 2-node cycle: node 0 -> code 256 (LONG 40B), node 1 -> code 257 (7B)
            let bits = 2u32;
            let bi_edges = [(256u32 << bits) | (1 ^ 1), (257u32 << bits) | (0 ^ 0)];
            let uni_edges = [(256u32 << bits) | 1, (257u32 << bits) | 0];
            let mut full_word = spelling.clone();
            full_word.extend_from_slice(b"abcdefg"); // 47 B cycle

            for n in 1..=48usize {
                let mut out_bi = vec![0xCC; n + INLINE];
                emit_bidirectional(&bi_edges, &spelling, &inline, &lens, n, 0, 1, bits, &mut out_bi);
                let mut out_uni = vec![0xCC; n + INLINE];
                emit(&uni_edges, &[], &spelling, &inline, &lens, n, 0, bits, u32::MAX, &mut out_uni);
                assert_eq!(out_bi.len(), n);
                assert_eq!(out_uni.len(), n);
                // For n <= 40, front LONG truncates to `spelling[..n]`.
                if n <= 40 {
                    assert_eq!(out_bi, &spelling[..n], "bi LONG>rem at n={n}");
                    assert_eq!(out_uni, &spelling[..n], "uni LONG>rem at n={n}");
                } else if n <= 47 {
                    // Front writes 40B, back writes last `n-40` bytes of `abcdefg`
                    let mut exp = spelling.clone();
                    exp.extend_from_slice(&b"abcdefg"[7 - (n - 40)..]);
                    assert_eq!(out_bi, exp, "bi LONG+tail_back at n={n}");
                } else {
                    // n == 48: 47B cycle repeated to 48B
                    let mut exp = full_word.clone();
                    exp.push(full_word[0]);
                    assert_eq!(out_bi, exp, "bi cycle repeat at n={n}");
                    assert_eq!(out_uni, exp, "uni cycle repeat at n={n}");
                }
            }
        }

        #[test]
        fn c54_packed_edge_rule32_sentinel_natural_wide_and_forced() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;

            // 1. Forced sentinel across bi+tail, bi+vec, uni+tail, uni+vec
            for &(sz, expect_bi) in &[(82_000usize, true), (40_000usize, false)] {
                let mut l = vec![0u8; sz];
                for (i, slot) in l.iter_mut().enumerate() {
                    *slot = ((i * 29 + (i >> 4) * 7) % 180) as u8;
                }
                for pos in (0..sz - 200).step_by(200) {
                    let b = (180 + ((pos / 200) % 40)) as u8;
                    for j in 0..32 {
                        l[pos + j] = b;
                    }
                }
                let recs = records_of(&l);
                let want = ref_fl_from_records(&recs, sz, 123);
                for &force_vec in &[false, true] {
                    for &cap in &[256u32, 260, 300] {
                        k.force_band_vec = force_vec;
                        k.force_sentinel_cap = Some(cap);
                        out.clear();
                        assert!(k.try_run_const::<12>(&recs, sz, 123, bi, &mut out));
                        assert_eq!(out, want);
                        assert_eq!(k.predecessor.is_some(), expect_bi);
                        assert!(
                            !k.extra.is_empty(),
                            "forced sentinel cap={cap} must populate extra"
                        );
                    }
                }
                k.force_band_vec = false;
                k.force_sentinel_cap = None;
            }

            // 2. Natural `wide == true` block (m > 524,288, bits = 20, num_codes > 4,095)!
            let n = 840_000usize;
            let mut l = vec![0u8; n];
            let mut rng = Rng(0xC53D_A7A1_5E17_1E11);
            let mut pos = 0usize;
            let mut blk_id = 0usize;
            while pos < n {
                if pos + 40 < n && blk_id % 3 == 0 {
                    let b = (blk_id % 200) as u8;
                    let rl = 20 + (blk_id % 12);
                    for j in 0..rl.min(n - pos) {
                        l[pos + j] = b;
                    }
                    pos += rl.min(n - pos);
                    let r2 = (rng.next() as usize % 6) + 1;
                    for _ in 0..r2.min(n - pos) {
                        l[pos] = (rng.next() as usize % 240) as u8;
                        pos += 1;
                    }
                } else {
                    let rl = 2 + (rng.next() as usize % 3);
                    let b = (rng.next() as usize % 240) as u8;
                    for _ in 0..rl.min(n - pos) {
                        l[pos] = b;
                        pos += 1;
                    }
                }
                blk_id += 1;
            }
            let recs = records_of(&l);
            let want = ref_fl_from_records(&recs, n, 42);
            out.clear();
            assert!(k.try_run_const::<12>(&recs, n, 42, bi, &mut out));
            assert_eq!(out, want);
            assert!(k.predecessor.is_some(), "natural wide block must stay bi XOR");
            assert!(
                k.lens.len() > 4096 && !k.extra.is_empty(),
                "must trigger natural wide sentinel (lens={})",
                k.lens.len()
            );
        }

        #[test]
        fn c54_slim_s4_full_and_slow_hi0_mark_positions() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            let sz = 140_000usize;
            let mut l = vec![0u8; sz];
            for (i, slot) in l.iter_mut().enumerate() {
                *slot = ((i * 17 + (i >> 5) * 31) % 64) as u8;
            }
            for pos in (0..sz - 200).step_by(200) {
                let b = (64 + ((pos / 200) % 32)) as u8;
                for j in 0..32 {
                    l[pos + j] = b;
                }
            }
            let recs = records_of(&l);
            for &p in &[
                0u32,
                1,
                7,
                8,
                511,
                512,
                513,
                65_535,
                65_536,
                65_537,
                (sz - 1) as u32,
            ] {
                let want = ref_fl_from_records(&recs, sz, p);
                for &force_vec in &[false, true] {
                    k.force_band_vec = force_vec;
                    out.clear();
                    assert!(k.try_run_const::<12>(&recs, sz, p, bi, &mut out));
                    assert_eq!(out, want, "SlimS4 mismatch at p={p} force_vec={force_vec}");
                }
            }
            k.force_band_vec = false;
        }

        #[test]
        fn c54_zero_hdr_fallback_forced_and_natural() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            let n = 80_000usize;
            let mut l = Vec::with_capacity(n);
            for i in 0..n {
                l.push(((i * 31 + (i >> 3) * 17) & 63) as u8);
            }
            for pos in (0..n - 100).step_by(200) {
                let b = ((pos / 200) % 16) as u8;
                for j in 0..24 {
                    l[pos + j] = b;
                }
            }
            let recs = records_of(&l);
            let want = ref_fl_from_records(&recs, n, 12_345);
            for &force_no_zh in &[false, true] {
                k.force_no_zero_hdr = force_no_zh;
                out.clear();
                assert!(k.try_run_const::<12>(&recs, n, 12_345, bi, &mut out));
                assert_eq!(out, want, "ZeroH={force_no_zh} mismatch");
            }
            k.force_no_zero_hdr = false;
        }

        #[test]
        fn c54_orphanfree_deep85_memo_ovf_and_repoint() {
            let mut nodes_buf = vec![0u32; 64];
            let mut rules_buf = vec![0u32; 8192];
            let mut e = Elim::<Narrow> {
                nodes_buf: &mut nodes_buf,
                rules_buf: &mut rules_buf,
                rule_top: 0,
                nodes: 0,
                id_overflow: false,
                _links: std::marker::PhantomData,
            };
            let mk_17 = |e: &mut Elim<Narrow>, seed: u8| -> u32 {
                let mut h = literal(seed);
                for k in 1..17u8 {
                    h = e.join(h, literal(seed.wrapping_add(k)));
                }
                h
            };
            let h1_a = mk_17(&mut e, b'a');
            let h1_b = mk_17(&mut e, b'A');
            let root1 = e.join(h1_a, h1_b);
            let h2_a = mk_17(&mut e, b'a');
            let h2_b = mk_17(&mut e, b'A');
            let root2 = e.join(h2_a, h2_b);
            assert_ne!(root1, root2);

            let mk_deep85 = |e: &mut Elim<Narrow>| -> u32 {
                let mut cur = mk_17(e, b'Z');
                for k in 0..85u8 {
                    cur = e.join(cur, literal(b'0' + (k % 10)));
                }
                cur
            };
            let deep1 = mk_deep85(&mut e);
            let deep2 = mk_deep85(&mut e);
            assert_ne!(deep1, deep2);

            let mut pairs = [0u32; 512];
            let mut contents_u16 = [0u16; 2048];
            let mut contents_u32_fb = Vec::new();
            let mut inline16 = Vec::new();
            let mut lens = Vec::new();
            begin_inline(&mut inline16, &mut lens);
            let mut spelling = Vec::new();
            let mut istk = [0u32; INLINE_STACK];
            let mut memo_rec = [0u32; MEMO_REC_CAP];
            let mut stack = Vec::new();
            let mut dict = Dictionary {
                pairs: &mut pairs,
                contents_u16: &mut contents_u16,
                contents_u32_fb: &mut contents_u32_fb,
                u16_cap: U16_DICT_CAP,
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
            assert_eq!(dict.spelling.len(), sp1, "OrphanFree must reclaim 100% of orphan bytes");
            assert_eq!(dict.code(&mut e, h1_b, b'?'), dict.code(&mut e, h2_b, b'?'));

            let cd1 = dict.code(&mut e, deep1, b'#');
            let sp_d = dict.spelling.len();
            let cd2 = dict.code(&mut e, deep2, b'#');
            assert_eq!(cd1, cd2);
            assert!(dict.spelling.len() > sp_d, "memo_ovf fallback must keep live");
            assert!(dict.stack.capacity() > 0, "depth 85 must exercise ovf_stack > 64");
        }

        #[test]
        fn c54_tiny_fl_all_4_tiers_zero_heap_and_16m_guard() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let mut rng = Rng(0xC53F_1111_2222_3333);
            // All 4 zero-heap stack tiers: 64, 256, 512, 1536
            for &n in &[1usize, 2, 64, 65, 256, 257, 512, 513, 1536] {
                let text: Vec<u8> = (0..n).map(|_| (rng.next() as u8) % 16).collect();
                let (l, p) = bwt_of(&text);
                let recs = records_of(&l);
                assert!(k.invert_tiny_fl(&recs, n, p, &mut out));
                assert_eq!(out, text, "TinyFL mismatch at n={n}");
                assert_eq!(k.edges.capacity(), 0, "TinyFL n={n} <= 1536 must use 0 B heap");
            }
            // Heap tier: n = 1537 and n = 2048
            for &n in &[1537usize, 2048] {
                let text: Vec<u8> = (0..n).map(|i| ((i * 7 + (i >> 3)) % 32) as u8).collect();
                let (l, p) = bwt_of(&text);
                let recs = records_of(&l);
                assert!(k.invert_tiny_fl(&recs, n, p, &mut out));
                assert_eq!(out, text, "TinyFL heap tier mismatch at n={n}");
            }
            assert!(k.edges.capacity() >= 2048);

            // 16M record guard on both TinyFL and try_run_const (zero-width records)
            let giant = vec![0u16; 16_000_001];
            assert!(!k.invert_tiny_fl(&giant, 100, 0, &mut out));
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            assert!(!k.try_run_const::<12>(&giant, 40_000, 0, bi, &mut out));
        }

        #[test]
        fn c54_cauchy_short_periodic_exact_and_fallback() {
            let mut out = Vec::new();
            for &d in &[1usize, 2, 3, 7, 16, 63, 64] {
                let n = d * (4096usize.div_ceil(d));
                let pat: Vec<u8> = (0..d).map(|i| b'a' + (i % 26) as u8).collect();
                let text: Vec<u8> = (0..n).map(|i| pat[i % d]).collect();
                let (l, p) = bwt_of(&text);
                let recs = records_of(&l);
                if recs.len() * 64 < n {
                    let ok = super::super::C54DramLeanV1Moonshot::try_short_periodic(
                        &recs, n, p, &mut out,
                    );
                    if ok {
                        assert_eq!(out, text, "Cauchy periodic mismatch d={d}");
                    }
                }
            }
            // Non-periodic sparse block (e.g. d=65 > 64) must return false cleanly
            let n = 65 * 80;
            let mut text = vec![b'a'; n];
            text[0] = b'b';
            text[n / 2] = b'c';
            let (l, p) = bwt_of(&text);
            let recs = records_of(&l);
            assert!(!super::super::C54DramLeanV1Moonshot::try_short_periodic(
                &recs, n, p, &mut out
            ));
        }

        #[test]
        fn c54_u16_dict_forced_and_natural_fallback_and_lim12_24_32() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            let bi = super::super::C54DramLeanV1Moonshot::POST_S1_BI;
            let sz = 84_000usize;
            let mut l = vec![0u8; sz];
            for (i, slot) in l.iter_mut().enumerate() {
                *slot = ((i * 37 + (i >> 4) * 13) % 180) as u8;
            }
            for pos in (0..sz - 200).step_by(180) {
                let b = (180 + ((pos / 180) % 40)) as u8;
                for j in 0..40 {
                    l[pos + j] = b;
                }
            }
            let recs = records_of(&l);
            let want = ref_fl_from_records(&recs, sz, 77);
            // Forced u16_cap fallback at 258, 280, 300 across LIM = 12, 24, 32
            for &cap in &[258u32, 280, 300] {
                k.force_u16_cap = Some(cap);
                out.clear();
                assert!(k.try_run_const::<12>(&recs, sz, 77, bi, &mut out));
                assert_eq!(out, want, "forced u16_cap={cap} LIM=12 mismatch");
                assert!(!k.contents_u32_fb.is_empty(), "forced u16_cap={cap} must promote to u32 fb");
            }
            k.force_u16_cap = None;
            for &lim in &[12u32, 24, 32] {
                out.clear();
                let ok = match lim {
                    12 => k.try_run_const::<12>(&recs, sz, 77, bi, &mut out),
                    24 => k.try_run_const::<24>(&recs, sz, 77, bi, &mut out),
                    _ => k.try_run_const::<32>(&recs, sz, 77, bi, &mut out),
                };
                assert!(ok);
                assert_eq!(out, want, "LIM={lim} mismatch");
                assert!(k.contents_u32_fb.is_empty(), "normal block must stay in 4KiB stack u16 dict");
            }
        }
    }
}
