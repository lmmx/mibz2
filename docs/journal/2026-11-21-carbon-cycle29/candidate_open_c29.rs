//! Carbon Cycle 29 Unified Drop-In `moonshot` Candidate (`candidate_open_c29.rs`)
//!
//! Built directly on `origin/master` (`90e67196`, `mibz2 0.5.11` tree with `#692` memory diet and
//! `#703` dense-block admission), combining ONLY native A/B-verified safe-Rust winners from
//! Carbon Cycles 20–28:
//! 1. `LeanRegBurst6_256` (`m >= 140_000`, Cycles 23, 24, 27): decoupled 6-lane scalar-register
//!    scout burst + `< 256`-step macro-cycle stitch + 6-lane scalar-register `16 B` sloppy emit
//!    burst with 6-way bitwise `LONG = 0x80` sentinel, `[...; 256]` stack arrays (`+0` extra
//!    `rep_stores` over `origin/master`, `+6.5 KiB` `W = 10` heap), and single-caller
//!    `#[inline(always)] emit_serial` separated from `#[cold] emit_serial_cold`.
//! 2. `Elim::replace` + `#[inline(always)] Elim::node` (Cycle 28): direct 4-RMW in-place splice
//!    on equal-width Rauzy steps (`aw == bw`), eliminating back-to-back packed-`u16` store-to-load
//!    forwarding stalls on `a[y * 4]`.
//!
//! Strict `#![forbid(unsafe_code)]` and Rust 1.80 compatible.
#![forbid(unsafe_code)]

use mibz2::bwt::{InverseBwt, LazyCells};
use mibz2::{runs::Runs, Kernels};

#[allow(dead_code)]
pub type Candidate = OpenCells;
pub type OpenCells = OpenCellsOf<false>;

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
        let need = self.arena_words(ns, m as usize);
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
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
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
        self.walk(n, stats.retained, mark, bits, out);
        Some(stats)
    }

    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
        let use_burst = self.extra.is_empty() && m >= BURST_MIN_M;
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
            &mut self.word_len,
            use_burst,
        );
        if use_burst {
            emit_lean_regburst6(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                &self.word_len,
                n,
                mark,
                bits,
                out,
            );
        } else {
            emit_serial(
                &self.edges[..m],
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
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

fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    word_len: &mut Vec<u16>,
    need_wlen: bool,
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
    let words = offsets.len().saturating_sub(1);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
    if need_wlen {
        if word_len.len() < 256 {
            word_len.clear();
            word_len.resize(256, 1);
        }
        word_len.truncate(256);
        word_len.reserve_exact(words);
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
}

/// Single-caller hot serial emit (`m < 140_000`), preserving `origin/master` inlining into `walk()`.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn emit_serial(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
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
    out.truncate(at.min(n));
    repeat(n, out);
}

#[cold]
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn emit_serial_cold(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
}

#[allow(clippy::too_many_arguments)]
fn emit_lean_regburst6(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    word_len: &[u16],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    let m = edges.len();
    let m32 = m as u32;
    if !extra.is_empty() || m < 16_384 || word_len.contains(&u16::MAX) {
        emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    // Algebraic invariant: `m <= (1 << bits)` and `shift = bits.saturating_sub(8)` => `(j >> shift) < 256`.
    let shift = bits.saturating_sub(8);
    let cm = (1u32 << shift) - 1;
    let rem_low = mark & cm;
    let stride = cm + 1;
    let mut cuts = [0u32; 256];
    let mut n_cuts = 0usize;
    let mut q = 0u32;
    while let Some(r) = q.checked_mul(stride).map(|v| v + rem_low) {
        if r >= m32 || n_cuts >= 256 {
            break;
        }
        cuts[n_cuts] = r;
        n_cuts += 1;
        q += 1;
    }
    if n_cuts < 12 {
        emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    let mask = (1u32 << bits) - 1;
    let mut seg_len = [0u32; 256];
    let mut seg_next = [u16::MAX; 256];
    let mut next_c = 6usize;
    let (mut c0, mut c1, mut c2, mut c3, mut c4, mut c5) = (0usize, 1, 2, 3, 4, 5);
    let (mut j0, mut j1, mut j2, mut j3, mut j4, mut j5) =
        (cuts[0], cuts[1], cuts[2], cuts[3], cuts[4], cuts[5]);
    let (mut a0, mut a1, mut a2, mut a3, mut a4, mut a5) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
    let mut guard = 0usize;

    while (c0 & c1 & c2 & c3 & c4 & c5) != usize::MAX {
        if (c0 | c1 | c2 | c3 | c4 | c5) < 256 {
            loop {
                let e0 = edges[j0 as usize];
                let e1 = edges[j1 as usize];
                let e2 = edges[j2 as usize];
                let e3 = edges[j3 as usize];
                let e4 = edges[j4 as usize];
                let e5 = edges[j5 as usize];
                a0 = a0.wrapping_add(u32::from(word_len[(e0 >> bits) as usize]));
                a1 = a1.wrapping_add(u32::from(word_len[(e1 >> bits) as usize]));
                a2 = a2.wrapping_add(u32::from(word_len[(e2 >> bits) as usize]));
                a3 = a3.wrapping_add(u32::from(word_len[(e3 >> bits) as usize]));
                a4 = a4.wrapping_add(u32::from(word_len[(e4 >> bits) as usize]));
                a5 = a5.wrapping_add(u32::from(word_len[(e5 >> bits) as usize]));
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
                emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
                return;
            }
            macro_rules! check_scout {
                ($ck:ident, $jk:ident, $ak:ident) => {
                    if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                        seg_len[$ck] = $ak;
                        seg_next[$ck] = ($jk >> shift) as u16;
                        if next_c < n_cuts {
                            $ck = next_c;
                            $jk = cuts[next_c];
                            $ak = 0;
                            next_c += 1;
                        } else {
                            $ck = usize::MAX;
                        }
                    }
                };
            }
            check_scout!(c0, j0, a0);
            check_scout!(c1, j1, a1);
            check_scout!(c2, j2, a2);
            check_scout!(c3, j3, a3);
            check_scout!(c4, j4, a4);
            check_scout!(c5, j5, a5);
        } else {
            macro_rules! step_drain {
                ($ck:ident, $jk:ident, $ak:ident) => {
                    if $ck != usize::MAX {
                        let e = edges[$jk as usize];
                        $ak = $ak.wrapping_add(u32::from(word_len[(e >> bits) as usize]));
                        $jk = e & mask;
                        guard += 1;
                        if ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                            seg_len[$ck] = $ak;
                            seg_next[$ck] = ($jk >> shift) as u16;
                            $ck = usize::MAX;
                        }
                    }
                };
            }
            step_drain!(c0, j0, a0);
            step_drain!(c1, j1, a1);
            step_drain!(c2, j2, a2);
            step_drain!(c3, j3, a3);
            step_drain!(c4, j4, a4);
            step_drain!(c5, j5, a5);
            if guard > 2 * m + 64 {
                emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
                return;
            }
        }
    }

    let mark_cid = (mark >> shift) as usize;
    let mut visited = [false; 256];
    let mut t_row = [0u32; 256];
    let mut t_start = [0u32; 256];
    let mut t_len = [0u32; 256];
    let mut n_tasks = 0usize;
    let mut cur_cid = mark_cid;
    let mut total_at = 0usize;
    let mut tail_row = None;

    while total_at < n {
        if visited[cur_cid] {
            if cur_cid == mark_cid {
                break;
            } else {
                emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
                return;
            }
        }
        visited[cur_cid] = true;
        let slen = seg_len[cur_cid] as usize;
        let nxt = seg_next[cur_cid] as usize;
        if nxt >= n_cuts {
            emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
            return;
        }
        if total_at + slen > n {
            tail_row = Some(cuts[cur_cid]);
            break;
        }
        if slen > 0 {
            t_row[n_tasks] = cuts[cur_cid];
            t_start[n_tasks] = total_at as u32;
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
        emit_serial_cold(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    out.resize(n + INLINE, 0);
    let mut next_t = 6usize;
    let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5) =
        (t_row[0], t_row[1], t_row[2], t_row[3], t_row[4], t_row[5]);
    let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5) = (
        t_start[0] as usize,
        t_start[1] as usize,
        t_start[2] as usize,
        t_start[3] as usize,
        t_start[4] as usize,
        t_start[5] as usize,
    );
    let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5) =
        (t_len[0], t_len[1], t_len[2], t_len[3], t_len[4], t_len[5]);

    while (rem0 | rem1 | rem2 | rem3 | rem4 | rem5) != 0 {
        if (rem0 != 0) & (rem1 != 0) & (rem2 != 0) & (rem3 != 0) & (rem4 != 0) & (rem5 != 0) {
            loop {
                let e0 = edges[rj0 as usize];
                let e1 = edges[rj1 as usize];
                let e2 = edges[rj2 as usize];
                let e3 = edges[rj3 as usize];
                let e4 = edges[rj4 as usize];
                let e5 = edges[rj5 as usize];
                let k0 = (e0 >> bits) as usize;
                let k1 = (e1 >> bits) as usize;
                let k2 = (e2 >> bits) as usize;
                let k3 = (e3 >> bits) as usize;
                let k4 = (e4 >> bits) as usize;
                let k5 = (e5 >> bits) as usize;
                let (l0, l1, l2, l3, l4, l5) =
                    (lens[k0], lens[k1], lens[k2], lens[k3], lens[k4], lens[k5]);
                if ((l0 | l1 | l2 | l3 | l4 | l5) & LONG) != 0 {
                    macro_rules! step_cold {
                        ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                            $rjk = $ek & mask;
                            if $lk != LONG {
                                out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
                                $atk += usize::from($lk);
                                $remk = $remk.saturating_sub(u32::from($lk));
                            } else {
                                let w = $kk - 256;
                                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
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
                out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                at0 += usize::from(l0);
                rem0 -= u32::from(l0);
                rj1 = e1 & mask;
                out[at1..at1 + INLINE].copy_from_slice(&inline16[k1]);
                at1 += usize::from(l1);
                rem1 -= u32::from(l1);
                rj2 = e2 & mask;
                out[at2..at2 + INLINE].copy_from_slice(&inline16[k2]);
                at2 += usize::from(l2);
                rem2 -= u32::from(l2);
                rj3 = e3 & mask;
                out[at3..at3 + INLINE].copy_from_slice(&inline16[k3]);
                at3 += usize::from(l3);
                rem3 -= u32::from(l3);
                rj4 = e4 & mask;
                out[at4..at4 + INLINE].copy_from_slice(&inline16[k4]);
                at4 += usize::from(l4);
                rem4 -= u32::from(l4);
                rj5 = e5 & mask;
                out[at5..at5 + INLINE].copy_from_slice(&inline16[k5]);
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
            macro_rules! refill_emit {
                ($rjk:ident, $atk:ident, $remk:ident) => {
                    if $remk == 0 && next_t < n_tasks {
                        $rjk = t_row[next_t];
                        $atk = t_start[next_t] as usize;
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
                        let e = edges[$rjk as usize];
                        let k = (e >> bits) as usize;
                        $rjk = e & mask;
                        let l = lens[k];
                        if l != LONG {
                            out[$atk..$atk + INLINE].copy_from_slice(&inline16[k]);
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

    for t in 1..n_tasks {
        let mut rj = t_row[t];
        let mut rat = t_start[t] as usize;
        let rend = (rat + INLINE - 1).min(total_at);
        while rat < rend {
            let e = edges[rj as usize];
            let k = (e >> bits) as usize;
            rj = e & mask;
            let l = lens[k];
            let w = if l != LONG {
                &inline16[k][..usize::from(l)]
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
            let e = edges[rj as usize];
            let k = (e >> bits) as usize;
            rj = e & mask;
            let l = lens[k];
            if l != LONG {
                out[rat..rat + INLINE].copy_from_slice(&inline16[k]);
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

mod endpoint {
    use super::{Band, Counts, Kernel};

    trait Links {
        const STRIDE: usize;
        const BOTTOM: usize;
        fn prev(a: &[u32], id: u32, side: usize) -> u32;
        fn next(a: &[u32], id: u32, side: usize) -> u32;
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    }

    struct Narrow;
    struct Wide;

    impl Links for Narrow {
        const STRIDE: usize = 4;
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
    }

    impl Links for Wide {
        const STRIDE: usize = 6;
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
    }

    const NARROW_NODES: usize = 1 << 16;
    const RULE: usize = 4;
    pub(super) const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;

    #[derive(Default)]
    pub(super) struct Endpoint {
        stack: Vec<u32>,
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
        fn bottom(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::BOTTOM]
        }
        #[inline(always)]
        fn set_bottom(&mut self, id: u32, x: u32) {
            self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
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
        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.a, at, side);
            L::set_prev(self.a, id, side, at);
            L::set_next(self.a, id, side, next);
            L::set_next(self.a, at, side, id);
            L::set_prev(self.a, next, side, id);
        }

        #[inline(always)]
        fn replace(&mut self, old: u32, id: u32, side: usize) {
            let (p, n) = (L::prev(self.a, old, side), L::next(self.a, old, side));
            L::set_prev(self.a, id, side, p);
            L::set_next(self.a, id, side, n);
            L::set_next(self.a, p, side, id);
            L::set_prev(self.a, n, side, id);
        }

        #[inline(always)]
        fn rule(&self, w: u32) -> usize {
            self.base + (w - 257) as usize * RULE
        }

        #[inline(always)]
        fn len(&self, w: u32) -> u32 {
            if w <= 256 {
                u32::from(w != 0)
            } else {
                self.a[self.rule(w) + 2]
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
            let id = ((self.top - self.base) / RULE) as u32 + 257;
            let len = self.len(left) + self.len(right);
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
            self.top += RULE;
            id
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

        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32, steps: &mut usize) -> bool {
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
                taken += 1;
                if x == y {
                    self.cut(x, 0);
                    self.cut(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw < bw {
                    self.cut(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    self.set_width(y, bw - w);
                } else if aw > bw {
                    self.cut(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                } else {
                    self.cut(x, 0);
                    self.replace(y, x, 0);
                    self.set_word(x, word);
                    self.cut(y, 1);
                }
            };
            (*remaining, *steps) = (left, taken);
            done
        }

        fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
            stack.clear();
            stack.push(word);
            while let Some(id) = stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let r = self.rule(id);
                    let rule = &mut self.a[r..r + RULE];
                    if rule[3] == u32::MAX {
                        rule[3] = dest.len() as u32;
                        stack.push(rule[1]);
                        stack.push(rule[0]);
                    } else {
                        let start = rule[3] as usize;
                        dest.extend_from_within(start..start + rule[2] as usize);
                    }
                }
            }
        }
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
            (ns + (stride + RULE) * Self::nodes_most(ns)).max(m)
        }

        pub(super) fn induced(
            &mut self,
            n: u32,
            thin: &[u32; 256],
            stats: &mut Counts,
        ) -> [usize; 256] {
            let most = Self::nodes_most(self.strips.len());
            if self.wide(self.strips.len()) {
                self.induced_with::<Wide>(n, thin, stats, most)
            } else {
                self.induced_with::<Narrow>(n, thin, stats, most)
            }
        }

        fn induced_with<L: Links>(
            &mut self,
            n: u32,
            thin: &[u32; 256],
            stats: &mut Counts,
            most: usize,
        ) -> [usize; 256] {
            let Kernel {
                endpoint,
                strips,
                bands,
                spelling,
                offsets,
                edges,
                ..
            } = self;
            let ns = strips.len();
            let nodes_words = L::STRIDE * most;
            assert!(edges.len() >= ns + nodes_words);
            let (image_order, arena) = edges.split_at_mut(ns);
            let mut e = Elim::<L> {
                a: arena,
                base: 0,
                top: 0,
                nodes: 0,
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            let mut widest = 0;
            for s in strips.iter() {
                e.node(s.width, u32::from(s.byte) + 1);
                widest = widest.max(s.width);
            }
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0;
            for (i, s) in strips.iter().enumerate() {
                if x < s.from {
                    widest = widest.max(s.from - x);
                    let id = e.node(s.from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s.from + s.width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for &i in image_order.iter() {
                let s = strips[i as usize];
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
                grow(edges, ns + top + RULE);
                e = Elim {
                    a: &mut edges[ns..],
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
            }
            stats.steps = steps;
            stats.rules = (e.top - e.base) / RULE;

            let mut id = L::next(e.a, 0, 1);
            x = 0;
            while id != 0 {
                let next = L::next(e.a, id, 1);
                let w = e.width(id);
                e.set_bottom(id, x);
                x += w;
                id = next;
            }
            assert_eq!(x as usize, stats.retained);

            let mut cursors = [0; 256];
            id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                while k < end {
                    let node_width = e.width(id);
                    let node_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = e.bottom(id) + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let height = e.len(node_word) as usize;
                        stats.words += 1;
                        stats.copies += width as usize;
                        stats.traced += height;
                        stats.thick_area += width as usize * height;
                        stats.unit_work += usize::from(width == 1) * height;
                        stats.longest = stats.longest.max(height);
                        e.append(node_word, &mut endpoint.stack, spelling);
                        spelling.push(a as u8);
                        offsets.push(spelling.len() as u32);
                        254 + offsets.len() as u32
                    };
                    bands.push(Band {
                        end: k + width,
                        delta: dst.wrapping_sub(k),
                        code,
                    });
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            cursors
        }
    }
}

mod outq {
    const CHUNK: usize = 512;
    const BYTES: usize = 2 * CHUNK;
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
        tails: [u32; 256],
        fill: [u32; 256],
        limit: u32,
        over: bool,
    }

    pub(super) struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    impl<'a> Queues<'a> {
        pub(super) fn new(out: &'a mut [u8], spill: &'a mut Spill, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            Queues {
                in_out: (out.len() / BYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE; 256],
                tails: [NONE; 256],
                fill: [CHUNK as u32; 256],
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
            }
        }

        #[inline(always)]
        pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = self.tails[a];
            if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
                let at = tail as usize * BYTES + 2 * f;
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = (f as u32) + w;
            } else {
                self.push_slow(a, m, w, thin);
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
                if self.fill[a] as usize == CHUNK {
                    self.take_chunk(a);
                }
                let f = self.fill[a] as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
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

        fn take_chunk(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push([0; BYTES]);
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE);
            }
            self.spill.next[id as usize] = NONE;
            if self.heads[a] == NONE {
                self.heads[a] = id;
            } else {
                self.spill.next[self.tails[a] as usize] = id;
            }
            self.tails[a] = id;
            self.fill[a] = 0;
        }

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
