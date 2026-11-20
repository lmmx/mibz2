//! The open kernel: the inverse BWT as the first return to a union of whole LF strips, with
//! one marked row retained (Astra's open-return decoder, #565,
//! `docs/journal/2026-11-18-open-return/`, ported for production by the B5 study, #145
//! CHECKPOINT 7004759994). [`OpenCells`] is the workers' kernel wherever production ran
//! `LazyCells`: the open kernel on the blocks `LazyCells` admits by its record count, `LazyCells`
//! as shipped on every other block (`docs/journal/2026-11-19-open-admission.md`).
//!
//! **The construction** (`Kernel::construct`, induced and queued, the instance that ships):
//! each byte's maximal runs are split into thin ones (at most `LIMIT` rows, retained) and thick
//! strips (whole LF strips, eliminated). The thick strips and the gaps between them form an
//! interval exchange whose elimination (`endpoint.rs`) gives, for every retained row, the word
//! spelled until the walk next returns to a retained row. `emit` walks the retained rows from the
//! marked one, appending each word; a cycle shorter than the block is repeated to `n` bytes.
//!
//! **Safety:** no `unsafe`. Every index is bounds-checked. The `assert!`s are invariants of the
//! construction for every column, BWT or not: `checks.rs` audits every arrow of every binary
//! column to length 8 and ternary to 5, at every mark and five limits, and the B5 tests decode
//! arbitrary columns against the plain inverse. [`OpenCells`] admits only blocks with
//! `0 < n <= 900_000` and `p < n`, the kernel's own entry condition.
use crate::bwt::{InverseBwt, LazyCells};
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
}

#[derive(Clone, Copy)]
struct Strip {
    from: u32,
    to: u32,
    width: u32,
    byte: u8,
}

#[derive(Clone, Copy)]
struct Tile {
    end: u32,
    delta: u32,
    // 0..255 spells one byte. 256 exits to the retained ranks.
    symbol: u16,
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
    queues: Vec<Vec<u32>>,
    /// the thin rows' queues as one slab, a range a byte (`construct_slab`)
    slab: Vec<u32>,
    strips: Vec<Strip>,
    image_order: Vec<u32>,
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
/// counts. `base` is each byte's first slot in `queues` (all 0 for a `Vec` a byte).
struct Pass<'a, Q: Queues> {
    queues: Q,
    strips: &'a mut Vec<Strip>,
    base: [u32; 256],
    f: First,
}

impl<Q: Queues> Pass<'_, Q> {
    /// The first pass on one maximal run of `width` rows of byte `c` from row `pos`: its thin
    /// rows queued (their ranks, as retained rows), its thick strips listed. (`split`'s cases
    /// written out, so that the common two inline into the front's walk with no call.)
    #[inline(always)]
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
        self.f.runs += 1;
        if width <= limit {
            self.piece::<QUEUE>(c, pos, width, true, p);
            return;
        }
        // (the records `runs::front` cuts a run into past its first, 127 rows a record: none
        // for a thin run while the limit is below 128, as `LIMIT` is)
        self.f.cut += (width as usize - 1) / 127;
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

    /// One piece of a run: retained (`keep`) or a thick strip. A retained piece moves its
    /// byte's cursor alone; a byte's rows so far, which a thick strip needs, are its retained
    /// rows (the cursor less `base`) and its thick rows (Carbon cycle 11's lazy `all_c`).
    #[inline(always)]
    fn piece<const QUEUE: bool>(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
        let a = usize::from(c);
        let f = &mut self.f;
        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }
            if QUEUE {
                self.queues.push(a, f.cur[a], f.m, w);
            }
            f.cur[a] += w;
            f.m += w;
        } else {
            self.strips.push(Strip {
                from: x,
                to: f.cur[a] - self.base[a] + f.thick_rows[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
            f.thick_rows[a] += w;
        }
    }
}

/// Where the first pass queues a byte's thin rows
trait Queues {
    /// The `w` rows from rank `m`, at slot `at` (the byte's cursor) of byte `a`'s queue
    fn push(&mut self, a: usize, at: u32, m: u32, w: u32);
}

/// A `Vec` a byte, each grown as it fills (the fused path's, which has no counts before its
/// pass, and the reference the slab is tested against)
impl Queues for &mut [Vec<u32>; 256] {
    #[inline(always)]
    fn push(&mut self, a: usize, at: u32, m: u32, w: u32) {
        push_rows(&mut self[a], at, m, w);
    }
}

/// The rows a fixed store writes for a run of at most this many (`push_rows`, `Slab`)
const BLOCK: usize = 16;

/// One slab for every byte's queue, partitioned before the pass (Carbon cycle 11's `FlatQ`):
/// byte `a`'s slots from `base[a]`, the cursor absolute. No capacity to read or grow: a store
/// is one bounds check against the slab's end. A store past that end is not made, and marks
/// the slab `over`; the partition's check after the pass (`construct_slab`) then sends the
/// block to the `Vec` queues.
struct Slab<'a> {
    rows: &'a mut [u32],
    over: bool,
}

impl Queues for Slab<'_> {
    #[inline(always)]
    fn push(&mut self, _: usize, at: u32, m: u32, w: u32) {
        let at = at as usize;
        let take = BLOCK.max(w as usize);
        match self.rows.get_mut(at..at + take) {
            Some(rows) if w as usize <= BLOCK => {
                let iota: [u32; BLOCK] = std::array::from_fn(|i| m.wrapping_add(i as u32));
                rows.copy_from_slice(&iota);
            }
            Some(rows) => {
                for (i, q) in rows.iter_mut().enumerate() {
                    *q = m + i as u32;
                }
            }
            None => self.over = true,
        }
    }
}

/// The counts `runs::front` leaves beside a block's records: each byte's rows and records
#[derive(Clone, Copy)]
pub(crate) struct Meta<'a> {
    pub(crate) counts: &'a [u32; 256],
    pub(crate) per: &'a [u32; 256],
}

/// Each byte's slots in the slab, from its rows and records: at most `limit` retained rows a
/// record (1 at a limit of 0: the marked row) and no more than its rows (the bound
/// `construct_slab` checks), and `BLOCK` - 1 slots for the last store's spare rows. A byte
/// with no rows has none.
fn room(meta: Meta, limit: u32) -> [u32; 256] {
    std::array::from_fn(|a| {
        let rows = meta.counts[a];
        if rows == 0 {
            0
        } else {
            rows.min(limit.max(1).saturating_mul(meta.per[a]))
                .saturating_add(BLOCK as u32 - 1)
        }
    })
}

/// The construction's first pass as it goes, run by run: each byte's rows, retained rows and
/// thick strips, the retained rows so far, the marked row's rank, and the maximal runs
#[derive(Clone, Copy)]
struct First {
    /// each byte's queue cursor: its first slot (`Pass::base`) plus its retained rows
    cur: [u32; 256],
    /// each byte's rows in thick strips, and its strips
    thick_rows: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
    /// the records past each thick run's first that `runs::front` would write (`Fused`'s
    /// record count, with `runs`, at a limit below 128)
    cut: usize,
}

impl Default for First {
    fn default() -> Self {
        First {
            cur: [0; 256],
            thick_rows: [0; 256],
            thick: [0; 256],
            m: 0,
            mark: 0,
            runs: 0,
            cut: 0,
        }
    }
}

/// Append the `w` rows from `m` to a queue of `len` rows. A thin run has at most `LIMIT` = 16
/// rows, so the queue takes a fixed block of 16 (an iota from `m`) and keeps the first `w`: no
/// loop whose trip count is the run's width, whose exit mispredicts on blocks of 48k-126k short
/// runs (Carbon cycle 2's `U16Q`, `docs/journal/2026-11-20-carbon-cycle2-bvsmc.md` §2.4).
///
/// The queue's buffer is kept at its full length, its rows past `len` stale or zero, and its
/// length is the caller's (`First::cur`, its byte's retained rows), so a run is one bounds
/// check and two 32-byte stores with no length written back to the heap (Carbon Kitchen Sink
/// cycle 3's `Vec16` queue: a cursor per byte, `q_cur[c] += w`). The rows past `w` land in the
/// spare rows and are overwritten by the next run. A wider run (a limit above 16, in the tests)
/// is written row by row.
#[inline(always)]
fn push_rows(queue: &mut Vec<u32>, len: u32, m: u32, w: u32) {
    let at = len as usize;
    let need = at + BLOCK.max(w as usize);
    if queue.len() < need {
        grow(queue, need);
    }
    if w as usize <= BLOCK {
        let iota: [u32; BLOCK] = std::array::from_fn(|i| m.wrapping_add(i as u32));
        queue[at..at + BLOCK].copy_from_slice(&iota);
    } else {
        for (i, q) in queue[at..at + w as usize].iter_mut().enumerate() {
            *q = m + i as u32;
        }
    }
}

/// A queue's buffer to `need` rows at least: doubled, from 1024 rows
#[cold]
#[inline(never)]
fn grow(queue: &mut Vec<u32>, need: usize) {
    queue.resize(need.max(2 * queue.len()).max(1024), 0);
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
            self.bands.push(Band {
                end: *k + width,
                delta: exit.wrapping_sub(*k),
                code,
            });
            *k += width;
            x0 += width;
        }
    }

    /// The first pass's state, emptied for a block, with a `Vec` a byte for the queues. (They
    /// keep their buffers and stale rows: a queue's length is its byte's `First::cur`.)
    fn begin(&mut self) -> Pass<'_, &mut [Vec<u32>; 256]> {
        self.strips.clear();
        self.queues.resize_with(256, Vec::new);
        Pass {
            queues: (&mut self.queues[..]).try_into().expect("256 queues"),
            strips: &mut self.strips,
            base: [0; 256],
            f: First::default(),
        }
    }

    /// The construction from a block's records, with its counts (`meta`) where `runs::front`
    /// gives them: the queues then go in the slab (`construct_slab`), else a `Vec` a byte
    fn construct<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        meta: Option<Meta>,
        n: u32,
        p: u32,
        limit: u32,
    ) -> (Counts, u32, u32) {
        if let (true, Some(meta)) = (QUEUE, meta) {
            if let Some(done) = self.construct_slab::<INDUCE>(records, meta, n, p, limit) {
                return done;
            }
        }
        let mut pass = self.begin();
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let f = pass.f;
        self.finish::<INDUCE, QUEUE>(&f, None, records, n, p, limit)
    }

    /// The construction with its queues in one slab, partitioned before the pass by `room`
    /// from the block's counts (Carbon cycle 11's `FlatQ`). None, if the counts are not the
    /// records' or the partition did not hold, leaves the block to the `Vec` queues.
    ///
    /// **Why it holds.** A byte's retained rows are rows of that byte, so at most its
    /// `counts`. They are its thin maximal runs' rows, at most `limit` each, and the marked
    /// row when it falls in a thick run. Every maximal run of the byte holds a record of it at
    /// least, so it has at most `per` maximal runs, and with the marked row's thick run among
    /// them at most `per - 1` thin ones: `limit * (per - 1) + 1 <= max(limit, 1) * per`
    /// retained rows.
    /// A store writes `BLOCK` slots from the cursor for a run of at most `BLOCK` rows (else
    /// the run's rows), so a byte's stores end at most `BLOCK - 1` past its last retained
    /// row: within `room`. **Checked, not trusted:** after the pass, each byte's retained
    /// rows plus `BLOCK - 1` are within its room (no store reached the next byte's), and no
    /// store fell past the slab's end; otherwise the block is built again with the `Vec`s.
    fn construct_slab<const INDUCE: bool>(
        &mut self,
        records: &[u16],
        meta: Meta,
        n: u32,
        p: u32,
        limit: u32,
    ) -> Option<(Counts, u32, u32)> {
        if meta.counts.iter().map(|&w| u64::from(w)).sum::<u64>() != u64::from(n) {
            return None;
        }
        let room = room(meta, limit);
        let base = starts(room);
        let total = (base[255] + room[255]) as usize;
        if self.slab.len() < total {
            // (a new zeroed buffer, not `resize`: its pages are touched only where stores land)
            self.slab = vec![0; total];
        }
        self.strips.clear();
        let mut pass = Pass {
            queues: Slab {
                rows: &mut self.slab[..total],
                over: false,
            },
            strips: &mut self.strips,
            base,
            f: First {
                cur: base,
                ..First::default()
            },
        };
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, limit)
        });
        let (f, over) = (pass.f, pass.queues.over);
        let fits = (0..256).all(|a| {
            let thin = f.cur[a] - base[a];
            thin == 0 || thin + (BLOCK as u32 - 1) <= room[a]
        });
        if over || !fits {
            return None;
        }
        Some(self.finish::<INDUCE, true>(&f, Some(&base), records, n, p, limit))
    }

    /// The construction after its first pass `f`, its queues in the slab from `base` where
    /// given, else a `Vec` a byte (`records` read again only without `QUEUE`)
    #[allow(clippy::too_many_arguments)]
    fn finish<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        f: &First,
        base: Option<&[u32; 256]>,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
    ) -> (Counts, u32, u32) {
        let First {
            cur,
            thick_rows,
            thick,
            m,
            mark,
            runs,
            cut: _,
        } = *f;
        let slab = base;
        let base = base.copied().unwrap_or([0; 256]);
        let thin: [u32; 256] = std::array::from_fn(|a| cur[a] - base[a]);
        let all: [u32; 256] = std::array::from_fn(|a| thin[a] + thick_rows[a]);
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
        self.image_order.resize(self.strips.len(), 0);
        self.tiles.clear();
        let (mut last_x, mut retained_x) = (0, 0u32);
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            self.image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if !INDUCE {
                if s.from > last_x {
                    self.tiles.push(Tile {
                        end: s.from,
                        delta: retained_x.wrapping_sub(last_x),
                        symbol: TERMINAL,
                    });
                    retained_x += s.from - last_x;
                }
                last_x = s.from + s.width;
                self.tiles.push(Tile {
                    end: last_x,
                    delta: s.to.wrapping_sub(s.from),
                    symbol: s.byte.into(),
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
        self.offsets.clear();
        self.offsets.push(0);
        let mut cursors = if INDUCE {
            self.induced(n, &thin, &mut stats)
        } else {
            let mut cursors = [0usize; 256];
            let (mut k, mut si) = (0, 0);
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let mut x = first[a];
                while si < write[a] as usize {
                    let s = self.strips[self.image_order[si] as usize];
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
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        self.edges.resize(m as usize, 0);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (self.edges.len() + self.extra.len());
        if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let queue: &[u32] = match slab {
                    Some(base) => &self.slab[base[a] as usize..],
                    None => &self.queues[a],
                };
                let mut from = 0;
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    let src = &queue[from..from + span];
                    for (target, &j) in self.edges[dst..dst + span].iter_mut().zip(src) {
                        *target = pack | j;
                    }
                    from += span;
                    k = b.end;
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
                    let b = self.bands[cursors[a]];
                    let span = (end.min(b.end) - z) as usize;
                    let dest = z.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dest..dest + span].fill(b.code);
                    }
                    let mut value = pack | j;
                    for e in &mut self.edges[dest..dest + span] {
                        *e = value;
                        value = value.wrapping_add(1);
                    }
                    j += span as u32;
                    z += span as u32;
                    cursors[a] += usize::from(z == b.end);
                }
                stream[a] = end;
            });
        }
        (stats, mark, bits)
    }

    /// The block of `records` (with their counts, `meta`, where known) in `n` rows from the
    /// marked row `p`, its bytes into `out`
    pub(crate) fn run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        meta: Option<Meta>,
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Counts {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, bits) =
            self.construct::<INDUCE, QUEUE>(records, meta, n as u32, p, limit);
        self.walk(n, mark, bits, out);
        stats
    }

    /// The block of symbols `b` from its symbols, the construction's first pass fused into the
    /// front's literal walk (`LRB`, Carbon Kitchen Sink cycle 3): no records, no cut at 127,
    /// no second read of them. Its rows and records (as `runs::front` would count them) once the
    /// first pass is done; `admit` then decides, from those, whether the kernel finishes the
    /// block. None leaves the block to the records' path, `out` untouched: where the front
    /// would refuse it or take its scalar loop, where a run repeats the byte before it, where
    /// the rows would pass `max_len` or 900,000, where `p` is not below the rows, and where
    /// `admit` refuses (`dense_from`: as soon as the maximal runs reach it, before the walk ends).
    /// `r` lends its bitmaps.
    pub(crate) fn run_symbols(
        &mut self,
        b: &crate::block::Symbols,
        r: &mut Runs,
        k: Kernels,
        dense_from: usize,
        admit: impl FnOnce(usize, usize) -> bool,
        out: &mut Vec<u8>,
    ) -> Option<usize> {
        let p = b.orig_ptr;
        let mut v = Fused {
            pass: self.begin(),
            pos: 0,
            last: 256,
            p,
            max_len: b.max_len.min(900_000) as u32,
            dense_from,
        };
        crate::runs::front_runs(b, r, k, &mut v)?;
        let (first, n) = (v.pass.f, v.pos);
        let records = first.runs + first.cut;
        if n == 0 || p >= n || !admit(records, n as usize) {
            return None;
        }
        let (_, mark, bits) = self.finish::<true, true>(&first, None, &[], n, p, LIMIT);
        self.walk(n as usize, mark, bits, out);
        Some(n as usize)
    }

    /// The words inlined and the walk from the marked row: the block's bytes into `out`
    fn walk(&mut self, n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        emit(
            &self.edges,
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

/// The open kernel's fused first pass: the front's runs (`runs::front_runs`), each taken as
/// the maximal run it is, in safe code
struct Fused<'a> {
    pass: Pass<'a, &'a mut [Vec<u32>; 256]>,
    /// the rows so far: the next run's first row
    pos: u32,
    /// the last run's byte (256 before the first)
    last: u16,
    p: u32,
    max_len: u32,
    /// the maximal runs at which the block is refused (past the density line)
    dense_from: usize,
}

impl crate::runs::RunVisitor for Fused<'_> {
    #[inline(always)]
    fn run(&mut self, c: u8, rows: u32) -> bool {
        // Every literal (an MTF index of 1 or more) names a byte other than the open run's
        // (index 0), so with distinct bytes in use each run the front gives is maximal (Carbon
        // Kitchen Sink cycle 1, Theorem 3). A repeat is refused, never merged: the records'
        // path then takes the block. So are rows past the bound, and records past the density
        // line. (One branch for the three: `|`, not `||`.)
        if (u16::from(c) == self.last)
            | (self.pos + rows > self.max_len)
            | (self.pass.f.runs >= self.dense_from)
        {
            return false;
        }
        self.last = c.into();
        self.pass.take::<true>(c, self.pos, rows, self.p, LIMIT);
        self.pos += rows;
        true
    }
}

/// Rows a thin run may have, as retained ranks (the arm `Open<16, true, true>` measured on the
/// i9: bench-open-return, the moonshot runs)
pub(crate) const LIMIT: u32 = 16;
// (`Fused` counts a thin run as one record: it is, below a record's 127 rows)
const _: () = assert!(LIMIT < 128);

/// The workers' kernel wherever production ran `LazyCells` (from 0.5.7): the open kernel on
/// every run-route block that `LazyCells`' record-count gates admit (its density line and the
/// sparse gate, `cells_lazy::lazy_counts_admit`), `LazyCells` as shipped on every other block and
/// on the column route. The output is the same either way.
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
    lazy: LazyCells,
    /// (test only) blocks the open kernel inverted
    #[cfg(test)]
    pub(crate) opened: usize,
    /// (test only) of those, the blocks it inverted from their symbols (`invert_symbols`)
    #[cfg(test)]
    pub(crate) fused: usize,
}

impl<const FORCED: bool> OpenCellsOf<FORCED> {
    /// Whether the open kernel takes a run-route block of `records` records in `n` rows with
    /// the walk's row `p`: `LazyCells`' record-count gates (not when `FORCED`), and the
    /// kernel's own entry condition
    pub(crate) fn admits(records: usize, n: usize, p: u32) -> bool {
        n > 0
            && n <= 900_000
            && (p as usize) < n
            && (FORCED || crate::cells::lazy_counts_admit(records, n))
    }
}

impl<const FORCED: bool> InverseBwt for OpenCellsOf<FORCED> {
    fn reads_runs(&self) -> bool {
        true
    }
    fn invert_runs(&mut self, r: &Runs, n: usize, p: u32, out: &mut Vec<u8>, k: Kernels) -> bool {
        if Self::admits(r.records().len(), n, p) {
            self.open
                .run::<true, true>(r.records(), Some(r.meta()), n, p, LIMIT, out);
            #[cfg(test)]
            {
                self.opened += 1;
            }
            return true;
        }
        self.lazy.invert_runs(r, n, p, out, k)
    }
    fn invert_symbols(
        &mut self,
        b: &crate::block::Symbols,
        r: &mut Runs,
        k: Kernels,
        out: &mut Vec<u8>,
    ) -> Option<usize> {
        // no block of `max_len` rows or fewer is admitted from this many records on (the
        // density line, `lazy_counts_admit`): stop there rather than at the block's end
        let dense_from = if FORCED {
            usize::MAX
        } else {
            (b.max_len * 3).div_ceil(20)
        };
        let n = self.open.run_symbols(
            b,
            r,
            k,
            dense_from,
            |records, n| Self::admits(records, n, b.orig_ptr),
            out,
        )?;
        #[cfg(test)]
        {
            self.opened += 1;
            self.fused += 1;
        }
        Some(n)
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

/// The bytes of a code's word kept inline, and the longest word kept whole there
const INLINE: usize = 16;
/// `lens` of a word longer than `INLINE`: its bytes are read from `spelling` (the cold path)
const LONG: u8 = u8::MAX;

/// Every code's word as 16 bytes, its first `INLINE` padded with zeros, and its length (`LONG`
/// past 16), indexed by code: the literals 0..256, then the block's words from 256. The
/// literals are written once, when the table is first made; the words, once a block.
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
fn emit(
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

/// A block's counts from its records, as `runs::front` gives them beside the records
#[cfg(test)]
fn meta_of(records: &[u16]) -> ([u32; 256], [u32; 256]) {
    let (mut counts, mut per) = ([0u32; 256], [0u32; 256]);
    for &r in records {
        counts[usize::from(r >> 8)] += u32::from(r & 255);
        per[usize::from(r >> 8)] += 1;
    }
    (counts, per)
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

#[cfg(test)]
mod checks;

#[cfg(test)]
mod b5;

#[cfg(test)]
mod fused;

#[cfg(test)]
mod slab;
