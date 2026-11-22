//! Cycle 47 Specialisation Drop-In Candidate for `#729` `bench/moonshot` (`0.5.12` `OpenCells`).
//!
//! Self-contained single-file `#![forbid(unsafe_code)]` implementation of `crate::bwt::InverseBwt`
//! combining:
//! 1. **Unified Open Kernel (`+0 B` persistent WS, `945.8 KiB` steady-state):** Fuses Round-5 head
//!    (`cand_r5_head` `XOR-BiEmit` + `OutQ16` packed `pt` + `endpoint.rs`) with `cand_r5_nobi`'s
//!    `W16` 16-wide SWAR serial wire, monomorphized over `const LIM ∈ {16, 24, 32}` (`0` extra
//!    instructions in the `S1` hot loop) plus a `pos != n` broken-tiling guard.
//! 2. **`Spec1-TinyDense-FL` (Dirichlet 1830s + Cauchy 1815):** Exact forward-`FL` `PlainRunWalk`
//!    for tiny blocks (`n < 32,768`, `<= 128 KiB` transient buffer, `-0.9%` `chaingrind` instr)
//!    and exact cycle-checked short-periodic fast path (`tot == n && x == p && n % d == 0`).
//! 3. **`Spec3-PostS1` (Markov-Dirichlet 1860s + Neyman-Pearson 1933 / Fisher 1936 LDA):**
//!    Zero-pass `~1 ns` post-S1 decision at `S4` choosing `μ = 1` (`W16` serial wire+emit) vs
//!    `μ = 2` (`XOR-BiEmit`) via the exact lower-moment singleton bound `md_w1 = 2*(recs - ns) - m`
//!    (`μ = 1` iff `m < 65_536 || (recs <= 57_000 && ns <= 2_400 && md_w1 <= 26_000)`).
//! 4. **`Spec4-SPRT-L` (Wald 1945 SPRT + Chebyshev-Markov 1867):** 2-stage (`64 -> 256` sample)
//!    sub-microsecond stride sampler (`C_recognise ~ 0.3-0.5 us`) selecting `L ∈ {16, 24, 32}`
//!    before `S1` to capture non-Wikidata JSONL/syslog headroom (`-4.5%` 1T / `-5.4%` 2W) while
//!    preserving full `R5-Head` speed on Wikidata (`431` blocks).
//! 5. **Unconditional Exact Fallback:** Delegates column-route, `Cap::dense` refusals, invalid
//!    `(n, p)`, and broken tilings directly to `crate::bwt::LazyCells`.
#![forbid(unsafe_code)]

use crate::bwt::{InverseBwt, LazyCells};
use crate::runs::Runs;
use crate::Kernels;

/// Exported `#729` moonshot candidate type.
pub type Candidate = C47SelectorMoonshot;

/// Cycle 47 unified adaptive selector kernel (`Spec1-TinyDense-FL` + `Spec3-PostS1` + `Spec4-SPRT-L`).
#[derive(Default)]
pub struct C47SelectorMoonshot {
    open: unified_open::Kernel,
    lazy: LazyCells,
    plain_fl: Vec<u32>,
}

impl C47SelectorMoonshot {
    const POST_S1_BI: unified_open::BiMode = unified_open::BiMode::PostS1Md {
        recs_max: 57_000,
        ns_max: 2_400,
        md_max: 26_000,
    };

    /// Exact forward-`FL` tiny-block walk (`n < 32_768`), matching `libbzip2` / `InverseBwt`
    /// on ALL columns including malformed multi-cycle inputs; returns `false` on broken tiling.
    #[inline(never)]
    fn invert_tiny_fl(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n {
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
        let mut starts = [0u32; 256];
        let mut tot = 0u32;
        for c in 0..256 {
            starts[c] = tot;
            tot += counts[c];
        }
        if tot as usize != n {
            return false;
        }
        if self.plain_fl.len() < n {
            self.plain_fl.resize(n, 0);
        }
        let fl = &mut self.plain_fl[..n];
        let mut pos = 0usize;
        for &rec in records {
            let b = (rec >> 8) as usize;
            let len = (rec & 255) as usize;
            let s0 = starts[b] as usize;
            starts[b] += len as u32;
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

    /// Cauchy (1815) exact short-cycle orbit detector (`recs*64 < n`, `n >= 4096`).
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

    /// Classical Wald (1945) SPRT + Chebyshev-Markov (1867) 2-stage stride sampler (`64 -> 256`).
    /// Selects `L ∈ {16, 24, 32}` in `< 0.5 us` before `S1`.
    #[inline]
    fn sprt_select_limit(records: &[u16], n: usize) -> u32 {
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

impl InverseBwt for C47SelectorMoonshot {
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
            return self.invert_tiny_fl(records, n, p, out);
        }
        let nrec = records.len();
        if nrec * 64 < n {
            if Self::try_short_periodic(records, n, p, out) {
                return true;
            }
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        if nrec * 25 >= n * 7 {
            return self.lazy.invert_runs(r, n, p, out, k);
        }
        let cap = if nrec * 20 < n * 3 {
            unified_open::Cap::NONE
        } else {
            unified_open::Cap::dense(n)
        };
        let lim = Self::sprt_select_limit(records, n);
        let ok = match lim {
            16 => self
                .open
                .try_run_const::<16>(records, n, p, cap, Self::POST_S1_BI, out)
                .is_some(),
            24 => self
                .open
                .try_run_const::<24>(records, n, p, cap, Self::POST_S1_BI, out)
                .is_some(),
            _ => self
                .open
                .try_run_const::<32>(records, n, p, cap, Self::POST_S1_BI, out)
                .is_some(),
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

#[allow(dead_code)]
mod unified_open {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BiMode {
        Never,
        AlwaysAt(u32),
        PostS1Md {
            recs_max: u32,
            ns_max: u32,
            md_max: i64,
        },
    }

    impl BiMode {
        #[inline(always)]
        pub fn should_use_bi(self, m: u32, ns: u32, recs: u32) -> bool {
            match self {
                BiMode::Never => false,
                BiMode::AlwaysAt(thr) => m >= thr,
                BiMode::PostS1Md {
                    recs_max,
                    ns_max,
                    md_max,
                } => {
                    if m < 65_536 {
                        return false;
                    }
                    let md_w1 = 2i64 * (i64::from(recs) - i64::from(ns)) - i64::from(m);
                    !(recs <= recs_max && ns <= ns_max && md_w1 <= md_max)
                }
            }
        }
    }

    #[derive(Default, Clone, Copy, Debug)]
    pub struct Counts {
        pub open_nodes: usize,
        pub steps: usize,
        pub rules: usize,
        pub n: usize,
        pub runs: usize,
        pub retained: usize,
        pub thick: usize,
        pub bands: usize,
        pub words: usize,
        pub copies: usize,
        pub traced: usize,
        pub thick_area: usize,
        pub unit_work: usize,
        pub longest: usize,
        pub table_bytes: usize,
        pub spirals: bool,
        pub used_bi: bool,
    }

    #[derive(Clone, Copy)]
    pub(super) struct Strip {
        pub(super) from: u32,
        pub(super) to: u32,
        pub(super) width: u32,
        pub(super) byte: u8,
    }

    impl Strip {
        #[inline(always)]
        pub(super) fn unpack(w: &[u32]) -> Self {
            Strip {
                from: w[0],
                to: w[1],
                width: w[2] >> 8,
                byte: (w[2] & 0xFF) as u8,
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Tile {
        end: u32,
        delta: u32,
        symbol: u16,
    }

    #[derive(Clone, Copy)]
    pub(super) struct Band(u64);

    impl Band {
        #[inline(always)]
        pub(super) fn new(end: u32, dst: u32, code: u32) -> Self {
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

    #[derive(Default)]
    pub struct Kernel {
        endpoint: endpoint::Endpoint,
        predecessor: Option<u32>,
        spill: outq::Spill,
        tiles: Vec<Tile>,
        directory: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        offsets: Vec<u32>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
    }

    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;
    pub(super) const INLINE: usize = 16;
    pub(super) const LONG: u8 = u8::MAX;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cap {
        pub m: u32,
        pub strips: usize,
    }

    impl Cap {
        pub const NONE: Cap = Cap {
            m: u32::MAX,
            strips: usize::MAX,
        };
        pub fn dense(n: usize) -> Cap {
            Cap {
                m: DENSE_M.min((n as u64 * 3 / 10) as u32),
                strips: DENSE_STRIPS,
            }
        }
    }

    #[inline(always)]
    fn split(
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
        mut visit: impl FnMut(u8, u32, u32, bool),
    ) {
        let end = pos + width;
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                visit(c, pos, p - pos, false);
            }
            visit(c, p, 1, true);
            if p + 1 < end {
                visit(c, p + 1, end - p - 1, false);
            }
        } else {
            visit(c, pos, width, width <= limit);
        }
    }

    struct Pass<'a> {
        queues: outq::Queues<'a>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl Pass<'_> {
        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            self.f.runs += 1;
            if width <= limit {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p, limit);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            split(c, pos, width, p, limit, |c, x, w, keep| {
                self.piece(c, x, w, keep, p)
            });
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

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    pub(super) fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
        let mut slot = [0; INLINE];
        slot[..4].copy_from_slice(&start.to_le_bytes());
        slot[4..8].copy_from_slice(&end.to_le_bytes());
        slot
    }

    fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes(slot[..4].try_into().unwrap()) as usize;
        let end = u32::from_le_bytes(slot[4..8].try_into().unwrap()) as usize;
        &spelling[start..end]
    }

    pub(super) fn begin_inline(
        words: usize,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
    ) {
        if inline16.len() < 256 {
            inline16.clear();
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
        inline16.reserve_exact(words);
        lens.reserve_exact(words);
    }

    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

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
        out.resize(n + INLINE, 0);
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
        out.resize(n + INLINE, 0);
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

    impl Kernel {
        fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32, cap_strips: usize) -> Pass<'a> {
            Pass {
                queues: outq::Queues::new(out, &mut self.spill, limit),
                edges: &mut self.edges,
                ns: 0,
                cap_strips,
                f: First::default(),
            }
        }

        #[inline(always)]
        fn construct_capped<const LIM: u32>(
            &mut self,
            records: &[u16],
            n: u32,
            p: u32,
            cap: Cap,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> Option<(Counts, u32, u32)> {
            out.resize(n as usize + INLINE, 0);
            let mut pass = self.begin(&mut out[..], cap.m, cap.strips);
            let mut pos = 0u32;
            {
                let mut i = 0usize;
                let nrec = records.len();
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
                                i += 1;
                            }
                        }
                        pass.take(c, pos, width, p, LIM);
                        pos += width;
                    }
                }
            }
            if pos != n {
                return None;
            }
            let ns = pass.ns;
            let mut f = pass.f;
            f.thin = pass.queues.extract_thin();
            let (heads, over) = (pass.queues.heads(), pass.queues.over());
            if over || f.m == 0 || f.m > cap.m || ns > cap.strips {
                return None;
            }
            Some(self.finish(&f, ns, n, records.len() as u32, &out[..], &heads, bi_mode))
        }

        #[inline(always)]
        fn finish(
            &mut self,
            f: &First,
            ns: usize,
            n: u32,
            nrec: u32,
            out: &[u8],
            heads: &outq::Heads,
            bi_mode: BiMode,
        ) -> (Counts, u32, u32) {
            let First {
                thick_rows,
                thin,
                thick,
                m,
                mark,
                mark_byte,
                runs,
            } = *f;
            let mut stats = Counts {
                n: n as usize,
                runs,
                retained: m as usize,
                thick: ns,
                ..Counts::default()
            };
            let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
            debug_assert!(m > 0 && all.iter().sum::<u32>() == n);
            let first = starts(all);
            let mut write = starts(thick);
            let need = self.arena_words(ns, m as usize);
            if self.edges.len() < need {
                let add = need - self.edges.len();
                self.edges.reserve_exact(add);
                self.edges.resize(need, 0);
            }
            self.bands.clear();
            if n >= 65_536 && self.bands.capacity() < 8192 {
                self.bands.reserve_exact(8192);
            }
            self.spelling.clear();
            if n >= 65_536 && self.spelling.capacity() < 48 * 1024 {
                self.spelling.reserve_exact(48 * 1024);
            }
            self.offsets.clear();
            let cursors = self.induced(ns, n, &thin, &first, &mut write, &mut stats);
            stats.bands = self.bands.len();
            let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
            let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m as usize {
                self.edges.resize(m as usize, 0);
            }
            self.extra.clear();
            if wide {
                self.extra.resize(m as usize, 0);
            }
            stats.table_bytes = 4 * (m as usize + self.extra.len());
            self.predecessor = None;
            let use_bi = !wide && bi_mode.should_use_bi(m, ns as u32, nrec);
            stats.used_bi = use_bi;
            if use_bi {
                let edges = &mut self.edges[..m as usize];
                edges.fill(0);
                let mut predecessor = mark;
                let mb = usize::from(mark_byte);
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
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
                self.predecessor = Some(predecessor);
            } else {
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let span = b.end() as usize;
                        let dst = b.dst() as usize;
                        let bcode = b.code();
                        let pack = if wide { 0 } else { bcode << bits };
                        if wide {
                            self.extra[dst..dst + span].fill(bcode);
                        }
                        queue.wire(&mut self.edges[dst..dst + span], pack);
                    }
                }
            }
            (stats, mark, bits)
        }

        fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
            if let Some(before) = self.predecessor {
                emit_bidirectional(
                    &self.edges[..m],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    before,
                    bits,
                    out,
                );
                return;
            }
            emit(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                out,
            );
        }

        #[inline(never)]
        pub fn try_run_const<const LIM: u32>(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            cap: Cap,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> Option<Counts> {
            if n == 0 || n > 900_000 || (p as usize) >= n {
                return None;
            }
            let (stats, mark, bits) =
                self.construct_capped::<LIM>(records, n as u32, p, cap, bi_mode, out)?;
            self.walk(n, stats.retained, mark, bits, out);
            Some(stats)
        }
    }

    mod endpoint {
        // Eliminate the open complement with an auxiliary interval exchange.
        // Its prefix contains terminal coordinates only. All narrow-run wiring is absent.
        //
        // **Where its state lives** (the Wormhole line's memory diet, W1,
        // `docs/journal/2026-11-21-wormhole-cycle5-pigeonhole-chaitin-synthesis.md` on
        // `research/carbon-wormhole`): the elimination's nodes and rules are dead before the kernel's
        // `edges` is first written, so they are kept in `edges`' own buffer, as `u32` words, and
        // `edges` is then written over them. A node is four words when every node id fits in 16 bits
        // (`Narrow`: a side's `prev` and `next` links in one word, then its width and
        // word), six otherwise (`Wide`). The two lists are linked as the nodes are made, with no
        // staging lists. After the elimination a node's side-1 links, read once more by the walk
        // along the bottom, give way to its bottom coordinate.
        use super::{begin_inline, Band, Counts, Kernel, Strip, INLINE, LONG};

        /// A node's links in the arena
        trait Links {
            /// `u32` words a node takes
            const STRIDE: usize;
            const IS_NARROW: bool;
            /// The word that holds a node's bottom coordinate once the elimination is done (one of
            /// its side-1 links, read before it is written)
            const BOTTOM: usize;
            fn prev(a: &[u32], id: u32, side: usize) -> u32;
            fn next(a: &[u32], id: u32, side: usize) -> u32;
            fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
            fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
            fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);
        }

        /// Every node id below 2^16: four words a node, a word per side (`prev` in its low half,
        /// `next` in its high half), then the width and the word
        struct Narrow;
        /// Any node id: six words a node, `prev` and `next` per side, then the width and the word
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

        /// The node count up to which `Narrow` links serve: ids 0..=65535
        const NARROW_NODES: usize = 1 << 16;

        /// Four arena words hold either 15 payload bytes plus a length byte, or
        /// two child ids, their combined length and a tagged spelling offset. An
        /// unexpanded long rule has the sentinel u32::MAX in the offset field.
        const RULE: usize = 4;

        // Only live node handles carry a length. Rule children remain plain ids. Each
        // ordinary step consumes at least one row and makes at most one rule. A spiral
        // of q steps consumes at least q rows and makes at most 2q rules by squaring,
        // so every id is at most 256 + 2 * 900_000 and fits in the low 22 bits.
        const ID_BITS: u32 = 22;
        const ID_MASK: u32 = (1 << ID_BITS) - 1;
        const SATURATED: u32 = u32::MAX >> ID_BITS;

        #[inline(always)]
        fn literal(byte: u8) -> u32 {
            (1 << ID_BITS) | u32::from(byte)
        }

        /// The width certificate's line (the Mechanisms line's cycle 19, Theorem 19.2a,
        /// `docs/journal/2026-11-21-carbon-cycle19-unified-synthesis.md` on
        /// `research/carbon-word-transport`): no step of the elimination widens a node, so a run of
        /// steps that peels one node against the same partner (a spiral) takes fewer steps than the
        /// widest node is wide. A block whose nodes are all at most this wide runs the plain loop;
        /// a wider one runs the loop that contracts each spiral into one step (`eliminate::<true>`).
        pub(super) const CERTIFIED_WIDTH: u32 = 32_768;
        /// The rules one contracted spiral may make: a power by squaring (two a bit of the count,
        /// which is below 2^20) and the join after it
        const SPIRAL_RULES: usize = 2 * 20 + 1;

        /// The elimination's scratch kept outside `edges`: the stack of `append`
        #[derive(Default)]
        pub(super) struct Endpoint {
            stack: Vec<u32>,
            /// (test only) make every elimination take `Wide` links
            #[cfg(test)]
            pub(super) force_wide: bool,
            /// (test only) start every elimination with no room for a rule, so that the arena grows
            #[cfg(test)]
            pub(super) tight: bool,
            /// (test only) make every elimination contract its spirals (`Some(true)`) or run the
            /// plain loop (`Some(false)`), whatever its widths
            #[cfg(test)]
            pub(super) spirals: Option<bool>,
            /// (test only) poison a region of `edges` at a point of the construction (`Clobber`)
            #[cfg(test)]
            pub(super) clobber: Clobber,
        }

        /// (test only) where the lifetime tests poison `edges` (`mux.rs`): after each region's last
        /// read, which the output must not see, or (the controls) before it, which it must
        #[cfg(test)]
        #[derive(Default, Clone, Copy, PartialEq, Debug)]
        pub(super) enum Clobber {
            #[default]
            None,
            /// the image order, once the node lists are made (dead)
            OrderAfterNodes,
            /// the image order, once written and before the node lists are made (live: a control)
            OrderBeforeNodes,
            /// all of `edges`, once the bands are made and before the wiring (dead)
            AllBeforeWire,
            /// the arena, once the elimination is done and before the bands are read off it (live:
            /// a control)
            ArenaBeforeBands,
        }

        impl Endpoint {
            /// (test only) the scratch it keeps, in bytes (its nodes and rules are in `edges`)
            #[cfg(test)]
            pub(super) fn scratch(&self) -> usize {
                4 * self.stack.capacity()
            }
        }

        /// The elimination over an arena: `STRIDE` words per node from 0, then `RULE` words per rule
        /// from `base` to `top`. The arena is `edges`' buffer, whose length only grows, so neither the
        /// nodes nor the edges after them are filled first. (A slice, not the `Vec`: its pointer and
        /// length stay in registers across the stores.)
        struct Elim<'a, L: Links> {
            a: &'a mut [u32],
            base: usize,
            top: usize,
            nodes: u32,
            _links: std::marker::PhantomData<L>,
        }

        /// The arena to `need` words at least: doubled
        #[cold]
        #[inline(never)]
        fn grow(a: &mut Vec<u32>, need: usize) {
            a.resize(need.max(2 * a.len()), 0);
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
            /// A node's bottom coordinate, once the elimination is done (in a side-1 link's word)
            #[inline(always)]
            fn bottom(&self, id: u32) -> u32 {
                self.a[id as usize * L::STRIDE + L::BOTTOM]
            }
            #[inline(always)]
            fn set_bottom(&mut self, id: u32, x: u32) {
                self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
            }

            /// Assign exclusive prefix sums from both ends of side 1. Narrow nodes
            /// reuse both links for the coordinate, so read both links before writing.
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

            /// A new node, its links unset
            fn node(&mut self, width: u32, word: u32) -> u32 {
                let id = self.nodes;
                self.nodes += 1;
                self.set_width(id, width);
                self.set_word(id, word);
                id
            }

            /// Append `id` to a list on `side` whose last node is `*tail` (0: the list is empty, and
            /// `*head` is set)
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

            /// The circular list on `side` through the sentinel: `first`'s nodes, then `second`'s
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

            /// The successor of a tail is the sentinel on this side.
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
                self.base + (id - 65_793) as usize * RULE
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

            /// Low 120 bits in spelling order for any non-empty handle with `grade < 16`.
            /// Both `grade == 1` (literal) and `grade == 2` (inline bigram) decode in 0 memory loads!
            #[inline(always)]
            fn short_payload_h(&self, h: u32) -> u128 {
                let g = h >> ID_BITS;
                let id = h & ID_MASK;
                if g <= 2 {
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
                    if sum == 2 {
                        return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
                    }
                    let id = ((self.top - self.base) / RULE) as u32 + 65_793;
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
                    // Since left_len, right_len >= 1, `sum <= SATURATED` implies BOTH `< SATURATED`!
                    let len = if sum <= SATURATED {
                        sum
                    } else {
                        self.len_handle(left) + self.len_handle(right)
                    };
                    let id = ((self.top - self.base) / RULE) as u32 + 65_793;
                    let out = &mut self.a[self.top..self.top + RULE];
                    out[0] = left;
                    out[1] = right;
                    out[2] = len;
                    out[3] = u32::MAX;
                    self.top += RULE;
                    (len.min(SATURATED) << ID_BITS) | id
                }
            }

            /// `word` repeated `q` times (`q > 0`), by squaring: at most `SPIRAL_RULES - 1` rules
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

            /// The elimination's steps until `remaining` is 0 (true), or until the arena has no room
            /// for a rule (false: it is grown, and the elimination resumed). With `SPIRALS` (the
            /// blocks the width certificate leaves out), a spiral's steps are taken as one:
            /// - the side-0 node `x` wider than the side-1 node `y` that follows it on side 1: each
            ///   step leaves both lists as they were, prepends `x`'s word to `y`'s and takes `y`'s
            ///   width off `x`'s, while `x` stays the wider. Its `q = (aw - 1) / bw` steps give `y`
            ///   the word `x^q y` and `x` the width `aw - q bw`.
            /// - `x` narrower than `y`, which `x` follows on side 0: each step appends `y`'s word to
            ///   `x`'s and takes `x`'s width off `y`'s, for `q = (bw - 1) / aw` steps: `x y^q`, and
            ///   `y` the width `bw - q aw`.
            ///
            /// The lists, the widths and the words' spellings are exactly the plain loop's (its rules
            /// are fewer); the steps are counted as the plain loop's.
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
                            // aw == bw: insert x after y on side 0, cut y from side 0, pop y from tail of side 1
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

            fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
                if !SPIRALS && L::IS_NARROW {
                    return self.eliminate_narrow_plain(remaining);
                }
                let (mut left, mut taken) = (*remaining, *steps);
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
                            taken += q as usize;
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
                            taken += q as usize;
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
                (*remaining, *steps) = (left, taken);
                done
            }

            /// An excursion followed by its retained byte. Only long words occupy
            /// spelling, so memoized offsets always refer to permanent bytes.
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
                    self.append(word, stack, spelling);
                    spelling.push(byte);
                    (super::long_slot(start, spelling.len() as u32), LONG)
                }
            }

            fn append(&mut self, handle: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
                stack.clear();
                stack.push(handle);
                while let Some(h) = stack.pop() {
                    let g = h >> ID_BITS;
                    if g < INLINE as u32 {
                        let bytes = self.short_payload_h(h).to_le_bytes();
                        dest.extend_from_slice(&bytes[..g as usize]);
                    } else {
                        let r = self.rule(h & ID_MASK);
                        let tag = self.a[r + 3];
                        let rule = &mut self.a[r..r + RULE];
                        if tag == u32::MAX {
                            rule[3] = 0x8000_0000 | dest.len() as u32;
                            stack.push(rule[1]);
                            stack.push(rule[0]);
                        } else {
                            let start = (tag & 0x7fff_ffff) as usize;
                            dest.extend_from_within(start..start + rule[2] as usize);
                        }
                    }
                }
            }
        }

        /// Two direct-mapped tables in the dead edge arena. The first compares both
        /// the rule id and the retained byte. The second compares the word itself.
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
                let g = handle >> ID_BITS;
                let id = handle & ID_MASK;
                let uid = id + 1 + (u32::from(g == 2) << 8);
                let key = (uid << 8) | u32::from(byte);
                let bucket = (key.wrapping_mul(0x9e37_79b9) >> 24) as usize;
                let slot = bucket * 2;
                if self.pairs[slot] == key {
                    return self.pairs[slot + 1];
                }
                self.code_miss(e, handle, id, byte, key, slot)
            }

            fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
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
                    let code = if (ent & 0xFF00_0000) == tag && {
                        let old = (ent & 0x00FF_FFFF) as usize;
                        self.lens[old] == len && self.inline16[old] == entry
                    } {
                        ent & 0x00FF_FFFF
                    } else {
                        let c = self.insert(entry, len);
                        self.contents[index] = tag | c;
                        c
                    };
                    self.pairs[slot] = key;
                    self.pairs[slot + 1] = code;
                    return code;
                }
                let start = self.spelling.len();
                let memoized = e.a[e.rule(id) + 3] != u32::MAX;
                e.append(handle, self.stack, self.spelling);
                self.spelling.push(byte);
                let entry = super::long_slot(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                let code = if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.lens[old] == LONG && super::long_word(&self.inline16[old], self.spelling) == bytes
                } {
                    if memoized {
                        self.spelling.truncate(start);
                    } else {
                        self.spelling.pop();
                    }
                    ent & 0x00FF_FFFF
                } else {
                    let c = self.insert(entry, LONG);
                    self.contents[index] = tag | c;
                    c
                };
                self.pairs[slot] = key;
                self.pairs[slot + 1] = code;
                code
            }

            #[inline(always)]
            fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
                let code = self.lens.len() as u32;
                self.inline16.push(entry);
                self.lens.push(len);
                code
            }
        }

        fn spelling_hash_long(bytes: &[u8]) -> u64 {
            let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ part(bytes.len() - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
                ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^ (h >> 30)
        }

        impl Kernel {
            /// The most nodes `ns` strips make: a node per strip, at most one gap before each strip
            /// and one after the last, on each side, and the sentinel
            fn nodes_most(ns: usize) -> usize {
                3 * ns + 3
            }

            /// Whether the elimination of `ns` strips takes `Wide` links
            fn wide(&self, ns: usize) -> bool {
                #[cfg(test)]
                let wide = Self::nodes_most(ns) > NARROW_NODES || self.endpoint.force_wide;
                #[cfg(not(test))]
                let wide = Self::nodes_most(ns) > NARROW_NODES;
                wide
            }

            /// The words `edges` is sized to for the induced construction of `ns` strips and `m`
            /// retained rows: packed strips and image order (`4 * ns` words), then the arena for the
            /// nodes and as many rules (the blocks make about one a node; more grow it); and at least
            /// the `m` edges written over them all
            pub(super) fn arena_words(&self, ns: usize, m: usize) -> usize {
                let stride = if self.wide(ns) {
                    Wide::STRIDE
                } else {
                    Narrow::STRIDE
                };
                (4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
            }

            pub(super) fn induced(
                &mut self,
                ns: usize,
                n: u32,
                thin: &[u32; 256],
                first: &[u32; 256],
                write: &mut [u32; 256],
                stats: &mut Counts,
            ) -> [usize; 256] {
                let most = Self::nodes_most(ns);
                if self.wide(ns) {
                    self.induced_with::<Wide>(ns, n, thin, first, write, stats, most)
                } else {
                    self.induced_with::<Narrow>(ns, n, thin, first, write, stats, most)
                }
            }

            #[allow(clippy::too_many_arguments)]
            fn induced_with<L: Links>(
                &mut self,
                ns: usize,
                n: u32,
                thin: &[u32; 256],
                first: &[u32; 256],
                write: &mut [u32; 256],
                stats: &mut Counts,
                most: usize,
            ) -> [usize; 256] {
                let Kernel {
                    endpoint,
                    bands,
                    spelling,
                    edges,
                    inline16,
                    lens,
                    ..
                } = self;
                let nodes_words = L::STRIDE * most;
                assert!(edges.len() >= 4 * ns + nodes_words);
                #[cfg(test)]
                if endpoint.tight {
                    edges.truncate(4 * ns + nodes_words);
                }
                let (hdr, arena) = edges.split_at_mut(4 * ns);
                let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
                #[cfg(test)]
                if endpoint.clobber == Clobber::OrderBeforeNodes {
                    image_order.fill(0xA5A5_A5A5);
                }
                let mut e = Elim::<L> {
                    a: arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                e.node(0, 0);
                e.nodes = ns as u32 + 1;
                let mut widest = 0u32;
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
                    let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
                    if x < s.to {
                        widest = widest.max(s.to - x);
                        let id = e.node(s.to - x, 0);
                        e.append_to(&mut order[1], id, 1);
                        e.append_to(&mut terminals[0], id, 0);
                    }
                    e.append_to(&mut order[1], i + 1, 1);
                    x = s.to + s.width;
                }
                if x < n {
                    widest = widest.max(n - x);
                    let id = e.node(n - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                // the image order is dead from here: the arena may be grown over it, and `edges` is
                // written over both
                #[cfg(test)]
                if endpoint.clobber == Clobber::OrderAfterNodes {
                    image_order.fill(0xA5A5_A5A5);
                }
                stats.open_nodes = e.nodes as usize - 1;
                for side in 0..2 {
                    e.close(side, terminals[side], order[side]);
                }
                // the rules follow the nodes made
                e.base = L::STRIDE * e.nodes as usize;
                e.top = e.base;
                let (mut remaining, mut steps) = (n, 0);
                #[cfg(test)]
                let spirals = endpoint.spirals.unwrap_or(widest > CERTIFIED_WIDTH);
                #[cfg(not(test))]
                let spirals = widest > CERTIFIED_WIDTH;
                stats.spirals = spirals;
                while !if spirals {
                    e.eliminate::<true>(&mut remaining, &mut steps)
                } else {
                    e.eliminate::<false>(&mut remaining, &mut steps)
                } {
                    let (base, top, nodes) = (e.base, e.top, e.nodes);
                    grow(edges, 4 * ns + top + RULE);
                    e = Elim {
                        a: &mut edges[4 * ns..],
                        base,
                        top,
                        nodes,
                        _links: std::marker::PhantomData,
                    };
                }
                stats.steps = steps;
                stats.rules = (e.top - e.base) / RULE;
                #[cfg(test)]
                if endpoint.clobber == Clobber::ArenaBeforeBands {
                    e.a[..e.top].fill(0xA5A5_A5A5);
                }

                e.assign_bottom(stats.retained as u32);
                // After elimination the strip/order prefix and unused arena tail are
                // available for two direct-mapped caches. Small blocks need neither.
                const PAIR_WORDS: usize = 512;
                const CACHE_WORDS: usize = PAIR_WORDS + 2048;
                let (base, top, nodes) = (e.base, e.top, e.nodes);
                let (header, arena) = edges.split_at_mut(4 * ns);
                let (live, tail) = arena.split_at_mut(top);
                let cache = if header.len() >= CACHE_WORDS {
                    header
                } else {
                    tail
                };
                let mut e = Elim::<L> {
                    a: live,
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
                begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
                if cache.len() < CACHE_WORDS {
                    let mut cursors = [0; 256];
                    let mut id = L::next(e.a, 0, 0);
                    let mut in_node = 0;
                    let mut k = 0;
                    for a in 0..256 {
                        cursors[a] = bands.len();
                        let end = k + thin[a];
                        while k < end {
                            let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else {
                                let (entry, len) = e.inline_word(node_word, a as u8, &mut endpoint.stack, spelling);
                                let c = lens.len() as u32;
                                inline16.push(entry);
                                lens.push(len);
                                c
                            };
                            bands.push(Band::new(width, dst, code));
                            in_node += width;
                            k += width;
                            if in_node == node_width {
                                id = next_id;
                                in_node = 0;
                            }
                        }
                    }
                    assert_eq!(id, 0);
                    return cursors;
                }
                cache[..CACHE_WORDS].fill(0);
                let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
                let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();
                let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
                let mut dictionary = Dictionary {
                    pairs,
                    contents,
                    inline16,
                    lens,
                    spelling,
                    stack: &mut endpoint.stack,
                };
                let mut cursors = [0; 256];
                let mut id = L::next(e.a, 0, 0);
                let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
                    L::band_node(e.a, id)
                } else {
                    (0, 0, 0, 0)
                };
                let mut in_node = 0u32;
                let mut k = 0u32;
                for a in 0..256 {
                    cursors[a] = bands.len();
                    let end = k + thin[a];
                    let mut last_w = u32::MAX;
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
                        bands.push(Band::new(width, dst, code));
                        in_node += width;
                        k += width;
                        if in_node == node_width {
                            id = next_id;
                            in_node = 0;
                            if id != 0 {
                                (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                cursors
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn dictionary_collisions_compare_complete_spellings() {
                let mut arena = [0; RULE * 64];
                let mut elim = Elim::<Narrow> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                let (mut inline, mut lens) = (vec![], vec![]);
                begin_inline(0, &mut inline, &mut lens);
                let mut dict = Dictionary {
                    pairs: &mut [0; 2048],
                    contents: &mut [0; 1],
                    inline16: &mut inline,
                    lens: &mut lens,
                    spelling: &mut vec![],
                    stack: &mut vec![],
                };
                for byte in 0..=255 {
                    for first in [1, 241] {
                        let handle = literal(first);
                        let code = dict.code(&mut elim, handle, byte);
                        assert_eq!(dict.code(&mut elim, handle, byte), code);
                        assert_eq!(dict.inline16[code as usize][..2], [first, byte]);
                    }
                }
                let left = elim.power(literal(7), 16);
                let right = elim.power(literal(8), 16);
                let c1 = dict.code(&mut elim, left, 9) as usize;
                let c2 = dict.code(&mut elim, right, 9) as usize;
                assert_ne!(c1, c2);
                assert_eq!(
                    super::super::long_word(&dict.inline16[c1], dict.spelling),
                    [&[7; 16][..], &[9]].concat()
                );
            }

            #[test]
            fn short_rules_keep_all_fifteen_high_bytes() {
                let mut arena = [0; 32 * RULE];
                let mut e = Elim::<Narrow> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                let mut word = 0;
                for b in 241..=255 {
                    word = e.join(literal(b), word);
                }
                let short = e.short_payload(word & ID_MASK).to_le_bytes();
                assert!(short[..15].iter().copied().eq((241..=255).rev()));
                let long = e.join(word, word);
                let mut bytes = Vec::new();
                for _ in 0..2 {
                    e.append(long & ID_MASK, &mut vec![], &mut bytes);
                }
                assert_eq!(bytes, short[..15].repeat(4));
            }

            #[test]
            fn bottom_coordinates_at_odd_and_even_meetings() {
                fn check<L: Links>() {
                    for count in 0..=7u32 {
                        let mut arena = [0; 64];
                        let mut e = Elim::<L> {
                            a: &mut arena,
                            base: 0,
                            top: 0,
                            nodes: 0,
                            _links: std::marker::PhantomData,
                        };
                        e.node(0, 0);
                        let mut list = (0, 0);
                        for width in 1..=count {
                            let id = e.node(width, 0);
                            e.append_to(&mut list, id, 1);
                        }
                        e.close(1, list, (0, 0));
                        e.assign_bottom(count * (count + 1) / 2);
                        for id in 1..=count {
                            assert_eq!(e.bottom(id), (id - 1) * id / 2);
                        }
                    }
                }
                check::<Narrow>();
                check::<Wide>();
            }

            #[test]
            fn graded_word_lengths_across_saturation() {
                let mut arena = [0; RULE * 40];
                let mut e = Elim::<Narrow> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                let run = e.power(literal(255), SATURATED - 1);
                let mut handle = e.join(0, run);
                let mut spelling = Vec::new();
                for len in SATURATED - 1..=SATURATED + 2 {
                    assert_eq!(handle >> ID_BITS, len.min(SATURATED));
                    assert_eq!(e.len(handle & ID_MASK), len);
                    // Memo offsets refer to this same growing spelling buffer.
                    let start = spelling.len();
                    e.append(handle & ID_MASK, &mut Vec::new(), &mut spelling);
                    assert_eq!(
                        &spelling[start..start + (SATURATED - 1) as usize],
                        &[255; 1022]
                    );
                    assert!(spelling[start + 1022..].iter().all(|&b| b == 0));
                    handle = e.join(handle, literal(0));
                }
            }

            #[test]
            fn prefix15_crosses_into_memoized_spelling() {
                let mut arena = [0; RULE * 40];
                let mut e = Elim::<Narrow> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                let (mut stack, mut spelling) = (Vec::new(), Vec::new());
                let mut handle = 0;
                for n in 1..=17u8 {
                    handle = e.join(handle, literal(n));
                    let start = spelling.len();
                    let (inline, len) = e.inline_word(handle, 255, &mut stack, &mut spelling);
                    let want: Vec<_> = (1..=n).chain([255]).collect();
                    if n < 16 {
                        assert_eq!(&inline[..usize::from(len)], want);
                        assert_eq!(spelling.len(), 0);
                    } else {
                        assert_eq!(len, LONG);
                        assert_eq!(spelling[start..], want);
                    }
                }
            }

            #[test]
            fn popping_a_tail_relinks_only_its_side() {
                let mut arena = [0; 3 * Narrow::STRIDE];
                let mut e = Elim::<Narrow> {
                    a: &mut arena,
                    base: 0,
                    top: 0,
                    nodes: 0,
                    _links: std::marker::PhantomData,
                };
                for side in 0..2 {
                    let mut list = (0, 0);
                    e.append_to(&mut list, 1, side);
                    e.append_to(&mut list, 2, side);
                    e.close(side, list, (0, 0));
                }
                e.pop_tail(2, 1);
                assert_eq!(Narrow::next(e.a, 1, 1), 0);
                assert_eq!(Narrow::prev(e.a, 0, 1), 1);
                assert_eq!(Narrow::next(e.a, 1, 0), 2);
                assert_eq!(Narrow::prev(e.a, 0, 0), 2);
            }
        }

        impl Endpoint { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.stack.capacity() } }
    }

    mod outq {
        // The thin queues in the block's output buffer (the Mechanisms line's `OutQ16`,
        // `docs/journal/2026-11-21-carbon-cycle16-swar-outq16.md` on `research/carbon-word-transport`).
        //
        // The first pass queues each byte's retained ranks; the wiring reads them; only then does the
        // walk write the block's bytes into `out`. So the queues live in `out` itself, which the walk
        // writes over: no buffer of their own, and the cache lines they use are the ones the walk is
        // about to write. (The line's ablation: the same chunks in a buffer of their own lost the
        // gain, so the reuse of `out`'s lines, not the chunking, is what it buys.)
        //
        // **The layout:** `out` (its `n + 16` bytes) is cut into chunks of `CHUNK` ranks, two bytes a
        // rank. Each byte's queue is a list of chunks, taken in turn as the queues fill, linked by
        // `Spill::next`; a chunk past the end of `out` is an overflow chunk in `Spill::ovf` (none on a
        // full block of up to about 318,000 retained rows; every chunk on a block under 1 KiB).
        //
        // **The ranks in 16 bits:** a queue keeps a rank's low 16 bits. Its high bits, the rank's
        // epoch, are recovered from `Spill::cuts`: when the retained rows reach a multiple of 2^16
        // (a run is split there), every byte's queue length is recorded, so the entries of byte `a`'s
        // queue before `cuts[e][a]` are of epochs up to `e`, and the queue's entries are read in order.
        //
        // **Safety:** no `unsafe`; every index is bounds-checked.

        /// Ranks a chunk holds
        const CHUNK: usize = 512;
        /// A chunk's bytes
        const BYTES: usize = 2 * CHUNK;
        /// No chunk
        const NONE: u32 = u32::MAX;
        /// Ranks an epoch holds
        const EPOCH: u32 = 1 << 16;

        /// The queues' state kept between blocks: the chunks' links, the overflow chunks and the
        /// epochs' cuts (on the admitted blocks, a few KiB in all)
        #[derive(Default)]
        pub(super) struct Spill {
            next: Vec<u32>,
            ovf: Vec<[u8; BYTES]>,
            cuts: Vec<[u32; 256]>,
            /// (test only) the chunks the last block's first pass took
            #[cfg(test)]
            pub(super) taken: u32,
        }

        impl Spill {
            /// (test only) the scratch it keeps, in bytes
            #[cfg(test)]
            pub(super) fn scratch(&self) -> usize {
                4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity()
            }

            /// (test only) the overflow chunks the last block took
            #[cfg(test)]
            pub(super) fn overflow(&self) -> usize {
                self.ovf.len()
            }
        }

        /// The queues while the first pass writes them
        pub(super) struct Queues<'a> {
            out: &'a mut [u8],
            spill: &'a mut Spill,
            in_out: u32,
            chunks: u32,
            heads: [u32; 256],
            /// Packed `(thin[a] << 32) | (tail << 10) | fill` (`u32::MAX` in low 32 before first chunk)
            pt: [u64; 256],
            limit: u32,
            over: bool,
        }

        /// Where the queues are, once written: each byte's first chunk
        pub(super) struct Heads {
            heads: [u32; 256],
            in_out: u32,
        }

        impl<'a> Queues<'a> {
            /// Empty queues in `out`, whose bytes are free until the walk, for at most `limit` ranks
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

            /// Whether a push went past the queues' limit (the block is refused)
            pub(super) fn over(&self) -> bool {
                self.over
            }

            /// (test only) the chunks taken
            #[cfg(test)]
            pub(super) fn chunks(&self) -> u32 {
                self.chunks
            }

            pub(super) fn heads(&self) -> Heads {
                Heads {
                    heads: self.heads,
                    in_out: self.in_out,
                }
            }

            /// Queue the `w` ranks from `m` for byte `a`, whose queue length (before them) is
            /// `thin[a]`. A thin run has at most 16 rows, so the common case is one fixed block of 16
            /// ranks written (an iota from `m`), its first `w` kept: within the byte's last chunk, in
            /// `out`, and within an epoch. Anything else takes `push_slow`.
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
                        let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
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
                        self.spill.cuts.push(cut);
                    }
                }
            }

            fn take_chunk_pt(&mut self, a: usize) {
                let id = self.chunks;
                self.chunks += 1;
                if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill.ovf.push([0; BYTES]);
                }
                if self.spill.next.len() <= id as usize {
                    self.spill.next.resize(id as usize + 1, NONE);
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

            /// `push` across a chunk's end or an epoch's, into an overflow chunk, or of more than 16
            /// ranks (a limit above 16, in the tests): rank by rank. Past the queues' `limit` it sets
            /// `over` and writes nothing: every chunk is taken here, so the queues take none once the
            /// block is refused, and `push` itself pays nothing for the limit (only a chunk's end
            /// brings it here, and its fast case writes within a chunk already taken).
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

        /// A byte's queue read in order: its chunk, the rank within it, the ranks read, and the
        /// epoch with the queue length at which it ends
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
            /// Byte `a`'s queue, from its first rank
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

            /// Each arrow from row u to v contributes v to u's low bits and u to
            /// v's low bits by XOR. The high bits of u hold its word code. Save the
            /// incoming neighbor of the mark to start the walk in either direction.

            #[inline(always)]
            pub(super) fn wire_xor_nomark(
                &mut self,
                edges: &mut [u32],
                dst_start: usize,
                span: usize,
                pack: u32,
            ) {
                if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
                if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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

            /// The next `span` ranks, each with `pack` added, into `dst`
            #[inline(always)]
            pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {
                let span = dst.len();
                if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
                    // the common case: one chunk of `out`, one epoch
                    let hi = pack | (self.epoch << 16);
                    let at = self.id as usize * BYTES + 2 * self.at;
                    let bytes = &self.out[at..at + 2 * span];
                    let mut i = 0usize;
                    while i + 16 <= span {
                        let b = &bytes[2 * i..2 * i + 32];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                        let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                        let d = &mut dst[i..i + 16];
                        d[0] = hi | (q0 as u32 & 0xFFFF);  d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF); d[3] = hi | ((q0 >> 48) as u32);
                        d[4] = hi | (q1 as u32 & 0xFFFF);  d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF); d[7] = hi | ((q1 >> 48) as u32);
                        d[8] = hi | (q2 as u32 & 0xFFFF);  d[9] = hi | ((q2 >> 16) as u32 & 0xFFFF);
                        d[10] = hi | ((q2 >> 32) as u32 & 0xFFFF); d[11] = hi | ((q2 >> 48) as u32);
                        d[12] = hi | (q3 as u32 & 0xFFFF); d[13] = hi | ((q3 >> 16) as u32 & 0xFFFF);
                        d[14] = hi | ((q3 >> 32) as u32 & 0xFFFF); d[15] = hi | ((q3 >> 48) as u32);
                        i += 16;
                    }
                    if i + 8 <= span {
                        let b = &bytes[2 * i..2 * i + 16];
                        let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                        let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                        let d = &mut dst[i..i + 8];
                        d[0] = hi | (q0 as u32 & 0xFFFF);
                        d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                        d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF);
                        d[3] = hi | ((q0 >> 48) as u32);
                        d[4] = hi | (q1 as u32 & 0xFFFF);
                        d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                        d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF);
                        d[7] = hi | ((q1 >> 48) as u32);
                        i += 8;
                    }
                    while i < span {
                        dst[i] = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                        i += 1;
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

        impl Spill { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity() } }
    }
}
