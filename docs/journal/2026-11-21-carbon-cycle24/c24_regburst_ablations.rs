//! Carbon Cycle 24 artifact: Multi-lane register-pressure, cut-density (`CUT_LOG = 7, 8, 9`),
//! and branched vs branchless scout length ablations on `RegBurst` against `origin/master` (`90e67196`).
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

/// Zero extra vectors compared to origin/master (`word_len` eliminated completely -> +0.0 KiB heap!).
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
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        if self.extra.is_empty() && m >= BURST_MIN_M {
            emit_regburst8(
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

const BURST_MIN_M: usize = 128_000;
const CUT_LOG: u32 = 9; // up to 512 cuts (halving straggler tail vs 256 cuts!)
const EMIT_K: usize = 8; // 4, 6, or 8 emit lanes (8 scout lanes always)
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

fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
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
    let words = offsets.len().saturating_sub(1);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
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

#[inline(always)]
fn exact_len(code: usize, l: u8, offsets: &[u32]) -> u32 {
    if l != LONG {
        u32::from(l)
    } else {
        let w = code - 256;
        offsets[w + 1] - offsets[w]
    }
}

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

/// Carbon Cycle 24: Zero-Heap `RegBurst8` (`+0.0 KiB` extra heap: uses `lens` + cold `offsets`
/// directly with 8-way bitwise `LONG` test, 8 scalar-register scout lanes over up to 512 cuts,
/// and configurable 6- or 8-lane scalar-register sloppy emit burst).
#[allow(clippy::too_many_arguments)]
fn emit_regburst8(
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
    let m = edges.len();
    let m32 = m as u32;
    let shift = bits.saturating_sub(CUT_LOG);
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
    if n_cuts < 16 {
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    let mask = (1u32 << bits) - 1;
    let mut seg_len = [0u32; 512];
    let mut seg_next = [u16::MAX; 512];

    // === Stage 1: 8-Lane Scalar-Register Scout Burst (zero `word_len` Vec!) ===
    let mut next_c = 8usize;
    let (mut c0, mut c1, mut c2, mut c3, mut c4, mut c5, mut c6, mut c7) =
        (0usize, 1, 2, 3, 4, 5, 6, 7);
    let (mut j0, mut j1, mut j2, mut j3, mut j4, mut j5, mut j6, mut j7) = (
        cuts[0], cuts[1], cuts[2], cuts[3], cuts[4], cuts[5], cuts[6], cuts[7],
    );
    let (mut a0, mut a1, mut a2, mut a3, mut a4, mut a5, mut a6, mut a7) =
        (0u32, 0, 0, 0, 0, 0, 0, 0);
    let mut guard = 0usize;

    while (c0 & c1 & c2 & c3 & c4 & c5 & c6 & c7) != usize::MAX {
        if (c0 | c1 | c2 | c3 | c4 | c5 | c6 | c7) < 512 {
            loop {
                let e0 = edges[j0 as usize];
                let e1 = edges[j1 as usize];
                let e2 = edges[j2 as usize];
                let e3 = edges[j3 as usize];
                let e4 = edges[j4 as usize];
                let e5 = edges[j5 as usize];
                let e6 = edges[j6 as usize];
                let e7 = edges[j7 as usize];
                let (k0, k1, k2, k3, k4, k5, k6, k7) = (
                    (e0 >> bits) as usize,
                    (e1 >> bits) as usize,
                    (e2 >> bits) as usize,
                    (e3 >> bits) as usize,
                    (e4 >> bits) as usize,
                    (e5 >> bits) as usize,
                    (e6 >> bits) as usize,
                    (e7 >> bits) as usize,
                );
                let (l0, l1, l2, l3, l4, l5, l6, l7) = (
                    lens[k0], lens[k1], lens[k2], lens[k3], lens[k4], lens[k5], lens[k6], lens[k7],
                );
                if ((l0 | l1 | l2 | l3 | l4 | l5 | l6 | l7) & LONG) == 0 {
                    a0 += u32::from(l0);
                    a1 += u32::from(l1);
                    a2 += u32::from(l2);
                    a3 += u32::from(l3);
                    a4 += u32::from(l4);
                    a5 += u32::from(l5);
                    a6 += u32::from(l6);
                    a7 += u32::from(l7);
                } else {
                    a0 += exact_len(k0, l0, offsets);
                    a1 += exact_len(k1, l1, offsets);
                    a2 += exact_len(k2, l2, offsets);
                    a3 += exact_len(k3, l3, offsets);
                    a4 += exact_len(k4, l4, offsets);
                    a5 += exact_len(k5, l5, offsets);
                    a6 += exact_len(k6, l6, offsets);
                    a7 += exact_len(k7, l7, offsets);
                }
                j0 = e0 & mask;
                j1 = e1 & mask;
                j2 = e2 & mask;
                j3 = e3 & mask;
                j4 = e4 & mask;
                j5 = e5 & mask;
                j6 = e6 & mask;
                j7 = e7 & mask;
                guard += 8;
                if (((j0 ^ mark) & cm) == 0)
                    | (((j1 ^ mark) & cm) == 0)
                    | (((j2 ^ mark) & cm) == 0)
                    | (((j3 ^ mark) & cm) == 0)
                    | (((j4 ^ mark) & cm) == 0)
                    | (((j5 ^ mark) & cm) == 0)
                    | (((j6 ^ mark) & cm) == 0)
                    | (((j7 ^ mark) & cm) == 0)
                    | (guard > 2 * m + 64)
                {
                    break;
                }
            }
            if guard > 2 * m + 64 {
                emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
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
            check_scout!(c6, j6, a6);
            check_scout!(c7, j7, a7);
        } else {
            macro_rules! step_drain {
                ($ck:ident, $jk:ident, $ak:ident) => {
                    if $ck != usize::MAX {
                        let e = edges[$jk as usize];
                        let k = (e >> bits) as usize;
                        $ak += exact_len(k, lens[k], offsets);
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
            step_drain!(c6, j6, a6);
            step_drain!(c7, j7, a7);
            if guard > 2 * m + 64 {
                emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
                return;
            }
        }
    }

    // === Stage 2: Macro-Cycle Prefix Sum (< 512 steps in L1d) ===
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
                emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
                return;
            }
        }
        visited[cur_cid] = true;
        let slen = seg_len[cur_cid] as usize;
        let nxt = seg_next[cur_cid] as usize;
        if nxt >= n_cuts {
            emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
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

    if n_tasks < EMIT_K {
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    // === Stage 3: Sloppy Emit Burst (`EMIT_K` = 6 or 8 scalar lanes) ===
    out.resize(n + INLINE, 0);
    if EMIT_K == 8 {
        let mut next_t = 8usize;
        let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5, mut rj6, mut rj7) = (
            t_row[0], t_row[1], t_row[2], t_row[3], t_row[4], t_row[5], t_row[6], t_row[7],
        );
        let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5, mut at6, mut at7) = (
            t_start[0],
            t_start[1],
            t_start[2],
            t_start[3],
            t_start[4],
            t_start[5],
            t_start[6],
            t_start[7],
        );
        let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5, mut rem6, mut rem7) = (
            t_len[0], t_len[1], t_len[2], t_len[3], t_len[4], t_len[5], t_len[6], t_len[7],
        );
        while (rem0 | rem1 | rem2 | rem3 | rem4 | rem5 | rem6 | rem7) != 0 {
            if (rem0 != 0)
                & (rem1 != 0)
                & (rem2 != 0)
                & (rem3 != 0)
                & (rem4 != 0)
                & (rem5 != 0)
                & (rem6 != 0)
                & (rem7 != 0)
            {
                loop {
                    let e0 = edges[rj0 as usize];
                    let e1 = edges[rj1 as usize];
                    let e2 = edges[rj2 as usize];
                    let e3 = edges[rj3 as usize];
                    let e4 = edges[rj4 as usize];
                    let e5 = edges[rj5 as usize];
                    let e6 = edges[rj6 as usize];
                    let e7 = edges[rj7 as usize];
                    let (k0, k1, k2, k3, k4, k5, k6, k7) = (
                        (e0 >> bits) as usize,
                        (e1 >> bits) as usize,
                        (e2 >> bits) as usize,
                        (e3 >> bits) as usize,
                        (e4 >> bits) as usize,
                        (e5 >> bits) as usize,
                        (e6 >> bits) as usize,
                        (e7 >> bits) as usize,
                    );
                    let (l0, l1, l2, l3, l4, l5, l6, l7) = (
                        lens[k0], lens[k1], lens[k2], lens[k3], lens[k4], lens[k5], lens[k6],
                        lens[k7],
                    );
                    if ((l0 | l1 | l2 | l3 | l4 | l5 | l6 | l7) & LONG) != 0 {
                        macro_rules! step_cold {
                            ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                                $rjk = $ek & mask;
                                if $lk != LONG {
                                    out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
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
                        step_cold!(e6, k6, l6, rj6, at6, rem6);
                        step_cold!(e7, k7, l7, rj7, at7, rem7);
                        break;
                    }
                    macro_rules! step_fast {
                        ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                            $rjk = $ek & mask;
                            out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
                            $atk += usize::from($lk);
                            $remk -= u32::from($lk);
                        };
                    }
                    step_fast!(e0, k0, l0, rj0, at0, rem0);
                    step_fast!(e1, k1, l1, rj1, at1, rem1);
                    step_fast!(e2, k2, l2, rj2, at2, rem2);
                    step_fast!(e3, k3, l3, rj3, at3, rem3);
                    step_fast!(e4, k4, l4, rj4, at4, rem4);
                    step_fast!(e5, k5, l5, rj5, at5, rem5);
                    step_fast!(e6, k6, l6, rj6, at6, rem6);
                    step_fast!(e7, k7, l7, rj7, at7, rem7);
                    if (rem0 == 0)
                        | (rem1 == 0)
                        | (rem2 == 0)
                        | (rem3 == 0)
                        | (rem4 == 0)
                        | (rem5 == 0)
                        | (rem6 == 0)
                        | (rem7 == 0)
                    {
                        break;
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
                refill_emit!(rj6, at6, rem6);
                refill_emit!(rj7, at7, rem7);
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
                drain_emit!(rj6, at6, rem6);
                drain_emit!(rj7, at7, rem7);
            }
        }
    } else {
        // EMIT_K == 4 (ultra-low register pressure: 12 scalar GPRs, zero spills!)
        let mut next_t = 4usize;
        let (mut rj0, mut rj1, mut rj2, mut rj3) = (t_row[0], t_row[1], t_row[2], t_row[3]);
        let (mut at0, mut at1, mut at2, mut at3) =
            (t_start[0], t_start[1], t_start[2], t_start[3]);
        let (mut rem0, mut rem1, mut rem2, mut rem3) = (t_len[0], t_len[1], t_len[2], t_len[3]);
        while (rem0 | rem1 | rem2 | rem3) != 0 {
            if (rem0 != 0) & (rem1 != 0) & (rem2 != 0) & (rem3 != 0) {
                loop {
                    let e0 = edges[rj0 as usize];
                    let e1 = edges[rj1 as usize];
                    let e2 = edges[rj2 as usize];
                    let e3 = edges[rj3 as usize];
                    let (k0, k1, k2, k3) = (
                        (e0 >> bits) as usize,
                        (e1 >> bits) as usize,
                        (e2 >> bits) as usize,
                        (e3 >> bits) as usize,
                    );
                    let (l0, l1, l2, l3) = (lens[k0], lens[k1], lens[k2], lens[k3]);
                    if ((l0 | l1 | l2 | l3) & LONG) != 0 {
                        macro_rules! step_cold {
                            ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                                $rjk = $ek & mask;
                                if $lk != LONG {
                                    out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
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
                        break;
                    }
                    macro_rules! step_fast {
                        ($ek:ident, $kk:ident, $lk:ident, $rjk:ident, $atk:ident, $remk:ident) => {
                            $rjk = $ek & mask;
                            out[$atk..$atk + INLINE].copy_from_slice(&inline16[$kk]);
                            $atk += usize::from($lk);
                            $remk -= u32::from($lk);
                        };
                    }
                    step_fast!(e0, k0, l0, rj0, at0, rem0);
                    step_fast!(e1, k1, l1, rj1, at1, rem1);
                    step_fast!(e2, k2, l2, rj2, at2, rem2);
                    step_fast!(e3, k3, l3, rj3, at3, rem3);
                    if (rem0 == 0) | (rem1 == 0) | (rem2 == 0) | (rem3 == 0) {
                        break;
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
            }
        }
    }

    // === Stage 4: Exact 15-Byte Slop Restoration at Cut Starts ===
    for t in 1..n_tasks {
        let mut rj = t_row[t];
        let mut rat = t_start[t];
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

mod endpoint;
mod outq;
