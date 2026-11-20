//! Research only (feature `open-phases`): two rewirings of the shipped open kernel's induced
//! construction (`Kernel::construct::<true, true>`), each a research copy; production's kernel
//! is untouched (docs/journal/2026-11-20-open-fused-wiring.md, examples/open_fused.rs).
//!
//! - **F (`FUSE`)**: the induced band loop writes `edges` (and `extra`) directly from
//!   `(a, width, dst, code)`. There is no `bands` buffer, no `cursors` and no second wiring loop.
//!   The codomain prefix sum (production's `Endpoint::bottom`) is kept in `Node::prev[1]`, dead
//!   once `eliminate` has returned. `wide` is decided lazily: the loop starts narrow and, when a
//!   code first exceeds `u32::MAX >> bits`, spills every edge written so far into `extra`, then
//!   continues wide. Shipped Open decides `wide` from the final word count (`255 + words`, the
//!   largest code), so the spill happens exactly when shipped Open is wide.
//! - **Q (`COMPACT`)**: the per-byte rank queues hold one `u32` per thin run instead of one per
//!   row, Neon's encoding `(m << 8) | (w - 1)` (n < 2^24, and `w <= limit <= 256`, asserted in
//!   release builds), and the scatter expands each run's ranks with Neon's cursor (entry, next
//!   rank, rows left). Same information, coarser grain, no new pass.
//! - **Qi (`INDEX`, without `COMPACT`)**: Q's other form, per-row queues pre-sized (they keep
//!   their length from block to block) and written by index rather than by `Vec::extend`.
//!
//! Both compose (`FUSE` and `COMPACT`). With neither, this is shipped Open's construction.
//! `PHASED` defers the spelling to a loop of its own and takes a timestamp between stages, as
//! the phase split's clone does (`phases.rs`), for the stage columns; without it the spelling is
//! interleaved as in production and nothing is timed.
use std::time::Instant;

use super::super::{each_run, emit, starts, Band, Counts, Kernel, Strip, LIMIT};
use super::Node;
use crate::open::Phases;

/// Test-only knobs for the wide spill: `cap` replaces `u32::MAX >> bits` as the largest code a
/// narrow edge may pack; `wide` starts wide whatever the codes
#[derive(Default, Clone, Copy)]
pub struct Spill {
    pub cap: Option<u32>,
    pub wide: bool,
}

/// The open kernel with one of the research wirings
#[derive(Default)]
pub struct ResearchOpen {
    k: Kernel,
    /// (`COMPACT`) per byte, its thin runs as `(first rank << 8) | (width - 1)`
    spans: Vec<Vec<u32>>,
    pending: Vec<(u32, u8)>,
}

/// The construction's result as a check compares it
pub struct State<'a> {
    pub edges: &'a [u32],
    pub extra: &'a [u32],
    pub spelling: &'a [u8],
    pub offsets: &'a [u32],
}

impl ResearchOpen {
    pub fn state(&self) -> State<'_> {
        State {
            edges: &self.k.edges,
            extra: &self.k.extra,
            spelling: &self.k.spelling,
            offsets: &self.k.offsets,
        }
    }

    /// Shipped Open's construction (`Kernel::construct::<true, true>`), untouched, for the
    /// internal-state comparison: `(mark, bits)`
    pub fn construct_shipped(&mut self, records: &[u16], n: usize, p: u32) -> (u32, u32) {
        let (_, mark, bits) = self.k.construct::<true, true>(records, n as u32, p, LIMIT);
        (mark, bits)
    }

    /// Production's `Kernel::run::<true, true>`, untouched
    pub fn run_shipped(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        self.k.run::<true, true>(records, n, p, LIMIT, out);
    }

    /// One research arm, construction then `emit`; the stage times when `PHASED`
    pub fn run<const FUSE: bool, const COMPACT: bool, const PHASED: bool, const INDEX: bool>(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        out: &mut Vec<u8>,
    ) -> Phases {
        assert!(n > 0 && n <= 900_000 && (p as usize) < n);
        let mut ph = Phases::default();
        let (stats, mark, bits) = self.construct::<FUSE, COMPACT, PHASED, INDEX>(
            records,
            n as u32,
            p,
            LIMIT,
            Spill::default(),
            &mut ph,
        );
        let t = Instant::now();
        emit(
            &self.k.edges,
            &self.k.extra,
            &self.k.spelling,
            &self.k.offsets,
            n,
            mark,
            bits,
            out,
        );
        if PHASED {
            ph.emit_ns = t.elapsed().as_nanos() as u64;
            self.sizes(&mut ph, stats);
        }
        ph
    }

    fn sizes(&self, ph: &mut Phases, stats: Counts) {
        use std::mem::size_of;
        let k = &self.k;
        let e = &k.endpoint;
        ph.counts = stats;
        ph.strips = k.strips.len();
        ph.queued = k.queues.iter().map(Vec::len).sum::<usize>()
            + self.spans.iter().map(Vec::len).sum::<usize>();
        ph.nodes = e.nodes.len();
        ph.rules = e.rules.len();
        ph.bands = k.bands.len();
        ph.spelling = k.spelling.len();
        ph.offsets = k.offsets.len();
        ph.edges = k.edges.len();
        ph.extra = k.extra.len();
        ph.queue_bytes = k.queues.iter().map(|q| 4 * q.capacity()).sum::<usize>()
            + self.spans.iter().map(|q| 4 * q.capacity()).sum::<usize>();
        ph.live_bytes = k.queues.iter().map(|q| 4 * q.capacity()).sum::<usize>()
            + self.spans.iter().map(|q| 4 * q.capacity()).sum::<usize>()
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
            + 5 * self.pending.capacity();
    }

    /// The research construction: `(stats, mark, bits)`, as `Kernel::construct` returns
    pub fn construct<
        const FUSE: bool,
        const COMPACT: bool,
        const PHASED: bool,
        const INDEX: bool,
    >(
        &mut self,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        spill: Spill,
        ph: &mut Phases,
    ) -> (Counts, u32, u32) {
        let now = || PHASED.then(Instant::now);
        let ns = |a: Option<Instant>, b: Option<Instant>| match (a, b) {
            (Some(a), Some(b)) => (b - a).as_nanos() as u64,
            _ => 0,
        };
        let t0 = now();
        let k = &mut self.k;
        let mut stats = Counts {
            n: n as usize,
            ..Counts::default()
        };
        // (a) the run pass
        let mut all = [0; 256];
        let mut thin = [0; 256];
        let mut thick = [0; 256];
        let mut m = 0;
        let mut mark = 0;
        k.strips.clear();
        if COMPACT {
            assert!(limit <= 256, "Q packs a thin run's width in 8 bits");
            self.spans.resize_with(256, Vec::new);
            for q in &mut self.spans {
                q.clear();
            }
        } else {
            k.queues.resize_with(256, Vec::new);
            if !INDEX {
                for q in &mut k.queues {
                    q.clear();
                }
            }
        }
        // (Q, indexed) each queue's write cursor; the queues keep their length from block to
        // block and grow only when a block needs more
        let mut written = [0u32; 256];
        let mut last = None;
        {
            let (queues, spans, strips) = (&mut k.queues, &mut self.spans, &mut k.strips);
            each_run(records, p, limit, |c, x, w, keep| {
                stats.runs += usize::from(last != Some(c));
                last = Some(c);
                let a = usize::from(c);
                if keep {
                    if (x..x + w).contains(&p) {
                        mark = m + (p - x);
                    }
                    if COMPACT {
                        // Neon's encoding: n < 2^24 and w <= limit <= 256
                        spans[a].push((m << 8) | (w - 1));
                    } else if INDEX {
                        let q = &mut queues[a];
                        let at = written[a] as usize;
                        if q.len() < at + w as usize {
                            q.resize((at + w as usize).max(2 * q.len()), 0);
                        }
                        for (slot, j) in q[at..at + w as usize].iter_mut().zip(m..) {
                            *slot = j;
                        }
                        written[a] += w;
                    } else {
                        queues[a].extend(m..m + w);
                    }
                    thin[a] += w;
                    m += w;
                } else {
                    strips.push(Strip {
                        from: x,
                        to: all[a],
                        width: w,
                        byte: c,
                    });
                    thick[a] += 1;
                }
                all[a] += w;
            });
        }
        assert!(m > 0 && all.iter().sum::<u32>() == n);
        stats.retained = m as usize;
        stats.thick = k.strips.len();
        let t1 = now();
        // (b) the image order
        let first = starts(all);
        let mut write = starts(thick);
        k.image_order.resize(k.strips.len(), 0);
        k.tiles.clear();
        for (i, s) in k.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            k.image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        k.bands.clear();
        k.spelling.clear();
        k.offsets.clear();
        k.offsets.push(0);
        self.pending.clear();
        let t2 = now();
        // (c1) the endpoint's nodes and lists, (c2) the elimination: production's `induced`
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
        let t3 = now();
        stats.steps = e.eliminate(n);
        stats.rules = e.rules.len();
        let t4 = now();

        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let mask = (1u32 << bits) - 1;
        let cap = spill.cap.unwrap_or(u32::MAX >> bits);
        // (c3) the bottoms and the bands (F: and the wiring)
        let mut cursors = [0usize; 256];
        let mut wide = spill.wide;
        if FUSE {
            // the codomain prefix sum, in `prev[1]`: `eliminate` was its last reader
            let mut id = e.nodes[0].next[1];
            x = 0;
            while id != 0 {
                let node = &mut e.nodes[id as usize];
                node.prev[1] = x;
                x += node.width;
                id = node.next[1];
            }
            assert_eq!(x, m);
            k.edges.resize(m as usize, 0);
            k.extra.clear();
            if wide {
                k.extra.resize(m as usize, 0);
            }
        } else {
            e.bottom.resize(e.nodes.len(), 0);
            let mut id = e.nodes[0].next[1];
            x = 0;
            while id != 0 {
                e.bottom[id as usize] = x;
                let node = e.nodes[id as usize];
                x += node.width;
                id = node.next[1];
            }
            assert_eq!(x, m);
        }
        let mut id = e.nodes[0].next[0];
        let mut in_node = 0;
        let mut kk = 0;
        for a in 0..256 {
            cursors[a] = k.bands.len();
            let end = kk + thin[a];
            // F: where byte `a`'s queue is read from (Q: the run and the row in it)
            let (mut from, mut run, mut off, mut rem) = (0usize, 0usize, 0u32, 0u32);
            while kk < end {
                let node = e.nodes[id as usize];
                let width = (end - kk).min(node.width - in_node);
                assert!(width > 0);
                let dst = if FUSE {
                    node.prev[1]
                } else {
                    e.bottom[id as usize]
                } + in_node;
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
                    if PHASED {
                        self.pending.push((node.word, a as u8));
                        255 + self.pending.len() as u32
                    } else {
                        e.append(node.word, &mut k.spelling);
                        k.spelling.push(a as u8);
                        k.offsets.push(k.spelling.len() as u32);
                        254 + k.offsets.len() as u32
                    }
                };
                if FUSE {
                    if !wide && code > cap {
                        // the lazy spill: every edge so far into its code and its rank (the
                        // rows not yet written are overwritten later, as wide edges)
                        k.extra.resize(m as usize, 0);
                        for (c, edge) in k.extra.iter_mut().zip(&mut k.edges) {
                            *c = *edge >> bits;
                            *edge &= mask;
                        }
                        wide = true;
                    }
                    let (dst, span) = (dst as usize, width as usize);
                    let pack = if wide { 0 } else { code << bits };
                    if wide {
                        k.extra[dst..dst + span].fill(code);
                    }
                    let target = &mut k.edges[dst..dst + span];
                    if COMPACT {
                        fill(target, pack, &self.spans[a], &mut run, &mut off, &mut rem);
                    } else {
                        let src = &k.queues[a][from..from + span];
                        for (t, &j) in target.iter_mut().zip(src) {
                            *t = pack | j;
                        }
                        from += span;
                    }
                } else {
                    k.bands.push(Band {
                        end: kk + width,
                        delta: dst.wrapping_sub(kk),
                        code,
                    });
                }
                in_node += width;
                kk += width;
                if in_node == node.width {
                    id = node.next[0];
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        let t5 = now();
        // (d) the spelling, when deferred
        if PHASED {
            for &(word, a) in &self.pending {
                e.append(word, &mut k.spelling);
                k.spelling.push(a);
                k.offsets.push(k.spelling.len() as u32);
            }
        }
        let t6 = now();
        // (e) the wiring (Q alone; F has done it)
        if FUSE {
            stats.bands = 0;
        } else {
            stats.bands = k.bands.len();
            wide = spill.wide || (255 + stats.words as u32) > cap;
            k.edges.resize(m as usize, 0);
            k.extra.clear();
            if wide {
                k.extra.resize(m as usize, 0);
            }
            let mut kk = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(k.bands.len());
                let (mut from, mut run, mut off, mut rem) = (0usize, 0usize, 0u32, 0u32);
                for b in &k.bands[cursors[a]..end] {
                    let span = (b.end - kk) as usize;
                    let dst = kk.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        k.extra[dst..dst + span].fill(b.code);
                    }
                    let target = &mut k.edges[dst..dst + span];
                    if COMPACT {
                        fill(target, pack, &self.spans[a], &mut run, &mut off, &mut rem);
                    } else {
                        let src = &k.queues[a][from..from + span];
                        for (t, &j) in target.iter_mut().zip(src) {
                            *t = pack | j;
                        }
                        from += span;
                    }
                    kk = b.end;
                }
            }
        }
        stats.table_bytes = 4 * (k.edges.len() + k.extra.len());
        let t7 = now();
        if PHASED {
            ph.runs_ns = ns(t0, t1);
            ph.image_ns = ns(t1, t2);
            ph.nodes_ns = ns(t2, t3);
            ph.elim_ns = ns(t3, t4);
            ph.bands_ns = ns(t4, t5);
            ph.spell_ns = ns(t5, t6);
            ph.wire_ns = ns(t6, t7);
        }
        (stats, mark, bits)
    }
}

/// (Q) `target`'s ranks from byte `a`'s thin runs, Neon's cursor: `run` the next entry, `j`
/// the next rank, `rem` the rows left in the current run
#[inline(always)]
#[allow(clippy::explicit_counter_loop)]
fn fill(target: &mut [u32], pack: u32, spans: &[u32], run: &mut usize, j: &mut u32, rem: &mut u32) {
    let mut d = 0;
    while d < target.len() {
        if *rem == 0 {
            let entry = spans[*run];
            *run += 1;
            *j = entry >> 8;
            *rem = (entry & 255) + 1;
        }
        let t = (*rem as usize).min(target.len() - d);
        let mut value = pack | *j;
        for edge in &mut target[d..d + t] {
            *edge = value;
            value += 1;
        }
        d += t;
        *j += t as u32;
        *rem -= t as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open::records_of;

    fn columns() -> impl Iterator<Item = Vec<u8>> {
        [(2usize, 8usize), (3, 5)]
            .into_iter()
            .flat_map(|(base, len)| {
                (1..=len).flat_map(move |n| {
                    (0..base.pow(n as u32)).map(move |mut z| {
                        (0..n)
                            .map(|_| {
                                let c = z % base;
                                z /= base;
                                c as u8
                            })
                            .collect::<Vec<u8>>()
                    })
                })
            })
    }

    type ArmFn = fn(&mut ResearchOpen, &[u16], usize, usize, usize) -> (Counts, u32, u32);

    fn same(r: &ResearchOpen, s: &ResearchOpen) {
        let (a, b) = (r.state(), s.state());
        assert_eq!(a.edges, b.edges);
        assert_eq!(a.extra, b.extra);
        assert_eq!(a.spelling, b.spelling);
        assert_eq!(a.offsets, b.offsets);
    }

    fn arm<const F: bool, const Q: bool, const P: bool, const I: bool>(
        r: &mut ResearchOpen,
        recs: &[u16],
        n: usize,
        p: usize,
        t: usize,
        spill: Spill,
    ) -> (Counts, u32, u32) {
        r.construct::<F, Q, P, I>(
            recs,
            n as u32,
            p as u32,
            t as u32,
            spill,
            &mut Phases::default(),
        )
    }

    /// Every arm's internal state equals shipped Open's, and every arm passes the exhaustive
    /// arrow audit (`checks.rs`), on every binary column to 8 and ternary to 5, every mark,
    /// five limits
    #[test]
    fn arms_equal_shipped_and_audit() {
        let mut shipped = ResearchOpen::default();
        let mut r = ResearchOpen::default();
        let mut cases = 0;
        for l in columns() {
            let n = l.len();
            let recs = records_of(&l);
            for p in 0..n {
                for t in [0usize, 1, 2, 4, 16] {
                    let (_, mark, bits) = shipped
                        .k
                        .construct::<true, true>(&recs, n as u32, p as u32, t as u32);
                    let runs: [ArmFn; 8] = [
                        |r, x, n, p, t| {
                            arm::<true, false, false, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<false, true, false, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<true, true, false, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<true, false, true, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<false, true, true, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<false, false, false, false>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<false, false, false, true>(r, x, n, p, t, Spill::default())
                        },
                        |r, x, n, p, t| {
                            arm::<true, false, false, true>(r, x, n, p, t, Spill::default())
                        },
                    ];
                    for run in runs {
                        let (s, m2, b2) = run(&mut r, &recs, n, p, t);
                        assert_eq!((m2, b2), (mark, bits));
                        same(&r, &shipped);
                        let st = r.state();
                        crate::open::checks::arrows(
                            &l,
                            p,
                            t,
                            &s,
                            mark,
                            bits,
                            st.edges,
                            st.extra,
                            st.spelling,
                            st.offsets,
                        );
                    }
                    cases += 1;
                }
            }
        }
        println!("{cases} triples, six research arms each equal to shipped and audited");
    }

    /// The forced spill: with the packing cap lowered so that a code exceeds it partway through
    /// the band loop, F (with and without Q) gives exactly the `edges` and `extra` of a run wide
    /// from the start, and of Q's unfused wiring at the same cap; its arrows audit, and `emit`
    /// gives the plain inverse's output
    #[test]
    fn forced_wide_spill() {
        use crate::bwt::{InverseBwt, Plain};
        let mut seed = 977u32;
        let mut spilled = 0;
        let (mut a, mut b, mut c) = (
            ResearchOpen::default(),
            ResearchOpen::default(),
            ResearchOpen::default(),
        );
        let mut plain = Plain::default();
        let (mut out, mut reference) = (Vec::new(), Vec::new());
        for n in [64usize, 257, 1025, 4099] {
            for alphabet in [2u32, 5, 40] {
                let l: Vec<u8> = (0..n)
                    .map(|i| {
                        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                        // runs of mixed length, so there are thick strips and words
                        ((((seed >> 8) % alphabet) + (i as u32 / 7) % 2) % alphabet) as u8
                    })
                    .collect();
                let mut counts = [0; 256];
                for &x in &l {
                    counts[x as usize] += 1;
                }
                let recs = records_of(&l);
                for p in [0, n / 3, n - 1] {
                    for t in [1usize, 2, 4] {
                        let (s, ..) = arm::<false, false, false, false>(
                            &mut c,
                            &recs,
                            n,
                            p,
                            t,
                            Spill::default(),
                        );
                        if s.words < 2 {
                            continue;
                        }
                        // a cap the middle word's code passes
                        let cap = 255 + (s.words as u32) / 2;
                        let forced = Spill {
                            cap: Some(cap),
                            wide: false,
                        };
                        let from_start = Spill {
                            cap: Some(cap),
                            wide: true,
                        };
                        for fq in [false, true] {
                            let (s1, mark, bits) = if fq {
                                arm::<true, true, false, false>(&mut a, &recs, n, p, t, forced)
                            } else {
                                arm::<true, false, false, false>(&mut a, &recs, n, p, t, forced)
                            };
                            // wide at the end, so the spill happened (narrow until the middle word)
                            assert_eq!(a.k.extra.len(), a.k.edges.len());
                            if fq {
                                arm::<true, true, false, false>(&mut b, &recs, n, p, t, from_start);
                            } else {
                                arm::<true, false, false, false>(
                                    &mut b, &recs, n, p, t, from_start,
                                );
                            }
                            same(&a, &b);
                            arm::<false, true, false, false>(&mut c, &recs, n, p, t, forced);
                            same(&a, &c);
                            let st = a.state();
                            crate::open::checks::arrows(
                                &l,
                                p,
                                t,
                                &s1,
                                mark,
                                bits,
                                st.edges,
                                st.extra,
                                st.spelling,
                                st.offsets,
                            );
                            emit(
                                st.edges,
                                st.extra,
                                st.spelling,
                                st.offsets,
                                n,
                                mark,
                                bits,
                                &mut out,
                            );
                            plain.invert(&l, &counts, p as u32, &mut reference);
                            assert_eq!(out, reference);
                            spilled += 1;
                        }
                    }
                }
            }
        }
        assert!(spilled > 20, "{spilled}");
    }

    /// Q's 8-bit width field is enforced in release builds: a limit above 256 is refused, never
    /// packed wrong (production's instance is `LIMIT = 16`)
    #[test]
    #[should_panic(expected = "Q packs a thin run's width in 8 bits")]
    fn compact_refuses_limits_above_256() {
        let l = vec![1u8; 300];
        let recs = records_of(&l);
        let mut r = ResearchOpen::default();
        arm::<false, true, false, false>(&mut r, &recs, l.len(), 0, 257, Spill::default());
    }
}
