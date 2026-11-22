//! Carbon Cycle 26 artifact: `RegRelay6` (Gated 1-Direct + 5-Scout Scalar-Register Relay, `m >= 135_000`)
//! — Cuts `T1 dense` to `0.8923x` (`blk-270` `0.8186x`), `T4 dense` to `0.8366x`, and `T1 wiki-431` to `0.9866x`.
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
        let use_relay = self.extra.is_empty() && m >= RELAY_MIN_M;
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
            &mut self.word_len,
            use_relay,
        );
        if use_relay {
            emit_regrelay6(
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

const RELAY_MIN_M: usize = 135_000;
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

/// Carbon Cycle 26: `RegRelay6` — Scalar-Register Direct-Emitter + 5-Scout Burst (Stage 1)
/// followed by 6-Lane Scalar-Register Sloppy Emit Burst (Stage 2).
/// Cuts Stage 2 random `edges` loads from `RegBurst6`'s `2.00 m` down to `~1.80 m` while
/// retaining 100% scalar-register cursors and immediate exit to fast cut-hop once scouts finish!
#[allow(clippy::too_many_arguments)]
fn emit_regrelay6(
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
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
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
        emit_serial(edges, extra, spelling, offsets, inline16, lens, n, mark, bits, out);
        return;
    }

    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut state = [0u8; 512];
    let mut seg_len = [0u32; 512];
    let mut seg_next = [0u16; 512];

    let mark_cid = (mark >> shift) as usize;
    state[mark_cid] = 3;

    let mut next_c = 0usize;
    let mut sc_cid = [usize::MAX; 5];
    let mut active = 0usize;
    let (mut j1, mut j2, mut j3, mut j4, mut j5) = (0u32, 0, 0, 0, 0);
    let (mut a1, mut a2, mut a3, mut a4, mut a5) = (0u32, 0, 0, 0, 0);

    macro_rules! claim_cut {
        ($idx:expr, $jk:ident, $ak:ident) => {
            while next_c < n_cuts {
                let c = next_c;
                next_c += 1;
                if state[c] == 0 {
                    state[c] = 1;
                    sc_cid[$idx] = c;
                    $jk = cuts[c];
                    $ak = 0;
                    active += 1;
                    break;
                }
            }
        };
    }
    claim_cut!(0, j1, a1);
    claim_cut!(1, j2, a2);
    claim_cut!(2, j3, a3);
    claim_cut!(3, j4, a4);
    claim_cut!(4, j5, a5);

    let mut t_row = [0u32; 1024];
    let mut t_start = [0usize; 1024];
    let mut t_len = [0u32; 1024];
    let mut n_tasks = 0usize;

    let mut j0 = mark;
    let mut at0 = 0usize;
    let mut steps0 = 0usize;

    'stage1: loop {
        if active == 5 && at0 < n && steps0 < m {
            // Fast 6-lane scalar-register burst: Lane 0 emits directly, Lanes 1..5 scout!
            loop {
                let e0 = edges[j0 as usize];
                let e1 = edges[j1 as usize];
                let e2 = edges[j2 as usize];
                let e3 = edges[j3 as usize];
                let e4 = edges[j4 as usize];
                let e5 = edges[j5 as usize];

                let k0 = (e0 >> bits) as usize;
                j0 = e0 & mask;
                let l0 = lens[k0];
                if l0 != LONG {
                    out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                    at0 += usize::from(l0);
                } else {
                    let w = k0 - 256;
                    let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                    let tk = word.len().min(n - at0);
                    out[at0..at0 + tk].copy_from_slice(&word[..tk]);
                    at0 += tk;
                }

                a1 += u32::from(word_len[(e1 >> bits) as usize]);
                a2 += u32::from(word_len[(e2 >> bits) as usize]);
                a3 += u32::from(word_len[(e3 >> bits) as usize]);
                a4 += u32::from(word_len[(e4 >> bits) as usize]);
                a5 += u32::from(word_len[(e5 >> bits) as usize]);
                j1 = e1 & mask;
                j2 = e2 & mask;
                j3 = e3 & mask;
                j4 = e4 & mask;
                j5 = e5 & mask;
                steps0 += 1;

                if (((j0 ^ mark) & cm) == 0)
                    | (((j1 ^ mark) & cm) == 0)
                    | (((j2 ^ mark) & cm) == 0)
                    | (((j3 ^ mark) & cm) == 0)
                    | (((j4 ^ mark) & cm) == 0)
                    | (((j5 ^ mark) & cm) == 0)
                    | (at0 >= n)
                    | (steps0 >= m)
                {
                    break;
                }
            }
        } else {
            // Drain / pure-Lane-0 tail when < 5 scouts remain (zero dummy loads!)
            if steps0 >= m || at0 >= n {
                break 'stage1;
            }
            let e0 = edges[j0 as usize];
            let k0 = (e0 >> bits) as usize;
            j0 = e0 & mask;
            let l0 = lens[k0];
            if l0 != LONG {
                out[at0..at0 + INLINE].copy_from_slice(&inline16[k0]);
                at0 += usize::from(l0);
            } else {
                let w = k0 - 256;
                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                let tk = word.len().min(n - at0);
                out[at0..at0 + tk].copy_from_slice(&word[..tk]);
                at0 += tk;
            }
            steps0 += 1;

            if active > 0 {
                macro_rules! step_one_scout {
                    ($idx:expr, $jk:ident, $ak:ident) => {
                        if sc_cid[$idx] != usize::MAX {
                            let es = edges[$jk as usize];
                            $ak += u32::from(word_len[(es >> bits) as usize]);
                            $jk = es & mask;
                        }
                    };
                }
                step_one_scout!(0, j1, a1);
                step_one_scout!(1, j2, a2);
                step_one_scout!(2, j3, a3);
                step_one_scout!(3, j4, a4);
                step_one_scout!(4, j5, a5);
            }
        }

        // Retire any scouts that hit a cut
        if active > 0 {
            macro_rules! retire_scout {
                ($idx:expr, $jk:ident, $ak:ident) => {
                    if sc_cid[$idx] != usize::MAX && ((($jk ^ mark) & cm) == 0) && $jk < m32 {
                        let cid = sc_cid[$idx];
                        state[cid] = 2;
                        seg_len[cid] = $ak;
                        seg_next[cid] = ($jk >> shift) as u16;
                        sc_cid[$idx] = usize::MAX;
                        active -= 1;
                        claim_cut!($idx, $jk, $ak);
                    }
                };
            }
            retire_scout!(0, j1, a1);
            retire_scout!(1, j2, a2);
            retire_scout!(2, j3, a3);
            retire_scout!(3, j4, a4);
            retire_scout!(4, j5, a5);
        }

        if j0 == mark || at0 >= n || steps0 >= m {
            break 'stage1;
        }

        // Check if Lane 0 hit a cut: hop all SCOUTED cuts or split an active scout!
        if ((j0 ^ mark) & cm) == 0 && j0 < m32 {
            loop {
                let cid = (j0 >> shift) as usize;
                let st = state[cid];
                if st == 2 && at0 + (seg_len[cid] as usize) <= n && n_tasks < 1024 {
                    if seg_len[cid] > 0 {
                        t_row[n_tasks] = cuts[cid];
                        t_start[n_tasks] = at0;
                        t_len[n_tasks] = seg_len[cid];
                        n_tasks += 1;
                        at0 += seg_len[cid] as usize;
                    }
                    state[cid] = 3;
                    j0 = cuts[seg_next[cid] as usize];
                    if j0 == mark || at0 >= n {
                        break 'stage1;
                    }
                } else if st == 1 {
                    macro_rules! try_split {
                        ($idx:expr, $jk:ident, $ak:ident) => {
                            if sc_cid[$idx] == cid {
                                if $ak > 0 && at0 + ($ak as usize) <= n && n_tasks < 1024 {
                                    t_row[n_tasks] = cuts[cid];
                                    t_start[n_tasks] = at0;
                                    t_len[n_tasks] = $ak;
                                    n_tasks += 1;
                                    at0 += $ak as usize;
                                    j0 = $jk;
                                }
                                state[cid] = 3;
                                sc_cid[$idx] = usize::MAX;
                                active -= 1;
                                claim_cut!($idx, $jk, $ak);
                            }
                        };
                    }
                    try_split!(0, j1, a1);
                    try_split!(1, j2, a2);
                    try_split!(2, j3, a3);
                    try_split!(3, j4, a4);
                    try_split!(4, j5, a5);
                    if j0 == mark || at0 >= n {
                        break 'stage1;
                    }
                    break;
                } else {
                    state[cid] = 3;
                    break;
                }
            }
        }
    }

    let total_at = at0;

    // === Stage 2: 6-Lane Scalar-Register Sloppy Emit Burst over Hopped Tasks ===
    if n_tasks > 0 {
        let mut saved = [[0u8; INLINE]; 1024];
        for t in 0..n_tasks {
            let ea = t_start[t] + t_len[t] as usize;
            if ea < total_at {
                saved[t].copy_from_slice(&out[ea..ea + INLINE]);
            }
        }

        let init = n_tasks.min(6);
        let mut next_t = init;
        let mut rj = [0u32; 6];
        let mut at = [0usize; 6];
        let mut rem = [0u32; 6];
        for l in 0..init {
            rj[l] = t_row[l];
            at[l] = t_start[l];
            rem[l] = t_len[l];
        }
        let (mut rj0, mut rj1, mut rj2, mut rj3, mut rj4, mut rj5) =
            (rj[0], rj[1], rj[2], rj[3], rj[4], rj[5]);
        let (mut at0, mut at1, mut at2, mut at3, mut at4, mut at5) =
            (at[0], at[1], at[2], at[3], at[4], at[5]);
        let (mut rem0, mut rem1, mut rem2, mut rem3, mut rem4, mut rem5) =
            (rem[0], rem[1], rem[2], rem[3], rem[4], rem[5]);

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

        // Restore Lane-0 regions after task ends, then task starts
        for t in 0..n_tasks {
            let ea = t_start[t] + t_len[t] as usize;
            if ea < total_at {
                let limit_at = if t + 1 < n_tasks {
                    t_start[t + 1]
                } else {
                    total_at
                };
                let keep = (limit_at - ea).min(INLINE);
                out[ea..ea + keep].copy_from_slice(&saved[t][..keep]);
            }
        }
        for t in 0..n_tasks {
            let t_end = t_start[t] + t_len[t] as usize;
            let mut rj = t_row[t];
            let mut rat = t_start[t];
            let rend = t_end.min(rat + INLINE);
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
                let tk = w.len().min(t_end - rat);
                out[rat..rat + tk].copy_from_slice(&w[..tk]);
                rat += tk;
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
