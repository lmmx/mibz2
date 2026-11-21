//! Carbon Cycle 25 artifact: `P2Serial + P2RegBurst6` (power-of-two bounds-check masking + `NoLong`
//! specialization) against `origin/master` (`90e67196`) and Cycle 23 `RegBurst6`.
//! All code is `#![forbid(unsafe_code)]`.
#![forbid(unsafe_code)]
use mibz2::bwt::{InverseBwt, LazyCells};
use mibz2::{runs::Runs, Kernels};

#[derive(Default, Clone, Copy, Debug)]
#[allow(dead_code)]
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
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    strips: Vec<Strip>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    word_len: Vec<u16>,
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0;
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        visit(c, pos, width);
        pos += width;
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
    strips: &'a mut Vec<Strip>,
    f: First,
}

impl Pass<'_> {
    #[inline(always)]
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<QUEUE>(c, pos, width, true, p);
            return;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<QUEUE>(c, pos, width, false, p);
        } else {
            self.take_marked::<QUEUE>(c, pos, width, p, limit);
        }
    }

    #[cold]
    #[inline(never)]
    fn take_marked<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<QUEUE>(c, x, w, keep, p)
        });
    }

    #[inline(always)]
    fn piece<const QUEUE: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            if QUEUE {
                self.queues.push(a, f.m, w, &f.thin);
            }
            f.thin[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip {
                from: x,
                to: f.all[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

#[derive(Clone, Copy)]
struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            all: [0; 256],
            thin: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
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

impl Kernel {
    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32) -> Pass<'a> {
        self.strips.clear();
        Pass {
            queues: outq::Queues::new(out, &mut self.spill, limit),
            strips: &mut self.strips,
            f: First::default(),
        }
    }

    fn construct_capped(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(Counts, u32, u32)> {
        out.resize(n as usize + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m);
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, limit)
        });
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish(&f, n, &out[..], &heads))
    }

    fn finish(
        &mut self,
        f: &First,
        n: u32,
        out: &[u8],
        heads: &outq::Heads,
    ) -> (Counts, u32, u32) {
        let First {
            all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;
        let mut stats = Counts {
            n: n as usize,
            runs,
            ..Counts::default()
        };
        assert!(m > 0 && all.iter().sum::<u32>() == n);
        stats.retained = m as usize;
        stats.thick = self.strips.len();
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.strips.len();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let p2_m = 1usize << bits;
        let need = self.arena_words(ns, m as usize).max(p2_m);
        if self.edges.len() < need {
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        let image_order = &mut self.edges[..ns];
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);
        let cursors = self.induced(n, &thin, &mut stats);
        stats.bands = self.bands.len();
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < p2_m {
            self.edges.resize(p2_m, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        let mut k = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(out, &self.spill, heads, a);
            for b in &self.bands[cursors[a]..end] {
                let span = (b.end - k) as usize;
                let dst = k.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    self.extra[dst..dst + span].fill(b.code);
                }
                queue.wire(&mut self.edges[dst..dst + span], pack);
                k = b.end;
            }
        }
        (stats, mark, bits)
    }

    pub fn try_run(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<Counts> {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, bits) = self.construct_capped(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, bits, stats.longest + 1 > INLINE, out);
        Some(stats)
    }

    fn walk(
        &mut self,
        n: usize,
        m: usize,
        mark: u32,
        bits: u32,
        has_long: bool,
        out: &mut Vec<u8>,
    ) {
        let use_burst = self.extra.is_empty() && m >= BURST_MIN_M;
        let code_cap = inline_words_p2(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
            &mut self.word_len,
            use_burst,
        );
        if use_burst {
            emit_p2_regburst6(
                &self.edges[..(1usize << bits)],
                &self.spelling,
                &self.offsets,
                &self.inline16[..code_cap],
                &self.lens[..code_cap],
                &self.word_len[..code_cap],
                m,
                n,
                mark,
                bits,
                has_long,
                out,
            );
        } else {
            emit_p2_serial(
                &self.edges[..(1usize << bits)],
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16[..code_cap],
                &self.lens[..code_cap],
                m,
                n,
                mark,
                bits,
                has_long,
                out,
            );
        }
    }
}

const BURST_MIN_M: usize = 140_000;
pub const LIMIT: u32 = 16;
const DENSE: (usize, usize) = (7, 25);
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

#[inline]
fn lazy_counts_admit(records: usize, n: usize) -> bool {
    records * 20 < n * 3 && records * 64 >= n
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub type OpenCells = OpenCellsOf<false>;

#[derive(Default)]
pub struct OpenCellsOf<const FORCED: bool> {
    open: Kernel,
    lazy: LazyCells,
}

impl<const FORCED: bool> OpenCellsOf<FORCED> {
    pub fn cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if FORCED || lazy_counts_admit(records, n) {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }
}

impl<const FORCED: bool> InverseBwt for OpenCellsOf<FORCED> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if let Some(cap) = Self::cap(r.records().len(), n, p) {
            if self.open.try_run(r.records(), n, p, LIMIT, cap, out).is_some() {
                return true;
            }
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

const INLINE: usize = 16;
const LONG: u8 = 0x80;

/// Builds `inline16`, `lens`, and (when `need_wlen`) `word_len` padded to `code_cap = next_power_of_two`,
/// so `code & (code_cap - 1)` eliminates every safe-Rust bounds check on dictionary lookups!
/// Padded slots `[256 + words .. code_cap)` get `lens = 1` (`!= LONG`) so `LONG` guarantees `code - 256 < words`.
fn inline_words_p2(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    word_len: &mut Vec<u16>,
    need_wlen: bool,
) -> usize {
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
    let words = offsets.len().saturating_sub(1);
    let code_cap = (256 + words).next_power_of_two();
    if inline16.capacity() < code_cap {
        inline16.reserve_exact(code_cap - inline16.len());
    }
    if lens.capacity() < code_cap {
        lens.reserve_exact(code_cap - lens.len());
    }
    if need_wlen {
        if word_len.len() < 256 {
            word_len.clear();
            word_len.resize(256, 1);
        }
        word_len.truncate(256);
        if word_len.capacity() < code_cap {
            word_len.reserve_exact(code_cap - word_len.len());
        }
        for w in offsets.windows(2) {
            let word = &spelling[w[0] as usize..w[1] as usize];
            let mut v = [0; INLINE];
            let head = word.len().min(INLINE);
            v[..head].copy_from_slice(&word[..head]);
            inline16.push(v);
            lens.push(if word.len() <= INLINE {
                word.len() as u8
            } else {
                LONG
            });
            word_len.push(word.len().min(u16::MAX as usize) as u16);
        }
        word_len.resize(code_cap, 1);
    } else {
        for w in offsets.windows(2) {
            let word = &spelling[w[0] as usize..w[1] as usize];
            let mut v = [0; INLINE];
            let head = word.len().min(INLINE);
            v[..head].copy_from_slice(&word[..head]);
            inline16.push(v);
            lens.push(if word.len() <= INLINE {
                word.len() as u8
            } else {
                LONG
            });
        }
    }
    inline16.resize(code_cap, [0; INLINE]);
    lens.resize(code_cap, 1);
    code_cap
}

#[allow(clippy::too_many_arguments)]
fn emit_p2_serial(
    edges_p2: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16_p2: &[[u8; INLINE]],
    lens_p2: &[u8],
    m: usize,
    n: usize,
    mark: u32,
    bits: u32,
    has_long: bool,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mut at = 0usize;
    let mask = (edges_p2.len() - 1) as u32;
    let cmask = inline16_p2.len() - 1;
    let mut j = mark & mask;

    if extra.is_empty() && !has_long {
        // Ultra-tight branch-minimal fast path when all words <= 16 B!
        for _ in 0..m {
            let edge = edges_p2[(j & mask) as usize];
            let code = ((edge >> bits) as usize) & cmask;
            j = edge & mask;
            out[at..at + INLINE].copy_from_slice(&inline16_p2[code]);
            at += usize::from(lens_p2[code]);
            if j == mark || at >= n {
                break;
            }
        }
    } else if extra.is_empty() {
        for _ in 0..m {
            let edge = edges_p2[(j & mask) as usize];
            let code = ((edge >> bits) as usize) & cmask;
            j = edge & mask;
            let len = lens_p2[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16_p2[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
    } else {
        for _ in 0..m {
            let edge = edges_p2[(j & mask) as usize];
            let code = (extra[j as usize] as usize) & cmask;
            j = edge & mask;
            let len = lens_p2[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16_p2[code]);
                at += usize::from(len);
            } else {
                let w = code - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
    }
    out.truncate(at.min(n));
    repeat(n, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_p2_regburst6(
    edges_p2: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16_p2: &[[u8; INLINE]],
    lens_p2: &[u8],
    word_len_p2: &[u16],
    m: usize,
    n: usize,
    mark: u32,
    bits: u32,
    has_long: bool,
    out: &mut Vec<u8>,
) {
    let m32 = m as u32;
    if word_len_p2.contains(&u16::MAX) {
        emit_p2_serial(
            edges_p2,
            &[],
            spelling,
            offsets,
            inline16_p2,
            lens_p2,
            m,
            n,
            mark,
            bits,
            has_long,
            out,
        );
        return;
    }

    let shift = bits.saturating_sub(8);
    let cm = (1u32 << shift) - 1;
    let rem_low = mark & cm;
    let stride = cm + 1;
    let mut cuts = [0u32; 512];
    let mut n_cuts = 0usize;
    let mut q = 0u32;
    while let Some(r) = q.checked_mul(stride).map(|v| v + rem_low) {
        if r >= m32 || n_cuts >= 512 {
            break;
        }
        cuts[n_cuts] = r;
        n_cuts += 1;
        q += 1;
    }
    if n_cuts < 12 {
        emit_p2_serial(
            edges_p2,
            &[],
            spelling,
            offsets,
            inline16_p2,
            lens_p2,
            m,
            n,
            mark,
            bits,
            has_long,
            out,
        );
        return;
    }

    let mask = (edges_p2.len() - 1) as u32;
    let cmask = inline16_p2.len() - 1;
    let mut seg_len = [0u32; 512];
    let mut seg_next = [u16::MAX; 512];

    // === Stage 1: Zero-Bounds-Check 6-Lane Scout Burst (`sc_cid` on stack, only 12 scalar GPRs live!) ===
    let mut next_c = 6usize;
    let mut sc_cid = [0usize, 1, 2, 3, 4, 5];
    let mut active = 6usize;
    let (mut j0, mut j1, mut j2, mut j3, mut j4, mut j5) =
        (cuts[0], cuts[1], cuts[2], cuts[3], cuts[4], cuts[5]);
    let (mut a0, mut a1, mut a2, mut a3, mut a4, mut a5) = (0u32, 0, 0, 0, 0, 0);
    let mut guard = 0usize;

    while active > 0 {
        if active == 6 {
            loop {
                let e0 = edges_p2[(j0 & mask) as usize];
                let e1 = edges_p2[(j1 & mask) as usize];
                let e2 = edges_p2[(j2 & mask) as usize];
                let e3 = edges_p2[(j3 & mask) as usize];
                let e4 = edges_p2[(j4 & mask) as usize];
                let e5 = edges_p2[(j5 & mask) as usize];
                a0 += u32::from(word_len_p2[((e0 >> bits) as usize) & cmask]);
                a1 += u32::from(word_len_p2[((e1 >> bits) as usize) & cmask]);
                a2 += u32::from(word_len_p2[((e2 >> bits) as usize) & cmask]);
                a3 += u32::from(word_len_p2[((e3 >> bits) as usize) & cmask]);
                a4 += u32::from(word_len_p2[((e4 >> bits) as usize) & cmask]);
                a5 += u32::from(word_len_p2[((e5 >> bits) as usize) & cmask]);
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
                emit_p2_serial(
                    edges_p2,
                    &[],
                    spelling,
                    offsets,
                    inline16_p2,
                    lens_p2,
                    m,
                    n,
                    mark,
                    bits,
                    has_long,
                    out,
                );
                return;
            }
            macro_rules! check_scout {
                ($idx:expr, $jk:ident, $ak:ident) => {
                    if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                        let cid = sc_cid[$idx];
                        seg_len[cid] = $ak;
                        seg_next[cid] = ($jk >> shift) as u16;
                        if next_c < n_cuts {
                            sc_cid[$idx] = next_c;
                            $jk = cuts[next_c];
                            $ak = 0;
                            next_c += 1;
                        } else {
                            sc_cid[$idx] = usize::MAX;
                            active -= 1;
                        }
                    }
                };
            }
            check_scout!(0, j0, a0);
            check_scout!(1, j1, a1);
            check_scout!(2, j2, a2);
            check_scout!(3, j3, a3);
            check_scout!(4, j4, a4);
            check_scout!(5, j5, a5);
        } else {
            macro_rules! step_drain {
                ($idx:expr, $jk:ident, $ak:ident) => {
                    if sc_cid[$idx] != usize::MAX {
                        let e = edges_p2[($jk & mask) as usize];
                        $ak += u32::from(word_len_p2[((e >> bits) as usize) & cmask]);
                        $jk = e & mask;
                        guard += 1;
                        if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                            let cid = sc_cid[$idx];
                            seg_len[cid] = $ak;
                            seg_next[cid] = ($jk >> shift) as u16;
                            sc_cid[$idx] = usize::MAX;
                            active -= 1;
                        }
                    }
                };
            }
            step_drain!(0, j0, a0);
            step_drain!(1, j1, a1);
            step_drain!(2, j2, a2);
            step_drain!(3, j3, a3);
            step_drain!(4, j4, a4);
            step_drain!(5, j5, a5);
            if guard > 2 * m + 64 {
                emit_p2_serial(
                    edges_p2,
                    &[],
                    spelling,
                    offsets,
                    inline16_p2,
                    lens_p2,
                    m,
                    n,
                    mark,
                    bits,
                    has_long,
                    out,
                );
                return;
            }
        }
    }

    // === Stage 2: Macro-Cycle Prefix Sum ===
    let mark_cid = (mark >> shift) as usize;
    let mut visited = [false; 512];
    let mut t_row = [0u32; 512];
    let mut t_start = [0usize; 512];
    let mut t_len = [0u32; 512];
    let mut n_tasks = 0usize;
    let mut cur_cid = mark_cid;
    let mut total_at = 0usize;
    let mut tail_row = None;

    while total_at < n {
        if visited[cur_cid] {
            if cur_cid == mark_cid {
                break;
            } else {
                emit_p2_serial(
                    edges_p2,
                    &[],
                    spelling,
                    offsets,
                    inline16_p2,
                    lens_p2,
                    m,
                    n,
                    mark,
                    bits,
                    has_long,
                    out,
                );
                return;
            }
        }
        visited[cur_cid] = true;
        let slen = seg_len[cur_cid] as usize;
        let nxt = seg_next[cur_cid] as usize;
        if nxt >= n_cuts {
            emit_p2_serial(
                edges_p2,
                &[],
                spelling,
                offsets,
                inline16_p2,
                lens_p2,
                m,
                n,
                mark,
                bits,
                has_long,
                out,
            );
            return;
        }
        if total_at + slen > n {
            tail_row = Some(cuts[cur_cid]);
            break;
        }
        if slen > 0 {
            t_row[n_tasks] = cuts[cur_cid];
            t_start[n_tasks] = total_at;
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
        emit_p2_serial(
            edges_p2,
            &[],
            spelling,
            offsets,
            inline16_p2,
            lens_p2,
            m,
            n,
            mark,
            bits,
            has_long,
            out,
        );
        return;
    }

    // === Stage 3: Zero-Bounds-Check 6-Lane Sloppy Emit Burst ===
    out.resize(n + INLINE, 0);
    let mut next_t = 6usize;
    let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5) =
        (t_row[0], t_row[1], t_row[2], t_row[3], t_row[4], t_row[5]);
    let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5) = (
        t_start[0],
        t_start[1],
        t_start[2],
        t_start[3],
        t_start[4],
        t_start[5],
    );
    let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5) =
        (t_len[0], t_len[1], t_len[2], t_len[3], t_len[4], t_len[5]);

    while (rem0 | rem1 | rem2 | rem3 | rem4 | rem5) != 0 {
        if (rem0 != 0) & (rem1 != 0) & (rem2 != 0) & (rem3 != 0) & (rem4 != 0) & (rem5 != 0) {
            if !has_long {
                // Zero-LONG-branch burst when all block words <= 16 B!
                loop {
                    let e0 = edges_p2[(rj0 & mask) as usize];
                    let e1 = edges_p2[(rj1 & mask) as usize];
                    let e2 = edges_p2[(rj2 & mask) as usize];
                    let e3 = edges_p2[(rj3 & mask) as usize];
                    let e4 = edges_p2[(rj4 & mask) as usize];
                    let e5 = edges_p2[(rj5 & mask) as usize];
                    let k0 = ((e0 >> bits) as usize) & cmask;
                    let k1 = ((e1 >> bits) as usize) & cmask;
                    let k2 = ((e2 >> bits) as usize) & cmask;
                    let k3 = ((e3 >> bits) as usize) & cmask;
                    let k4 = ((e4 >> bits) as usize) & cmask;
                    let k5 = ((e5 >> bits) as usize) & cmask;
                    let (l0, l1, l2, l3, l4, l5) = (
                        lens_p2[k0],
                        lens_p2[k1],
                        lens_p2[k2],
                        lens_p2[k3],
                        lens_p2[k4],
                        lens_p2[k5],
                    );
                    rj0 = e0 & mask;
                    out[at0..at0 + INLINE].copy_from_slice(&inline16_p2[k0]);
                    at0 += usize::from(l0);
                    rem0 -= u32::from(l0);

                    rj1 = e1 & mask;
                    out[at1..at1 + INLINE].copy_from_slice(&inline16_p2[k1]);
                    at1 += usize::from(l1);
                    rem1 -= u32::from(l1);

                    rj2 = e2 & mask;
                    out[at2..at2 + INLINE].copy_from_slice(&inline16_p2[k2]);
                    at2 += usize::from(l2);
                    rem2 -= u32::from(l2);

                    rj3 = e3 & mask;
                    out[at3..at3 + INLINE].copy_from_slice(&inline16_p2[k3]);
                    at3 += usize::from(l3);
                    rem3 -= u32::from(l3);

                    rj4 = e4 & mask;
                    out[at4..at4 + INLINE].copy_from_slice(&inline16_p2[k4]);
                    at4 += usize::from(l4);
                    rem4 -= u32::from(l4);

                    rj5 = e5 & mask;
                    out[at5..at5 + INLINE].copy_from_slice(&inline16_p2[k5]);
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
            } else {
                loop {
                    let e0 = edges_p2[(rj0 & mask) as usize];
                    let e1 = edges_p2[(rj1 & mask) as usize];
                    let e2 = edges_p2[(rj2 & mask) as usize];
                    let e3 = edges_p2[(rj3 & mask) as usize];
                    let e4 = edges_p2[(rj4 & mask) as usize];
                    let e5 = edges_p2[(rj5 & mask) as usize];
                    let k0 = ((e0 >> bits) as usize) & cmask;
                    let k1 = ((e1 >> bits) as usize) & cmask;
                    let k2 = ((e2 >> bits) as usize) & cmask;
                    let k3 = ((e3 >> bits) as usize) & cmask;
                    let k4 = ((e4 >> bits) as usize) & cmask;
                    let k5 = ((e5 >> bits) as usize) & cmask;
                    let (l0, l1, l2, l3, l4, l5) = (
                        lens_p2[k0],
                        lens_p2[k1],
                        lens_p2[k2],
                        lens_p2[k3],
                        lens_p2[k4],
                        lens_p2[k5],
                    );
                    if ((l0 | l1 | l2 | l3 | l4 | l5) & LONG) != 0 {
                        macro_rules! step_cold {
                            ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                                $rjk = $ek & mask;
                                if $lk != LONG {
                                    out[$atk..$atk + INLINE].copy_from_slice(&inline16_p2[$kk]);
                                    $atk += usize::from($lk);
                                    $remk = $remk.saturating_sub(u32::from($lk));
                                } else {
                                    let w = $kk - 256;
                                    let word =
                                        &spelling[offsets[w] as usize..offsets[w + 1] as usize];
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
                    out[at0..at0 + INLINE].copy_from_slice(&inline16_p2[k0]);
                    at0 += usize::from(l0);
                    rem0 -= u32::from(l0);

                    rj1 = e1 & mask;
                    out[at1..at1 + INLINE].copy_from_slice(&inline16_p2[k1]);
                    at1 += usize::from(l1);
                    rem1 -= u32::from(l1);

                    rj2 = e2 & mask;
                    out[at2..at2 + INLINE].copy_from_slice(&inline16_p2[k2]);
                    at2 += usize::from(l2);
                    rem2 -= u32::from(l2);

                    rj3 = e3 & mask;
                    out[at3..at3 + INLINE].copy_from_slice(&inline16_p2[k3]);
                    at3 += usize::from(l3);
                    rem3 -= u32::from(l3);

                    rj4 = e4 & mask;
                    out[at4..at4 + INLINE].copy_from_slice(&inline16_p2[k4]);
                    at4 += usize::from(l4);
                    rem4 -= u32::from(l4);

                    rj5 = e5 & mask;
                    out[at5..at5 + INLINE].copy_from_slice(&inline16_p2[k5]);
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
            }
            macro_rules! refill_emit {
                ($rjk:ident, $atk:ident, $remk:ident) => {
                    if $remk == 0 && next_t < n_tasks {
                        $rjk = t_row[next_t];
                        $atk = t_start[next_t];
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
                        let e = edges_p2[($rjk & mask) as usize];
                        let k = ((e >> bits) as usize) & cmask;
                        $rjk = e & mask;
                        let l = lens_p2[k];
                        if l != LONG {
                            out[$atk..$atk + INLINE].copy_from_slice(&inline16_p2[k]);
                            $atk += usize::from(l);
                            $remk = $remk.saturating_sub(u32::from(l));
                        } else {
                            let w = k - 256;
                            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
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

    // === Stage 4: Exact 15-Byte Slop Restoration at Cut Starts ===
    for t in 1..n_tasks {
        let mut rj = t_row[t];
        let mut rat = t_start[t];
        let rend = (rat + INLINE - 1).min(total_at);
        while rat < rend {
            let e = edges_p2[(rj & mask) as usize];
            let k = ((e >> bits) as usize) & cmask;
            rj = e & mask;
            let l = lens_p2[k];
            let w = if l != LONG {
                &inline16_p2[k][..usize::from(l)]
            } else {
                let wi = k - 256;
                &spelling[offsets[wi] as usize..offsets[wi + 1] as usize]
            };
            let tk = w.len().min(total_at - rat);
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
            let e = edges_p2[(rj & mask) as usize];
            let k = ((e >> bits) as usize) & cmask;
            rj = e & mask;
            let l = lens_p2[k];
            if l != LONG {
                out[rat..rat + INLINE].copy_from_slice(&inline16_p2[k]);
                rat += usize::from(l);
            } else {
                let wi = k - 256;
                let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
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
}

fn repeat(n: usize, out: &mut Vec<u8>) {
    assert!(!out.is_empty() && out.len() <= n);
    while out.len() < n {
        let take = out.len().min(n - out.len());
        out.extend_from_within(..take);
    }
}

mod endpoint;
mod outq;
