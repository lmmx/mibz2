/// A block's construction counts (the journal's; read by the tests and the arrow audit)
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
    /// the elimination contracted its spirals (the width certificate failed)
    pub spirals: bool,
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

impl Strip {
    #[inline(always)]
    fn unpack(w: &[u32]) -> Self {
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

/// 64-bit bit-lattice Band: `[end: 20 | dst: 20 | code: 24]` (8 bytes instead of 12 bytes!)
#[derive(Clone, Copy)]
struct Band(u64);

impl Band {
    #[inline(always)]
    fn new(end: u32, dst: u32, code: u32) -> Self {
        debug_assert!(end <= 900_000 && dst <= 900_000 && code < (1 << 24));
        Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
    }
    #[inline(always)]
    fn end(self) -> u32 { (self.0 & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn dst(self) -> u32 { ((self.0 >> 20) & 0xF_FFFF) as u32 }
    #[inline(always)]
    fn code(self) -> u32 { (self.0 >> 40) as u32 }
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
}

// Production records cap a run at 127. Selection is on maximal runs.
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

// A maximal run of `width` rows from `pos`, classified: thin (kept) or thick. A primary inside
// a thick strip is a retained singleton. The two fragments keep the classification of their
// parent even if they are short.
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

fn each_run(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
    maximal(records, |c, pos, width| {
        split(c, pos, width, p, limit, &mut visit)
    });
}

/// The first pass over a block's runs: the queues and the thick strips it writes, and its
/// counts
struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    strip_over: bool,
    f: First,
}

impl Pass<'_> {
    /// The first pass on one maximal run of `width` rows of byte `c` from row `pos`: its thin
    /// rows queued (their ranks, as retained rows), its thick strips listed. (`split`'s cases
    /// written out, so that the common two inline into the walk over the records with no call.)
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

    /// `take` on the thick run holding the marked row
    #[cold]
    #[inline(never)]
    fn take_marked<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        split(c, pos, width, p, limit, |c, x, w, keep| {
            self.piece::<QUEUE>(c, x, w, keep, p)
        });
    }

    /// One piece of a run: retained (`keep`) or a thick strip
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
            if self.ns >= self.cap_strips {
                self.strip_over = true;
            } else {
                let idx = 3 * self.ns;
                if self.edges.len() < idx + 3 {
                    let grow_to = (idx + 3).max(2 * self.edges.len()).max(1024);
                    let capped = if self.cap_strips < (usize::MAX / 4) {
                        grow_to.min(3 * self.cap_strips + 3)
                    } else {
                        grow_to
                    };
                    self.edges.resize(capped.max(idx + 3), 0);
                }
                self.edges[idx] = x;
                self.edges[idx + 1] = f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
        }
        f.all[a] += w;
    }
}

/// The construction's first pass as it goes, run by run: each byte's rows, retained rows and
/// thick strips, the retained rows so far, the marked row's rank, and the maximal runs
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

const PAGE: usize = 64;
const TERMINAL: u16 = 256;

impl Kernel {
    // Trace the leftmost uncovered row of an interval in f(S). Its least
    // right slack is the entire bundle certified by this itinerary.
    fn trace(&mut self, mut x0: u32, end: u32, c: u8, k: &mut u32, stats: &mut Counts) {
        while x0 < end {
            let mut width = end - x0;
            let mut x = x0;
            let start = self.spelling.len();
            let exit = loop {
                let mut q = self.directory[x as usize / PAGE] as usize;
                while x >= self.tiles[q].end {
                    q += 1;
                }
                let t = self.tiles[q];
                width = width.min(t.end - x);
                x = x.wrapping_add(t.delta);
                if t.symbol == TERMINAL {
                    break x;
                }
                self.spelling.push(t.symbol as u8);
            };
            let height = self.spelling.len() - start;
            let code = if height == 0 {
                u32::from(c)
            } else {
                stats.words += 1;
                stats.copies += width as usize;
                stats.traced += height;
                stats.thick_area += width as usize * height;
                stats.unit_work += usize::from(width == 1) * height;
                stats.longest = stats.longest.max(height);
                self.spelling[start..].reverse();
                self.spelling.push(c);
                let wlen = self.spelling.len() - start;
                let code = self.inline16.len() as u32;
                let mut v = [0u8; INLINE];
                if wlen <= INLINE {
                    v[..wlen].copy_from_slice(&self.spelling[start..start + wlen]);
                    self.lens.push(wlen as u8);
                } else {
                    v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                    v[4..8].copy_from_slice(&(self.spelling.len() as u32).to_le_bytes());
                    self.lens.push(LONG);
                }
                self.inline16.push(v);
                code
            };
            self.bands.push(Band::new(*k + width, exit, code));
            *k += width;
            x0 += width;
        }
    }

    /// The first pass's state, emptied for a block, its queues in `out` (`outq.rs`) for at
    /// most `limit` retained rows
    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32, cap_strips: usize) -> Pass<'a> {
        Pass {
            queues: outq::Queues::new(out, &mut self.spill, limit),
            edges: &mut self.edges,
            ns: 0,
            cap_strips,
            strip_over: false,
            f: First::default(),
        }
    }

    fn construct_capped<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(Counts, u32, u32, u32)> {
        out.resize(n as usize + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m, cap.strips);
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let (f, heads, over, ns, strip_over) = (pass.f, pass.queues.heads(), pass.queues.over(), pass.ns, pass.strip_over);
        if over || strip_over || f.m > cap.m || ns > cap.strips {
            return None;
        }
        Some(self.finish::<INDUCE, QUEUE>(&f, ns, records, n, p, limit, &out[..], &heads))
    }

    /// The construction after its first pass `f` (`records` read again only without `QUEUE`;
    /// with it, the queues in `out` from `heads`)
    #[allow(clippy::too_many_arguments)]
    fn finish<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        f: &First,
        ns: usize,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        out: &[u8],
        heads: &outq::Heads,
    ) -> (Counts, u32, u32, u32) {
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
        stats.thick = ns;
        let first = starts(all);
        let mut write = starts(thick);
        let need = if INDUCE {
            self.arena_words(ns, m as usize)
        } else {
            4 * ns
        };
        if self.edges.len() < need {
            let add = need - self.edges.len();
            self.edges.reserve_exact(add);
            self.edges.resize(need, 0);
        }
        let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
        let image_order = &mut rest_edges[..ns];
        self.tiles.clear();
        let (mut last_x, mut retained_x) = (0, 0u32);
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let width = sw[2] >> 8;
            let from = sw[0];
            let a = usize::from(byte);
            sw[1] += first[a];
            let to = sw[1];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if !INDUCE {
                if from > last_x {
                    self.tiles.push(Tile {
                        end: from,
                        delta: retained_x.wrapping_sub(last_x),
                        symbol: TERMINAL,
                    });
                    retained_x += from - last_x;
                }
                last_x = from + width;
                self.tiles.push(Tile {
                    end: last_x,
                    delta: to.wrapping_sub(from),
                    symbol: byte.into(),
                });
            }
        }
        if !INDUCE {
            if last_x < n {
                self.tiles.push(Tile {
                    end: n,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
            }
            self.directory.clear();
            let mut ti = 0;
            for x in (0..n).step_by(PAGE) {
                while self.tiles[ti].end <= x {
                    ti += 1;
                }
                self.directory.push(ti as u32);
            }
        }
        self.bands.clear();
        self.spelling.clear();
        if self.inline16.len() < 256 {
            let cap_hint = if n >= 65_536 { 1792 } else { 256 };
            self.inline16.clear();
            self.inline16.reserve_exact(cap_hint);
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.reserve_exact(cap_hint);
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
        let mut cursors = if INDUCE {
            self.induced(ns, n, &thin, &mut stats)
        } else {
            let mut cursors = [0usize; 256];
            let (mut k, mut si) = (0, 0);
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let mut x = first[a];
                while si < write[a] as usize {
                    let s_idx = self.edges[3 * ns + si] as usize;
                    let s = Strip::unpack(&self.edges[3 * s_idx..3 * s_idx + 3]);
                    self.trace(x, s.to, a as u8, &mut k, &mut stats);
                    x = s.to + s.width;
                    si += 1;
                }
                self.trace(x, first[a] + all[a], a as u8, &mut k, &mut stats);
            }
            assert_eq!(k, m);
            cursors
        };
        stats.bands = self.bands.len();
        // the image order and the arena are dead from here: `edges` is written over them
        #[cfg(test)]
        if self.endpoint.clobber == endpoint::Clobber::AllBeforeWire {
            self.edges.fill(0xA5A5_A5A5);
        }
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        // (the memory diet) `edges` keeps its length: its rows from `m` are stale, and its
        // first `m` are all written below (the bands partition the retained ranks)
        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        const XOR_GATE: u32 = 65_536;
        let xor_mode = m >= XOR_GATE && !wide;
        let mut j_neg1 = mark;
        if xor_mode {
            self.edges[..m as usize].fill(0);
            if QUEUE {
                let mut k = 0;
                let edges_m = &mut self.edges[..m as usize];
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let bend = b.end();
                        let span = (bend - k) as usize;
                        let dst = b.dst() as usize;
                        let pack = b.code() << bits;
                        queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);
                        k = bend;
                    }
                }
            } else {
                let mut stream = starts(thin);
                let mut j = 0;
                let (bands_s, edges_m) = (&self.bands[..], &mut self.edges[..m as usize]);
                each_run(records, p, limit, |c, _, w, keep| {
                    if !keep {
                        return;
                    }
                    let a = usize::from(c);
                    let mut z = stream[a];
                    let end = z + w;
                    while z < end {
                        let ci = cursors[a];
                        let b = bands_s[ci];
                        let bend = b.end();
                        let bstart = if ci == 0 { 0 } else { bands_s[ci - 1].end() };
                        let span = (end.min(bend) - z) as usize;
                        let dest = (b.dst() + (z - bstart)) as usize;
                        let pack = b.code() << bits;
                        for i in 0..span {
                            let u = dest + i;
                            let v = j + i as u32;
                            if v == mark {
                                j_neg1 = u as u32;
                            }
                            edges_m[u] ^= pack | v;
                            edges_m[v as usize] ^= u as u32;
                        }
                        j += span as u32;
                        z += span as u32;
                        cursors[a] += usize::from(z == bend);
                    }
                    stream[a] = end;
                });
            }
        } else if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = bend;
                }
            }
        } else {
            let mut stream = starts(thin);
            let mut j = 0;
            each_run(records, p, limit, |c, _, w, keep| {
                if !keep {
                    return;
                }
                let a = usize::from(c);
                let mut z = stream[a];
                let end = z + w;
                while z < end {
                    let ci = cursors[a];
                    let b = self.bands[ci];
                    let bend = b.end();
                    let bstart = if ci == 0 { 0 } else { self.bands[ci - 1].end() };
                    let span = (end.min(bend) - z) as usize;
                    let dest = (b.dst() + (z - bstart)) as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dest..dest + span].fill(bcode);
                    }
                    let mut value = pack | j;
                    for e in &mut self.edges[dest..dest + span] {
                        *e = value;
                        value = value.wrapping_add(1);
                    }
                    j += span as u32;
                    z += span as u32;
                    cursors[a] += usize::from(z == bend);
                }
                stream[a] = end;
            });
        }
        (stats, mark, j_neg1, bits)
    }

    #[cfg(test)]
    pub fn run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Counts {
        self.try_run::<INDUCE, QUEUE>(records, n, p, limit, Cap::NONE, out)
            .expect("no cap")
    }

    /// `run`, or `None` (and `out` left to the caller) if the block is past `cap`
    pub fn try_run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<Counts> {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, j_neg1, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, j_neg1, bits, out);
        Some(stats)
    }

    /// The words inlined and the walk from the marked row: the block's bytes into `out`
    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, bits: u32, out: &mut Vec<u8>) {
        if m >= 65_536 && self.extra.is_empty() {
            emit_xor_bi_no_off(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                j_neg1,
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
                mark,
                bits,
                out,
            );
        }
    }
}

/// Rows a thin run may have, as retained ranks (the arm `Open<16, true, true>` measured on the
/// i9: bench-open-return, the moonshot runs)
pub const LIMIT: u32 = 16;

/// The dense band's density line: the open kernel takes a block up to `DENSE.0` records in
/// `DENSE.1` rows (r/n below 0.28; text starts there) where `LazyCells`' own line (r/n 0.15)
/// refuses it. Wikidata's five full blocks between the two lie at r/n 0.151 to 0.206
/// (blocks 41, 56, 165, 270 and 278 of the bench corpus), and went to Segmented.
const DENSE: (usize, usize) = (7, 25);
/// The dense band's structural gate, from the first pass (the Mechanisms line's cycle 18,
/// `docs/journal/2026-11-21-carbon-cycle18-aht-whole-corpus.md` on
/// `research/carbon-word-transport`): at most `DENSE_M` retained rows and 3 in 10 of the
/// block's, and at most `DENSE_STRIPS` thick strips. Wikidata's five keep 199,189 to 231,600
/// retained rows and 568 to 2,729 strips. Past either the block goes to `LazyCells` as before,
/// and the first pass has taken no more than the cap's queues and strips.
const DENSE_M: u32 = 256_000;
const DENSE_STRIPS: usize = 16_384;

/// What the open kernel may take of a block, checked by its first pass
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cap {
    /// retained rows
    pub m: u32,
    /// thick strips
    pub strips: usize,
}

impl Cap {
    /// No cap: the blocks `LazyCells`' count gates admit, and the test route
    pub const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };

    /// The dense band's, for a block of `n` rows
    pub fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * 3 / 10) as u32),
            strips: DENSE_STRIPS,
        }
    }
}
/// The bytes of a code's word kept inline, and the longest word kept whole there
const INLINE: usize = 16;
/// `lens` of a word longer than `INLINE`: its bytes are read from `spelling` (the cold path)
const LONG: u8 = u8::MAX;

/// Every code's word as 16 bytes, its first `INLINE` padded with zeros, and its length (`LONG`
/// past 16), indexed by code: the literals 0..256, then the block's words from 256. The
/// literals are written once, when the table is first made; the words, once a block.
#[allow(dead_code)]
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
    // (the memory diet) the table to its exact size, not doubled: 256 literals and the words
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

/// The walk over the retained rows from the marked one, each row's word appended; a cycle
/// shorter than the block is repeated to `n` bytes (the journal's code, with Carbon cycle 2's
/// `U16` emit). A word of at most 16 bytes is one unconditional 16-byte copy from `inline16`
/// at the cursor, which then advances by the word's length; a longer one is a slice copy from
/// `spelling`. `out` is sized `n + 16` for the copies' overrun and truncated at the end. Its
/// bytes from an earlier block are not cleared: every byte kept is written first.
#[allow(clippy::too_many_arguments)]

#[allow(clippy::too_many_arguments)]
fn emit_xor_bi_no_off(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut prev_f = j_neg1;
    let mut cur_f = mark;
    let mut next_b = mark;
    let mut cur_b = j_neg1;
    let mut at_f = 0usize;
    let mut at_b = n;

    let max_pairs = edges.len() / 2 + 1;
    let mut started = false;
    let mut rem_pairs = max_pairs;
    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f + 32 <= at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];

        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;

        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
            at_f += usize::from(lf);
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            if at_f + INLINE <= at_b {
                let raw = u128::from_le_bytes(inline16[cb]);
                let shifted = if lbu == 16 { raw } else { raw << ((16 - lbu) * 8) };
                out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                at_b -= lbu;
            } else {
                let take = lbu.min(at_b - at_f);
                out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                at_b -= take;
            }
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f < at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;
        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            let take = lbu.min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
            at_b -= take;
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    if cur_f == cur_b && at_f < at_b {
        let cf = (edges[cur_f as usize] >> bits) as usize;
        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }

    if at_f < at_b {
        let right_len = n - at_b;
        if right_len > 0 {
            out.copy_within(at_b..n, at_f);
        }
        let period = at_f + right_len;
        out.truncate(period);
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}

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
    // (the port's one change to the walk) the cycle through the marked row visits each retained
    // row once at most, so a valid construction returns to it within `edges.len()` steps; the
    // bound and the clamp to `n` make every walk end, whatever the construction: a wrong one
    // gives a wrong block, which the block's CRC rejects, never a walk that grows without end
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
            // `at < n` here, so the copy ends within `n + 16`
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let meta = &inline16[code];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
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

/// The walk as shipped before the 16-byte emit, kept as the equivalence tests' reference
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn emit_reference(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.clear();
    out.reserve(n);
    let mask = (1 << bits) - 1;
    let mut j = mark;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        };
        j = edge & mask;
        append_code(code, spelling, offsets, out);
        if j == mark || out.len() >= n {
            break;
        }
    }
    out.truncate(n);
    repeat(n, out);
}

#[cfg(test)]
#[inline]
fn append_code(code: u32, spelling: &[u8], offsets: &[u32], out: &mut Vec<u8>) {
    if code < 256 {
        out.push(code as u8);
    } else {
        let w = (code - 256) as usize;
        out.extend_from_slice(&spelling[offsets[w] as usize..offsets[w + 1] as usize]);
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

mod endpoint;
mod outq;
impl Kernel {
    #[inline(never)]
    pub fn run_entry(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, LIMIT, cap, out)
    }
    #[inline(never)]
    pub fn run_stages(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> (f64, f64) {
        let t0 = std::time::Instant::now();
        let (stats, mark, j_neg1, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, j_neg1, bits, out);
        let tw = t1.elapsed().as_secs_f64() * 1e6;
        (tc, tw)
    }
    pub fn scratch_report(&self) -> (usize, usize, String) {
        use std::mem::size_of;
        let q = self.spill.scratch_bytes();
        let strips = 0usize;
        let tiles_dir = size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
        let bands = size_of::<Band>() * self.bands.capacity();
        let spelling = self.spelling.capacity();
        let offsets = 0usize;
        let edges = 4 * self.edges.capacity();
        let extra = 4 * self.extra.capacity();
        let inline16 = 16 * self.inline16.capacity();
        let lens = self.lens.capacity();
        let ep = self.endpoint.scratch_bytes();
        let tot = q + strips + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
        let dict_l1 = 16 * self.inline16.len() + self.lens.len();
        let s = format!(
            "queues={q} strips={strips} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}"
        );
        (tot, dict_l1, s)
    }
}