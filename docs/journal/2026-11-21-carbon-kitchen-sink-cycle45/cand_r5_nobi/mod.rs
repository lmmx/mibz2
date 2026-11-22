//! The open kernel: the inverse BWT as the first return to a union of whole LF strips, with
//! one marked row retained (Astra's open-return decoder, #565,
//! `docs/journal/2026-11-18-open-return/`, ported for production by the B5 study, #145
//! CHECKPOINT 7004759994). [`OpenCells`] is the workers' kernel wherever production ran
//! `LazyCells`: the open kernel on the blocks `LazyCells` admits by its record count
//! (`docs/journal/2026-11-19-open-admission.md`) and on the dense band above them within a
//! first-pass cap (`DENSE`, `docs/journal/2026-11-21-open-dense-admission.md`), `LazyCells` as
//! shipped on every other block.
//!
//! **The construction** (`Kernel::construct`, induced and queued, the instance that ships):
//! each byte's maximal runs are split into thin ones (at most `LIMIT` rows, retained) and thick
//! strips (whole LF strips, eliminated). The thick strips and the gaps between them form an
//! interval exchange whose elimination (`endpoint.rs`) gives, for every retained row, the word
//! spelled until the walk next returns to a retained row. Large queued permutations whose
//! codes fit alongside the links store the XOR of each row's neighbors. The emitter reads
//! those cycles from both ends. A short marked cycle is repeated to `n` bytes.
//!
//! **Safety:** no `unsafe`. Every index is bounds-checked. The `assert!`s are invariants of the
//! construction for every column, BWT or not: `checks.rs` audits every arrow of every binary
//! column to length 8 and ternary to 5, at every mark and five limits, and the B5 tests decode
//! arbitrary columns against the plain inverse. [`OpenCells`] admits only blocks with
//! `0 < n <= 900_000` and `p < n`, the kernel's own entry condition.

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
    // 0..255 spells one byte. 256 exits to the retained ranks.
    symbol: u16,
}

/// Absolute source end and destination start each fit in 20 bits (n <= 900,000).
/// The high 24 bits hold the dictionary code.
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
                f.mark_byte = c;
            }
            if QUEUE {
                self.queues.push_thin(a, f.m, w);
            } else {
                f.thin[a] += w;
            }
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
                let thin_a = if QUEUE { self.queues.thin_of(a) } else { f.thin[a] };
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

/// The construction's first pass as it goes, run by run: each byte's rows, retained rows and
/// thick strips, the retained rows so far, the marked row's rank, and the maximal runs
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
                self.offsets.push(self.spelling.len() as u32);
                254 + self.offsets.len() as u32
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
            f: First::default(),
        }
    }

    #[cfg(test)]
    fn construct<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> (Counts, u32, u32) {
        self.construct_capped::<INDUCE, QUEUE>(records, n, p, limit, Cap::NONE, out)
            .expect("no cap")
    }

    /// `construct`, or `None` once the first pass finds the block past `cap` (its retained
    /// rows or its thick strips): nothing after the first pass is done, and `out` holds
    /// nothing of the block's
    fn construct_capped<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(Counts, u32, u32)> {
        // the walk's buffer, `n` bytes and its copies' 16: the queues' until the walk
        out.resize(n as usize + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m, cap.strips);
        {
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= limit {
                    // Canonical runs.rs invariant (CAP = 127 > LIMIT = 16): no continuation possible!
                    pass.piece::<QUEUE>(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take::<QUEUE>(c, pos, width, p, limit);
                    pos += width;
                }
            }
        }
        let ns = pass.ns;
        let mut f = pass.f;
        if QUEUE {
            f.thin = pass.queues.extract_thin();
        }
        let (heads, over) = (pass.queues.heads(), pass.queues.over());
        #[cfg(test)]
        {
            self.spill.taken = pass.queues.chunks();
        }
        if over || f.m > cap.m || ns > cap.strips {
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
            ..Counts::default()
        };
        let all = std::array::from_fn(|a| thin[a] + thick_rows[a]);
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
        if !INDUCE {
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
        if n >= 65_536 && self.bands.capacity() < 8192 { self.bands.reserve_exact(8192); }
        self.spelling.clear();
        if INDUCE && n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        self.offsets.clear();
        if !INDUCE {
            self.offsets.push(0);
        }
        let mut cursors = if INDUCE {
            self.induced(ns, n, &thin, &first, &mut write, &mut stats)
        } else {
            let mut cursors = [0usize; 256];
            let (mut k, mut si) = (0, 0);
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let mut x = first[a];
                while si < write[a] as usize {
                    let i = self.edges[3 * ns + si] as usize;
                    let s = Strip::unpack(&self.edges[3 * i..3 * i + 3]);
                    self.trace(x, s.to, a as u8, &mut k, &mut stats);
                    x = s.to + s.width;
                    si += 1;
                }
                self.trace(x, first[a] + all[a], a as u8, &mut k, &mut stats);
            }
            assert_eq!(k, m);
            inline_words(
                &self.spelling,
                &self.offsets,
                &mut self.inline16,
                &mut self.lens,
            );
            cursors
        };
        stats.bands = self.bands.len();
        // the image order and the arena are dead from here: `edges` is written over them
        #[cfg(test)]
        if self.endpoint.clobber == endpoint::Clobber::AllBeforeWire {
            self.edges.fill(0xA5A5_A5A5);
        }
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
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
        self.predecessor = None;
        if false && QUEUE && !wide && m >= 65_536 {
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
        } else if QUEUE {
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
        (stats, mark, bits)
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
    pub(crate) fn try_run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<Counts> {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, bits, out);
        Some(stats)
    }

    /// The walk from the marked row using the constructed dictionary: the block's bytes into `out`
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

/// Long words use their otherwise unused inline slot for two spelling offsets.
fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
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

/// Keep the literal entries across blocks and reserve exactly the new word count.
fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
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
    inline16.reserve_exact(words);
    lens.reserve_exact(words);
}

/// Build the tracing constructor's dictionary from its contiguous spellings.
fn inline_words(
    spelling: &[u8],
    offsets: &[u32],
    inline16: &mut Vec<[u8; INLINE]>,
    lens: &mut Vec<u8>,
) {
    begin_inline(offsets.len() - 1, inline16, lens);
    for w in offsets.windows(2) {
        let word = &spelling[w[0] as usize..w[1] as usize];
        let mut v = [0; INLINE];
        if word.len() <= INLINE {
            v[..word.len()].copy_from_slice(word);
        } else {
            v = long_slot(w[0], w[1]);
        }
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

/// Consume the marked cycle from both ends. Each packed edge holds the XOR
/// of its neighbors, so either neighbor determines the other. For a short
/// cycle, move the suffix beside the prefix before repeating the period.
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
        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);
        let tw = t1.elapsed().as_secs_f64() * 1e6;
        (tc, tw)
    }
    pub fn scratch_report(&self) -> (usize, usize, String) {
        use std::mem::size_of;
        let q = self.spill.scratch_bytes();
        let tiles_dir = size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
        let bands = size_of::<Band>() * self.bands.capacity();
        let spelling = self.spelling.capacity();
        let offsets = 4 * self.offsets.capacity();
        let edges = 4 * self.edges.capacity();
        let extra = 4 * self.extra.capacity();
        let inline16 = 16 * self.inline16.capacity();
        let lens = self.lens.capacity();
        let ep = self.endpoint.scratch_bytes();
        let tot = q + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
        let dict_l1 = 16 * self.inline16.len() + self.lens.len();
        let s = format!("queues={q} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}");
        (tot, dict_l1, s)
    }
}
