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
use crate::bwt::{InverseBwt, Segmented};
use crate::{runs::Runs, Kernels};

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
    #[cfg(feature = "forward-anchor-lanes")]
    lanes: lanes::Arena,
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
/// counts (`512/512` on the shipped path; `FOLD_CHUNK/FOLD_STRIDE` with a 16-rank guard zone
/// under `mtf-fold`)
struct Pass<'a, const CHUNK: usize = 512, const STRIDE: usize = 512> {
    queues: outq::QueuesOf<'a, CHUNK, STRIDE>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    f: First,
}

impl<const CHUNK: usize, const STRIDE: usize> Pass<'_, CHUNK, STRIDE> {
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

    #[cold]
    #[inline(never)]
    fn grow_edges(&mut self, at: usize) {
        let need = (at + 3)
            .max(2 * self.edges.len())
            .max(1024)
            .min(self.cap_strips.saturating_mul(3));
        self.edges.reserve_exact(need - self.edges.len());
        self.edges.resize(need, 0);
    }

    /// One piece of a run: retained (`keep`) or a thick strip
    #[inline(always)]
    fn piece<const QUEUE: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        if keep {
            let f = &mut self.f;
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            if QUEUE {
                self.queues.push(a, f.m, w, &f.thin);
            }
            f.thin[a] += w;
            f.m += w;
        } else {
            if self.ns < self.cap_strips {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    self.grow_edges(at);
                }
                let f = &self.f;
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            self.f.thick[a] += 1;
            self.f.thick_rows[a] += w;
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

    /// Research only (`mtf-fold`): first pass using `OutQ16`'s guarded chunk (`FOLD_CHUNK` /
    /// `FOLD_STRIDE`)
    #[cfg(feature = "mtf-fold")]
    fn begin_folded<'a>(
        &'a mut self,
        out: &'a mut [u8],
        limit: u32,
        cap_strips: usize,
    ) -> Pass<'a, { outq::FOLD_CHUNK }, { outq::FOLD_STRIDE }> {
        Pass {
            queues: outq::QueuesOf::<{ outq::FOLD_CHUNK }, { outq::FOLD_STRIDE }>::new(
                out,
                &mut self.spill,
                limit,
            ),
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
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let ns = pass.ns;
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        #[cfg(test)]
        {
            self.spill.taken = pass.queues.chunks();
        }
        if over || f.m > cap.m || ns > cap.strips {
            return None;
        }
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::OpenPass1End, crate::trace::NONE);
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
        self.finish_with::<INDUCE, QUEUE, 512, 512>(f, ns, records, n, p, limit, out, heads)
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_with<
        const INDUCE: bool,
        const QUEUE: bool,
        const CHUNK: usize,
        const STRIDE: usize,
    >(
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
        #[cfg(feature = "a6b-endpoint-redesign")]
        let use_redesign = INDUCE && !self.wide(ns);
        #[cfg(not(feature = "a6b-endpoint-redesign"))]
        let use_redesign = false;
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
        self.tiles.clear();
        if !use_redesign {
            let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
            let image_order = &mut rest_edges[..ns];
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
        }
        self.bands.clear();
        self.spelling.clear();
        if INDUCE && n >= 65_536 {
            self.spelling.reserve_exact(48 * 1024);
        }
        self.offsets.clear();
        if !INDUCE {
            self.offsets.push(0);
        }
        let mut cursors = if INDUCE {
            #[cfg(feature = "a6b-endpoint-redesign")]
            if use_redesign {
                self.induced_redesigned(ns, n, &first, &mut write, &thin, &mut stats)
            } else {
                self.induced(ns, n, &thin, &mut stats)
            }
            #[cfg(not(feature = "a6b-endpoint-redesign"))]
            self.induced(ns, n, &thin, &mut stats)
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
        // (research, `forward-anchor-lanes`) a block the four-lane walk takes keeps forward links
        #[cfg(feature = "forward-anchor-lanes")]
        let xor = !lanes::fits(wide, self.lens.len(), m, bits);
        #[cfg(not(feature = "forward-anchor-lanes"))]
        let xor = true;
        if QUEUE && !wide && m >= 65_536 && xor {
            let edges = &mut self.edges[..m as usize];
            edges.fill(0);
            let mut predecessor = mark;
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::ReaderOf::<CHUNK, STRIDE>::new(out, &self.spill, heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor(
                        edges,
                        band.dst() as usize,
                        (band.end() - k) as usize,
                        band.code() << bits,
                        mark,
                        &mut predecessor,
                    );
                    k = band.end();
                }
            }
            self.predecessor = Some(predecessor);
        } else if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::ReaderOf::<CHUNK, STRIDE>::new(out, &self.spill, heads, a);
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
        #[cfg(feature = "ideal-construction")]
        {
            use crate::ideal::{construction as ic, Mode};
            if ic::mode() == Mode::Replay {
                std::hint::black_box(records);
                if let Some(idx) = crate::ideal::current_block() {
                    match ic::with_built(idx, n as u32, |b| {
                        walk_slices(
                            b.predecessor,
                            &b.edges,
                            &b.extra,
                            &b.spelling,
                            &b.inline16,
                            &b.lens,
                            n,
                            b.mark,
                            b.bits,
                            out,
                        );
                        Counts {
                            n,
                            retained: b.m as usize,
                            ..Counts::default()
                        }
                    }) {
                        ic::Replay::Built(stats) => return Some(stats),
                        ic::Replay::Refused => return None,
                        ic::Replay::Miss => {}
                    }
                }
            }
        }
        let (stats, mark, bits) =
            self.construct_and_record::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, bits, out);
        Some(stats)
    }

    /// Runs `construct_capped` and, under `ideal-construction`, counts the real construction
    /// and records Open's built structures (with the 256 literal slots pre-populated) in
    /// `Mode::Record`.
    #[inline(always)]
    fn construct_and_record<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        cap: Cap,
        out: &mut Vec<u8>,
    ) -> Option<(Counts, u32, u32)> {
        #[cfg(feature = "ideal-construction")]
        crate::ideal::construction::count_constructed();
        let res = self.construct_capped::<INDUCE, QUEUE>(records, n, p, limit, cap, out);
        #[cfg(feature = "ideal-construction")]
        if crate::ideal::construction::mode() == crate::ideal::Mode::Record {
            if let Some(idx) = crate::ideal::current_block() {
                match &res {
                    Some((stats, mark, bits)) => crate::ideal::construction::record_built(
                        idx,
                        n,
                        stats.retained as u32,
                        *mark,
                        *bits,
                        self.predecessor,
                        &self.edges[..stats.retained],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                    ),
                    None => crate::ideal::construction::record_refused(idx),
                }
            }
        }
        res
    }

    /// The walk from the marked row using the constructed dictionary: the block's bytes into `out`
    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
        #[cfg(feature = "forward-anchor-lanes")]
        if self.predecessor.is_none()
            && lanes::fits(!self.extra.is_empty(), self.lens.len(), m as u32, bits)
        {
            lanes::walk(
                &mut self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                bits,
                &mut self.lanes,
                out,
            );
            return;
        }
        walk_slices(
            self.predecessor,
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

/// Emit from borrowed Open slices (zero copy): called both from `Kernel::walk` and directly on
/// `&ideal::construction::Built`.
#[allow(clippy::too_many_arguments)]
fn walk_slices(
    predecessor: Option<u32>,
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
    if let Some(before) = predecessor {
        emit_bidirectional(edges, spelling, inline16, lens, n, mark, before, bits, out);
        return;
    }
    emit(edges, extra, spelling, inline16, lens, n, mark, bits, out);
}

/// Rows a thin run may have, as retained ranks (the arm `Open<16, true, true>` measured on the
/// i9: bench-open-return, the moonshot runs)
pub(crate) const LIMIT: u32 = 16;

/// Post-#1080 + #1210 two-tier open-kernel admission
/// (`docs/journal/2026-11-27-explore/route-crossover-post-1080.md`,
/// `docs/journal/2026-11-27-explore/composed-post-0516-stack.md`):
/// - `1/64 <= r/n < 0.10` (`OPEN_OUTRIGHT`): admitted outright (`Cap::NONE`), zero `within_cap`
///   overhead on 194/233 full `wikidata` blocks;
/// - `0.10 <= r/n < 0.21` (`OPEN_BAND`): checked by `within_cap` against `Cap::dense(n)`
///   (`m <= min(256_000, 0.27 * n)`, `strips <= 16_384`), keeping all 233/233 full `wikidata`
///   blocks (including the 5 dense blocks at `r/n = 0.151..0.206`, `m/n = 0.221..0.257`) inside
///   `OpenCells` so 8-worker `OpenCells` workers never allocate `Segmented`'s ~6.5–8.4 MiB/worker
///   scratch (`<= 1.01x` 8W RSS), while catching high-`m/n` `text` blocks (`r/n = 0.142..0.146`,
///   `m/n = 0.42`) and routing them straight to `Segmented`;
/// - `r/n >= 0.21` (and `r/n < 1/64`): straight to `Segmented` without running `within_cap`,
///   dropping the dead `[0.21, 0.28)` dense tail.
const OPEN_OUTRIGHT: (usize, usize) = (1, 10);
const OPEN_BAND: (usize, usize) = (21, 100);
const DENSE_M: u32 = 256_000;
const DENSE_M_RATIO: (u64, u64) = (27, 100);
const DENSE_STRIPS: usize = 16_384;

/// What the open kernel may take of a block, checked by its first pass
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Cap {
    /// retained rows
    pub(crate) m: u32,
    /// thick strips
    pub(crate) strips: usize,
}

impl Cap {
    /// No cap: blocks in `1/64 <= r/n < 0.10` (`OPEN_OUTRIGHT`), and the test route
    pub(crate) const NONE: Cap = Cap {
        m: u32::MAX,
        strips: usize::MAX,
    };

    /// Structural cap for `0.10 <= r/n < 0.21` (`m <= min(256_000, 0.27 * n)`)
    pub(crate) fn dense(n: usize) -> Cap {
        Cap {
            m: DENSE_M.min((n as u64 * DENSE_M_RATIO.0 / DENSE_M_RATIO.1) as u32),
            strips: DENSE_STRIPS,
        }
    }
}

/// The workers' default inverse-BWT kernel: the open kernel outright on run-route blocks in
/// `1/64 <= r/n < 0.10`, and in `0.10 <= r/n < 0.21` when `within_cap` finds `m <= 0.27 * n`
/// and `strips <= 16_384`; `Segmented` directly on every other block (`r/n >= 0.21`, `r/n < 1/64`,
/// and the column route). The output is the same either way.
pub type OpenCells = OpenCellsOf<false>;
/// Test and fuzz route: the open kernel on every run-route block its entry condition admits,
/// whatever the record count (`OpenCells` otherwise)
#[doc(hidden)]
pub type ForcedOpenCells = OpenCellsOf<true>;

/// `OpenCells`, or with `FORCED` its test and fuzz route
#[doc(hidden)]
#[derive(Default)]
pub struct OpenCellsOf<const FORCED: bool> {
    open: Kernel,
    fallback: Segmented,
    /// (test only) blocks the open kernel inverted
    #[cfg(test)]
    pub(crate) opened: usize,
    /// (test only) blocks refused by `within_cap` or the open kernel's first pass
    #[cfg(test)]
    pub(crate) capped: usize,
}

impl<const FORCED: bool> OpenCellsOf<FORCED> {
    /// What the open kernel may take of a run-route block of `records` records in `n` rows
    /// with the walk's row `p`, by its record count and the kernel's own entry condition:
    /// `Cap::NONE` for `1/64 <= r/n < 0.10` (and when `FORCED`), `Cap::dense(n)` for
    /// `0.10 <= r/n < 0.21`, and `None` (`Segmented`) elsewhere
    pub(crate) fn cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if FORCED {
            Some(Cap::NONE)
        } else if crate::cells::lazy_counts_admit(records, n) {
            if records * OPEN_OUTRIGHT.1 < n * OPEN_OUTRIGHT.0 {
                Some(Cap::NONE)
            } else {
                Some(Cap::dense(n))
            }
        } else if records * 64 >= n && records * OPEN_BAND.1 < n * OPEN_BAND.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }

    /// (test only) whether the open kernel takes the block outright, by its record count
    /// alone (`cap` with no cap)
    #[cfg(test)]
    pub(crate) fn admits(records: usize, n: usize, p: u32) -> bool {
        Self::cap(records, n, p) == Some(Cap::NONE)
    }
}

/// Research only (`mtf-fold`): the open kernel's first pass fed by the decoupled L1d-batched
/// MTF/RLE2 front (`runs::fold`), in place of `maximal` over the run records. On real bzip2
/// blocks (`injective == true`), `flush_batch_direct` processes packed `(c << 24) | width`
/// maximal runs with `m`/`in_out` hoisted in registers and `f.runs` incremented once per batch,
/// writing into `OutQ16`'s guarded `CHUNK = 1008 / STRIDE = 1024` layout; non-injective synthetic
/// blocks fall back to `flush_batch_merging` so `Pass::take` sees identical maximal runs.
#[cfg(feature = "mtf-fold")]
struct FoldSink<'a> {
    pass: Pass<'a, { outq::FOLD_CHUNK }, { outq::FOLD_STRIDE }>,
    p: u32,
    /// pending maximal run (for `run()` and non-injective merging; none while `width` is 0)
    c: u8,
    pos: u32,
    width: u32,
}

#[cfg(feature = "mtf-fold")]
impl FoldSink<'_> {
    /// The maximal run so far to the first pass
    #[inline(always)]
    fn flush(&mut self) {
        if self.width != 0 {
            self.pass
                .take::<true>(self.c, self.pos, self.width, self.p, LIMIT);
            self.pos += self.width;
            self.width = 0;
        }
    }

    /// Fast 100% safe batch consumer (injective `seq_to_unseq`, 100% of real bzip2 blocks):
    /// all 15 GPRs dedicated to `Pass::take + OutQ16`, zero `c == prev_c` check, `m` hoisted in a
    /// register, `f.runs` updated once per batch, and epoch-boundary checks hoisted across the
    /// whole batch whenever `(m & 0xFFFF) + runs.len() * LIMIT < 65536` (~87.5% of 512-run batches).
    #[inline(always)]
    fn flush_batch_direct(&mut self, runs: &[u32]) -> u64 {
        let no_epoch = (self.pass.f.m & 0xFFFF) + (runs.len() as u32) * LIMIT < (1 << 16);
        if no_epoch {
            self.flush_batch_no_epoch(runs)
        } else {
            self.flush_batch_fallback(runs)
        }
    }

    #[inline(never)]
    fn flush_batch_no_epoch(&mut self, runs: &[u32]) -> u64 {
        self.flush_batch_inner::<true>(runs)
    }

    #[cold]
    #[inline(never)]
    fn flush_batch_fallback(&mut self, runs: &[u32]) -> u64 {
        self.flush_batch_inner::<false>(runs)
    }

    #[inline(always)]
    fn flush_batch_inner<const NO_EPOCH: bool>(&mut self, runs: &[u32]) -> u64 {
        let mut cur = self.pos;
        let mut m = self.pass.f.m;
        let p = self.p;
        self.pass.f.runs += runs.len();
        for &packed in runs {
            let c = (packed >> 24) as u8;
            let width = packed & 0x00FF_FFFF;
            if width <= LIMIT {
                let a = usize::from(c);
                if p.wrapping_sub(cur) < width {
                    self.pass.f.mark = m + (p - cur);
                }
                self.pass
                    .queues
                    .push_thin_fast::<NO_EPOCH>(a, m, width, &self.pass.f.thin);
                self.pass.f.thin[a] += width;
                m += width;
            } else if p.wrapping_sub(cur) >= width {
                self.pass.piece::<true>(c, cur, width, false, p);
            } else {
                self.pass.f.m = m;
                self.pass.take_marked::<true>(c, cur, width, p, LIMIT);
                m = self.pass.f.m;
            }
            cur += width;
        }
        let delta = cur.wrapping_sub(self.pos);
        self.pass.f.m = m;
        self.pos = cur;
        u64::from(delta)
    }

    /// Safe fallback batch consumer when `seq_to_unseq` is non-injective: merges adjacent
    /// identical output bytes seamlessly across batch boundaries.
    #[cold]
    #[inline(never)]
    fn flush_batch_merging(&mut self, runs: &[u32]) -> u64 {
        let mut sum = 0u64;
        let mut cur = self.pos;
        let mut pc = self.c;
        let mut pw = self.width;
        let p = self.p;
        for &packed in runs {
            let c = (packed >> 24) as u8;
            let width = packed & 0x00FF_FFFF;
            sum += u64::from(width);
            if c == pc && pw != 0 {
                pw += width;
            } else {
                if pw != 0 {
                    self.pass.take::<true>(pc, cur, pw, p, LIMIT);
                    cur += pw;
                }
                pc = c;
                pw = width;
            }
        }
        self.pos = cur;
        self.c = pc;
        self.width = pw;
        sum
    }
}

#[cfg(feature = "mtf-fold")]
impl crate::runs::fold::Sink for FoldSink<'_> {
    #[inline(always)]
    fn run(&mut self, c: u8, total: u32) {
        // (the rows of all a block's runs are below 2^32 whatever its symbols: the front's
        // runs over 127 rows sum to at most `max_len` < 2^31, the others to at most 127 a
        // symbol, under 2^24 symbols)
        if c == self.c && self.width != 0 {
            self.width += total;
            return;
        }
        self.flush();
        self.c = c;
        self.width = total;
    }

    #[inline(always)]
    fn batch(&mut self, runs: &[u32], injective: bool) -> u64 {
        if injective {
            // Any pending run (e.g. the block's opening run) has a byte distinct from the first
            // literal in `runs` because `seq_to_unseq` over all accessed indices is injective.
            self.flush();
            self.flush_batch_direct(runs)
        } else {
            self.flush_batch_merging(runs)
        }
    }
}

#[cfg(feature = "mtf-fold")]
impl<const FORCED: bool> OpenCellsOf<FORCED> {
    /// Research only (`mtf-fold`): the block `b` (a run-route block of bound at most 900,000)
    /// inverted by the open kernel with its first pass fed straight from the MTF/RLE2 front,
    /// its bytes into `buf` (the worker's scratch: whatever it held). `Done(n)` only where the
    /// shipped path, `runs::front` and then `invert_runs`, would have taken the open kernel to
    /// the same bytes; otherwise nothing is handed over and the caller takes the shipped path.
    ///
    /// **`admitted(n, syms)`** is called once the first pass is done and the block admitted,
    /// before the rest of the construction and the walk: with the block's rows and its symbols'
    /// buffer (taken from `b`, which nothing reads after). That is the point at which the
    /// shipped path's worker hands the symbols back and takes its output buffer
    /// (`decode_owned`'s `take`), as near as a fused pass allows.
    ///
    /// **Its buffer before the rows are known:** `buf` is sized by the block's bound (with
    /// the pool's slack for the RLE1's growth), and the queues laid out over it
    /// (`outq::Queues`, chunks from its start); the walk sizes it to `n` itself (`emit`). The
    /// queues' contents are the shipped pass's: only where their chunks lie can differ, which
    /// the wiring reads through the same `Heads`.
    ///
    /// **Its caps, before the admission's is known:** the retained rows at most the bound
    /// (every block within its bound has at most that many, so no admitted block meets it) and
    /// the thick strips at most `bound / 17 + 2` (a strip is wider than `LIMIT`, but for the
    /// two fragments of the one holding the marked row). They bound what a block past its
    /// bound can take before the front refuses it. Once `n` and the record count are known,
    /// `cap` decides as on the shipped path: a block it refuses, or one past a dense cap, is
    /// `NotOpen`, exactly the blocks the shipped path gives to `LazyCells` (and, unforced, a
    /// block refused by its literals alone, `runs::fold::refused`, before its runs are sent).
    pub(crate) fn fold(
        &mut self,
        b: &mut crate::block::Symbols,
        scratch: &mut Runs,
        buf: &mut Vec<u8>,
        k: Kernels,
        admitted: &mut dyn FnMut(usize, Vec<u16>),
    ) -> crate::runs::fold::Outcome {
        use crate::runs::fold::{self, Outcome};
        let bound = b.max_len;
        if bound > fold::MAX_LEN {
            return Outcome::Bailed;
        }
        let p = b.orig_ptr;
        let strips = bound / (LIMIT as usize + 1) + 2;
        // (the pool's capacity for a block of this many rows, `parallel.rs`'s worker)
        let room = bound + bound / 16 + 64;
        if buf.capacity() < room {
            buf.reserve_exact(room - buf.len());
        }
        buf.resize(bound + INLINE, 0);
        let mut sink = FoldSink {
            pass: self.open.begin_folded(&mut buf[..], bound as u32, strips),
            p,
            c: 0,
            pos: 0,
            width: 0,
        };
        let refused = if FORCED {
            usize::MAX
        } else {
            fold::refused(bound)
        };
        let (n, records) = match fold::front(b, scratch, k, &mut sink, refused) {
            Ok(x) => x,
            Err(o) => return o,
        };
        sink.flush();
        let pass = sink.pass;
        let ns = pass.ns;
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        // (the shipped path's BadOrigPtr, and a block past either of the fold's own caps,
        // which no block within its bound reaches)
        if p as usize >= n || over || ns > strips {
            return Outcome::Bailed;
        }
        let Some(cap) = Self::cap(records, n, p) else {
            return Outcome::NotOpen;
        };
        if f.m > cap.m || ns > cap.strips {
            return Outcome::NotOpen;
        }
        admitted(n, std::mem::take(&mut b.syms));
        #[cfg(feature = "trace")]
        crate::trace::mark(crate::trace::Kind::OpenPass1End, crate::trace::NONE);
        let (stats, mark, bits) = self
            .open
            .finish_with::<true, true, { outq::FOLD_CHUNK }, { outq::FOLD_STRIDE }>(
                &f,
                ns,
                &[],
                n as u32,
                p,
                LIMIT,
                &buf[..],
                &heads,
            );
        self.open.walk(n, stats.retained, mark, bits, buf);
        #[cfg(test)]
        {
            self.opened += 1;
        }
        Outcome::Done(n)
    }
}

/// One pass over the run records checking whether the block fits within `cap` (retained rows
/// `m <= cap.m` and thick strips `ns <= cap.strips`), stopping early once either cap is exceeded.
/// Matches `maximal` + `split` + `Pass::take` identically: consecutive records of one byte merge
/// (cap-127 splits), runs of at most `limit` rows add to `m`, and longer runs add one thick strip
/// (or, when holding the marked row `p`, one retained row plus up to two thick fragments).
fn within_cap(records: &[u16], p: u32, limit: u32, cap: Cap) -> bool {
    let (mut pos, mut m, mut ns) = (0u32, 0u32, 0usize);
    let mut i = 0;
    while i < records.len() {
        let rec = records[i];
        let c = rec >> 8;
        let mut width = u32::from(rec & 0xFF);
        i += 1;
        while i < records.len() && (records[i] >> 8) == c {
            width += u32::from(records[i] & 0xFF);
            i += 1;
        }
        if width <= limit {
            m += width;
        } else if p.wrapping_sub(pos) >= width {
            ns += 1;
        } else {
            m += 1;
            ns += usize::from(pos < p) + usize::from(p + 1 < pos + width);
        }
        if m > cap.m || ns > cap.strips {
            return false;
        }
        pos += width;
    }
    true
}

impl<const FORCED: bool> InverseBwt for OpenCellsOf<FORCED> {
    fn reads_runs(&self) -> bool {
        true
    }
    #[cfg(feature = "mtf-fold")]
    fn folds(&self) -> bool {
        true
    }
    #[cfg(feature = "mtf-fold")]
    fn fold_runs(
        &mut self,
        b: &mut crate::block::Symbols,
        scratch: &mut Runs,
        buf: &mut Vec<u8>,
        k: Kernels,
        admitted: &mut dyn FnMut(usize, Vec<u16>),
    ) -> crate::runs::fold::Outcome {
        self.fold(b, scratch, buf, k, admitted)
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        let recs = r.records();
        if let Some(cap) = Self::cap(recs.len(), n, p) {
            // Skip the scalar `within_cap` pre-scan on `lazy_counts_admit` blocks
            // (`0.10 <= r/n < 0.15`): on Wikidata all 30/30 blocks pass the cap, so the
            // separate scan was pure wasted CPU (+2.63% 8T / +1.52% 10T on 0.5.17 i9,
            // `5182d546`). `try_run` enforces the exact same `cap` inside Pass 1 and falls
            // back cleanly to `Segmented` on overflow, preserving identical routes on every
            // block. Keep `within_cap` in `[0.15, 0.21)` where most off-Wikidata blocks refuse.
            if cap.m != u32::MAX
                && !crate::cells::lazy_counts_admit(recs.len(), n)
                && !within_cap(recs, p, LIMIT, cap)
            {
                #[cfg(test)]
                {
                    self.capped += 1;
                }
                return self.fallback.invert_runs(r, n, p, out, k);
            }
            let ran = self.open.try_run::<true, true>(recs, n, p, LIMIT, cap, out);
            #[cfg(feature = "census")]
            if let Some(c) = &ran {
                crate::census::record(r.records(), n, p, LIMIT, c);
            }
            if ran.is_some() {
                #[cfg(test)]
                {
                    self.opened += 1;
                }
                return true;
            }
            #[cfg(test)]
            {
                self.capped += 1;
            }
        }
        self.fallback.invert_runs(r, n, p, out, k)
    }
    fn invert(&mut self, l: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
        self.fallback.invert(l, c, p, out);
    }
    fn invert_in_place(&mut self, l: &mut Vec<u8>, c: &[u32; 256], p: u32) -> bool {
        self.fallback.invert_in_place(l, c, p)
    }
    fn wants_checkpoints(&self) -> bool {
        self.fallback.wants_checkpoints()
    }
    fn invert_checkpointed(
        &mut self,
        l: &[u8],
        c: &[u32; 256],
        checkpoints: &[[u32; 256]],
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.fallback.invert_checkpointed(l, c, checkpoints, p, out);
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
#[cfg(feature = "forward-anchor-lanes")]
mod lanes;
mod outq;

#[cfg(test)]
mod checks;

#[cfg(test)]
mod b5;

#[cfg(test)]
mod dense;

#[cfg(test)]
mod spirals;

#[cfg(test)]
mod mux;
