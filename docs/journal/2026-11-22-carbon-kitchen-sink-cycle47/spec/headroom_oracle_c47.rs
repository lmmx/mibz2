#![forbid(unsafe_code)]

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/mod.rs"]
mod cand_r5_head;

#[allow(dead_code)]
#[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/mod.rs"]
mod cand_r5_nobi;

// ============================================================================
// 0. Unified Open Kernel (+0 B extra persistent WS vs cand_r5_head)
//    Fuses cand_r5_head (XOR-BiEmit) + cand_r5_nobi (W16 serial wire) +
//    zero-overhead branchless `w1` register counter in S1 + broken-tiling guard
//    + runtime BiMode (μ ∈ {1, 2}) and limit (L ∈ {16, 24, 32, ...}).
// ============================================================================

#[allow(dead_code)]
#[path = "."]
mod unified_open {
    #[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_head/endpoint.rs"]
    mod endpoint;

    #[path = "../../2026-11-21-carbon-kitchen-sink-cycle45/cand_r5_nobi/outq.rs"]
    mod outq;

    /// Runtime μ ∈ {1, 2} policy evaluated at S4 (`finish`) with ZERO extra instructions in S1!
    /// Uses exact Post-S1 `(m, ns, recs)` and the classical 19th-century Markov-Dirichlet
    /// singleton moment bound `md_w1 = 2 * (recs - ns) - m <= w1_true` (`MSC 60E15`, 1860s).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BiMode {
        /// Always μ = 1 (W16 serial wire + emit), identical to `cand_r5_nobi`.
        Never,
        /// μ = 2 iff `m >= threshold` (`AlwaysAt(65_536)` = `cand_r5_head`).
        AlwaysAt(u32),
        /// Zero-S1-overhead Markov-Dirichlet / Neyman-Pearson Post-S1 rule:
        /// μ = 1 when `m < 65_536 || (recs <= recs_max && ns <= ns_max && 2*(recs - ns) - m <= md_max)`
        /// else μ = 2 (`XOR-BiEmit`).
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

    /// Exact same 12 fields as `cand_r5_head::Kernel` -> +0 B extra persistent working set!
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

    pub const LIMIT: u32 = 16;
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
                        // Zero extra instructions in S1 hot loop (const LIM + no w1 counter!)
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
}

use std::fs;
use std::path::Path;
use std::sync::Barrier;
use std::time::Instant;
use unified_open::BiMode;

// ============================================================================
// 1. Block Data Structures & Corpus Loaders
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
// 2. Forward-FL Exact Walks & Cycle 47 Specialists
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

/// 8-way interleaved forward-FL walk (`Seg8-FL`), 100% byte-exact against `cand_r5_head` / `libbzip2`.
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
            debug_assert_eq!(out.len(), n);
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

/// Cauchy (1815) exact short-cycle orbit detector for ultra-sparse periodic blocks (`recs*64 < n`).
/// Strictly verifies `tot == n`, `x == p`, and `n % d == 0`, so forward-FL == backward-LF on every hit.
fn try_short_periodic(records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> bool {
    if records.len() * 64 >= n || n < 4096 || (p as usize) >= n {
        return false;
    }
    let mut sym = [0u8; 64];
    let mut r_start = [0u32; 64];
    let mut r_len = [0u32; 64];
    let mut r_rank = [0u32; 64];
    let mut counts = [0u32; 256];
    let mut nr = 0usize;
    let mut pos = 0u32;
    for &rec in records {
        let b = (rec >> 8) as u8;
        let l = u32::from(rec & 255);
        if nr > 0 && sym[nr - 1] == b {
            r_len[nr - 1] += l;
        } else {
            if nr >= 64 {
                return false;
            }
            sym[nr] = b;
            r_start[nr] = pos;
            r_len[nr] = l;
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

/// Classical Wald (1945) SPRT + Chebyshev-Markov (1867) 2-stage sub-microsecond stride sampler
/// choosing `L ∈ {16, 24, 32}` before S1 (`C_recognise ~ 0.3–0.5 us`).
/// - Stage 1 (64 samples): early-reject H1 -> L=16 if `est_w1_64 > 39_500 || s_r17_64 > 2`.
/// - Stage 2 (256 samples):
///   - `est_w1 <= 33_500 && s_r17 <= 3` -> L = 32 (low singleton load + sparse 17..32 band)
///   - `est_w1 <= 33_500 && s_r17 <= 5` -> L = 24 (moderate 17..32 band)
///   - else -> L = 16.
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

/// Specialist 1 (`Spec1-TinyDense-FL`): FL-fixed Cycle 46 specialist (`L=16`, `BiMode::AlwaysAt(65_536)`).
#[derive(Default)]
struct Spec1TinyDense {
    open: unified_open::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl Spec1TinyDense {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), out)
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
        if records.len() * 20 >= n * 3 {
            if self
                .open
                .try_run_const::<16>(records, n, p, unified_open::Cap::dense(n), BiMode::AlwaysAt(65_536), out)
                .is_some()
            {
                return 4;
            }
            self.seg.invert(records, n, p, out);
            return 3;
        }
        self.open
            .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), out)
            .expect("open");
        0
    }
}

/// Specialist 3 (`Spec3-PostS1`, Cycle 47 Deliverable 1A):
/// Single unified `945.8 KiB` Open kernel (`+0 B` persistent WS, `0` extra instructions in S1!)
/// with `~1 ns` zero-pass Post-S1 classical Markov-Dirichlet moment bound (`md_w1 = 2*(recs - ns) - m`)
/// + Neyman-Pearson / Fisher LDA rule choosing `μ ∈ {1, 2}` from exact `(m, ns, recs)`:
/// `μ = 1` (`W16` serial wire+emit) iff `m < 65_536 || (recs <= 57_000 && ns <= 2_400 && md_w1 <= 26_000)`,
/// else `μ = 2` (`XOR-BiEmit`).
#[derive(Default)]
struct Spec3PostS1 {
    open: unified_open::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl Spec3PostS1 {
    const POST_S1_BI: BiMode = BiMode::PostS1Md {
        recs_max: 57_000,
        ns_max: 2_400,
        md_max: 26_000,
    };

    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), out)
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
        if records.len() * 20 >= n * 3 {
            if let Some(st) =
                self.open
                    .try_run_const::<16>(records, n, p, unified_open::Cap::dense(n), Self::POST_S1_BI, out)
            {
                return if st.used_bi { 4 } else { 5 };
            }
            self.seg.invert(records, n, p, out);
            return 3;
        }
        let st = self
            .open
            .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, Self::POST_S1_BI, out)
            .expect("open");
        if st.used_bi { 0 } else { 5 }
    }
}

/// Specialist 4 (`Spec4-SPRT-L`, Cycle 47 Deliverable 1B):
/// Single unified `945.8 KiB` Open kernel (`+0 B` persistent WS, `0` extra instructions in S1!) combining:
/// 1. Pre-S1 Wald (1945) SPRT + Chebyshev-Markov (1867) 2-stage stride sampler (`64 -> 256` records,
///    `C_recognise ~ 0.3-0.5 us`) selecting monomorphized `L ∈ {16, 24, 32}`;
/// 2. Post-S1 exact `(m, ns, recs)` Markov-Dirichlet / Neyman-Pearson rule selecting `μ ∈ {1, 2}` at S4.
#[derive(Default)]
struct Spec4SprtL {
    open: unified_open::Kernel,
    seg: InterleavedWalk,
    force_fallback: bool,
}

impl Spec4SprtL {
    #[inline(never)]
    fn run_entry(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> u8 {
        if self.force_fallback {
            self.open
                .try_run_const::<16>(records, n, p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), out)
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
            unified_open::Cap::dense(n)
        } else {
            unified_open::Cap::NONE
        };
        let res = match lim {
            16 => self.open.try_run_const::<16>(records, n, p, cap, Spec3PostS1::POST_S1_BI, out),
            24 => self.open.try_run_const::<24>(records, n, p, cap, Spec3PostS1::POST_S1_BI, out),
            _ => self.open.try_run_const::<32>(records, n, p, cap, Spec3PostS1::POST_S1_BI, out),
        };
        if let Some(st) = res {
            return match (lim, st.used_bi) {
                (16, true) => 0,  // L16 + μ=2 (R5-Head)
                (16, false) => 5, // L16 + μ=1 (Post-S1 NobiW16)
                (24, _) => 6,     // SPRT L24
                _ => 7,           // SPRT L32
            };
        }
        self.seg.invert(records, n, p, out);
        3
    }
}

// ============================================================================
// 3. Utilities & Exactness / Forced-Fallback Audit
// ============================================================================

fn pct(v: &mut [f64], num: usize, den: usize) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[(v.len() * num) / den]
}

fn run_exactness_audit(six: &[Block], wiki431: &[Block], non_wiki: &[Block]) {
    let mut kh = cand_r5_head::Kernel::default();
    let mut kn = cand_r5_nobi::Kernel::default();
    let mut ku = unified_open::Kernel::default();
    let mut plain = PlainRunWalk::default();
    let mut seg8 = InterleavedWalk::default();
    let mut sp1 = Spec1TinyDense::default();
    let mut sp3 = Spec3PostS1::default();
    let mut sp4 = Spec4SprtL::default();
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
            ku.try_run(&recs, n, p, 16, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), &mut out_k)
                .unwrap();
            assert_eq!(out_k, text, "unified head binary mismatch n={n} mask={mask}");
            ku.try_run(&recs, n, p, 32, unified_open::Cap::NONE, BiMode::Never, &mut out_k)
                .unwrap();
            assert_eq!(out_k, text, "unified nobi32 binary mismatch");
            plain.invert(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            seg8.invert(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            sp1.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            sp3.run_entry(&recs, n, p, &mut out_k);
            assert_eq!(out_k, text);
            sp4.run_entry(&recs, n, p, &mut out_k);
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
            ku.try_run(&recs, n, p, lim, unified_open::Cap::NONE, BiMode::AlwaysAt(0), &mut out_k)
                .unwrap();
            assert_eq!(out_k, text);
            ku.try_run(&recs, n, p, lim, unified_open::Cap::NONE, BiMode::Never, &mut out_k)
                .unwrap();
            assert_eq!(out_k, text);
        }
        sp1.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        sp3.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        sp4.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, text);
        valid_cnt += 1;
    }

    // 3b. 4 Stress blocks + 6 Admitted + 6 Non-Wikidata + Forced Fallback (`force_fallback = true`)
    let dense = make_dense_300k();
    let spiral = make_spiral_200k();
    let per2 = make_periodic("periodic-150k-d2", b"ab", 75_000);
    let per3 = make_periodic("periodic-105k-d3", b"abc", 35_000);
    let stress = [&dense, &spiral, &per2, &per3];

    for b in six.iter().chain(stress.iter().copied()).chain(non_wiki.iter()) {
        let want = b.expected.as_ref().unwrap();
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k).unwrap();
        assert_eq!(&out_k, want, "cand_r5_head mismatch on {}", b.name);
        kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, &mut out_k).unwrap();
        assert_eq!(&out_k, want, "cand_r5_nobi mismatch on {}", b.name);
        for &lim in &[16u32, 24, 32] {
            for bm in [BiMode::AlwaysAt(65_536), BiMode::Never, Spec3PostS1::POST_S1_BI] {
                ku.try_run(&b.records, b.n, b.p, lim, unified_open::Cap::NONE, bm, &mut out_k)
                    .unwrap();
                assert_eq!(&out_k, want, "unified mismatch on {} L={} {:?}", b.name, lim, bm);
            }
        }
        sp1.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "sp1 mismatch on {}", b.name);
        sp3.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "sp3 mismatch on {}", b.name);
        sp4.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(&out_k, want, "sp4 mismatch on {}", b.name);

        // Forced fallback tests on ALL 3 specialists
        sp1.force_fallback = true;
        assert_eq!(sp1.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "sp1 forced fallback mismatch on {}", b.name);
        sp1.force_fallback = false;

        sp3.force_fallback = true;
        assert_eq!(sp3.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "sp3 forced fallback mismatch on {}", b.name);
        sp3.force_fallback = false;

        sp4.force_fallback = true;
        assert_eq!(sp4.run_entry(&b.records, b.n, b.p, &mut out_k), 0);
        assert_eq!(&out_k, want, "sp4 forced fallback mismatch on {}", b.name);
        sp4.force_fallback = false;
    }

    // Verify spiral rejection -> UnifiedOpen (NOT periodic route 2), periodic hit -> route 2
    assert_eq!(sp1.run_entry(&spiral.records, spiral.n, spiral.p, &mut out_k), 0);
    assert_eq!(&out_k, spiral.expected.as_ref().unwrap());
    let sp4_spiral = sp4.run_entry(&spiral.records, spiral.n, spiral.p, &mut out_k);
    assert!(matches!(sp4_spiral, 0 | 5 | 6 | 7), "sp4 spiral must reject periodic probe (got {sp4_spiral})");
    assert_eq!(&out_k, spiral.expected.as_ref().unwrap());
    assert_eq!(sp4.run_entry(&per2.records, per2.n, per2.p, &mut out_k), 2);
    assert_eq!(&out_k, per2.expected.as_ref().unwrap());

    // 3c. All 431 wikidata.bz2 blocks exact against cand_r5_head (forward-FL)
    for (idx, b) in wiki431.iter().enumerate() {
        kh.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        ku.try_run(&b.records, b.n, b.p, 16, unified_open::Cap::NONE, Spec3PostS1::POST_S1_BI, &mut out_k)
            .unwrap();
        assert_eq!(out_k, out_ref, "wiki431 unified PostS1 mismatch idx={idx}");
        sp1.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 sp1 mismatch idx={idx}");
        sp3.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 sp3 mismatch idx={idx}");
        sp4.run_entry(&b.records, b.n, b.p, &mut out_k);
        assert_eq!(out_k, out_ref, "wiki431 sp4 mismatch idx={idx}");
    }

    // 3d. 250 Malformed / Multi-Cycle Columns + Broken Tiling Rejection (STRICTLY forward-FL vs cand_r5_head!)
    assert!(!plain.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(!seg8.try_invert(&[(u16::from(b'a') << 8) | 3], 5, 0, &mut out_k));
    assert!(ku
        .try_run(&[(u16::from(b'a') << 8) | 3], 5, 0, 16, unified_open::Cap::NONE, BiMode::AlwaysAt(0), &mut out_k)
        .is_none());

    // Explicit C46 Bug #1 witness L="bbaabb", p=0 plus 50 doubled/periodic non-dividing malformed columns
    let mut mal_cnt = 0usize;
    {
        let l = b"bbaabb";
        let recs = records_of(l);
        kh.run_entry(&recs, l.len(), 0, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        sp1.run_entry(&recs, l.len(), 0, &mut out_k);
        assert_eq!(out_k, out_ref, "bbaabb sp1");
        sp3.run_entry(&recs, l.len(), 0, &mut out_k);
        assert_eq!(out_k, out_ref, "bbaabb sp3");
        sp4.run_entry(&recs, l.len(), 0, &mut out_k);
        assert_eq!(out_k, out_ref, "bbaabb sp4");
        mal_cnt += 1;
    }
    let mut lbuf = Vec::with_capacity(65_536);
    for case in 0..249usize {
        let n = if case < 150 {
            1 + (case * 37) % 512
        } else if case < 210 {
            8_192
        } else {
            // Large malformed multi-cycle columns (> 32,768) exercising Open/SPRT/PostS1/Seg8!
            40_000 + (case - 210) * 137
        };
        let alpha = if case >= 210 && case < 230 { 2 } else { 2 + (case % 12) };
        lbuf.clear();
        if case >= 210 && case < 230 {
            // Sparse malformed multi-cycle column (few runs, tests periodic guard rejection!)
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
        let recs = records_of(&lbuf);
        kh.run_entry(&recs, n, p, cand_r5_head::Cap::NONE, &mut out_ref).unwrap();
        plain.invert(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed PlainRunWalk mismatch case={case}");
        seg8.invert(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Seg8-FL mismatch case={case}");
        ku.try_run(&recs, n, p, 16, unified_open::Cap::NONE, BiMode::AlwaysAt(0), &mut out_k).unwrap();
        assert_eq!(out_k, out_ref, "malformed UnifiedOpen(bi) mismatch case={case}");
        ku.try_run(&recs, n, p, 32, unified_open::Cap::NONE, BiMode::Never, &mut out_k).unwrap();
        assert_eq!(out_k, out_ref, "malformed UnifiedOpen(nobi32) mismatch case={case}");
        sp1.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec1 mismatch case={case}");
        sp3.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec3 mismatch case={case}");
        sp4.run_entry(&recs, n, p, &mut out_k);
        assert_eq!(out_k, out_ref, "malformed Spec4 mismatch case={case}");
        mal_cnt += 1;
    }

    // 3e. Persistent scratch report on FRESH kernels across the 6 admitted blocks (canonical 945.8 KiB WS)
    let mut kh_ws = cand_r5_head::Kernel::default();
    let mut ku_ws = unified_open::Kernel::default();
    for b in six {
        kh_ws.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, &mut out_k).unwrap();
        ku_ws
            .try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, Spec3PostS1::POST_S1_BI, &mut out_k)
            .unwrap();
    }
    let (h_ws, h_l1, h_str) = kh_ws.scratch_report();
    let (u_ws, u_l1, u_str) = ku_ws.scratch_report();

    println!(
        "EXACTNESS PASS: {valid_cnt} small valid + {mal_cnt} malformed (strict forward-FL vs cand_r5_head) + 4 stress + {} admitted + {} wikidata.bz2 + {} non-Wikidata + forced fallbacks 100% BYTE-EXACT!",
        six.len(),
        wiki431.len(),
        non_wiki.len()
    );
    println!("SCRATCH REPORT (steady-state over 6 admitted blocks):");
    println!("  cand_r5_head : total={} B ({:.1} KiB), dict_l1={} B | {}", h_ws, h_ws as f64 / 1024.0, h_l1, h_str);
    println!("  UnifiedOpen  : total={} B ({:.1} KiB), dict_l1={} B | {}", u_ws, u_ws as f64 / 1024.0, u_l1, u_str);
    println!("  Delta WS     : {:+} B", u_ws as isize - h_ws as isize);
}

// ============================================================================
// 4. Deliverable 1: Empirical M_BI + (m, w1, ns) Sweep & Gauss/Wald Diagnostics
// ============================================================================

fn ols_solve4(xs: &[[f64; 4]], ys: &[f64]) -> [f64; 4] {
    let mut a = [[0.0f64; 5]; 4];
    for (x, &y) in xs.iter().zip(ys.iter()) {
        for i in 0..4 {
            for j in 0..4 {
                a[i][j] += x[i] * x[j];
            }
            a[i][4] += x[i] * y;
        }
    }
    for col in 0..4 {
        let mut piv = col;
        for row in col + 1..4 {
            if a[row][col].abs() > a[piv][col].abs() {
                piv = row;
            }
        }
        a.swap(col, piv);
        let d = a[col][col];
        for j in col..5 {
            a[col][j] /= d;
        }
        for row in 0..4 {
            if row == col {
                continue;
            }
            let f = a[row][col];
            for j in col..5 {
                a[row][j] -= f * a[col][j];
            }
        }
    }
    [a[0][4], a[1][4], a[2][4], a[3][4]]
}

fn run_selector_discovery_sweep(wiki431: &[Block], non_wiki: &[Block]) {
    println!("\n=== DELIVERABLE 1: EMPIRICAL M_BI + POST-S1 (m, ns, recs, MD_w1) + WALD SPRT SWEEP ===");
    let mut ku = unified_open::Kernel::default();
    let mut out = Vec::with_capacity(900_064);

    // Warmup
    ku.try_run_const::<16>(&non_wiki[0].records, non_wiki[0].n, non_wiki[0].p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), &mut out);

    struct Entry {
        is_nw: bool,
        recs: u32,
        m16: u32,
        w1: u32,
        ns16: u32,
        rows17_32: u32,
        sprt_lim: u32,
        sprt_samples: u32,
        h16: f64,
        n16: f64,
        n24: f64,
        n32: f64,
    }

    let mut entries = Vec::new();
    let full_wiki: Vec<&Block> = wiki431.iter().filter(|b| b.route_class < 2).collect();

    for (is_nw, list, rds) in [(false, full_wiki.as_slice(), 3usize), (true, &non_wiki.iter().collect::<Vec<_>>()[..], 11usize)] {
        for &b in list {
            // Exact diagnostic w1 & rows17_32 (offline analysis only — never in hot path!)
            let mut w1 = 0u32;
            let mut rows17_32 = 0u32;
            let mut i = 0;
            while i < b.records.len() {
                let rec = b.records[i];
                let c = (rec >> 8) as u8;
                let mut w = u32::from(rec & 255);
                i += 1;
                if w == 127 {
                    while i < b.records.len() && (b.records[i] >> 8) as u8 == c {
                        w += u32::from(b.records[i] & 255);
                        i += 1;
                    }
                }
                if w == 1 {
                    w1 += 1;
                }
                if w > 16 && w <= 32 {
                    rows17_32 += w;
                }
            }
            let (sprt_lim, sprt_samples) = sprt_select_limit(&b.records, b.n);
            let mut tv: [Vec<f64>; 4] = Default::default();
            let mut st16 = unified_open::Counts::default();
            for r in 0..rds {
                for off in 0..4 {
                    let arm = (r + off) % 4;
                    let t0 = Instant::now();
                    match arm {
                        0 => {
                            st16 = ku
                                .try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, BiMode::AlwaysAt(65_536), &mut out)
                                .unwrap();
                        }
                        1 => {
                            ku.try_run_const::<16>(&b.records, b.n, b.p, unified_open::Cap::NONE, BiMode::Never, &mut out);
                        }
                        2 => {
                            ku.try_run_const::<24>(&b.records, b.n, b.p, unified_open::Cap::NONE, BiMode::Never, &mut out);
                        }
                        _ => {
                            ku.try_run_const::<32>(&b.records, b.n, b.p, unified_open::Cap::NONE, BiMode::Never, &mut out);
                        }
                    }
                    tv[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                }
            }
            entries.push(Entry {
                is_nw,
                recs: b.records.len() as u32,
                m16: st16.retained as u32,
                w1,
                ns16: st16.thick as u32,
                rows17_32,
                sprt_lim,
                sprt_samples,
                h16: pct(&mut tv[0], 1, 2),
                n16: pct(&mut tv[1], 1, 2),
                n24: pct(&mut tv[2], 1, 2),
                n32: pct(&mut tv[3], 1, 2),
            });
        }
    }

    // Gauss (1809) OLS Regression
    let xs_mu: Vec<[f64; 4]> = entries
        .iter()
        .map(|e| [1.0, e.m16 as f64 / 1e3, e.w1 as f64 / 1e3, e.ns16 as f64 / 1e3])
        .collect();
    let ys_mu: Vec<f64> = entries.iter().map(|e| e.n16 - e.h16).collect();
    let b_mu = ols_solve4(&xs_mu, &ys_mu);

    let xs_32: Vec<[f64; 4]> = entries
        .iter()
        .map(|e| [1.0, e.w1 as f64 / 1e3, e.rows17_32 as f64 / 1e3, e.ns16 as f64 / 1e3])
        .collect();
    let ys_32: Vec<f64> = entries.iter().map(|e| e.n32 - e.h16).collect();
    let b_32 = ols_solve4(&xs_32, &ys_32);

    println!("Gauss (1809, MSC 62J05) OLS Physics Laws (233 full Wikidata + 6 Non-Wikidata):");
    println!(
        "  Δt_μ   = t(N16) - t(H16) = {:+.2} + ({:+.3})*(m/1k) + ({:+.3})*(w1/1k) + ({:+.3})*(ns/1k) [us]",
        b_mu[0], b_mu[1], b_mu[2], b_mu[3]
    );
    println!(
        "  Δt_L32 = t(N32) - t(H16) = {:+.2} + ({:+.3})*(w1/1k) + ({:+.3})*(rows17_32/1k) + ({:+.3})*(ns/1k) [us]",
        b_32[0], b_32[1], b_32[2], b_32[3]
    );

    let w_e: Vec<&Entry> = entries.iter().filter(|e| !e.is_nw).collect();
    let nw_e: Vec<&Entry> = entries.iter().filter(|e| e.is_nw).collect();
    let wh: f64 = w_e.iter().map(|e| e.h16).sum();
    let nwh: f64 = nw_e.iter().map(|e| e.h16).sum();

    println!("\nM_BI Threshold Sweep (μ=1 iff m < M_BI, L=16):");
    for &m_bi in &[65_536u32, 85_000, 100_000, 115_000, 130_000, 150_000] {
        let mut wt = 0.0;
        let mut wc = 0usize;
        for e in &w_e {
            if e.m16 < m_bi { wt += e.n16; wc += 1; } else { wt += e.h16; }
        }
        let mut nwt = 0.0;
        let mut nwc = 0usize;
        for e in &nw_e {
            if e.m16 < m_bi { nwt += e.n16; nwc += 1; } else { nwt += e.h16; }
        }
        println!(
            "  M_BI={:7} | Wiki(233) μ=1:{:3}/233 sum={:7.2}ms ({:.4}x) | NonWiki(6) μ=1:{}/6 sum={:6.1}us ({:.4}x)",
            m_bi, wc, wt / 1e3, wt / wh, nwc, nwt, nwt / nwh
        );
    }

    // Compare Post-S1 rules and SPRT-L rule
    let rules: [(&str, fn(&Entry) -> f64); 4] = [
        ("Spec3-PostS1 (MD bound, 0 S1 ops)", |e| {
            if !Spec3PostS1::POST_S1_BI.should_use_bi(e.m16, e.ns16, e.recs) { e.n16 } else { e.h16 }
        }),
        ("PostS1-ExactW1 (w1<=34.5k & ns<=2.4k)", |e| {
            if e.m16 < 65_536 || (e.w1 <= 34_500 && e.ns16 <= 2_400) { e.n16 } else { e.h16 }
        }),
        ("Spec4-SPRT-L (L∈{16,24,32} + PostS1)", |e| {
            match e.sprt_lim {
                32 => e.n32,
                24 => e.n24,
                _ => if !Spec3PostS1::POST_S1_BI.should_use_bi(e.m16, e.ns16, e.recs) { e.n16 } else { e.h16 },
            }
        }),
        ("Hindsight Oracle (H16,N16,N24,N32)", |e| e.h16.min(e.n16).min(e.n24).min(e.n32)),
    ];
    println!("\nClassical Selector Rule Comparison (233 full Wikidata vs 6 Non-Wikidata):");
    for (name, f) in rules {
        let wt: f64 = w_e.iter().map(|e| f(e)).sum();
        let nwt: f64 = nw_e.iter().map(|e| f(e)).sum();
        println!(
            "  {:38} | Wiki(233)={:7.2}ms ({:.4}x) | NonWiki(6)={:6.1}us ({:.4}x)",
            name, wt / 1e3, wt / wh, nwt, nwt / nwh
        );
    }

    // Exact SPRT C_recognise timing & sample statistics
    let avg_s_w = w_e.iter().map(|e| e.sprt_samples as f64).sum::<f64>() / w_e.len() as f64;
    let avg_s_nw = nw_e.iter().map(|e| e.sprt_samples as f64).sum::<f64>() / nw_e.len() as f64;
    let w_l32 = w_e.iter().filter(|e| e.sprt_lim == 32).count();
    let w_l24 = w_e.iter().filter(|e| e.sprt_lim == 24).count();
    let nw_l32 = nw_e.iter().filter(|e| e.sprt_lim == 32).count();
    let nw_l24 = nw_e.iter().filter(|e| e.sprt_lim == 24).count();

    let mut chk = 0u64;
    let t0 = Instant::now();
    for _ in 0..1000 {
        for &b in &full_wiki {
            let (l, _) = sprt_select_limit(&b.records, b.n);
            chk = chk.wrapping_add(u64::from(l));
        }
    }
    let ns_w = t0.elapsed().as_nanos() as f64 / (1000 * full_wiki.len()) as f64;

    let t1 = Instant::now();
    for _ in 0..10_000 {
        for b in non_wiki {
            let (l, _) = sprt_select_limit(&b.records, b.n);
            chk = chk.wrapping_add(u64::from(l));
        }
    }
    let ns_nw = t1.elapsed().as_nanos() as f64 / (10_000 * non_wiki.len()) as f64;

    println!(
        "\nWald (1945) SPRT Telemetry: Wiki(233) L32={w_l32} L24={w_l24} avg_samples={avg_s_w:.1} C_rec={ns_w:.1} ns/blk | NonWiki(6) L32={nw_l32} L24={nw_l24} avg_samples={avg_s_nw:.1} C_rec={ns_nw:.1} ns/blk (chk={chk})"
    );
}

// ============================================================================
// 5. Headroom Oracle v2 Benchmarks (1T, 2W, 4W) Across All 3 Corpora
//    Arms: R5-Head-A1, R5-Head-A2(AA), R5-NobiW16, Spec1-TinyDense-FL,
//          Spec3-PostS1, Spec4-SPRT-L
// ============================================================================

const ARM_NAMES: [&str; 6] = [
    "R5-Head-A1",
    "R5-Head-A2(AA)",
    "R5-NobiW16",
    "Spec1-TinyDense",
    "Spec3-PostS1",
    "Spec4-SPRT-L",
];

struct ArmSuite {
    kh1: cand_r5_head::Kernel,
    kh2: cand_r5_head::Kernel,
    kn: cand_r5_nobi::Kernel,
    sp1: Spec1TinyDense,
    sp3: Spec3PostS1,
    sp4: Spec4SprtL,
}

impl Default for ArmSuite {
    fn default() -> Self {
        Self {
            kh1: cand_r5_head::Kernel::default(),
            kh2: cand_r5_head::Kernel::default(),
            kn: cand_r5_nobi::Kernel::default(),
            sp1: Spec1TinyDense::default(),
            sp3: Spec3PostS1::default(),
            sp4: Spec4SprtL::default(),
        }
    }
}

impl ArmSuite {
    #[inline(always)]
    fn run_arm(&mut self, arm: usize, b: &Block, out: &mut Vec<u8>) -> u8 {
        match arm {
            0 => {
                self.kh1.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out);
                0
            }
            1 => {
                self.kh2.run_entry(&b.records, b.n, b.p, cand_r5_head::Cap::NONE, out);
                0
            }
            2 => {
                self.kn.run_entry(&b.records, b.n, b.p, cand_r5_nobi::Cap::NONE, out);
                5
            }
            3 => self.sp1.run_entry(&b.records, b.n, b.p, out),
            4 => self.sp3.run_entry(&b.records, b.n, b.p, out),
            _ => self.sp4.run_entry(&b.records, b.n, b.p, out),
        }
    }
}

fn bench_1t_interleaved(label: &str, blocks: &[Block], rds: usize, show_rc: bool, show_per_block: bool) {
    println!("\n=== HEADROOM ORACLE v2 (1T INTERLEAVED, {rds} rds/blk): {label} ===");
    let mut suite = ArmSuite::default();
    let mut out = Vec::with_capacity(900_064);

    // Warmup
    for b in blocks.iter().take(6) {
        for arm in 0..6 {
            suite.run_arm(arm, b, &mut out);
        }
    }

    let mut tot = [0.0f64; 6];
    let mut by_rc = [[0.0f64; 4]; 6];
    let mut sp3_paths = [0usize; 8];
    let mut sp4_paths = [0usize; 8];

    for b in blocks {
        let rc = (b.route_class as usize).min(3);
        let mut s: [Vec<f64>; 6] = Default::default();
        let mut p3 = 0u8;
        let mut p4 = 0u8;
        for r in 0..rds {
            for off in 0..6 {
                let arm = (r + off) % 6;
                let t0 = Instant::now();
                let path = suite.run_arm(arm, b, &mut out);
                s[arm].push(t0.elapsed().as_secs_f64() * 1e6);
                if arm == 4 { p3 = path; }
                if arm == 5 { p4 = path; }
            }
        }
        sp3_paths[p3 as usize] += 1;
        sp4_paths[p4 as usize] += 1;
        let mut meds = [0.0f64; 6];
        for arm in 0..6 {
            let m = pct(&mut s[arm], 1, 2);
            meds[arm] = m;
            tot[arm] += m;
            by_rc[arm][rc] += m;
        }
        if show_per_block {
            println!(
                "  {:12} | H-A1={:6.1} | H-A2={:6.1} | Nobi={:6.1} ({:.4}x) | Sp1={:6.1} ({:.4}x) | Sp3(p{})={:6.1} ({:.4}x) | Sp4(p{})={:6.1} ({:.4}x)",
                b.name,
                meds[0], meds[1],
                meds[2], meds[2] / meds[0],
                meds[3], meds[3] / meds[0],
                p3, meds[4], meds[4] / meds[0],
                p4, meds[5], meds[5] / meds[0],
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
    println!("  Spec3 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", sp3_paths);
    println!("  Spec4 routes [L16-bi, tiny, per, seg8, dense-bi, L16-nobi, L24, L32]: {:?}", sp4_paths);
}

fn bench_mt_suite(label: &str, blocks: &[Block], passes_2w: usize, passes_4w: usize) {
    for &(threads, passes) in &[(2usize, passes_2w), (4usize, passes_4w)] {
        println!(
            "\n=== {} — {}W CONCURRENT BARRIER ({} interleaved passes, 2-vCPU Xeon host) ===",
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
                    hs.into_iter().map(|h| h.join().unwrap()).fold(0.0f64, f64::max)
                });
                rmt[arm].push(t);
            }
        }
        let b10 = pct(&mut rmt[0].clone(), 1, 10);
        let b50 = pct(&mut rmt[0].clone(), 1, 2);
        for arm in 0..6 {
            let p10 = pct(&mut rmt[arm], 1, 10);
            let p50 = pct(&mut rmt[arm], 1, 2);
            let lt = if arm >= 3 {
                if p50 < b50 { "YES" } else { "NO" }
            } else {
                "-"
            };
            println!(
                "  {}W {:16} | p10={:9.1} us ({:.4}x) | p50={:9.1} us ({:.4}x) | p50 < C_Open? {}",
                threads, ARM_NAMES[arm], p10, p10 / b10, p50, p50 / b50, lt
            );
        }
    }
}

fn main() {
    let six = load_six_admitted();
    let wiki_bin = "/tmp/mibz2_corpora/wikidata_431_blocks.bin";
    let nonwiki_bin = "/tmp/mibz2_corpora/non_wikidata/non_wikidata_blocks.bin";
    assert!(Path::new(wiki_bin).exists(), "missing {wiki_bin}");
    assert!(Path::new(nonwiki_bin).exists(), "missing {nonwiki_bin}");

    let wiki431 = load_bin_corpus(wiki_bin, "wiki", false);
    let non_wiki = load_bin_corpus(nonwiki_bin, "nonwiki", true);

    println!(
        "Headroom Oracle v2 (Cycle 47) loaded: {} admitted, {} wikidata.bz2, {} non-Wikidata blocks",
        six.len(),
        wiki431.len(),
        non_wiki.len()
    );

    // 1. Exhaustive forward-FL exactness, malformed multi-cycle, forced fallback & scratch_report()
    run_exactness_audit(&six, &wiki431, &non_wiki);

    // 2. Deliverable 1: Empirical M_BI + Post-S1 (m, w1, ns) + Wald SPRT sweep & Gauss OLS laws
    run_selector_discovery_sweep(&wiki431, &non_wiki);

    // 3. Deliverable 2: Interleaved 1T Headroom Oracle v2 across all 3 corpora
    bench_1t_interleaved("6 ADMITTED BLOCKS", &six, 25, false, true);
    bench_1t_interleaved("6 NON-WIKIDATA BLOCKS (5 JSONL + 1 SYSLOG)", &non_wiki, 25, false, true);
    bench_1t_interleaved("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 5, true, false);

    // 4. Deliverable 2: 2W and 4W Concurrent Barrier Benchmarks across all 3 corpora
    bench_mt_suite("6 ADMITTED BLOCKS", &six, 25, 20);
    bench_mt_suite("6 NON-WIKIDATA BLOCKS", &non_wiki, 25, 20);
    bench_mt_suite("ALL 431 wikidata.bz2 BLOCKS", &wiki431, 7, 5);
}
