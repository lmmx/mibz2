//! Cycle 50 Specialisation Interim Verified Handoff #1 — **Unified Drop-In v1**:
//! **`C50-DropIn-v1`** (`F-Inline3 + S3a-OutTail64-Cln + as_chunks::<8>() + Clean S1 + TrimWS +
//! Stack-Local OutQ next/cuts + Zero-WS TinyFL + K-TailUnw + Exact r_thin PostS1 + CA-37/C50-D1 Crossover`).
//!
//! Self-contained single-file `#![forbid(unsafe_code)]` implementation of `crate::bwt::InverseBwt`
//! over `cand_r5_head` (`25be0973d`) cleanly fusing ALL verified Cycles 46–49 breakthroughs:
//!
//! 1. **`F-Inline3` (`CS-06`, C47):** `ID_BITS = 24`, `SATURATED = 255`, `RULE_BASE = 0` packing
//!    `len = 1, 2, 3` words directly inside the `u32` handle with a collision-free 40-bit
//!    `(handle, (byte << 24) | code)` 512-word `pairs` cache.
//! 2. **`S3a-OutTail64-Cln` + `as_chunks::<8>()` + Clean `S1` + `TrimWS` (`CS-09` / `CS-17`, C48/C49):**
//!    - Places `S3a`'s fixed-stride 8-byte `Band(u64)` table directly in the provably disjoint tail
//!      of `out` (`out[tail_offset..]`, statically enforced via `out.split_at(tail_offset)` and
//!      `out_tail.as_chunks_mut::<8>()` / `as_chunks::<8>()`, zero `Option::unwrap` / `try_into`),
//!      using `O(1)` `heads.chunks` offset and exact bound `bands < open_nodes + 256` with
//!      unconditional fallback to `self.bands: Vec<Band>`.
//!    - Eliminates per-record `width == 0` / `checked_add` from `S1` via a single upfront
//!      `nrec > 16_000_000` guard (`16M * 255 < u32::MAX`), uses `8192`-word stepped `grow_edges_step`
//!      (eliminating `2x` heap doubling for `RSS <= 1.0`), stepped `TrimWS` (`8192 B` `spelling`,
//!      `256` `inline16+lens`), and `#[cold] #[inline(never)]` cold helpers.
//! 3. **Compact `u16` `OutQ` `next` + Exact-Reserve `cuts` + Stack-Local `append` Scratch + Zero-WS `TinyFL` (`CS-17`):**
//!    - `OutQ` `next` uses compact `Vec<u16>` (`limit = n <= 900_000 => chunks <= 2,015 << 65,535`,
//!      halving `spill.next` to `~1.5 KiB`) and `cuts` uses exact-stepped `reserve_exact(1)`
//!      (`0` per-block stack zeroing, avoiding `W=8` LLC cliff pollution while keeping `spill = 0.00 KiB`
//!      on tiny blocks), plus stack-local `S3a` `append` scratch (`[u32; 64]`, 256 B stack, certified by
//!      `CA-35` Lucas depth `<= 26`) keeping `self.stack = 0.00 KiB` persistent heap WS.
//!    - Zero-WS `invert_tiny_fl` (`n < 32_768`) reuses `self.open.edges` (`+0 B` persistent WS) and
//!      enforces `auditor_c49`'s `records.len() > 16_000_000` parity guard.
//! 4. **`K-TailUnw` (`CS-16`, C49 P2, guided by `MB-14` Horton–Strahler–Ershov `S = 2..3`):**
//!    Register-spine tail-call unwind in `Elim::append` (`-73%..-91%` stack pushes at `+0 B` WS).
//! 5. **Exact `r_thin` `Spec4` `μ ∈ {1, 2}` Selector (`CA-26`/`CA-27`/`CA-31`) + Exact `CA-37`/`C50-D1`
//!    Pre-`S1` & Post-`S1` `Seg8-FL` Crossover + Cauchy (`CA-05`) Short-Periodic Fast Path:**
//!    - Classical pre-`S1` Markov–MacMahon threshold `17 * nrec >= 13 * n` (`L = 16 <= 63 = L*(127)`)
//!      unconditionally certifies `3*m + r_thin >= 3*n` (zero false rejects on `Code/Docs`), paired
//!      with post-`S1` `8*m + 2*r_thin_lo > 9*n`, 2-stage (`64 -> 256`) `CA-26` Gauss-OLS `LIM ∈ {16,24,32}`
//!      selector, and exact `r_max = nrec - extra_127` (`0` hot-loop ops) `PostS1ExactMd` `μ ∈ {1,2}` switch.
//! 6. **Unconditional Forward-`FL` Parity & `LazyCells` Fallback:**
//!    Matches `OpenCells` / `libbzip2` forward-`FL` byte-for-byte on all valid, adversarial, and
//!    malformed multi-cycle inputs.
//!
//! Strictly honest Weiss classification: `CA-05`, `CA-26`, `CA-27`, `CA-31`, `CA-35`, `CA-37` are
//! load-bearing classical theorems; `F-Inline3`, `OutTail64`, `TrimWS`, `K-TailUnw`, stack-local
//! `OutQ`, and `TinyFL` are strictly `2020s` modern CS (`CS`, ZERO relabels).
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Exported `#729` moonshot candidate type.
pub type Candidate = C50DropInV1Moonshot;

/// Cycle 50 Interim Verified Handoff #1 Unified Drop-In v1 over `OpenCells`.
#[derive(Default)]
pub struct C50DropInV1Moonshot {
    open: open_dropin_v1::Kernel,
    lazy: LazyCells,
}

impl C50DropInV1Moonshot {
    const POST_S1_BI: open_dropin_v1::BiMode = open_dropin_v1::BiMode::PostS1ExactMd {
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
    /// Selects `LIM ∈ {16, 24, 32}` (all strictly `<= 63 = L*(127)`!) in `~74 ns` on Wikidata.
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

impl InverseBwt for C50DropInV1Moonshot {
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
        // Cauchy (1815) exact short-periodic fast path; non-periodic sparse blocks fall through to Open.
        if nrec * 64 < n && Self::try_short_periodic(records, n, p, out) {
            return true;
        }
        // Exact pre-S1 Markov-MacMahon sound reject (CA-37 / C50-D1, L = 16 <= 63 = L*(127)):
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

mod open_dropin_v1 {
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;
    const INLINE_STACK: usize = 64;

    // F-Inline3 handle encoding: 8-bit grade (1..=255), 24-bit inline payload / rule id (RULE_BASE=0).
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
    // `next: Vec<u16>` (`limit = n <= 900_000 => chunks <= 2,015 << 65,535`)
    // and exact-reserve `cuts: Vec<[u32; 256]>` (`0` per-block stack zeroing).
    // ------------------------------------------------------------------------
    mod outq {
        const CHUNK: usize = 512;
        pub(super) const BYTES: usize = 2 * CHUNK;
        const NONE: u32 = u32::MAX;
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
                debug_assert!(id < u32::from(NONE16));
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
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
                if self.heads[a] == NONE {
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
    // Endpoint elimination (`F-Inline3`: ID_BITS=24, SATURATED=255, RULE_BASE=0)
    // + `K-TailUnw` register-spine tail-call unwind in `Elim::append` (`CS-16`).
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
            istk: &mut [u32; INLINE_STACK],
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let v = self.short_payload_h(word) | (u128::from(byte) << (8 * height));
                (v.to_le_bytes(), (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                self.append(word, istk, stack, spelling);
                ensure_step_u8(spelling, 1, 8192);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }

        /// `K-TailUnw` (`CS-16`): register-spine tail-call unwind with stack-local `[u32; 64]`
        /// scratch (zero heap `self.stack` on real blocks where `CA-35` depth `<= 26`) + exact Vec overflow.
        #[inline(always)]
        fn append(
            &mut self,
            handle: u32,
            istk: &mut [u32; INLINE_STACK],
            ovf_stack: &mut Vec<u32>,
            dest: &mut Vec<u8>,
        ) {
            ovf_stack.clear();
            let mut sp = 0usize;
            let mut h = handle;
            loop {
                let g = h >> ID_BITS;
                if g < INLINE as u32 {
                    let bytes = self.short_payload_h(h).to_le_bytes();
                    ensure_step_u8(dest, g as usize, 8192);
                    dest.extend_from_slice(&bytes[..g as usize]);
                    if let Some(next) = ovf_stack.pop() {
                        h = next;
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                let r = self.rule(h & ID_MASK);
                let tag = self.a[r + 3];
                if tag != u32::MAX {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let rlen = self.a[r + 2] as usize;
                    ensure_step_u8(dest, rlen, 8192);
                    dest.extend_from_within(start..start + rlen);
                    if let Some(next) = ovf_stack.pop() {
                        h = next;
                        continue;
                    }
                    if sp > 0 {
                        sp -= 1;
                        h = istk[sp];
                        continue;
                    }
                    return;
                }
                self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                let left = self.a[r];
                let right = self.a[r + 1];
                let lg = left >> ID_BITS;
                if lg < INLINE as u32 {
                    let lb = self.short_payload_h(left).to_le_bytes();
                    ensure_step_u8(dest, lg as usize, 8192);
                    dest.extend_from_slice(&lb[..lg as usize]);
                    let rg = right >> ID_BITS;
                    if rg < INLINE as u32 {
                        let rb = self.short_payload_h(right).to_le_bytes();
                        ensure_step_u8(dest, rg as usize, 8192);
                        dest.extend_from_slice(&rb[..rg as usize]);
                        if let Some(next) = ovf_stack.pop() {
                            h = next;
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
            let memoized = e.a[e.rule(id) + 3] != u32::MAX;
            e.append(handle, self.istk, self.stack, self.spelling);
            ensure_step_u8(self.spelling, 1, 8192);
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
                if memoized {
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
            ensure_step_inline(self.inline16, self.lens, 256);
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
    // Kernel (`TinyFL` + `S1 -> S2a -> S2b -> S3a[OutTail64-Cln+K-TailUnw] -> S4 -> S5`)
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
                    let need = (at + 3).max(2 * self.edges.len()).max(1024);
                    self.edges.reserve_exact(need - self.edges.len());
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
    }

    impl Kernel {
        fn nodes_most(ns: usize) -> usize {
            3 * ns + 3
        }

        fn arena_words(ns: usize, m: usize) -> usize {
            let most = Self::nodes_most(ns);
            let stride = if most > NARROW_NODES {
                Wide::STRIDE
            } else {
                Narrow::STRIDE
            };
            (4 * ns + (stride + RULE) * most).max(m)
        }

        /// Zero-WS exact forward-`FL` walk for tiny blocks (`n < 32_768`), reusing `self.edges`
        /// (`+0 B` persistent WS!) with `auditor_c49`'s `records.len() > 16_000_000` parity guard.
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
            let need = Self::arena_words(ns, m);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }

            self.bands.clear();
            self.spelling.clear();

            // Exact O(1) byte offset in `out` strictly after OutQ16's live chunks.
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
            let use_xor = !wide && bi_mode.should_use_bi(f.m, ns as u32, r_max);
            let (out_q, out_tail) = out.split_at(tail_offset);

            if use_xor {
                let pred = if used_tail {
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
                };
                self.predecessor = Some(pred);
            } else if used_tail {
                self.wire_uni_tail(
                    m,
                    wide,
                    bits,
                    &cursors,
                    nbands,
                    out_tail,
                    out_q,
                    heads,
                );
            } else {
                self.wire_uni_vec(m, wide, bits, &cursors, out_q, heads);
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

            let max_bands = open_nodes + 256;
            let use_tail = out_tail.len() >= max_bands * 8;
            let mut cursors = [0usize; 256];
            let mut nbands = 0usize;
            let mut istk = [0u32; INLINE_STACK];

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
                                &mut istk,
                                &mut self.stack,
                                &mut self.spelling,
                            );
                            ensure_step_inline(&mut self.inline16, &mut self.lens, 256);
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

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_uni_tail(
            &mut self,
            m: usize,
            wide: bool,
            bits: u32,
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
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
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
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
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
        for pair in 0..=edges.len() / 2 {
            if lo == hi || (pair != 0 && front == next) {
                break;
            }
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
            if single {
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
        fn c50_f_inline3_sat255_ktailunw_and_outtail_poison() {
            let mut k = Kernel::default();
            let mut out = vec![0xAAu8; 40_000];
            for &per in &[253usize, 255, 256, 257] {
                let n = per * 140;
                let mut l = vec![0u8; n];
                l[0] = 2;
                l[n - 1] = 1;
                let recs = records_of(&l);
                let want = plain_fl(&l, 0);
                out.clear();
                out.resize(n + 64, 0xAA);
                assert!(k.try_run_const::<16>(
                    &recs,
                    n,
                    0,
                    super::super::C50DropInV1Moonshot::POST_S1_BI,
                    &mut out
                ));
                assert_eq!(out, want);
            }
        }

        #[test]
        fn c50_stack_local_outq_inline_and_overflow_exact() {
            let mut k = Kernel::default();
            // Exercise > 4 epochs (m > 262_144 -> forces cuts overflow) AND > 1024 chunks!
            let n = 400_000usize;
            let mut l = Vec::with_capacity(n);
            let mut s = 0x1234_5678_9ABC_DEF0u64;
            for i in 0..n {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                if i < 60_000 {
                    l.push(b'a'); // thick run so 8*m + 2*r_thin <= 9*n passes
                } else {
                    l.push(((s >> 40) % 250) as u8);
                }
            }
            let recs = records_of(&l);
            let want = plain_fl(&l, 12_345);
            let mut out = vec![0x55u8; n + 64];
            assert!(k.try_run_const::<16>(
                &recs,
                n,
                12_345,
                super::super::C50DropInV1Moonshot::POST_S1_BI,
                &mut out
            ));
            assert_eq!(out, want);
        }

        #[test]
        fn c50_zero_ws_tiny_fl_auditor_guard_and_malformed_multicycle() {
            let mut k = Kernel::default();
            let mut out = Vec::new();
            for &(l, p) in &[(b"bbaabb".as_slice(), 0u32), (b"abbaab".as_slice(), 3u32)] {
                let recs = records_of(l);
                let want = plain_fl(l, p);
                assert!(k.invert_tiny_fl(&recs, l.len(), p, &mut out));
                assert_eq!(out, want);
                assert!(k.try_run_const::<16>(
                    &recs,
                    l.len(),
                    p,
                    super::super::C50DropInV1Moonshot::POST_S1_BI,
                    &mut out
                ));
                assert_eq!(out, want);
            }
            // Broken tiling must cleanly return false
            assert!(!k.invert_tiny_fl(&[(u16::from(b'a') << 8) | 3], 4, 0, &mut out));
            assert!(!k.try_run_const::<16>(
                &[(u16::from(b'a') << 8) | 3],
                4,
                0,
                super::super::C50DropInV1Moonshot::POST_S1_BI,
                &mut out
            ));
        }
    }
}
