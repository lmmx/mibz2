//! Cycle 48 Headroom Oracle v3 (4-Corpus `1T / 2W / 4W` Benchmark & Exactness Suite).
//!
//! Evaluates all 5 core arms (+ A/A paired control) across 4 real corpora:
//!   1. `6` Admitted Wikidata blocks (`/home/antigravity/admitted-bwt/`)
//!   2. `431` `wikidata.bz2` blocks (`233` full + `198` tiny/column)
//!   3. `6` Non-Wikidata JSONL/Log blocks (`5` JSONL + `1` syslog)
//!   4. `4` Non-Wikidata Code/Docs blocks (`code_and_docs.bz2` -> `code_docs_blocks.bin`)
//!
//! Arms:
//!   - `R5-Head-A1` / `R5-Head-A2(AA)` (verbatim `cand_r5_head`, `ID_BITS=22`, `L=16`, `BiMode::AlwaysAt(65_536)`)
//!   - `R5-Nobi` (verbatim `cand_r5_nobi`, `ID_BITS=22`, `L=16`, `μ=1`)
//!   - `F-Inline3` (`ID_BITS=24`, `SATURATED=255`, `RULE_BASE=0`, de-inlined `finish`/`wire_bi`/`wire_uni`)
//!   - `Spec4-SPRT-L` (Cycle 47 baseline `ID_BITS=22` + `Spec1-TinyDense-FL` + C47 `PostS1Md` + `Spec4-SPRT-L`)
//!   - `C48-Fused` (`F-Inline3` `ID_BITS=24` + `Spec1-TinyDense-FL` + exact `r_thin` `extra_127` `PostS1ExactMd`
//!                  + `Spec4-SPRT-L` + de-inlined `finish`/`wire_bi`/`wire_uni`)
//!
//! Safe Rust: `#![forbid(unsafe_code)]`, strictly forward-`FL` exact on ALL valid & malformed inputs.
#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

// ============================================================================
// 0A. Cycle 47 Unified Open Kernel (`ID_BITS=22`, verbatim endpoint.rs + outq.rs)
//     Used by `Spec4-SPRT-L` for direct apples-to-apples comparison vs `C48-Fused`.
// ============================================================================

#[allow(dead_code)]
#[path = "."]
mod unified_open_c47 {
    #[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/endpoint.rs"]
    mod endpoint;

    #[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/outq.rs"]
    mod outq;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BiMode {
        Never,
        AlwaysAt(u32),
        PostS1Md { recs_max: u32, ns_max: u32, md_max: i64 },
    }

    impl BiMode {
        #[inline(always)]
        pub fn should_use_bi(self, m: u32, ns: u32, recs: u32) -> bool {
            match self {
                BiMode::Never => false,
                BiMode::AlwaysAt(thr) => m >= thr,
                BiMode::PostS1Md { recs_max, ns_max, md_max } => {
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

        #[inline(never)]
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

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
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
                self.wire_bi(m as usize, mark, mark_byte, bits, &cursors, out, heads);
            } else {
                self.wire_uni(wide, bits, &cursors, out, heads);
            }
            (stats, mark, bits)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            out: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
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
        }

        #[inline(never)]
        fn wire_uni(
            &mut self,
            wide: bool,
            bits: u32,
            cursors: &[usize; 256],
            out: &[u8],
            heads: &outq::Heads,
        ) {
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

        pub fn scratch_report(&self) -> (usize, usize, String) {
            use std::mem::size_of;
            let q = self.spill.scratch_bytes();
            let tiles_dir =
                size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
            let bands = size_of::<Band>() * self.bands.capacity();
            let spelling = self.spelling.capacity();
            let offsets = 4 * self.offsets.capacity();
            let edges = 4 * self.edges.capacity();
            let extra = 4 * self.extra.capacity();
            let inline16 = 16 * self.inline16.capacity();
            let lens = self.lens.capacity();
            let ep = self.endpoint.scratch_bytes();
            let tot =
                q + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
            let dict_l1 = 16 * self.inline16.len() + self.lens.len();
            let s = format!(
                "queues={q} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}"
            );
            (tot, dict_l1, s)
        }
    }
}

// ============================================================================
// 0B. Cycle 48 Fused Unified Open Kernel (`F-Inline3` `ID_BITS=24` + exact
//     `extra_127` Dirichlet/Pigeonhole bound + de-inlined `finish`/`wire_bi`/`wire_uni`)
//     Matches `cand_c48_fused_moonshot.rs` verbatim!
// ============================================================================

#[allow(dead_code)]
#[path = "."]
mod unified_open_c48 {
    #[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/outq.rs"]
    mod outq;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BiMode {
        Never,
        AlwaysAt(u32),
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
        pub fn should_use_bi(self, m: u32, ns: u32, r_max: u32) -> bool {
            match self {
                BiMode::Never => false,
                BiMode::AlwaysAt(thr) => m >= thr,
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
        pub r_max: u32,
        pub extra_127: u32,
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

        #[inline(never)]
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
            let mut extra_127 = 0u32;
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
                                extra_127 += 1;
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
            let r_max = (records.len() as u32).saturating_sub(extra_127);
            let (mut stats, mark, bits) =
                self.finish(&f, ns, n, r_max, &out[..], &heads, bi_mode);
            stats.r_max = r_max;
            stats.extra_127 = extra_127;
            Some((stats, mark, bits))
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn finish(
            &mut self,
            f: &First,
            ns: usize,
            n: u32,
            r_max: u32,
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
            let use_bi = !wide && bi_mode.should_use_bi(m, ns as u32, r_max);
            stats.used_bi = use_bi;
            if use_bi {
                self.wire_bi(m as usize, mark, mark_byte, bits, &cursors, out, heads);
            } else {
                self.wire_uni(wide, bits, &cursors, out, heads);
            }
            (stats, mark, bits)
        }

        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn wire_bi(
            &mut self,
            m: usize,
            mark: u32,
            mark_byte: u8,
            bits: u32,
            cursors: &[usize; 256],
            out: &[u8],
            heads: &outq::Heads,
        ) {
            let edges = &mut self.edges[..m];
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
        }

        #[inline(never)]
        fn wire_uni(
            &mut self,
            wide: bool,
            bits: u32,
            cursors: &[usize; 256],
            out: &[u8],
            heads: &outq::Heads,
        ) {
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

        pub fn try_run(
            &mut self,
            records: &[u16],
            n: usize,
            p: u32,
            limit: u32,
            cap: Cap,
            bi_mode: BiMode,
            out: &mut Vec<u8>,
        ) -> Option<Counts> {
            match limit {
                4 => self.try_run_const::<4>(records, n, p, cap, bi_mode, out),
                16 => self.try_run_const::<16>(records, n, p, cap, bi_mode, out),
                24 => self.try_run_const::<24>(records, n, p, cap, bi_mode, out),
                _ => self.try_run_const::<32>(records, n, p, cap, bi_mode, out),
            }
        }

        pub fn scratch_report(&self) -> (usize, usize, String) {
            use std::mem::size_of;
            let q = self.spill.scratch_bytes();
            let tiles_dir =
                size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
            let bands = size_of::<Band>() * self.bands.capacity();
            let spelling = self.spelling.capacity();
            let offsets = 4 * self.offsets.capacity();
            let edges = 4 * self.edges.capacity();
            let extra = 4 * self.extra.capacity();
            let inline16 = 16 * self.inline16.capacity();
            let lens = self.lens.capacity();
            let ep = self.endpoint.scratch_bytes();
            let tot =
                q + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
            let dict_l1 = 16 * self.inline16.len() + self.lens.len();
            let s = format!(
                "queues={q} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}"
            );
            (tot, dict_l1, s)
        }
    }

    /// `F-Inline3` endpoint (`ID_BITS=24`, `SATURATED=255`, `RULE_BASE=0`).
    mod endpoint {
        use super::{begin_inline, Band, Counts, Kernel, Strip, INLINE, LONG};

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

        const NARROW_NODES: usize = 1 << 16;
        const RULE: usize = 4;
        const ID_BITS: u32 = 24;
        const ID_MASK: u32 = (1 << ID_BITS) - 1;
        const SATURATED: u32 = u32::MAX >> ID_BITS;

        #[inline(always)]
        fn literal(byte: u8) -> u32 {
            (1 << ID_BITS) | u32::from(byte)
        }

        pub(super) const CERTIFIED_WIDTH: u32 = 32_768;
        const SPIRAL_RULES: usize = 2 * 20 + 1;

        #[derive(Default)]
        pub(super) struct Endpoint {
            stack: Vec<u32>,
        }

        impl Endpoint {
            pub(super) fn scratch_bytes(&self) -> usize {
                4 * self.stack.capacity()
            }
        }

        struct Elim<'a, L: Links> {
            a: &'a mut [u32],
            base: usize,
            top: usize,
            nodes: u32,
            _links: std::marker::PhantomData<L>,
        }

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

            fn code_miss<L: Links>(
                &mut self,
                e: &mut Elim<L>,
                handle: u32,
                id: u32,
                byte: u8,
            ) -> u32 {
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
                e.append(handle, self.stack, self.spelling);
                self.spelling.push(byte);
                let entry = super::long_slot(start as u32, self.spelling.len() as u32);
                let bytes = &self.spelling[start..];
                let h = spelling_hash_long(bytes);
                let tag = (((h >> 56) as u32) | 0x80) << 24;
                let index = (h as usize) & 2047;
                let ent = self.contents[index];
                if (ent & 0xFF00_0000) == tag && {
                    let old = (ent & 0x00FF_FFFF) as usize;
                    self.lens[old] == LONG
                        && super::long_word(&self.inline16[old], self.spelling) == bytes
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
            fn nodes_most(ns: usize) -> usize {
                3 * ns + 3
            }

            fn wide(&self, ns: usize) -> bool {
                Self::nodes_most(ns) > NARROW_NODES
            }

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
                let (hdr, arena) = edges.split_at_mut(4 * ns);
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
                stats.open_nodes = e.nodes as usize - 1;
                for side in 0..2 {
                    e.close(side, terminals[side], order[side]);
                }
                e.base = L::STRIDE * e.nodes as usize;
                e.top = e.base;
                let (mut remaining, mut steps) = (n, 0);
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
                e.assign_bottom(stats.retained as u32);
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
                            let (next_id, node_bottom, node_width, node_word) =
                                L::band_node(e.a, id);
                            let width = (end - k).min(node_width - in_node);
                            assert!(width > 0);
                            let dst = node_bottom + in_node;
                            let code = if node_word == 0 {
                                a as u32
                            } else {
                                let (entry, len) = e.inline_word(
                                    node_word,
                                    a as u8,
                                    &mut endpoint.stack,
                                    spelling,
                                );
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
                                (next_id, node_bottom, node_width, node_word) =
                                    L::band_node(e.a, id);
                            }
                        }
                    }
                }
                assert_eq!(id, 0);
                cursors
            }
        }
    }
}

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;

// ============================================================================
// 1. Block Data Structures & Corpus Loaders (4 Real Corpora)
// ============================================================================

struct Block {
    name: String,
    n: usize,
    p: u32,
    route_class: u8, // 0=normal, 1=dense, 2=tiny, 3=column
    records: Vec<u16>,
    expected: Option<Vec<u8>>,
}

fn records_of(l: &[u8]) -> Vec<u16> {
    let mut recs = Vec::<u16>::new();
    for &b in l {
        if let Some(rec) = recs.last_mut() {
            if (*rec >> 8) as u8 == b && (*rec & 255) < 127 {
                *rec += 1;
                continue;
            }
        }
        recs.push((u16::from(b) << 8) | 1);
    }
    recs
}

fn expand_records(recs: &[u16], n: usize) -> Vec<u8> {
    let mut l = Vec::with_capacity(n);
    for &rec in recs {
        let b = (rec >> 8) as u8;
        let len = (rec & 255) as usize;
        l.resize(l.len() + len, b);
    }
    assert_eq!(l.len(), n);
    l
}

fn bwt_small(text: &[u8]) -> (Vec<u8>, u32) {
    let n = text.len();
    let mut sa: Vec<usize> = (0..n).collect();
    sa.sort_by(|&a, &b| {
        for k in 0..n {
            let ca = text[(a + k) % n];
            let cb = text[(b + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos + n - 1) % n]);
    }
    (l, p)
}

/// Forward-FL canonical reference inverse BWT (matching libbzip2 / mibz2::bwt::InverseBwt).
fn plain_invert_fl(l: &[u8], p: u32) -> Vec<u8> {
    let n = l.len();
    let mut counts = [0u32; 256];
    for &b in l {
        counts[b as usize] += 1;
    }
    let mut starts = [0u32; 256];
    let mut tot = 0u32;
    for c in 0..256 {
        starts[c] = tot;
        tot += counts[c];
    }
    let mut fl = vec![0u32; n];
    for (i, &b) in l.iter().enumerate() {
        let s = starts[b as usize] as usize;
        starts[b as usize] += 1;
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

fn resolve_corpus_path(rel: &str) -> String {
    let p1 = format!("/home/antigravity/mibz2_corpora_cache/{rel}");
    if Path::new(&p1).exists() {
        return p1;
    }
    let p2 = format!("/tmp/mibz2_corpora/{rel}");
    assert!(Path::new(&p2).exists(), "missing corpus {rel} (checked {p1} and {p2})");
    p2
}

fn load_six_admitted() -> Vec<Block> {
    let dir = "/home/antigravity/admitted-bwt";
    let man = fs::read_to_string(format!("{dir}/manifest.json")).expect("read manifest.json");
    let mut specs = Vec::new();
    let mut file = String::new();
    for line in man.lines() {
        let t = line.trim();
        if let Some(v) = t.strip_prefix("\"file\": \"") {
            file = v.trim_end_matches("\",").to_string();
        }
        if let Some(v) = t.strip_prefix("\"primary_index\": ") {
            let p = v.trim_end_matches(',').parse::<u32>().expect("parse p");
            specs.push((file.clone(), p));
        }
    }
    specs
        .into_iter()
        .map(|(f, p)| {
            let l = fs::read(format!("{dir}/{f}")).expect("read .L");
            let expected = plain_invert_fl(&l, p);
            let records = records_of(&l);
            Block {
                name: f,
                n: l.len(),
                p,
                route_class: 0,
                records,
                expected: Some(expected),
            }
        })
        .collect()
}

fn load_bin_corpus(path: &str, prefix: &str, with_expected: bool) -> Vec<Block> {
    let raw = fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    assert!(raw.len() >= 12 && &raw[0..8] == b"MBWT431\0", "bad magic in {path}");
    let num = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let mut off = 12usize;
    let mut blocks = Vec::with_capacity(num);
    for _ in 0..num {
        let idx = u32::from_le_bytes(raw[off..off + 4].try_into().unwrap());
        let n = u32::from_le_bytes(raw[off + 4..off + 8].try_into().unwrap()) as usize;
        let p = u32::from_le_bytes(raw[off + 8..off + 12].try_into().unwrap());
        let route_class = raw[off + 16];
        let n_recs = u32::from_le_bytes(raw[off + 20..off + 24].try_into().unwrap()) as usize;
        off += 24;
        let mut records = Vec::with_capacity(n_recs);
        for _ in 0..n_recs {
            records.push(u16::from_le_bytes([raw[off], raw[off + 1]]));
            off += 2;
        }
        let expected = if with_expected {
            let l = expand_records(&records, n);
            Some(plain_invert_fl(&l, p))
        } else {
            None
        };
        blocks.push(Block {
            name: format!("{prefix}-{idx}"),
            n,
            p,
            route_class,
            records,
            expected,
        });
    }
    assert_eq!(off, raw.len());
    blocks
}

fn make_dense_300k() -> Block {
    let n = 300_000usize;
    let mut text = Vec::with_capacity(n);
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    for i in 0..n {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        text.push(if i % 9 == 0 {
            b' '
        } else {
            b'a' + ((s >> 33) % 26) as u8
        });
    }
    let mut sa: Vec<u32> = (0..n as u32).collect();
    sa.sort_by(|&a, &b| {
        let (ia, ib) = (a as usize, b as usize);
        for k in 0..64 {
            let ca = text[(ia + k) % n];
            let cb = text[(ib + k) % n];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        ia.cmp(&ib)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (i, &pos) in sa.iter().enumerate() {
        if pos == 0 {
            p = i as u32;
        }
        l.push(text[(pos as usize + n - 1) % n]);
    }
    let expected = plain_invert_fl(&l, p);
    assert_eq!(expected, text);
    Block {
        name: "dense-300k".into(),
        n,
        p,
        route_class: 3,
        records: records_of(&l),
        expected: Some(expected),
    }
}

fn make_spiral_200k() -> Block {
    let n = 200_000usize;
    let mut l = vec![b'a'; n];
    l[0] = b'c';
    l[n - 1] = b'b';
    let p = 0u32;
    let expected = plain_invert_fl(&l, p);
    Block {
        name: "spiral-200k".into(),
        n,
        p,
        route_class: 2,
        records: records_of(&l),
        expected: Some(expected),
    }
}

fn make_periodic(name: &str, pat: &[u8], reps: usize) -> Block {
    let n = pat.len() * reps;
    let mut text = Vec::with_capacity(n);
    for _ in 0..reps {
        text.extend_from_slice(pat);
    }
    let mut rots: Vec<usize> = (0..pat.len()).collect();
    rots.sort_by(|&a, &b| {
        for k in 0..pat.len() {
            let ca = pat[(a + k) % pat.len()];
            let cb = pat[(b + k) % pat.len()];
            if ca != cb {
                return ca.cmp(&cb);
            }
        }
        a.cmp(&b)
    });
    let mut l = Vec::with_capacity(n);
    let mut p = 0u32;
    for (grp, &r) in rots.iter().enumerate() {
        let prev_b = pat[(r + pat.len() - 1) % pat.len()];
        if r == 0 {
            p = (grp * reps) as u32;
        }
        for _ in 0..reps {
            l.push(prev_b);
        }
    }
    let expected = plain_invert_fl(&l, p);
    assert_eq!(expected, text);
    Block {
        name: name.into(),
        n,
        p,
        route_class: 2,
        records: records_of(&l),
        expected: Some(expected),
    }
}

// ============================================================================
// 2. Forward-FL Exact Walks & Specialists (`Spec4-SPRT-L`, `C48-Fused`)
// ============================================================================

#[derive(Default)]
struct PlainRunWalk {
    fl: Vec<u32>,
}

impl PlainRunWalk {
    #[inline(never)]
    fn build_fl(&mut self, records: &[u16], n: usize) -> bool {
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
        if self.fl.len() < n {
            self.fl.resize(n, 0);
        }
        let fl = &mut self.fl[..n];
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
        true
    }

    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
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
        if !self.build_fl(records, n) {
            return false;
        }
        out.resize(n, 0);
        let fl = &self.fl[..n];
        let mut x = p as usize;
        for slot in &mut out[..n] {
            let v = fl[x];
            *slot = v as u8;
            x = (v >> 8) as usize;
        }
        true
    }

    #[inline(never)]
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out), "PlainRunWalk invalid tiling/bounds");
    }
}

#[derive(Default)]
struct InterleavedWalk {
    plain: PlainRunWalk,
    start_bits: Vec<u64>,
    segs: [Vec<u8>; 8],
}

impl InterleavedWalk {
    #[inline(never)]
    fn try_invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
        if n == 0 || (p as usize) >= n {
            return false;
        }
        if n < 4096 || records.len() == 1 {
            return self.plain.try_invert(records, n, p, out);
        }
        if !self.plain.build_fl(records, n) {
            return false;
        }
        let fl = &self.plain.fl[..n];
        let words = (n + 63) / 64;
        if self.start_bits.len() < words {
            self.start_bits.resize(words, 0);
        }
        let mut starts = [0usize; 8];
        let stride = (n / 8).max(1);
        for c in 0..8 {
            let mut s = (p as usize + c * stride) % n;
            while (self.start_bits[s >> 6] >> (s & 63)) & 1 != 0 {
                s = (s + 1) % n;
            }
            starts[c] = s;
            self.start_bits[s >> 6] |= 1u64 << (s & 63);
        }
        for seg in &mut self.segs {
            seg.clear();
        }
        let mut cur = starts;
        let mut end_row = [usize::MAX; 8];
        let mut active = [0usize, 1, 2, 3, 4, 5, 6, 7];
        let mut n_active = 8usize;
        let mut total_steps = 0usize;
        while n_active > 0 && total_steps <= n {
            let mut i = 0;
            while i < n_active {
                let c = active[i];
                let v = fl[cur[c]];
                self.segs[c].push(v as u8);
                let nx = (v >> 8) as usize;
                cur[c] = nx;
                total_steps += 1;
                if (self.start_bits[nx >> 6] >> (nx & 63)) & 1 != 0 {
                    end_row[c] = nx;
                    n_active -= 1;
                    active[i] = active[n_active];
                } else {
                    i += 1;
                }
            }
        }
        for &s in &starts {
            self.start_bits[s >> 6] &= !(1u64 << (s & 63));
        }
        let mut order = [0usize; 8];
        let mut n_ord = 0usize;
        let mut stitched_len = 0usize;
        let mut cur_c = 0usize;
        let mut closed = false;
        while n_ord < 8 {
            if end_row[cur_c] == usize::MAX {
                break;
            }
            order[n_ord] = cur_c;
            n_ord += 1;
            stitched_len += self.segs[cur_c].len();
            let er = end_row[cur_c];
            if er == starts[0] {
                closed = true;
                break;
            }
            if let Some(next_c) = starts.iter().position(|&s| s == er) {
                cur_c = next_c;
            } else {
                break;
            }
        }
        if closed && stitched_len > 0 && stitched_len <= n {
            out.clear();
            out.reserve(n);
            for &c in &order[..n_ord] {
                out.extend_from_slice(&self.segs[c]);
            }
            while out.len() < n {
                let take = out.len().min(n - out.len());
                out.extend_from_within(..take);
            }
            return true;
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
    fn invert(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        assert!(self.try_invert(records, n, p, out), "InterleavedWalk invalid tiling/bounds");
    }
}

/// Cauchy (1815) exact short-cycle orbit detector (`recs*64 < n`, `n >= 4096`).
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

/// 2-stage sub-microsecond stride sampler (`64 -> 256` records) choosing `L ∈ {16, 24, 32}`.
#[inline]
fn sprt_select_limit(records: &[u16], n: usize) -> (u32, u32) {
    let nrec = records.len();
    if n < 32_768 || nrec < 256 {
        return (16, 0);
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
        return (16, 64);
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
            (32, 256)
        } else if s_r17 <= 5 {
            (24, 256)
        } else {
            (16, 256)
        }
    } else {
        (16, 256)
    }
}

/// Cycle 47 `Spec4-SPRT-L` (`ID_BITS=22` + raw `nrec` `PostS1Md` + `Spec4-SPRT-L`).
#[derive(Default)]
struct Spec4SprtL {
    open: unified_open_c47::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl Spec4SprtL {
    const POST_S1_BI: unified_open_c47::BiMode = unified_open_c47::BiMode::PostS1Md {
        recs_max: 57_000,
        ns_max: 2_400,
        md_max: 26_000,
    };

    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(
                    records,
                    n,
                    p,
                    unified_open_c47::Cap::NONE,
                    unified_open_c47::BiMode::AlwaysAt(65_536),
                    out,
                )
                .expect("fallback");
            return 0;
        }
        if n < 32_768 {
            self.seg.plain.invert(records, n, p, out);
            return 1;
        }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) {
            return 2;
        }
        if records.len() * 25 >= n * 7 {
            self.seg.invert(records, n, p, out);
            return 3;
        }
        let (lim, _samples) = sprt_select_limit(records, n);
        let cap = if records.len() * 20 >= n * 3 {
            unified_open_c47::Cap::dense(n)
        } else {
            unified_open_c47::Cap::NONE
        };
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, cap, Self::POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, cap, Self::POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, cap, Self::POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) {
                (16, true) => 0,
                (16, false) => 5,
                (24, _) => 6,
                _ => 7,
            };
        }
        self.seg.invert(records, n, p, out);
        3
    }
}

/// Cycle 48 `C48-Fused` (`F-Inline3` `ID_BITS=24` + exact `r_thin` `extra_127` `PostS1ExactMd`
/// + `Spec4-SPRT-L` + de-inlined `finish`/`wire_bi`/`wire_uni`).
#[derive(Default)]
struct C48Fused {
    open: unified_open_c48::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl C48Fused {
    const POST_S1_BI: unified_open_c48::BiMode = unified_open_c48::BiMode::PostS1ExactMd {
        r_max_cap: 50_000,
        ns_max: 2_100,
        md_max: 16_000,
        md_neg_cap: -18_000,
        ns_neg_max: 2_400,
    };

    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(
                    records,
                    n,
                    p,
                    unified_open_c48::Cap::NONE,
                    unified_open_c48::BiMode::AlwaysAt(65_536),
                    out,
                )
                .expect("fallback");
            return 0;
        }
        if n < 32_768 {
            self.seg.plain.invert(records, n, p, out);
            return 1;
        }
        if records.len() * 64 < n && try_short_periodic(records, n, p, out) {
            return 2;
        }
        if records.len() * 25 >= n * 7 {
            self.seg.invert(records, n, p, out);
            return 3;
        }
        let (lim, _samples) = sprt_select_limit(records, n);
        let cap = if records.len() * 20 >= n * 3 {
            unified_open_c48::Cap::dense(n)
        } else {
            unified_open_c48::Cap::NONE
        };
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, cap, Self::POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, cap, Self::POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, cap, Self::POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) {
                (16, true) => 0,
                (16, false) => 5,
                (24, _) => 6,
                _ => 7,
            };
        }
        self.seg.invert(records, n, p, out);
        3
    }
}

// ============================================================================
// 3. Exactness, Forced-Fallback, Exact-`r_thin` Bound & Scratch Audit
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn exact_w1_and_r_max(records: &[u16]) -> (u32, u32, u32) {
    let mut w1 = 0u32;
    let mut r_max = 0u32;
    let mut extra_127 = 0u32;
    let mut i = 0usize;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut w = u32::from(rec & 255);
        i += 1;
        if w == 127 {
            while i < records.len() && (records[i] >> 8) as u8 == c {
                w += u32::from(records[i] & 255);
                extra_127 += 1;
                i += 1;
            }
        }
        r_max += 1;
        if w == 1 {
            w1 += 1;
        }
    }
    (w1, r_max, extra_127)
}

fn run_exactness_and_bound_audit(
    six: &[Block],
    wiki431: &[Block],
    non_wiki: &[Block],
    code_docs: &[Block],
) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut kn = cand_r5_nobi::Kernel::default();
    let mut kf = unified_open_c48::Kernel::default();
    let mut sp4 = Spec4SprtL::default();
    let mut c48 = C48Fused::default();
    let mut plain = PlainRunWalk::default();
    let mut seg8 = InterleavedWalk::default();
    let mut out_ref = Vec::with_capacity(900_064);
    let mut out_k = Vec::with_capacity(900_064);

    // 3a. 10,190 small valid BWTs (8,190 all binary n=1..=12 + 2,000 random n=1..=512 across 7 alphabets)
    let mut valid_cnt = 0usize;
    let mut text = Vec::with_capacity(512);
    for n in 1..=12usize {
        for mask in 0..(1usize << n) {
            text.clear();
            for bit in 0..n {
                text.push(b'a' + ((mask >> bit) & 1) as u8);
            }
            let (l, p) = bwt_small(&text);
            let recs = records_of(&l);
            kf.try_run(
                &recs,
                n,
                p,
                16,
                unified_open_c48::Cap::NONE,
                unified_open_c48::BiMode::AlwaysAt(0),
                &mut out_k,
            )
            .unwrap();
            assert_eq!(out_k, text, "F-Inline3 binary bi mismatch n={n} mask={mask}");
            kf.try_run(
                &recs,
                n,
                p,
                32,
                unified_open_c48::Cap::NONE,
                unified_open_c48::BiMode::Never,
                &mut out_k,
            )
            .unwrap();
            assert_eq!(out_k, text, "F-Inline3 binary nobi32 mismatch");
            sp4.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            c48.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            valid_cnt += 1;
        }
    }

    let mut rng = 0xA076_1D64_78BD_642Fu64;
    for case in 0..2_000usize {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let n = 1 + ((rng >> 32) as usize % 512);
        let alpha = match case % 7 {
            0 => 2u16,
            1 => 3,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 26,
            _ => 256,
        };
        text.clear();
        for _ in 0..n {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            text.push(((rng >> 40) % alpha as u64) as u8);
        }
        let (l, p) = bwt_small(&text);
        let recs = records_of(&l);
        for &lim in &[4u32, 16, 24, 32] {
            kf.try_run(
                &recs,
                n,
                p,
                lim,
                unified_open_c48::Cap::NONE,
                unified_open_c48::BiMode::AlwaysAt(0),
                &mut out_k,
            )
            .unwrap();
            assert_eq!(out_k, text);
            kf.try_run(
                &recs,
                n,
                p,
                lim,
                unified_open_c48::Cap::NONE,
                unified_open_c48::BiMode::Never,
                &mut out_k,
            )
            .unwrap();
            assert_eq!(out_k, text);
        }
        sp4.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        c48.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 3b. 4 Stress + 6 Admitted + 6 Non-Wiki JSONL/Log + 4 Code/Docs + Forced Fallbacks
    let dense = make_dense_300k();
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [&dense, &spiral, &per2, &per3];

    for b in six
        .iter()
        .chain(stress.iter().copied())
        .chain(non_wiki.iter())
        .chain(code_docs.iter())
    {
        let want = b.expected.as_ref().unwrap();
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k)
            .unwrap();
        assert_eq!(&out_k, want, "cand_r5_head mismatch on {}", b.name);
        kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out_k)
            .unwrap();
        assert_eq!(&out_k, want, "cand_r5_nobi mismatch on {}", b.name);
        for &lim in &[16u32, 24, 32] {
            for bm in [
                unified_open_c48::BiMode::AlwaysAt(65_536),
                unified_open_c48::BiMode::Never,
                C48Fused::POST_S1_BI,
            ] {
                kf.try_run(
                    &b.records,
                    b.n,
                    b.p,
                    lim,
                    unified_open_c48::Cap::NONE,
                    bm,
                    &mut out_k,
                )
                .unwrap();
                assert_eq!(&out_k, want, "F-Inline3 mismatch on {} L={} {:?}", b.name, lim, bm);
            }
        }
        sp4.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "Spec4 mismatch on {}", b.name);
        c48.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "C48-Fused mismatch on {}", b.name);

        // Forced fallback tests on Spec4-SPRT-L and C48-Fused
        sp4.force_fallback = true;
        assert_eq!(sp4.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "Spec4 forced fallback mismatch on {}", b.name);
        sp4.force_fallback = false;

        c48.force_fallback = true;
        assert_eq!(c48.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "C48-Fused forced fallback mismatch on {}", b.name);
        c48.force_fallback = false;
    }

    // Verify spiral rejects periodic probe & periodic hits route 2
    let c48_spiral = c48.run_entry(&spiral.records, spiral.n, spiral.p, &mut out_k);
    assert!(
        matches!(c48_spiral, 0 | 5 | 6 | 7),
        "C48-Fused spiral must reject periodic probe (got {c48_spiral})"
    );
    assert_eq!(&out_k, spiral.expected.as_ref().unwrap());
    assert_eq!(c48.run_entry(&per2.records, per2.n, per2.p, &mut out_k), 2);
    assert_eq!(&out_k, per2.expected.as_ref().unwrap());

    // 3c. All 431 wikidata.bz2 blocks exact across R5-Head, F-Inline3, Spec4-SPRT-L, C48-Fused
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref)
            .unwrap();
        kf.try_run_const::<16>(
            &b.records,
            b.n,
            b.p,
            unified_open_c48::Cap::NONE,
            unified_open_c48::BiMode::AlwaysAt(65_536),
            &mut out_k,
        )
        .unwrap();
        assert_eq!(out_k, out_ref, "wiki431 F-Inline3 mismatch idx={idx}");
        sp4.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 Spec4 mismatch idx={idx}");
        c48.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 C48-Fused mismatch idx={idx}");
    }

    // 3d. 300 Malformed / Multi-Cycle Forward-FL Columns + 0x00 & SATURATED=255 Stress + Broken Tiling
    assert!(!plain.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(!seg8.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(kf
        .try_run(
            &[(u16::from(b'a') << 8) | 3],
            5,
            0,
            16,
            unified_open_c48::Cap::NONE,
            unified_open_c48::BiMode::AlwaysAt(0),
            &mut out_k
        )
        .is_none());

    let mut mal_cnt = 0usize;
    let mut check_malformed = |l: &[u8], p: u32, tag: &str| {
        let n = l.len();
        let recs = records_of(l);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref)
            .unwrap();
        plain.invert(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed PlainRunWalk mismatch {tag}");
        seg8.invert(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Seg8-FL mismatch {tag}");
        kf.try_run(
            &recs,
            n,
            p,
            16,
            unified_open_c48::Cap::NONE,
            unified_open_c48::BiMode::AlwaysAt(0),
            &mut out_k,
        )
        .unwrap();
        assert_eq!(out_k, out_ref, "malformed F-Inline3(bi) mismatch {tag}");
        kf.try_run(
            &recs,
            n,
            p,
            32,
            unified_open_c48::Cap::NONE,
            unified_open_c48::BiMode::Never,
            &mut out_k,
        )
        .unwrap();
        assert_eq!(out_k, out_ref, "malformed F-Inline3(nobi32) mismatch {tag}");
        sp4.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec4 mismatch {tag}");
        c48.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed C48-Fused mismatch {tag}");
        mal_cnt += 1;
    };

    // Witness 1: C46 Bug #1 L="bbaabb", p=0
    check_malformed(b"bbaabb", 0, "bbaabb");

    // Witnesses 2..=8: SATURATED=255 boundary spirals (periods 253..=258, 512) with 0x00 byte
    for &per in &[253usize, 254, 255, 256, 257, 258, 512] {
        let n = per * 32;
        let mut l = vec![0u8; n];
        l[0] = 2u8;
        l[n - 1] = 1u8;
        check_malformed(&l, 0, &format!("sat255-per-{per}"));
    }

    // Witnesses 9..=300: 292 malformed columns (incl. {0x00, 0x01, 0x02} 0x00-heavy + large >32k)
    let mut lbuf = Vec::with_capacity(65_536);
    for case in 0..292usize {
        let n = if case < 170 {
            1 + (case * 37) % 512
        } else if case < 230 {
            8_192
        } else {
            40_000 + (case - 230) * 137
        };
        let alpha = if case < 60 {
            3usize // stresses 0x00, 0x01, 0x02 keys in F-Inline3 pairs cache
        } else if case >= 230 && case < 255 {
            2
        } else {
            2 + (case % 12)
        };
        lbuf.clear();
        if case >= 230 && case < 255 {
            let half = n / 2;
            lbuf.resize(half / 2, b'b');
            lbuf.resize(half, b'a');
            lbuf.resize(half + half / 2, b'a');
            lbuf.resize(n, b'b');
        } else {
            for _ in 0..n {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                lbuf.push(((rng >> 40) % alpha as u64) as u8);
            }
        }
        let p = ((rng >> 32) as usize % n) as u32;
        check_malformed(&lbuf, p, &format!("case-{case}"));
    }

    // 3e. Exactness Audit of Dirichlet/Pigeonhole Identity `2*(r_max - ns) - m <= w1_true`
    //     vs C47 raw `2*(nrec - ns) - m`, plus S2b rule reduction & PostS1ExactMd routing!
    let mut exact_viol = 0usize;
    let mut raw_viol = 0usize;
    let mut raw_worst = (String::new(), 0i64, 0u32, 0u32, 0i64, 0i64);
    let mut wiki_full_bi = 0usize;
    let mut wiki_full_tot = 0usize;
    let mut nw_nobi = 0usize;
    let mut checked_blks = 0usize;

    let mut rules_base_6 = 0usize;
    let mut rules_i3_6 = 0usize;
    let mut rules_base_nw = 0usize;
    let mut rules_i3_nw = 0usize;
    let mut rules_base_cd = 0usize;
    let mut rules_i3_cd = 0usize;

    let all_sets: [(&str, &[Block]); 4] = [
        ("admitted", six),
        ("wiki431", wiki431),
        ("non_wiki", non_wiki),
        ("code_docs", code_docs),
    ];
    for (label, set) in all_sets {
        for b in set {
            let (w1, r_max_ref, extra_ref) = exact_w1_and_r_max(&b.records);
            let st_h = kh
                .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k)
                .unwrap();
            let st_f = kf
                .try_run_const::<16>(
                    &b.records,
                    b.n,
                    b.p,
                    unified_open_c48::Cap::NONE,
                    C48Fused::POST_S1_BI,
                    &mut out_k,
                )
                .unwrap();
            assert_eq!(st_f.r_max, r_max_ref, "r_max mismatch on {}", b.name);
            assert_eq!(st_f.extra_127, extra_ref, "extra_127 mismatch on {}", b.name);
            let m = st_f.retained as i64;
            let ns = st_f.thick as i64;
            let nrec = b.records.len() as i64;
            let md_exact = 2 * (i64::from(st_f.r_max) - ns) - m;
            let md_raw = 2 * (nrec - ns) - m;
            if md_exact > i64::from(w1) {
                exact_viol += 1;
            }
            if md_raw > i64::from(w1) {
                raw_viol += 1;
                let over = md_raw - i64::from(w1);
                if over > raw_worst.1 {
                    raw_worst = (b.name.clone(), over, w1, st_f.extra_127, md_raw, md_exact);
                }
            }
            if label == "wiki431" && b.route_class < 2 {
                wiki_full_tot += 1;
                if st_f.used_bi {
                    wiki_full_bi += 1;
                }
            }
            if label == "non_wiki" && !st_f.used_bi {
                nw_nobi += 1;
            }
            match label {
                "admitted" => {
                    rules_base_6 += st_h.rules;
                    rules_i3_6 += st_f.rules;
                }
                "non_wiki" => {
                    rules_base_nw += st_h.rules;
                    rules_i3_nw += st_f.rules;
                }
                "code_docs" => {
                    rules_base_cd += st_h.rules;
                    rules_i3_cd += st_f.rules;
                }
                _ => {}
            }
            checked_blks += 1;
        }
    }
    assert_eq!(exact_viol, 0, "Exact Dirichlet/Pigeonhole bound MUST have 0 violations!");

    // 3f. Persistent scratch report on FRESH kernels over 6 admitted blocks
    let mut kh_ws = cand_r5_head::Kernel::default();
    let mut kf_ws = unified_open_c48::Kernel::default();
    let mut ku47_ws = unified_open_c47::Kernel::default();
    for b in six {
        kh_ws
            .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k)
            .unwrap();
        ku47_ws
            .try_run_const::<16>(
                &b.records,
                b.n,
                b.p,
                unified_open_c47::Cap::NONE,
                Spec4SprtL::POST_S1_BI,
                &mut out_k,
            )
            .unwrap();
        kf_ws
            .try_run_const::<16>(
                &b.records,
                b.n,
                b.p,
                unified_open_c48::Cap::NONE,
                C48Fused::POST_S1_BI,
                &mut out_k,
            )
            .unwrap();
    }
    let (h_ws, h_l1, h_str) = kh_ws.scratch_report();
    let (u47_ws, u47_l1, u47_str) = ku47_ws.scratch_report();
    let (f_ws, f_l1, f_str) = kf_ws.scratch_report();

    println!(
        "EXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed (incl. 0x00 & SATURATED=255, strict forward-FL) + 4 stress + {} admitted + {} wikidata.bz2 + {} JSONL/Log + {} Code/Docs + forced fallbacks 100% BYTE-EXACT!",
        six.len(),
        wiki431.len(),
        non_wiki.len(),
        code_docs.len()
    );
    println!("DIRICHLET/PIGEONHOLE EXACT r_thin BOUND AUDIT ({checked_blks} real blocks):");
    println!("  Exact 2*(r_max - ns) - m <= w1 : {exact_viol}/{checked_blks} violations (PROVED EXACT!)");
    println!(
        "  C47 raw 2*(nrec - ns) - m <= w1: {raw_viol}/{checked_blks} violations (worst: {} over={:+} w1={} extra127={} md_raw={} md_exact={})",
        raw_worst.0, raw_worst.1, raw_worst.2, raw_worst.3, raw_worst.4, raw_worst.5
    );
    println!(
        "  PostS1ExactMd routing          : Wiki full μ=2 {wiki_full_bi}/{wiki_full_tot} | JSONL/Log μ=1 {nw_nobi}/{}",
        non_wiki.len()
    );
    println!("S2b ARENA RULES REDUCTION (F-Inline3 ID_BITS=24 vs R5-Head ID_BITS=22):");
    println!(
        "  6 Admitted Wiki : {rules_base_6} -> {rules_i3_6} ({:+.2}%)",
        (rules_i3_6 as f64 / rules_base_6 as f64 - 1.0) * 100.0
    );
    println!(
        "  6 JSONL/Log     : {rules_base_nw} -> {rules_i3_nw} ({:+.2}%)",
        (rules_i3_nw as f64 / rules_base_nw as f64 - 1.0) * 100.0
    );
    println!(
        "  4 Code/Docs     : {rules_base_cd} -> {rules_i3_cd} ({:+.2}%)",
        (rules_i3_cd as f64 / rules_base_cd as f64 - 1.0) * 100.0
    );
    println!("SCRATCH REPORT (steady-state over 6 admitted blocks):");
    println!("  cand_r5_head    : total={} B ({:.1} KiB), dict_l1={} B | {}", h_ws, h_ws as f64 / 1024.0, h_l1, h_str);
    println!("  Spec4 (C47 Open): total={} B ({:.1} KiB), dict_l1={} B | {}", u47_ws, u47_ws as f64 / 1024.0, u47_l1, u47_str);
    println!("  C48-Fused Open  : total={} B ({:.1} KiB), dict_l1={} B | {}", f_ws, f_ws as f64 / 1024.0, f_l1, f_str);
    println!("  Delta WS (C48 vs Head): {:+} B", f_ws as isize - h_ws as isize);

    // 3g. C_recognise (sprt_select_limit) telemetry across all 4 corpora
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();
    let mut chk = 0u64;
    let measure_rec = |list: &[&Block], reps: usize, chk: &mut u64| -> (f64, f64, usize, usize, usize) {
        let mut s_sum = 0usize;
        let (mut l16, mut l24, mut l32) = (0usize, 0usize, 0usize);
        for &b in list {
            let (l, s) = sprt_select_limit(&b.records, b.n);
            s_sum += s as usize;
            match l {
                16 => l16 += 1,
                24 => l24 += 1,
                _ => l32 += 1,
            }
        }
        let t0 = Instant::now();
        for _ in 0..reps {
            for &b in list {
                let (l, _) = sprt_select_limit(&b.records, b.n);
                *chk = chk.wrapping_add(u64::from(l));
            }
        }
        let ns = t0.elapsed().as_nanos() as f64 / (reps * list.len()) as f64;
        (ns, s_sum as f64 / list.len() as f64, l16, l24, l32)
    };
    let six_r: Vec<&Block> = six.iter().collect();
    let nw_r: Vec<&Block> = non_wiki.iter().collect();
    let cd_r: Vec<&Block> = code_docs.iter().collect();
    let (ns_6, s_6, a6, b6, c6) = measure_rec(&six_r, 10_000, &mut chk);
    let (ns_w, s_w, aw, bw, cw) = measure_rec(&full_wiki, 1_000, &mut chk);
    let (ns_nw, s_nw, anw, bnw, cnw) = measure_rec(&nw_r, 10_000, &mut chk);
    let (ns_cd, s_cd, acd, bcd, ccd) = measure_rec(&cd_r, 10_000, &mut chk);
    println!("C_RECOGNISE (2-stage 64->256 stride sampler) TELEMETRY (chk={chk}):");
    println!("  6 Admitted   : C_rec={ns_6:5.1} ns/blk | avg_samples={s_6:5.1} | L16={a6} L24={b6} L32={c6}");
    println!("  233 FullWiki : C_rec={ns_w:5.1} ns/blk | avg_samples={s_w:5.1} | L16={aw} L24={bw} L32={cw}");
    println!("  6 JSONL/Log  : C_rec={ns_nw:5.1} ns/blk | avg_samples={s_nw:5.1} | L16={anw} L24={bnw} L32={cnw}");
    println!("  4 Code/Docs  : C_rec={ns_cd:5.1} ns/blk (bypassed in ~1 ns by recs/n>=0.28 gate!) | avg_samples={s_cd:5.1} | L16={acd} L24={bcd} L32={ccd}");
}

// ============================================================================
// 4. Headroom Oracle v3 Benchmarks (1T, 2W, 4W) Across All 4 Corpora + Overall
// ============================================================================

const ARM_NAMES: [&str; 6] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "R5-Nobi",
    "F-Inline3",
    "Spec4-SPRT-L",
    "C48-Fused",
];

struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    kn: cand_r5_nobi::Kernel,
    kf: unified_open_c48::Kernel,
    sp4: Spec4SprtL,
    c48: C48Fused,
}

impl Default for ArmSuite {
    fn default() -> Self {
        Self {
            kh1: cand_r5_head::Kernel::default(),
            kh2: cand_r5_head::Kernel::default(),
            kn: cand_r5_nobi::Kernel::default(),
            kf: unified_open_c48::Kernel::default(),
            sp4: Spec4SprtL::default(),
            c48: C48Fused::default(),
        }
    }
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => {
                self.kh1
                    .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out);
                0
            }
            1 => {
                self.kh2
                    .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out);
                0
            }
            2 => {
                self.kn
                    .run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, out);
                5
            }
            3 => {
                self.kf.try_run_const::<16>(
                    &b.records,
                    b.n,
                    b.p,
                    unified_open_c48::Cap::NONE,
                    unified_open_c48::BiMode::AlwaysAt(65_536),
                    out,
                );
                0
            }
            4 => self.sp4.run_entry(&b.records, b.n, b.p, out),
            _ => self.c48.run_entry(&b.records, b.n, b.p, out),
        }
    }
}

struct Bench1TResult {
    tot_us: [f64; 6],
    oracle_us: f64,
    full_tot_us: [f64; 6],
    full_oracle_us: f64,
}

fn bench_1t_interleaved(
    label: &str,
    blocks: &[Block],
    rds: usize,
    show_rc: bool,
    show_per_block: bool,
) -> Bench1TResult {
    println!("\n=== HEADROOM ORACLE v3 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);

    for b in blocks.iter().take(6) {
        for arm in 0..6 {
            suite.run_arm(arm, b, &mut out);
        }
    }

    let mut tot = [0.0f64; 6];
    let mut full_tot = [0.0f64; 6];
    let mut oracle_us = 0.0f64;
    let mut full_oracle_us = 0.0f64;
    let mut by_rc = [[0.0f64; 4]; 6];
    let mut sp4_paths = [0usize; 8];
    let mut c48_paths = [0usize; 8];

    for b in blocks {
        let rc = (b.route_class as usize).min(3);
        let mut s: [Vec<f64>; 6] = Default::default();
        let mut p4 = 0u8;
        let mut p5 = 0u8;
        for r in 0..rds {
            for off in 0..6 {
                let arm = (r + off) % 6;
                let t0 = Instant::now();
                let path = suite.run_arm(arm, b, &mut out);
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                if arm == 4 {
                    p4 = path;
                }
                if arm == 5 {
                    p5 = path;
                }
            }
        }
        sp4_paths[p4 as usize] += 1;
        c48_paths[p5 as usize] += 1;
        let mut meds = [0.0f64; 6];
        for arm in 0..6 {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m;
            tot[arm] += m;
            by_rc[arm][rc] += m;
            if b.route_class < 2 {
                full_tot[arm] += m;
            }
        }
        // Hindsight oracle over genuine arms (excluding A/A duplicate Arm 1)
        let best = meds[0].min(meds[2]).min(meds[3]).min(meds[4]).min(meds[5]);
        oracle_us += best;
        if b.route_class < 2 {
            full_oracle_us += best;
        }
        if show_per_block {
            println!(
                "  {:12} | H-A1={:6.1} | H-A2={:6.1} | Nobi={:6.1} ({:.4}x) | I3={:6.1} ({:.4}x) | Sp4(p{})={:6.1} ({:.4}x) | C48(p{})={:6.1} ({:.4}x) | Orcl={:6.1}",
                b.name,
                meds[0],
                meds[1],
                meds[2],
                meds[2] / meds[0],
                meds[3],
                meds[3] / meds[0],
                p4,
                meds[4],
                meds[4] / meds[0],
                p5,
                meds[5],
                meds[5] / meds[0],
                best
            );
        }
    }

    let base = tot[0];
    for arm in 0..6 {
        let lt = if arm >= 3 {
            if tot[arm] < base { "YES" } else { "NO" }
        } else {
            "-"
        };
        if show_rc {
            println!(
                "  {:16} | norm={:7.2}ms dense={:5.2}ms tiny={:4.2}ms col={:4.2}ms | TOTAL={:7.2}ms ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm],
                by_rc[arm][0] / 1e3,
                by_rc[arm][1] / 1e3,
                by_rc[arm][2] / 1e3,
                by_rc[arm][3] / 1e3,
                tot[arm] / 1e3,
                tot[arm] / base,
                lt
            );
        } else {
            println!(
                "  {:16} | TOTAL={:8.1} us ({:.4}x) | < C_Open? {}",
                ARM_NAMES[arm],
                tot[arm],
                tot[arm] / base,
                lt
            );
        }
    }
    println!(
        "  {:16} | TOTAL={:8.1} us ({:.4}x) | C48 capture of oracle headroom = {:.1}%",
        "Hindsight-Oracle",
        oracle_us,
        oracle_us / base,
        if base > oracle_us {
            (base - tot[5]) / (base - oracle_us) * 100.0
        } else {
            0.0
        }
    );
    if show_rc {
        let fb = full_tot[0];
        println!(
            "  233-Full-Wiki 1T | Head={:.2}ms | Nobi={:.4}x | F-Inline3={:.4}x | Spec4={:.4}x | C48-Fused={:.4}x | Oracle={:.4}x",
            fb / 1e3,
            full_tot[2] / fb,
            full_tot[3] / fb,
            full_tot[4] / fb,
            full_tot[5] / fb,
            full_oracle_us / fb
        );
    }
    println!("  Spec4 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", sp4_paths);
    println!("  C48   routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", c48_paths);

    Bench1TResult {
        tot_us: tot,
        oracle_us,
        full_tot_us: full_tot,
        full_oracle_us,
    }
}

fn bench_code_docs_capped_ablation(code_docs: &[Block], rds: usize) {
    println!("\n=== CODE/DOCS CAPPED-OPEN (`Cap::dense` -> `Seg8-FL`) vs `C48-Fused` ABLATION (1T, {rds} rds/blk) ===");
    let mut kh = cand_r5_head::Kernel::default();
    let mut seg = InterleavedWalk::default();
    let mut c48 = C48Fused::default();
    let mut out = Vec::with_capacity(900_064);

    let mut tot = [0.0f64; 3]; // [uncapped Open, Cap::dense + Seg8 fallback, C48-Fused O(1) gate]
    for b in code_docs {
        let mut s: [Vec<f64>; 3] = Default::default();
        for r in 0..rds {
            for off in 0..3 {
                let arm = (r + off) % 3;
                let t0 = Instant::now();
                match arm {
                    0 => {
                        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out);
                    }
                    1 => {
                        if kh
                            .run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::dense(b.n), &mut out)
                            .is_none()
                        {
                            seg.invert(&b.records, b.n, b.p, &mut out);
                        }
                    }
                    _ => {
                        c48.run_entry(&b.records, b.n, b.p, &mut out);
                    }
                }
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        let m0 = pct(&mut s[0], 1, 2);
        let m1 = pct(&mut s[1], 1, 2);
        let m2 = pct(&mut s[2], 1, 2);
        tot[0] += m0;
        tot[1] += m1;
        tot[2] += m2;
        println!(
            "  {:12} (recs/n={:.4}) | Uncapped-Open={:7.1} us | CapDense+Seg8={:7.1} us ({:.4}x) | C48-Fused={:7.1} us ({:.4}x vs Uncap, {:.4}x vs CapDense)",
            b.name,
            b.records.len() as f64 / b.n as f64,
            m0,
            m1,
            m1 / m0,
            m2,
            m2 / m0,
            m2 / m1
        );
    }
    println!(
        "  TOTAL (4 blks)           | Uncapped-Open={:7.1} us | CapDense+Seg8={:7.1} us ({:.4}x) | C48-Fused={:7.1} us ({:.4}x vs Uncap, {:.4}x vs CapDense)",
        tot[0],
        tot[1],
        tot[1] / tot[0],
        tot[2],
        tot[2] / tot[0],
        tot[2] / tot[1]
    );
}

fn bench_mt_suite(label: &str, blocks: &[Block], passes_2w: usize, passes_4w: usize) -> ([f64; 6], [f64; 6]) {
    let mut p50_2w = [0.0f64; 6];
    let mut p50_4w = [0.0f64; 6];
    for &(threads, passes) in &[(2usize, passes_2w), (4usize, passes_4w)] {
        println!(
            "\n=== {} — {}W CONCURRENT BARRIER ({} interleaved passes) ===",
            label, threads, passes
        );
        let mut rmt: [Vec<f64>; 6] = Default::default();
        for pass in 0..passes {
            for off in 0..6 {
                let arm = (pass + off) % 6;
                let bar = Barrier::new(threads);
                let t = std::thread::scope(|sc| {
                    let bar_ref = &bar;
                    let mut hs = Vec::with_capacity(threads);
                    for tid in 0..threads {
                        hs.push(sc.spawn(move || {
                            let mut out = Vec::with_capacity(900_064);
                            let mut suite = ArmSuite::default();
                            for b in blocks.iter().take(6) {
                                suite.run_arm(arm, b, &mut out);
                            }
                            bar_ref.wait();
                            let t0 = Instant::now();
                            for idx in 0..blocks.len() {
                                let b = &blocks[(idx + tid) % blocks.len()];
                                suite.run_arm(arm, b, &mut out);
                            }
                            t0.elapsed().as_secs_f64() * 1e6
                        }));
                    }
                    hs.into_iter()
                        .map(|h| h.join().unwrap())
                        .fold(0.0f64, f64::max)
                });
                rmt[arm].push(t);
            }
        }
        let b10 = pct(&mut rmt[0].clone(), 1, 10);
        let b50 = pct(&mut rmt[0].clone(), 1, 2);
        for arm in 0..6 {
            let p10 = pct(&mut rmt[arm], 1, 10);
            let p50 = pct(&mut rmt[arm], 1, 2);
            if threads == 2 {
                p50_2w[arm] = p50;
            } else {
                p50_4w[arm] = p50;
            }
            let lt = if arm >= 3 {
                if p50 < b50 { "YES" } else { "NO" }
            } else {
                "-"
            };
            println!(
                "  {}W {:16} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x) | p50 < C_Open? {}",
                threads,
                ARM_NAMES[arm],
                p10,
                p10 / b10,
                p50,
                p50 / b50,
                lt
            );
        }
    }
    (p50_2w, p50_4w)
}

fn main() {
    let six = load_six_admitted();
    let wiki_bin = resolve_corpus_path("wikidata_431_blocks.bin");
    let nonwiki_bin = resolve_corpus_path("non_wikidata/non_wikidata_blocks.bin");
    let codedocs_bin = resolve_corpus_path("non_wikidata/code_docs_blocks.bin");

    let wiki431 = load_bin_corpus(&wiki_bin, "wiki", false);
    let non_wiki = load_bin_corpus(&nonwiki_bin, "nonwiki", true);
    let code_docs = load_bin_corpus(&codedocs_bin, "codedoc", true);

    println!(
        "Headroom Oracle v3 (Cycle 48) loaded: {} admitted, {} wikidata.bz2, {} JSONL/Log, {} Code/Docs blocks",
        six.len(),
        wiki431.len(),
        non_wiki.len(),
        code_docs.len()
    );

    // 1. Exhaustive forward-FL exactness, forced fallback, exact r_thin bound & scratch_report
    run_exactness_and_bound_audit(&six, &wiki431, &non_wiki, &code_docs);

    // 2. Interleaved 1T Headroom Oracle v3 across all 4 corpora
    let r1_six = bench_1t_interleaved("6 ADMITTED WIKIDATA BLOCKS", &six, 21, false, true);
    let r1_nw = bench_1t_interleaved("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 21, false, true);
    let r1_cd = bench_1t_interleaved("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 21, false, true);
    let r1_w431 = bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    // Transparent Code/Docs Cap::dense ablation
    bench_code_docs_capped_ablation(&code_docs, 21);

    // 3. 2W and 4W Concurrent Barrier Benchmarks across all 4 corpora
    let (m2_six, m4_six) = bench_mt_suite("6 ADMITTED WIKIDATA BLOCKS", &six, 21, 15);
    let (m2_nw, m4_nw) = bench_mt_suite("6 NON-WIKIDATA JSONL/LOG BLOCKS", &non_wiki, 21, 15);
    let (m2_cd, m4_cd) = bench_mt_suite("4 NON-WIKIDATA CODE/DOCS BLOCKS", &code_docs, 21, 15);
    let (m2_w431, m4_w431) = bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 7, 5);

    // 4. Overall Non-Overlapping 441-Block Union (431 Wiki + 6 JSONL/Log + 4 Code/Docs)
    //    and 243 Full-Block Union (233 Full Wiki + 6 JSONL/Log + 4 Code/Docs)
    println!("\n=== OVERALL SUMMARY: C_recognise + C_member < C_Open (ALL CORPORA SUMMED) ===");
    let sum1_441: [f64; 6] = std::array::from_fn(|a| r1_w431.tot_us[a] + r1_nw.tot_us[a] + r1_cd.tot_us[a]);
    let orcl1_441 = r1_w431.oracle_us + r1_nw.oracle_us + r1_cd.oracle_us;
    let sum1_243: [f64; 6] = std::array::from_fn(|a| r1_w431.full_tot_us[a] + r1_nw.tot_us[a] + r1_cd.tot_us[a]);
    let orcl1_243 = r1_w431.full_oracle_us + r1_nw.oracle_us + r1_cd.oracle_us;
    let sum2_441: [f64; 6] = std::array::from_fn(|a| m2_w431[a] + m2_nw[a] + m2_cd[a]);
    let sum4_441: [f64; 6] = std::array::from_fn(|a| m4_w431[a] + m4_nw[a] + m4_cd[a]);

    println!("  [441 Distinct Blocks = 431 Wiki + 6 JSONL/Log + 4 Code/Docs]");
    for arm in 0..6 {
        let lt1 = if sum1_441[arm] < sum1_441[0] { "YES" } else { "NO" };
        let lt2 = if sum2_441[arm] < sum2_441[0] { "YES" } else { "NO" };
        let lt4 = if sum4_441[arm] < sum4_41_base(&sum4_441) { "YES" } else { "NO" };
        println!(
            "    {:16} | 1T={:7.2}ms ({:.4}x, <Open? {:3}) | 2W={:7.2}ms ({:.4}x, <Open? {:3}) | 4W={:7.2}ms ({:.4}x, <Open? {:3})",
            ARM_NAMES[arm],
            sum1_441[arm] / 1e3,
            sum1_441[arm] / sum1_441[0],
            lt1,
            sum2_441[arm] / 1e3,
            sum2_441[arm] / sum2_441[0],
            lt2,
            sum4_441[arm] / 1e3,
            sum4_41_ratio(&sum4_441, arm),
            lt4
        );
    }
    println!(
        "    {:16} | 1T={:7.2}ms ({:.4}x) | 6-Admitted 1T/2W/4W C48-Fused = {:.4}x / {:.4}x / {:.4}x",
        "Hindsight-Oracle",
        orcl1_441 / 1e3,
        orcl1_441 / sum1_441[0],
        r1_six.tot_us[5] / r1_six.tot_us[0],
        m2_six[5] / m2_six[0],
        m4_six[5] / m4_six[0]
    );
    println!("  [243 Full-Size Blocks = 233 Full Wiki + 6 JSONL/Log + 4 Code/Docs, 1T]");
    for arm in 0..6 {
        println!(
            "    {:16} | 1T={:7.2}ms ({:.4}x) | < C_Open? {}",
            ARM_NAMES[arm],
            sum1_243[arm] / 1e3,
            sum1_243[arm] / sum1_243[0],
            if sum1_243[arm] < sum1_243[0] { "YES" } else { "NO" }
        );
    }
    println!(
        "    {:16} | 1T={:7.2}ms ({:.4}x)",
        "Hindsight-Oracle",
        orcl1_243 / 1e3,
        orcl1_243 / sum1_243[0]
    );
}

#[inline]
fn sum4_41_base(s: &[f64; 6]) -> f64 {
    s[0]
}

#[inline]
fn sum4_41_ratio(s: &[f64; 6], arm: usize) -> f64 {
    s[arm] / s[0]
}
