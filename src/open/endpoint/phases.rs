//! Research only (feature `open-phases`): a timed clone of the open kernel's induced
//! construction (`Kernel::construct::<true, QUEUE>`) and `emit`, cut into stages, each stage an
//! `#[inline(never)]` function so a profiler (callgrind) attributes its cost to it. Never on a
//! production path; without the feature nothing here is compiled
//! (docs/journal/2026-11-20-open-phase-split.md, examples/open_phases.rs).
//!
//! The clone does what production does, in production's order, with one rearrangement: the
//! spelling of the words (`Endpoint::append` into `spelling`, and `offsets`), which production
//! interleaves with the bands' loop at the end of `induced`, is deferred to a loop of its own
//! over the same words in the same order, so that it can be timed apart. The appends see the
//! same `spelling` lengths in the same order (a rule's `first` is a position in it), the codes
//! are the same (`255 + words`), so the output is the same: the example checks it byte for byte
//! against production's `Kernel::run` every round it times.
use std::time::Instant;

use super::super::{each_run, emit, starts, Band, Counts, Kernel, Strip, LIMIT};
use super::Node;

/// A block's stage times (nanoseconds) and the sizes each stage leaves
#[derive(Default, Clone, Copy, Debug)]
pub struct Phases {
    /// (a) the run pass: `each_run`, the strips, the queues (QUEUE)
    pub runs_ns: u64,
    /// (b) the image ordering and setup (`starts`, `image_order`)
    pub image_ns: u64,
    /// (c1) the endpoint nodes, the two orders and their lists
    pub nodes_ns: u64,
    /// (c2) the elimination itself, rule creation included
    pub elim_ns: u64,
    /// (c3) the bottoms and the bands (production's band loop, spelling deferred)
    pub bands_ns: u64,
    /// (d) the words' spelling materialised (`spelling`, `offsets`)
    pub spell_ns: u64,
    /// (e) the wiring that fills `edges`
    pub wire_ns: u64,
    /// (f) `emit`
    pub emit_ns: u64,
    pub counts: Counts,
    pub strips: usize,
    pub queued: usize,
    pub nodes: usize,
    pub rules: usize,
    pub bands: usize,
    pub spelling: usize,
    pub offsets: usize,
    pub edges: usize,
    pub extra: usize,
    /// every buffer's capacity, in bytes, after the block (the kernel keeps them all live)
    pub live_bytes: usize,
}

/// The open kernel, run either as production runs it or as the timed clone
#[derive(Default)]
pub struct PhasedOpen {
    k: Kernel,
    pending: Vec<(u32, u8)>,
}

impl PhasedOpen {
    /// Production's `Kernel::run::<true, QUEUE>`, untouched, at the shipped limit
    pub fn run_production<const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) {
        self.k.run::<true, QUEUE>(records, n, p, LIMIT, out);
    }

    /// The timed clone: the same construction and walk, stage by stage
    pub fn run_phased<const QUEUE: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Phases {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let n32 = n as u32;
        let limit = LIMIT;
        let mut ph = Phases::default();
        let mut stats = Counts {
            n,
            ..Counts::default()
        };
        let t0 = Instant::now();
        let (thin, all, thick, m, mark) =
            stage_runs::<QUEUE>(&mut self.k, records, p, limit, &mut stats);
        let t1 = Instant::now();
        stage_image(&mut self.k, &all, &thick);
        let t2 = Instant::now();
        self.k.bands.clear();
        self.k.spelling.clear();
        self.k.offsets.clear();
        self.k.offsets.push(0);
        stage_nodes(&mut self.k, n32, &mut stats);
        let t3 = Instant::now();
        stage_elim(&mut self.k, n32, &mut stats);
        let t4 = Instant::now();
        let cursors = stage_bands(&mut self.k, &mut self.pending, &thin, &mut stats);
        let t5 = Instant::now();
        stage_spell(&mut self.k, &self.pending);
        let t6 = Instant::now();
        let bits = stage_wire::<QUEUE>(
            &mut self.k,
            records,
            p,
            limit,
            m,
            &thin,
            cursors,
            &mut stats,
        );
        let t7 = Instant::now();
        stage_emit(&self.k, n, mark, bits, out);
        let t8 = Instant::now();
        let ns = |a: Instant, b: Instant| (b - a).as_nanos() as u64;
        ph.runs_ns = ns(t0, t1);
        ph.image_ns = ns(t1, t2);
        ph.nodes_ns = ns(t2, t3);
        ph.elim_ns = ns(t3, t4);
        ph.bands_ns = ns(t4, t5);
        ph.spell_ns = ns(t5, t6);
        ph.wire_ns = ns(t6, t7);
        ph.emit_ns = ns(t7, t8);
        let k = &self.k;
        ph.counts = stats;
        ph.strips = k.strips.len();
        ph.queued = if QUEUE {
            k.queues.iter().map(Vec::len).sum()
        } else {
            0
        };
        ph.nodes = k.endpoint.nodes.len();
        ph.rules = k.endpoint.rules.len();
        ph.bands = k.bands.len();
        ph.spelling = k.spelling.len();
        ph.offsets = k.offsets.len();
        ph.edges = k.edges.len();
        ph.extra = k.extra.len();
        ph.live_bytes = self.live_bytes();
        ph
    }

    /// Research only: Queue-Free Backward Open (Neon cycle 2,
    /// docs/journal/2026-11-20-open-backward-native.md) at the shipped limit. The run pass is
    /// the unqueued one (`stage_runs::<false>`); the image ordering, the elimination, the bands
    /// and the spelling are shipped Open's, unchanged; the wiring stores g = P ∘ LF_v directly,
    /// `edges[j] = code | g(j)`, written sequentially in j; the walk follows g from the marked
    /// row and writes the words right to left from `n`. `ZERO` zero-fills the whole output every
    /// block (`clear` then `resize`, the cost of a fresh buffer's initialisation); otherwise the
    /// worker's buffer is reused and only resized to `n`, a no-op from block to block at equal n.
    pub fn run_backward<const ZERO: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Phases {
        self.run_backward_at::<ZERO>(records, n, p, LIMIT, out)
    }

    fn run_backward_at<const ZERO: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> Phases {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let n32 = n as u32;
        let mut ph = Phases::default();
        let mut stats = Counts {
            n,
            ..Counts::default()
        };
        let t0 = Instant::now();
        let (thin, all, thick, m, mark) =
            stage_runs::<false>(&mut self.k, records, p, limit, &mut stats);
        let t1 = Instant::now();
        stage_image(&mut self.k, &all, &thick);
        let t2 = Instant::now();
        self.k.bands.clear();
        self.k.spelling.clear();
        self.k.offsets.clear();
        self.k.offsets.push(0);
        stage_nodes(&mut self.k, n32, &mut stats);
        let t3 = Instant::now();
        stage_elim(&mut self.k, n32, &mut stats);
        let t4 = Instant::now();
        let cursors = stage_bands(&mut self.k, &mut self.pending, &thin, &mut stats);
        let t5 = Instant::now();
        stage_spell(&mut self.k, &self.pending);
        let t6 = Instant::now();
        let bits = stage_wire_backward(
            &mut self.k,
            records,
            p,
            limit,
            m,
            &thin,
            cursors,
            &mut stats,
        );
        let t7 = Instant::now();
        stage_emit_backward::<ZERO>(&self.k, n, mark, bits, out);
        let t8 = Instant::now();
        let ns = |a: Instant, b: Instant| (b - a).as_nanos() as u64;
        ph.runs_ns = ns(t0, t1);
        ph.image_ns = ns(t1, t2);
        ph.nodes_ns = ns(t2, t3);
        ph.elim_ns = ns(t3, t4);
        ph.bands_ns = ns(t4, t5);
        ph.spell_ns = ns(t5, t6);
        ph.wire_ns = ns(t6, t7);
        ph.emit_ns = ns(t7, t8);
        let k = &self.k;
        ph.counts = stats;
        ph.strips = k.strips.len();
        ph.nodes = k.endpoint.nodes.len();
        ph.rules = k.endpoint.rules.len();
        ph.bands = k.bands.len();
        ph.spelling = k.spelling.len();
        ph.offsets = k.offsets.len();
        ph.edges = k.edges.len();
        ph.extra = k.extra.len();
        ph.live_bytes = self.live_bytes();
        ph
    }

    fn live_bytes(&self) -> usize {
        use std::mem::size_of;
        let k = &self.k;
        let e = &k.endpoint;
        k.queues.iter().map(|q| 4 * q.capacity()).sum::<usize>()
            + size_of::<Strip>() * k.strips.capacity()
            + 4 * k.image_order.capacity()
            + size_of::<Band>() * k.bands.capacity()
            + k.spelling.capacity()
            + 4 * (k.offsets.capacity() + k.edges.capacity() + k.extra.capacity())
            + size_of::<Node>() * e.nodes.capacity()
            + 16 * e.rules.capacity()
            + 4 * (e
                .order
                .iter()
                .chain(&e.terminals)
                .map(Vec::capacity)
                .sum::<usize>()
                + e.bottom.capacity()
                + e.stack.capacity())
    }
}

type RunsOut = ([u32; 256], [u32; 256], [u32; 256], u32, u32);

#[inline(never)]
fn stage_runs<const QUEUE: bool>(
    k: &mut Kernel,
    records: &[u16],
    p: u32,
    limit: u32,
    stats: &mut Counts,
) -> RunsOut {
    let mut all = [0; 256];
    let mut thin = [0; 256];
    let mut thick = [0; 256];
    let mut m = 0;
    let mut mark = 0;
    k.strips.clear();
    if QUEUE {
        k.queues.resize_with(256, Vec::new);
        for q in &mut k.queues {
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
                k.queues[a].extend(m..m + w);
            }
            thin[a] += w;
            m += w;
        } else {
            k.strips.push(Strip {
                from: x,
                to: all[a],
                width: w,
                byte: c,
            });
            thick[a] += 1;
        }
        all[a] += w;
    });
    assert!(m > 0 && all.iter().sum::<u32>() == stats.n as u32);
    stats.retained = m as usize;
    stats.thick = k.strips.len();
    (thin, all, thick, m, mark)
}

#[inline(never)]
fn stage_image(k: &mut Kernel, all: &[u32; 256], thick: &[u32; 256]) {
    let first = starts(*all);
    let mut write = starts(*thick);
    k.image_order.resize(k.strips.len(), 0);
    k.tiles.clear();
    for (i, s) in k.strips.iter_mut().enumerate() {
        let a = usize::from(s.byte);
        s.to += first[a];
        k.image_order[write[a] as usize] = i as u32;
        write[a] += 1;
    }
}

#[inline(never)]
fn stage_nodes(k: &mut Kernel, n: u32, stats: &mut Counts) {
    let e = &mut k.endpoint;
    e.nodes.clear();
    e.nodes.push(Node::default());
    e.rules.clear();
    for order in e.order.iter_mut().chain(&mut e.terminals) {
        order.clear();
    }
    for s in &k.strips {
        e.node(s.width, u32::from(s.byte) + 1);
    }
    let mut x = 0;
    for (i, s) in k.strips.iter().enumerate() {
        e.gaps(0, x, s.from);
        e.order[0].push(i as u32 + 1);
        x = s.from + s.width;
    }
    e.gaps(0, x, n);
    x = 0;
    for &i in &k.image_order {
        let s = k.strips[i as usize];
        e.gaps(1, x, s.to);
        e.order[1].push(i + 1);
        x = s.to + s.width;
    }
    e.gaps(1, x, n);
    stats.open_nodes = e.nodes.len() - 1;
    e.lists();
}

#[inline(never)]
fn stage_elim(k: &mut Kernel, n: u32, stats: &mut Counts) {
    let e = &mut k.endpoint;
    stats.steps = e.eliminate(n);
    stats.rules = e.rules.len();
}

#[inline(never)]
fn stage_bands(
    k: &mut Kernel,
    pending: &mut Vec<(u32, u8)>,
    thin: &[u32; 256],
    stats: &mut Counts,
) -> [usize; 256] {
    let e = &mut k.endpoint;
    pending.clear();
    e.bottom.resize(e.nodes.len(), 0);
    let mut id = e.nodes[0].next[1];
    let mut x = 0;
    while id != 0 {
        e.bottom[id as usize] = x;
        let node = e.nodes[id as usize];
        x += node.width;
        id = node.next[1];
    }
    assert_eq!(x as usize, stats.retained);
    let mut cursors = [0; 256];
    id = e.nodes[0].next[0];
    let mut in_node = 0;
    let mut kk = 0;
    for a in 0..256 {
        cursors[a] = k.bands.len();
        let end = kk + thin[a];
        while kk < end {
            let node = e.nodes[id as usize];
            let width = (end - kk).min(node.width - in_node);
            assert!(width > 0);
            let dst = e.bottom[id as usize] + in_node;
            let code = if node.word == 0 {
                a as u32
            } else {
                let height = e.len(node.word) as usize;
                stats.words += 1;
                stats.copies += width as usize;
                stats.traced += height;
                stats.thick_area += width as usize * height;
                stats.unit_work += usize::from(width == 1) * height;
                stats.longest = stats.longest.max(height);
                // (the clone's one change) the spelling is deferred to `stage_spell`, in the
                // same order; production's code is `254 + offsets.len()` after its push
                pending.push((node.word, a as u8));
                255 + pending.len() as u32
            };
            k.bands.push(Band {
                end: kk + width,
                delta: dst.wrapping_sub(kk),
                code,
            });
            in_node += width;
            kk += width;
            if in_node == node.width {
                id = node.next[0];
                in_node = 0;
            }
        }
    }
    assert_eq!(id, 0);
    cursors
}

#[inline(never)]
fn stage_spell(k: &mut Kernel, pending: &[(u32, u8)]) {
    for &(word, a) in pending {
        k.endpoint.append(word, &mut k.spelling);
        k.spelling.push(a);
        k.offsets.push(k.spelling.len() as u32);
    }
}

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn stage_wire<const QUEUE: bool>(
    k: &mut Kernel,
    records: &[u16],
    p: u32,
    limit: u32,
    m: u32,
    thin: &[u32; 256],
    mut cursors: [usize; 256],
    stats: &mut Counts,
) -> u32 {
    stats.bands = k.bands.len();
    let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
    let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
    k.edges.resize(m as usize, 0);
    k.extra.clear();
    if wide {
        k.extra.resize(m as usize, 0);
    }
    stats.table_bytes = 4 * (k.edges.len() + k.extra.len());
    if QUEUE {
        let mut kk = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(k.bands.len());
            let mut from = 0;
            for b in &k.bands[cursors[a]..end] {
                let span = (b.end - kk) as usize;
                let dst = kk.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    k.extra[dst..dst + span].fill(b.code);
                }
                let src = &k.queues[a][from..from + span];
                for (target, &j) in k.edges[dst..dst + span].iter_mut().zip(src) {
                    *target = pack | j;
                }
                from += span;
                kk = b.end;
            }
        }
    } else {
        let mut stream = starts(*thin);
        let mut j = 0;
        each_run(records, p, limit, |c, _, w, keep| {
            if !keep {
                return;
            }
            let a = usize::from(c);
            let mut z = stream[a];
            let end = z + w;
            while z < end {
                let b = k.bands[cursors[a]];
                let span = (end.min(b.end) - z) as usize;
                let dest = z.wrapping_add(b.delta) as usize;
                let pack = if wide { 0 } else { b.code << bits };
                if wide {
                    k.extra[dest..dest + span].fill(b.code);
                }
                let mut value = pack | j;
                for e in &mut k.edges[dest..dest + span] {
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
    bits
}

#[inline(never)]
fn stage_emit(k: &Kernel, n: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
    emit(
        &k.edges,
        &k.extra,
        &k.spelling,
        &k.offsets,
        n,
        mark,
        bits,
        out,
    );
}

/// Backward Open's wiring: a second scan of the records, as `stage_wire::<false>`, but storing
/// each arrow at its source, `edges[j] = code | g(j)`, so `edges` is written in increasing j
/// (shipped Open stores it at its target, `edges[g(j)] = code | j`). Each byte's current band is
/// held in a small table (`Active`, the reference's `ActiveBand[256]`, g5_backward_open.c): a
/// thin run inside it (the reference's fast path) costs one table read and write
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn stage_wire_backward(
    k: &mut Kernel,
    records: &[u16],
    p: u32,
    limit: u32,
    m: u32,
    thin: &[u32; 256],
    cursors: [usize; 256],
    stats: &mut Counts,
) -> u32 {
    #[derive(Clone, Copy, Default)]
    struct Active {
        rank: u32,
        end: u32,
        delta: u32,
        code: u32,
        pack: u32,
        band: u32,
    }
    stats.bands = k.bands.len();
    let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
    let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
    k.edges.resize(m as usize, 0);
    k.extra.clear();
    if wide {
        k.extra.resize(m as usize, 0);
    }
    stats.table_bytes = 4 * (k.edges.len() + k.extra.len());
    let shift = if wide { 32 } else { bits };
    let pack = |code: u32| code.checked_shl(shift).unwrap_or(0);
    let bands = &k.bands;
    let mut active = [Active::default(); 256];
    let mut rank = 0;
    for a in 0..256 {
        active[a].rank = rank;
        active[a].band = cursors[a] as u32;
        if thin[a] > 0 {
            let b = bands[cursors[a]];
            active[a].end = b.end;
            active[a].delta = b.delta;
            active[a].code = b.code;
            active[a].pack = pack(b.code);
        } else {
            active[a].end = rank;
        }
        rank += thin[a];
    }
    let (edges, extra) = (&mut k.edges, &mut k.extra);
    let mut j = 0usize;
    each_run(records, p, limit, |c, _, w, keep| {
        if !keep {
            return;
        }
        let cb = &mut active[usize::from(c)];
        let mut left = w;
        while left > 0 {
            let span = left.min(cb.end - cb.rank);
            assert!(span > 0);
            let mut value = cb.pack | cb.rank.wrapping_add(cb.delta);
            let to = j + span as usize;
            for e in &mut edges[j..to] {
                *e = value;
                value = value.wrapping_add(1);
            }
            if wide {
                extra[j..to].fill(cb.code);
            }
            j = to;
            cb.rank += span;
            left -= span;
            if cb.rank == cb.end {
                let next = cb.band as usize + 1;
                let a = usize::from(c);
                let last = cursors.get(a + 1).copied().unwrap_or(bands.len());
                if next < last {
                    let b = bands[next];
                    cb.band = next as u32;
                    cb.end = b.end;
                    cb.delta = b.delta;
                    cb.code = b.code;
                    cb.pack = pack(b.code);
                }
            }
        }
    });
    assert_eq!(j, m as usize);
    bits
}

/// Backward Open's walk: from the marked row along g, each arrow's word written right to left
/// from `n`. On a cycle shorter than the block the walk returns to the mark with `w > 0`, the
/// cycle's text in `out[w..n]`, in forward order from the mark's successor; it is moved to the
/// front and repeated to `n` bytes, which is what shipped Open's `emit` and `repeat` give. The
/// walk is bounded by `edges.len()` steps and clamped at the buffer's start, as `emit` is
#[inline(never)]
fn stage_emit_backward<const ZERO: bool>(
    k: &Kernel,
    n: usize,
    mark: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    if ZERO {
        out.clear();
    }
    out.resize(n, 0);
    let (edges, extra, spelling, offsets) = (&k.edges, &k.extra, &k.spelling, &k.offsets);
    let mask = (1 << bits) - 1;
    let mut j = mark;
    let mut w = n;
    for _ in 0..edges.len() {
        let edge = edges[j as usize];
        let code = if extra.is_empty() {
            edge >> bits
        } else {
            extra[j as usize]
        };
        j = edge & mask;
        if code < 256 {
            w -= 1;
            out[w] = code as u8;
        } else {
            let id = (code - 256) as usize;
            let word = &spelling[offsets[id] as usize..offsets[id + 1] as usize];
            let take = word.len().min(w);
            out[w - take..w].copy_from_slice(&word[word.len() - take..]);
            w -= take;
        }
        if j == mark || w == 0 {
            break;
        }
    }
    if w > 0 {
        let cycle = n - w;
        out.copy_within(w..n, 0);
        let mut have = cycle;
        while have < n {
            let take = have.min(n - have);
            out.copy_within(..take, have);
            have += take;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::records_of;
    use super::*;

    // Every binary column to length 8 and ternary to 5, at every mark and five limits (the
    // arrow audit's enumeration, checks.rs): Backward Open's arrows are shipped Open's arrows
    // reversed (edges[j] = code | g(j) wherever the shipped table has edges[g(j)] = code | j,
    // the shipped table being the one checks.rs audits arrow by arrow), and its output, both
    // buffer variants, equals shipped Open's, short cycles included
    #[test]
    fn backward_arrows_and_output_match_shipped() {
        let mut shipped = Kernel::default();
        let mut bwd = PhasedOpen::default();
        let (mut want, mut got, mut got_z) = (Vec::new(), Vec::new(), Vec::new());
        let mut cases = 0;
        for (base, len) in [(2usize, 8usize), (3, 5)] {
            for n in 1..=len {
                for mut z in 0..base.pow(n as u32) {
                    let l: Vec<_> = (0..n)
                        .map(|_| {
                            let c = z % base;
                            z /= base;
                            c as u8
                        })
                        .collect();
                    let recs = records_of(&l);
                    for p in 0..n as u32 {
                        for t in [0, 1, 2, 4, 16] {
                            let (_, _, bits) =
                                shipped.construct::<true, true>(&recs, n as u32, p, t);
                            let mask = (1u32 << bits) - 1;
                            let fwd = shipped.edges.clone();
                            let fwd_extra = shipped.extra.clone();
                            shipped.run::<true, true>(&recs, n, p, t, &mut want);
                            bwd.run_backward_at::<false>(&recs, n, p, t, &mut got);
                            bwd.run_backward_at::<true>(&recs, n, p, t, &mut got_z);
                            assert_eq!(got, want, "{l:?} p={p} t={t}");
                            assert_eq!(got_z, want, "{l:?} p={p} t={t}");
                            let b = &bwd.k;
                            assert_eq!(b.edges.len(), fwd.len());
                            assert_eq!(b.extra.len(), fwd_extra.len());
                            for (i, &e) in fwd.iter().enumerate() {
                                let j = (e & mask) as usize;
                                let code_f = if fwd_extra.is_empty() {
                                    e >> bits
                                } else {
                                    fwd_extra[i]
                                };
                                let be = b.edges[j];
                                assert_eq!((be & mask) as usize, i, "{l:?} p={p} t={t}");
                                let code_b = if b.extra.is_empty() {
                                    be >> bits
                                } else {
                                    b.extra[j]
                                };
                                assert_eq!(code_b, code_f, "{l:?} p={p} t={t}");
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
        println!("{cases} column/mark/limit triples: arrows reversed and output equal");
    }

    // Random columns with short and multiple cycles, every limit up to above n, against the
    // plain inverse (as checks.rs's residual-segment test does for shipped Open)
    #[test]
    fn backward_short_cycles_match_plain() {
        use crate::bwt::{InverseBwt, Plain};
        let mut bwd = PhasedOpen::default();
        let mut plain = Plain::default();
        let (mut actual, mut reference) = (Vec::new(), Vec::new());
        let mut seed = 12383217u32;
        for n in [1, 2, 3, 31, 127, 128, 129, 385, 1025, 4099] {
            for alphabet in [1, 2, 11, 256] {
                let l: Vec<_> = (0..n)
                    .map(|_| {
                        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                        ((seed >> 8) % alphabet) as u8
                    })
                    .collect();
                let mut counts = [0; 256];
                for &c in &l {
                    counts[c as usize] += 1;
                }
                let recs = records_of(&l);
                for p in [0, (n / 2) as u32, (n - 1) as u32] {
                    plain.invert(&l, &counts, p, &mut reference);
                    for limit in [0, 1, 4, 16, 1025] {
                        bwd.run_backward_at::<false>(&recs, n, p, limit, &mut actual);
                        assert_eq!(actual, reference);
                        bwd.run_backward_at::<true>(&recs, n, p, limit, &mut actual);
                        assert_eq!(actual, reference);
                    }
                }
            }
        }
    }
}
