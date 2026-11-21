//! Carbon Cycle 20 artifact: Single-Touch `CodeTape` multi-lane walk + `Relay8Ref` diagnostic
//! against `origin/master` (`90e67196`). Preserved for reproducibility of Cycle 20's kill decision.
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

/// Chunk size in `u32` words (256 u32s = 512 u16 codes = 1 KiB).
const TAPE_U32S: usize = 256;
const TAPE_CODES: usize = TAPE_U32S * 2;
/// Max `u32` capacity we reuse in `self.edges` for dead-tail tape (within existing 1,269 KiB worker cap).
const TAPE_EDGE_CAP: usize = 231_600;

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
    tape_next: Vec<u16>,
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
        let need = self.arena_words(ns, m as usize).max(if m >= 2048 { TAPE_EDGE_CAP } else { 0 });
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
            &mut self.word_len,
        );
        const MODE: usize = 12; // 0 = Relay8Ref, 8 = CodeTape8, 12 = CodeTape12
        if MODE == 0 {
            emit_relay::<8>(
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
            emit_codetape::<MODE>(
                &mut self.edges,
                &mut self.bands,
                &mut self.tape_next,
                &self.extra,
                &self.spelling,
                &self.offsets,
                &self.inline16,
                &self.lens,
                &self.word_len,
                n,
                m,
                mark,
                bits,
                out,
            );
        }
    }
}

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
const LONG: u8 = u8::MAX;

fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
    word_len: &mut Vec<u16>,
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
        word_len.clear();
        word_len.resize(256, 1);
    }
    inline16.truncate(256);
    lens.truncate(256);
    word_len.truncate(256);
    let words = offsets.len().saturating_sub(1);
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
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

/// Reference 2-pass Relay (Wormhole DietRelay8 baseline for diagnostic comparison).
#[allow(clippy::too_many_arguments)]
fn emit_relay<const MU: usize>(
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
    let m = edges.len() as u32;
    if !extra.is_empty() || m < 2048 || inline16.len() > 65_536 {
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }
    out.resize(n + INLINE, 0);
    let shift = bits.saturating_sub(8);
    let cm = (1u32 << shift) - 1;
    let rem_low = mark & cm;
    let stride = cm + 1;
    let mut cuts = [0u32; 512];
    let mut n_cuts = 0usize;
    let mut q = 0u32;
    while let Some(r) = q.checked_mul(stride).map(|v| v + rem_low) {
        if r >= m || n_cuts >= 512 {
            break;
        }
        cuts[n_cuts] = r;
        n_cuts += 1;
        q += 1;
    }
    if n_cuts <= 2 {
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    let mask = (1u32 << bits) - 1;
    let mut state = [0u8; 512];
    let mut seg_len = [0usize; 512];
    let mut seg_next = [0usize; 512];

    let mut t_row = [0u32; 1024];
    let mut t_start = [0usize; 1024];
    let mut t_end = [0usize; 1024];
    let mut n_tasks = 0usize;

    let mark_cid = (mark >> shift) as usize;
    state[mark_cid] = 3;

    let mut next_claim = 0usize;
    let mut s_cid = [usize::MAX; MU];
    let mut s_j = [0u32; MU];
    let mut s_acc = [0usize; MU];
    let mut active = 0usize;

    for s in 1..MU {
        while next_claim < n_cuts {
            let c = next_claim;
            next_claim += 1;
            if state[c] == 0 {
                state[c] = 1;
                s_cid[s] = c;
                s_j[s] = cuts[c];
                s_acc[s] = 0;
                active += 1;
                break;
            }
        }
    }

    let mut j = mark;
    let mut at = 0usize;
    for _ in 0..edges.len() {
        s_j[0] = j;
        let e_batch: [u32; MU] = std::array::from_fn(|s| edges[s_j[s] as usize]);

        // Lane 0 direct emit step
        let e0 = e_batch[0];
        let c0 = (e0 >> bits) as usize;
        let nj0 = e0 & mask;
        let l0 = lens[c0];
        if l0 != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[c0]);
            at += usize::from(l0);
        } else {
            let w = c0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        j = nj0;

        // Scouts 1..MU
        if active > 0 {
            for s in 1..MU {
                let cid = s_cid[s];
                if cid == usize::MAX {
                    continue;
                }
                let es = e_batch[s];
                let cs = (es >> bits) as usize;
                let njs = es & mask;
                s_acc[s] += usize::from(word_len[cs]);
                if ((njs ^ mark) & cm) == 0 && njs < m {
                    state[cid] = 2;
                    seg_len[cid] = s_acc[s];
                    seg_next[cid] = (njs >> shift) as usize;
                    s_cid[s] = usize::MAX;
                    s_j[s] = 0;
                    active -= 1;
                    while next_claim < n_cuts {
                        let nc = next_claim;
                        next_claim += 1;
                        if state[nc] == 0 {
                            state[nc] = 1;
                            s_cid[s] = nc;
                            s_j[s] = cuts[nc];
                            s_acc[s] = 0;
                            active += 1;
                            break;
                        }
                    }
                } else {
                    s_j[s] = njs;
                }
            }
        }

        if j == mark || at >= n {
            break;
        }

        // Check if Lane 0 hit a cut
        if ((j ^ mark) & cm) == 0 && j < m {
            loop {
                let cid = (j >> shift) as usize;
                let st = state[cid];
                if st == 2 && at + seg_len[cid] <= n && n_tasks < 1024 {
                    t_row[n_tasks] = cuts[cid];
                    t_start[n_tasks] = at;
                    t_end[n_tasks] = at + seg_len[cid];
                    n_tasks += 1;
                    at += seg_len[cid];
                    state[cid] = 3;
                    j = cuts[seg_next[cid]];
                    if j == mark || at >= n {
                        break;
                    }
                } else if st == 1 {
                    let mut owner = usize::MAX;
                    for s in 1..MU {
                        if s_cid[s] == cid {
                            owner = s;
                            break;
                        }
                    }
                    if owner != usize::MAX && s_acc[owner] > 0 && at + s_acc[owner] <= n && n_tasks < 1024 {
                        t_row[n_tasks] = cuts[cid];
                        t_start[n_tasks] = at;
                        t_end[n_tasks] = at + s_acc[owner];
                        n_tasks += 1;
                        at += s_acc[owner];
                        j = s_j[owner];
                        state[cid] = 3;
                        s_cid[owner] = usize::MAX;
                        s_j[owner] = 0;
                        active -= 1;
                        while next_claim < n_cuts {
                            let nc = next_claim;
                            next_claim += 1;
                            if state[nc] == 0 {
                                state[nc] = 1;
                                s_cid[owner] = nc;
                                s_j[owner] = cuts[nc];
                                s_acc[owner] = 0;
                                active += 1;
                                break;
                            }
                        }
                    } else {
                        if owner != usize::MAX {
                            s_cid[owner] = usize::MAX;
                            s_j[owner] = 0;
                            active -= 1;
                        }
                        state[cid] = 3;
                    }
                    break;
                } else {
                    state[cid] = 3;
                    break;
                }
            }
            if j == mark || at >= n {
                break;
            }
        }
    }

    // Phase 4b for Relay8Ref: save boundaries first, then 8-lane sloppy emit, then restore!
    if n_tasks > 0 {
        let mut saved = [[0u8; INLINE]; 1024];
        for t in 0..n_tasks {
            let ea = t_end[t];
            if ea < n {
                saved[t].copy_from_slice(&out[ea..ea + INLINE]);
            }
        }
        let mut next_t = 0usize;
        let mut lj = [0u32; MU];
        let mut lat = [0usize; MU];
        let mut lend = [0usize; MU];
        let mut lbusy = 0usize;
        for l in 0..MU {
            if next_t < n_tasks {
                lj[l] = t_row[next_t];
                lat[l] = t_start[next_t];
                lend[l] = t_end[next_t];
                next_t += 1;
                lbusy += 1;
            } else {
                lj[l] = 0;
                lat[l] = 1;
                lend[l] = 0;
            }
        }
        while lbusy > 0 {
            let e_batch: [u32; MU] = std::array::from_fn(|l| edges[lj[l] as usize]);
            for l in 0..MU {
                if lat[l] >= lend[l] {
                    continue;
                }
                let e = e_batch[l];
                let c = (e >> bits) as usize;
                lj[l] = e & mask;
                let ln = lens[c];
                if ln != LONG {
                    let a = lat[l];
                    out[a..a + INLINE].copy_from_slice(&inline16[c]);
                    lat[l] = a + usize::from(ln);
                } else {
                    let w = c - 256;
                    let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                    let take = word.len().min(lend[l] - lat[l]);
                    let a = lat[l];
                    out[a..a + take].copy_from_slice(&word[..take]);
                    lat[l] = a + take;
                }
                if lat[l] >= lend[l] {
                    if next_t < n_tasks {
                        lj[l] = t_row[next_t];
                        lat[l] = t_start[next_t];
                        lend[l] = t_end[next_t];
                        next_t += 1;
                    } else {
                        lj[l] = 0;
                        lbusy -= 1;
                    }
                }
            }
        }
        // Restore boundaries in reverse order so adjacent zero/short tasks never clobber!
        for t in (0..n_tasks).rev() {
            let ea = t_end[t];
            if ea < n {
                // Only bytes >= ea that do NOT belong to a later task!
                // Wait: restoring in FORWARD order? Check!
                let _ = saved[t];
            }
        }
        // Wait: let's check exact boundary restoration order!
        // Before Phase 4b, `out` had valid Lane-0 bytes OUTSIDE union_t [t_start[t], t_end[t]).
        // If we save ONLY at `ea = t_end[t]` where `ea` is NOT inside any `[t_start[u], t_end[u])` (i.e. `t + 1 == n_tasks || t_start[t + 1] > ea`), wait: what if task `t` slopped over task `t+1` (because task `t+1` ran BEFORE task `t` in parallel Phase 4b!)?!
        // In parallel Phase 4b, task `t+1` holds ALREADY-WRITTEN bytes in `[t_start[t+1], t_end[t+1])`, which were NOT in `out` before Phase 4b!
        // That's why Wormhole's Relay8 saved `inline16` of the FIRST row `t_row[t+1]` or re-emitted the first 16 B of each cut!
        for t in 0..n_tasks {
            let ea = t_end[t];
            if ea < n {
                // Re-emit 16 B starting from the row at `ea`:
                // If `t + 1 < n_tasks && t_start[t + 1] == ea`, walk from `t_row[t + 1]`, else restore `saved[t]` up to next task!
                let limit_at = if t + 1 < n_tasks { t_start[t + 1] } else { n };
                let keep_lane0 = (limit_at - ea).min(INLINE);
                out[ea..ea + keep_lane0].copy_from_slice(&saved[t][..keep_lane0]);
            }
        }
        // And re-emit up to 16 B at the start of every task `t` (`t_start[t]`) in forward order!
        for t in 0..n_tasks {
            let mut rj = t_row[t];
            let mut rat = t_start[t];
            let rend = t_end[t].min(t_start[t] + INLINE);
            while rat < rend {
                let e = edges[rj as usize];
                let c = (e >> bits) as usize;
                rj = e & mask;
                let w = if lens[c] != LONG {
                    &inline16[c][..usize::from(lens[c])]
                } else {
                    let wi = c - 256;
                    &spelling[offsets[wi] as usize..offsets[wi + 1] as usize]
                };
                let take = w.len().min(t_end[t] - rat);
                out[rat..rat + take].copy_from_slice(&w[..take]);
                rat += take;
            }
        }
    }

    out.truncate(at.min(n));
    repeat(n, out);
}

/// Carbon Cycle 20: Single-Touch `CodeTape` Multi-Lane Walk!
/// Phase 4a walks `edges[..m]` ONCE across `MU` lanes (Lane 0 direct emit + `MU-1` tape scouts),
/// storing packed `u16` code pairs into dead `edges[m..]` + dead `bands` chunks.
/// Phase 4b drains the taped `u16` codes sequentially Left-to-Right with ZERO `edges[j]` loads!
#[allow(clippy::too_many_arguments)]
fn emit_codetape<const MU: usize>(
    edges_buf: &mut [u32],
    bands_buf: &mut [Band],
    tape_next: &mut Vec<u16>,
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    word_len: &[u16],
    n: usize,
    m: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    let m32 = m as u32;
    if !extra.is_empty() || m32 < 2048 || inline16.len() > 65_536 {
        emit_serial(
            &edges_buf[..m],
            extra,
            spelling,
            offsets,
            inline16,
            lens,
            n,
            mark,
            bits,
            out,
        );
        return;
    }

    let (edges, dead_tail) = edges_buf.split_at_mut(m);
    // Chunks of 256 u32s (512 u16 codes) in `dead_tail`, plus chunks of 86 Bands (258 u32s >= 256 u32s) in `bands_buf`:
    let tail_chunks = dead_tail.len() / TAPE_U32S;
    let band_chunks = bands_buf.len() / 86;
    let total_chunks = (tail_chunks + band_chunks).min(2048);
    if tape_next.len() < total_chunks {
        tape_next.resize(total_chunks, u16::MAX);
    }

    out.resize(n + INLINE, 0);
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
    if n_cuts <= 2 {
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    let mask = (1u32 << bits) - 1;
    let mut state = [0u8; 512];
    let mut seg_len = [0usize; 512];
    let mut seg_next = [0usize; 512];
    let mut seg_head = [u16::MAX; 512];
    let mut seg_codes = [0u32; 512];
    let mut seg_long = [false; 512];

    let mut t_head = [u16::MAX; 1024];
    let mut t_codes = [0u32; 1024];
    let mut t_long = [false; 1024];
    let mut t_row = [0u32; 1024];
    let mut t_start = [0usize; 1024];
    let mut t_end = [0usize; 1024];
    let mut n_tasks = 0usize;

    let mark_cid = (mark >> shift) as usize;
    state[mark_cid] = 3;

    let mut free_chunk = 0usize;
    let mut next_claim = 0usize;

    let mut s_cid = [usize::MAX; MU];
    let mut s_j = [0u32; MU];
    let mut s_acc = [0usize; MU];
    let mut s_head = [u16::MAX; MU];
    let mut s_tail = [u16::MAX; MU];
    let mut s_cnt = [0u32; MU];
    let mut s_pend = [0u32; MU];
    let mut s_long = [false; MU];
    let mut s_taping = [false; MU];
    let mut active = 0usize;

    let write_pair = |chunk_id: usize, pair_idx: usize, val: u32, dead_tail: &mut [u32], bands_buf: &mut [Band]| {
        if chunk_id < tail_chunks {
            dead_tail[chunk_id * TAPE_U32S + pair_idx] = val;
        } else {
            let b_base = (chunk_id - tail_chunks) * 86;
            let b = &mut bands_buf[b_base + pair_idx / 3];
            match pair_idx % 3 {
                0 => b.end = val,
                1 => b.delta = val,
                _ => b.code = val,
            }
        }
    };

    for s in 1..MU {
        while next_claim < n_cuts {
            let c = next_claim;
            next_claim += 1;
            if state[c] == 0 {
                state[c] = 1;
                s_cid[s] = c;
                s_j[s] = cuts[c];
                s_acc[s] = 0;
                s_cnt[s] = 0;
                s_long[s] = false;
                if free_chunk < total_chunks {
                    let ch = free_chunk as u16;
                    free_chunk += 1;
                    tape_next[ch as usize] = u16::MAX;
                    s_head[s] = ch;
                    s_tail[s] = ch;
                    s_taping[s] = true;
                } else {
                    s_head[s] = u16::MAX;
                    s_taping[s] = false;
                }
                active += 1;
                break;
            }
        }
    }

    let mut j = mark;
    let mut at = 0usize;
    for _ in 0..edges.len() {
        s_j[0] = j;
        let e_batch: [u32; MU] = std::array::from_fn(|s| edges[s_j[s] as usize]);

        // Lane 0 direct emit step
        let e0 = e_batch[0];
        let c0 = (e0 >> bits) as usize;
        let nj0 = e0 & mask;
        let l0 = lens[c0];
        if l0 != LONG {
            out[at..at + INLINE].copy_from_slice(&inline16[c0]);
            at += usize::from(l0);
        } else {
            let w = c0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }
        j = nj0;

        // Scouts 1..MU: single-touch walk + sequential u16 pair taping
        if active > 0 {
            for s in 1..MU {
                let cid = s_cid[s];
                if cid == usize::MAX {
                    continue;
                }
                let es = e_batch[s];
                let cs = (es >> bits) as usize;
                let njs = es & mask;
                let wl = word_len[cs];
                s_acc[s] += usize::from(wl);
                if wl > INLINE as u16 {
                    s_long[s] = true;
                }
                if s_taping[s] {
                    let cnt = s_cnt[s] as usize;
                    let in_chunk = cnt & (TAPE_CODES - 1);
                    if cnt > 0 && in_chunk == 0 {
                        if free_chunk < total_chunks {
                            let nch = free_chunk as u16;
                            free_chunk += 1;
                            tape_next[nch as usize] = u16::MAX;
                            tape_next[s_tail[s] as usize] = nch;
                            s_tail[s] = nch;
                        } else {
                            s_taping[s] = false;
                            s_head[s] = u16::MAX;
                        }
                    }
                    if s_taping[s] {
                        if (in_chunk & 1) == 0 {
                            s_pend[s] = cs as u32;
                        } else {
                            let val = s_pend[s] | ((cs as u32) << 16);
                            write_pair(s_tail[s] as usize, in_chunk >> 1, val, dead_tail, bands_buf);
                        }
                        s_cnt[s] += 1;
                    }
                }

                if ((njs ^ mark) & cm) == 0 && njs < m32 {
                    if s_taping[s] && (s_cnt[s] & 1) == 1 {
                        let in_chunk = ((s_cnt[s] - 1) as usize) & (TAPE_CODES - 1);
                        write_pair(s_tail[s] as usize, in_chunk >> 1, s_pend[s], dead_tail, bands_buf);
                    }
                    state[cid] = 2;
                    seg_len[cid] = s_acc[s];
                    seg_next[cid] = (njs >> shift) as usize;
                    seg_head[cid] = if s_taping[s] { s_head[s] } else { u16::MAX };
                    seg_codes[cid] = s_cnt[s];
                    seg_long[cid] = s_long[s];
                    s_cid[s] = usize::MAX;
                    s_j[s] = 0;
                    active -= 1;
                    while next_claim < n_cuts {
                        let nc = next_claim;
                        next_claim += 1;
                        if state[nc] == 0 {
                            state[nc] = 1;
                            s_cid[s] = nc;
                            s_j[s] = cuts[nc];
                            s_acc[s] = 0;
                            s_cnt[s] = 0;
                            s_long[s] = false;
                            if free_chunk < total_chunks {
                                let ch = free_chunk as u16;
                                free_chunk += 1;
                                tape_next[ch as usize] = u16::MAX;
                                s_head[s] = ch;
                                s_tail[s] = ch;
                                s_taping[s] = true;
                            } else {
                                s_head[s] = u16::MAX;
                                s_taping[s] = false;
                            }
                            active += 1;
                            break;
                        }
                    }
                } else {
                    s_j[s] = njs;
                }
            }
        }

        if j == mark || at >= n {
            break;
        }

        if ((j ^ mark) & cm) == 0 && j < m32 {
            loop {
                let cid = (j >> shift) as usize;
                let st = state[cid];
                if st == 2 && at + seg_len[cid] <= n && n_tasks < 1024 {
                    t_head[n_tasks] = seg_head[cid];
                    t_codes[n_tasks] = seg_codes[cid];
                    t_long[n_tasks] = seg_long[cid];
                    t_row[n_tasks] = cuts[cid];
                    t_start[n_tasks] = at;
                    t_end[n_tasks] = at + seg_len[cid];
                    n_tasks += 1;
                    at += seg_len[cid];
                    state[cid] = 3;
                    j = cuts[seg_next[cid]];
                    if j == mark || at >= n {
                        break;
                    }
                } else if st == 1 {
                    let mut owner = usize::MAX;
                    for s in 1..MU {
                        if s_cid[s] == cid {
                            owner = s;
                            break;
                        }
                    }
                    if owner != usize::MAX && s_acc[owner] > 0 && at + s_acc[owner] <= n && n_tasks < 1024 {
                        if s_taping[owner] && (s_cnt[owner] & 1) == 1 {
                            let in_chunk = ((s_cnt[owner] - 1) as usize) & (TAPE_CODES - 1);
                            write_pair(
                                s_tail[owner] as usize,
                                in_chunk >> 1,
                                s_pend[owner],
                                dead_tail,
                                bands_buf,
                            );
                        }
                        t_head[n_tasks] = if s_taping[owner] { s_head[owner] } else { u16::MAX };
                        t_codes[n_tasks] = s_cnt[owner];
                        t_long[n_tasks] = s_long[owner];
                        t_row[n_tasks] = cuts[cid];
                        t_start[n_tasks] = at;
                        t_end[n_tasks] = at + s_acc[owner];
                        n_tasks += 1;
                        at += s_acc[owner];
                        j = s_j[owner];
                        state[cid] = 3;
                        s_cid[owner] = usize::MAX;
                        s_j[owner] = 0;
                        active -= 1;
                        while next_claim < n_cuts {
                            let nc = next_claim;
                            next_claim += 1;
                            if state[nc] == 0 {
                                state[nc] = 1;
                                s_cid[owner] = nc;
                                s_j[owner] = cuts[nc];
                                s_acc[owner] = 0;
                                s_cnt[owner] = 0;
                                s_long[owner] = false;
                                if free_chunk < total_chunks {
                                    let ch = free_chunk as u16;
                                    free_chunk += 1;
                                    tape_next[ch as usize] = u16::MAX;
                                    s_head[owner] = ch;
                                    s_tail[owner] = ch;
                                    s_taping[owner] = true;
                                } else {
                                    s_head[owner] = u16::MAX;
                                    s_taping[owner] = false;
                                }
                                active += 1;
                                break;
                            }
                        }
                    } else {
                        if owner != usize::MAX {
                            s_cid[owner] = usize::MAX;
                            s_j[owner] = 0;
                            active -= 1;
                        }
                        state[cid] = 3;
                    }
                    break;
                } else {
                    state[cid] = 3;
                    break;
                }
            }
            if j == mark || at >= n {
                break;
            }
        }
    }

    // Phase 4b: Left-to-Right Sequential Drain of Taped Cuts (ZERO `edges[j]` random loads!)
    for t in 0..n_tasks {
        let mut cur_at = t_start[t];
        let end_at = t_end[t];
        let mut saved = [0u8; INLINE];
        if end_at < n {
            saved.copy_from_slice(&out[end_at..end_at + INLINE]);
        }
        if t_head[t] != u16::MAX {
            let mut ch = t_head[t] as usize;
            let mut rem = t_codes[t] as usize;
            let has_long = t_long[t];
            while rem > 0 {
                let take = rem.min(TAPE_CODES);
                let full_pairs = take >> 1;
                if ch < tail_chunks {
                    let slice = &dead_tail[ch * TAPE_U32S..ch * TAPE_U32S + full_pairs];
                    if !has_long {
                        for &w in slice {
                            let c0 = (w & 0xFFFF) as usize;
                            let c1 = (w >> 16) as usize;
                            out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c0]);
                            cur_at += usize::from(lens[c0]);
                            out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c1]);
                            cur_at += usize::from(lens[c1]);
                        }
                    } else {
                        for &w in slice {
                            for c in [(w & 0xFFFF) as usize, (w >> 16) as usize] {
                                let ln = lens[c];
                                if ln != LONG {
                                    out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c]);
                                    cur_at += usize::from(ln);
                                } else {
                                    let wi = c - 256;
                                    let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
                                    let tk = word.len().min(end_at - cur_at);
                                    out[cur_at..cur_at + tk].copy_from_slice(&word[..tk]);
                                    cur_at += tk;
                                }
                            }
                        }
                    }
                    if (take & 1) == 1 {
                        let c = (dead_tail[ch * TAPE_U32S + full_pairs] & 0xFFFF) as usize;
                        let ln = lens[c];
                        if ln != LONG {
                            out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c]);
                            cur_at += usize::from(ln);
                        } else {
                            let wi = c - 256;
                            let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
                            let tk = word.len().min(end_at - cur_at);
                            out[cur_at..cur_at + tk].copy_from_slice(&word[..tk]);
                            cur_at += tk;
                        }
                    }
                } else {
                    let b_base = (ch - tail_chunks) * 86;
                    for p in 0..full_pairs {
                        let b = &bands_buf[b_base + p / 3];
                        let w = match p % 3 {
                            0 => b.end,
                            1 => b.delta,
                            _ => b.code,
                        };
                        for c in [(w & 0xFFFF) as usize, (w >> 16) as usize] {
                            let ln = lens[c];
                            if ln != LONG {
                                out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c]);
                                cur_at += usize::from(ln);
                            } else {
                                let wi = c - 256;
                                let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
                                let tk = word.len().min(end_at - cur_at);
                                out[cur_at..cur_at + tk].copy_from_slice(&word[..tk]);
                                cur_at += tk;
                            }
                        }
                    }
                    if (take & 1) == 1 {
                        let b = &bands_buf[b_base + full_pairs / 3];
                        let w = match full_pairs % 3 {
                            0 => b.end,
                            1 => b.delta,
                            _ => b.code,
                        };
                        let c = (w & 0xFFFF) as usize;
                        let ln = lens[c];
                        if ln != LONG {
                            out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c]);
                            cur_at += usize::from(ln);
                        } else {
                            let wi = c - 256;
                            let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
                            let tk = word.len().min(end_at - cur_at);
                            out[cur_at..cur_at + tk].copy_from_slice(&word[..tk]);
                            cur_at += tk;
                        }
                    }
                }
                rem -= take;
                if rem > 0 {
                    ch = tape_next[ch] as usize;
                }
            }
        } else {
            // Fallback re-walk if dead_tail + bands ever ran out of chunks
            let mut rj = t_row[t];
            while cur_at < end_at {
                let e = edges[rj as usize];
                let c = (e >> bits) as usize;
                rj = e & mask;
                let ln = lens[c];
                if ln != LONG {
                    out[cur_at..cur_at + INLINE].copy_from_slice(&inline16[c]);
                    cur_at += usize::from(ln);
                } else {
                    let wi = c - 256;
                    let word = &spelling[offsets[wi] as usize..offsets[wi + 1] as usize];
                    let tk = word.len().min(end_at - cur_at);
                    out[cur_at..cur_at + tk].copy_from_slice(&word[..tk]);
                    cur_at += tk;
                }
            }
        }
        assert_eq!(cur_at, end_at);
        if end_at < n {
            out[end_at..end_at + INLINE].copy_from_slice(&saved);
        }
    }

    out.truncate(at.min(n));
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
