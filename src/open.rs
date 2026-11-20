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
    strips: Vec<Strip>,
    image_order: Vec<u32>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
}

// Production records cap a run at 127. Selection is on maximal runs.
// A primary inside a thick strip is a retained singleton. The two fragments
// keep the classification of their parent even if they are short.
fn each_run(records: &[u16], p: u32, limit: u32, mut visit: impl FnMut(u8, u32, u32, bool)) {
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
        pos = end;
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
            self.bands.push(Band {
                end: *k + width,
                delta: exit.wrapping_sub(*k),
                code,
            });
            *k += width;
            x0 += width;
        }
    }

    fn construct<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
    ) -> (Counts, u32, u32) {
        let mut stats = Counts {
            n: n as usize,
            ..Counts::default()
        };
        let mut all = [0; 256];
        let mut thin = [0; 256];
        let mut thick = [0; 256];
        let mut m = 0;
        let mut mark = 0;
        self.strips.clear();
        if QUEUE {
            self.queues.resize_with(256, Vec::new);
            for q in &mut self.queues {
                q.clear();
            }
        }
        let mut last = None;
        each_run(records, p, limit, |c, x, w, keep| {
            stats.runs += usize::from(last != Some(c));
            last = Some(c);
            let a = usize::from(c);
            if keep {
                if (x..x + w).contains(&p) {
                    mark = m + (p - x);
                }
                if QUEUE {
                    self.queues[a].extend(m..m + w);
                }
                thin[a] += w;
                m += w;
            } else {
                self.strips.push(Strip {
                    from: x,
                    to: all[a],
                    width: w,
                    byte: c,
                });
                thick[a] += 1;
            }
            all[a] += w;
        });
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
                let mut from = 0;
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    let src = &self.queues[a][from..from + span];
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

    pub fn run<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Counts {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records, n as u32, p, limit);
        emit(
            &self.edges,
            &self.extra,
            &self.spelling,
            &self.offsets,
            n,
            mark,
            bits,
            out,
        );
        stats
    }
}

/// Rows a thin run may have, as retained ranks (the arm `Open<16, true, true>` measured on the
/// i9: bench-open-return, the moonshot runs)
pub(crate) const LIMIT: u32 = 16;

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
            self.open.run::<true, true>(r.records(), n, p, LIMIT, out);
            #[cfg(test)]
            {
                self.opened += 1;
            }
            return true;
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

/// The walk over the retained rows from the marked one, each row's word appended; a cycle
/// shorter than the block is repeated to `n` bytes (the journal's code, unchanged)
#[allow(clippy::too_many_arguments)]
fn emit(
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

// Research only: the construction's stage split (`open-phases`, examples/open_phases.rs)
#[cfg(feature = "open-phases")]
pub use endpoint::phases::{PhasedOpen, Phases};

#[cfg(test)]
mod checks;

#[cfg(test)]
mod b5;
