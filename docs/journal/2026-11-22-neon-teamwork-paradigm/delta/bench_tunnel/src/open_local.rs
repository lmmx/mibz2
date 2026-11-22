#![allow(dead_code)]

use std::time::Instant;

#[derive(Default, Clone, Copy, Debug)]
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
struct Tile {
    end: u32,
    delta: u32,
    symbol: u16,
}

#[derive(Clone, Copy)]
struct Band {
    end: u32,
    delta: u32,
    code: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum EmitMode {
    Plain,
    LazyTunnel {
        min_steps: u32,
        sub_tunnels: bool,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum ZeroRvMode {
    Exact { min_w: u8 },
    Contained { min_w: u8 },
}

#[derive(Clone, Copy, Default)]
struct Cand {
    from: u32,
    to: u32,        // initially local count in byte; + first[byte] after Pass 1
    ret_start: u32, // rank in S (0..m)
    thin_local: u32, // rank within byte's thin queue
    byte: u8,
    width: u8,
}

#[derive(Clone, Copy)]
struct TailPatch {
    tail_cand: u32,
    tail_off: u8,
    width: u8,
    head_ret: u32,
    tun_code: u32,
}

#[derive(Default)]
pub struct Kernel {
    endpoint: endpoint::Endpoint,
    spill: outq::Spill,
    strips: Vec<Strip>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,
    spelling: Vec<u8>,
    offsets: Vec<u32>,
    edges: Vec<u32>,
    extra: Vec<u32>,
    inline16: Vec<[u8; INLINE]>,
    lens: Vec<u8>,
    tunnels: Vec<(u32, u32)>,
    // ZeroRV-Tunnel reusable scratch
    cands: Vec<Cand>,
    page_dir: Vec<u32>,
    next_c: Vec<u32>,
    has_pred: Vec<u8>,
    in_mask: Vec<u16>,
    claimed_mask: Vec<u16>,
    tail_patches: Vec<TailPatch>,
    tun_word_buf: Vec<u8>,
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

const INLINE: usize = 16;
const LONG: u8 = u8::MAX;
const TUNNEL: u8 = u8::MAX - 1;
const SENTINEL: u32 = u32::MAX - 32;

impl Kernel {
    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32) -> Pass<'a> {
        self.strips.clear();
        Pass {
            queues: outq::Queues::new(out, &mut self.spill, limit),
            strips: &mut self.strips,
            f: First::default(),
        }
    }

    pub fn run_timed(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        mode: EmitMode,
        out: &mut Vec<u8>,
    ) -> (Counts, f64, f64) {
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], u32::MAX);
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, limit)
        });
        let (f, heads) = (pass.f, pass.queues.heads());
        let (stats, mark, bits) = self.finish(&f, n as u32, &out[..], &heads);
        let t1 = Instant::now();
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        match mode {
            EmitMode::Plain => {
                emit_plain(
                    &self.edges[..stats.retained],
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
            EmitMode::LazyTunnel {
                min_steps,
                sub_tunnels,
            } => {
                emit_lazy_tunnel(
                    &mut self.edges,
                    &self.extra,
                    &self.spelling,
                    &self.offsets,
                    &self.inline16,
                    &mut self.lens,
                    &mut self.tunnels,
                    stats.retained,
                    n,
                    mark,
                    bits,
                    min_steps,
                    sub_tunnels,
                    out,
                );
            }
        }
        let t2 = Instant::now();
        (
            stats,
            (t1 - t0).as_secs_f64(),
            (t2 - t1).as_secs_f64(),
        )
    }

    pub fn run_staged(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        out: &mut Vec<u8>,
    ) -> (Counts, [f64; 5]) {
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        let mut pass = self.begin(&mut out[..], u32::MAX);
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, limit)
        });
        let (f, heads) = (pass.f, pass.queues.heads());
        let t1 = Instant::now();

        let First { all, thin, thick, m, mark, runs } = f;
        let mut stats = Counts { n, runs, retained: m as usize, thick: self.strips.len(), ..Counts::default() };
        let first = starts(all);
        let mut write = starts(thick);
        let ns = self.strips.len();
        let need = self.arena_words(ns, m as usize).max(m as usize + 1);
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
        let cursors = self.induced(n as u32, &thin, &mut stats);
        stats.bands = self.bands.len();
        let t2 = Instant::now();

        let bits = (u32::BITS - m.leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize + 1 {
            self.edges.resize(m as usize + 1, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        let mut k = 0;
        for a in 0..256 {
            let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
            let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
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
        let t3 = Instant::now();

        inline_words(&self.spelling, &self.offsets, &mut self.inline16, &mut self.lens);
        let t4 = Instant::now();

        emit_plain(
            &self.edges[..stats.retained],
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
        let t5 = Instant::now();
        (stats, [
            (t1 - t0).as_secs_f64(),
            (t2 - t1).as_secs_f64(),
            (t3 - t2).as_secs_f64(),
            (t4 - t3).as_secs_f64(),
            (t5 - t4).as_secs_f64(),
        ])
    }

    pub fn get_bands_info(&self) -> Vec<(u32, u32, u32)> {
        let mut out = Vec::with_capacity(self.bands.len());
        let mut k = 0u32;
        for b in &self.bands {
            let span = b.end - k;
            let dst = k.wrapping_add(b.delta);
            out.push((dst, span, b.code));
            k = b.end;
        }
        out
    }

    /// Zero-Induction Tunnel Elimination (ZeroRV-Tunnel):
    /// Recognizes exact-match (or contained-match) thin-run tunnels via an L1 page directory,
    /// leaving `endpoint.rs` (Induce), `OutQ16`/`queue.wire`, and `emit_plain` 100% unchanged,
    /// patching ONLY tunnel tails into `edges[]` before unmodified `emit_plain`!
    /// Returns (Counts, [1stPass, Induce, TunRec, Wire, Patch+Inline, Emit], elim_rows, tunnels_count).
    pub fn run_zerorv_staged(
        &mut self,
        records: &[u16],
        n: usize,
        p: u32,
        limit: u32,
        mode: ZeroRvMode,
        out: &mut Vec<u8>,
    ) -> (Counts, [f64; 6], usize, usize) {
        let min_w = match mode {
            ZeroRvMode::Exact { min_w } | ZeroRvMode::Contained { min_w } => u32::from(min_w),
        };
        let t0 = Instant::now();
        out.resize(n + INLINE, 0);
        self.cands.clear();
        self.strips.clear();
        let mut f = First::default();
        {
            let mut queues = outq::Queues::new(&mut out[..], &mut self.spill, u32::MAX);
            maximal(records, |c, pos, width| {
                f.runs += 1;
                if width <= limit {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        f.mark = f.m + (p - pos);
                    }
                    if width >= min_w {
                        self.cands.push(Cand {
                            from: pos,
                            to: f.all[a],
                            ret_start: f.m,
                            thin_local: f.thin[a],
                            byte: c,
                            width: width as u8,
                        });
                    }
                    queues.push(a, f.m, width, &f.thin);
                    f.thin[a] += width;
                    f.m += width;
                    f.all[a] += width;
                } else if p.wrapping_sub(pos) >= width {
                    let a = usize::from(c);
                    self.strips.push(Strip { from: pos, to: f.all[a], width, byte: c });
                    f.thick[a] += 1;
                    f.all[a] += width;
                } else {
                    split(c, pos, width, p, limit, |cc, x, w, keep| {
                        let a = usize::from(cc);
                        if keep {
                            if p.wrapping_sub(x) < w {
                                f.mark = f.m + (p - x);
                            }
                            queues.push(a, f.m, w, &f.thin);
                            f.thin[a] += w;
                            f.m += w;
                        } else {
                            self.strips.push(Strip { from: x, to: f.all[a], width: w, byte: cc });
                            f.thick[a] += 1;
                        }
                        f.all[a] += w;
                    });
                }
            });
            let heads = queues.heads();
            let t1 = Instant::now();

            // Stage 2: Exact unmodified Open(L=16) Rauzy-Veech induction!
            let First { all, thin, thick, m, mark, runs } = f;
            let mut stats = Counts { n, runs, retained: m as usize, thick: self.strips.len(), ..Counts::default() };
            let first = starts(all);
            let thin_starts = starts(thin);
            let mut write = starts(thick);
            let ns = self.strips.len();
            let need = self.arena_words(ns, m as usize).max(m as usize + 1);
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
            let cursors = self.induced(n as u32, &thin, &mut stats);
            stats.bands = self.bands.len();
            let t2 = Instant::now();

            // Stage 3: ZeroRV Tunnel Recognition & Word Spelling (L1 Page Directory on cands!)
            let bits = (u32::BITS - m.leading_zeros()).max(1);
            let max_codes = (u32::MAX >> bits) as usize;
            let wide_base = (255 + stats.words as u32) > (u32::MAX >> bits);

            let mut elim_rows = 0usize;
            self.tail_patches.clear();

            if !wide_base && !self.cands.is_empty() {
                // Convert to_local -> global LF row `to` & build 128-row L1 page directory!
                const SHIFT: u32 = 7; // 128 rows/page -> ~7032 entries
                let n_pages = (n >> SHIFT) + 2;
                if self.page_dir.len() < n_pages {
                    self.page_dir.resize(n_pages, 0);
                }
                let nc = self.cands.len();
                let mut cur_page = 0usize;
                for (ci, c) in self.cands.iter_mut().enumerate() {
                    c.to += first[usize::from(c.byte)];
                    let pg = (c.from as usize) >> SHIFT;
                    while cur_page <= pg {
                        self.page_dir[cur_page] = ci as u32;
                        cur_page += 1;
                    }
                }
                while cur_page < n_pages {
                    self.page_dir[cur_page] = nc as u32;
                    cur_page += 1;
                }

                match mode {
                    ZeroRvMode::Exact { .. } => {
                        if self.next_c.len() < nc { self.next_c.resize(nc, u32::MAX); }
                        if self.has_pred.len() < nc { self.has_pred.resize(nc, 0); }
                        self.next_c[..nc].fill(u32::MAX);
                        self.has_pred[..nc].fill(0);

                        for i in 0..nc {
                            let ci = self.cands[i];
                            let to = ci.to;
                            let pg = (to as usize) >> SHIFT;
                            let lo = self.page_dir[pg] as usize;
                            let hi = self.page_dir[pg + 1] as usize;
                            for j in lo..hi {
                                let cj = self.cands[j];
                                if cj.from >= to {
                                    if cj.from == to && cj.width == ci.width && j != i {
                                        let j_has_p = p.wrapping_sub(cj.from) < u32::from(cj.width);
                                        if !j_has_p {
                                            self.next_c[i] = j as u32;
                                            self.has_pred[j] = 1;
                                        }
                                    }
                                    break;
                                }
                            }
                        }

                        // Walk heads -> spell tunnel words -> record tail patches!
                        for head in 0..nc {
                            if self.has_pred[head] == 0 && self.next_c[head] != u32::MAX {
                                // Check code budget (leave headroom for ~20% merged thick-word tails)
                                if 256 + self.offsets.len() + 2 >= max_codes {
                                    break;
                                }
                                let w = self.cands[head].width;
                                self.tun_word_buf.clear();
                                self.tun_word_buf.push(self.cands[head].byte);
                                let mut cur = head;
                                let mut steps = 1usize;
                                while self.next_c[cur] != u32::MAX {
                                    let nxt = self.next_c[cur] as usize;
                                    // Mark visited by clearing next_c? Acyclic guaranteed since has_pred[head]==0!
                                    self.tun_word_buf.push(self.cands[nxt].byte);
                                    elim_rows += usize::from(w);
                                    cur = nxt;
                                    steps += 1;
                                }
                                if steps >= 2 {
                                    self.tun_word_buf.reverse();
                                    let tun_code = (255 + self.offsets.len()) as u32;
                                    self.spelling.extend_from_slice(&self.tun_word_buf);
                                    self.offsets.push(self.spelling.len() as u32);
                                    self.tail_patches.push(TailPatch {
                                        tail_cand: cur as u32,
                                        tail_off: 0,
                                        width: w,
                                        head_ret: self.cands[head].ret_start,
                                        tun_code,
                                    });
                                }
                            }
                        }
                    }
                    ZeroRvMode::Contained { .. } => {
                        if self.in_mask.len() < nc { self.in_mask.resize(nc, 0); }
                        if self.claimed_mask.len() < nc { self.claimed_mask.resize(nc, 0); }
                        self.in_mask[..nc].fill(0);
                        self.claimed_mask[..nc].fill(0);

                        // Helper: find candidate j containing [to, to + w)
                        let find_containing = |cands: &[Cand], page_dir: &[u32], to: u32, w: u8| -> Option<(usize, u8)> {
                            let pg = (to as usize) >> SHIFT;
                            // Note: candidate containing `to` might have started in `pg - 1` (at most 15 rows earlier!)
                            let lo = if pg > 0 && page_dir[pg] > page_dir[pg - 1] {
                                (page_dir[pg] as usize).saturating_sub(1)
                            } else {
                                page_dir[pg] as usize
                            };
                            let hi = page_dir[pg + 1] as usize;
                            for j in lo..hi {
                                let cj = cands[j];
                                if cj.from > to { break; }
                                let off = to - cj.from;
                                if off + u32::from(w) <= u32::from(cj.width) {
                                    return Some((j, off as u8));
                                }
                            }
                            None
                        };

                        // Pre-mark p in claimed_mask & compute in_mask
                        for i in 0..nc {
                            let ci = self.cands[i];
                            if p.wrapping_sub(ci.from) < u32::from(ci.width) {
                                self.claimed_mask[i] |= 1u16 << (p - ci.from);
                            }
                            if let Some((j, off)) = find_containing(&self.cands, &self.page_dir, ci.to, ci.width) {
                                if j != i {
                                    let bits_w = (((1u32 << ci.width) - 1) << off) as u16;
                                    self.in_mask[j] |= bits_w;
                                }
                            }
                        }

                        // Trace maximal unclaimed head sub-intervals of width >= min_w!
                        for head in 0..nc {
                            let ch = self.cands[head];
                            let full_mask = ((1u32 << ch.width) - 1) as u16;
                            let avail = full_mask & !self.in_mask[head];
                            if avail == 0 { continue; }

                            let mut off0 = 0u8;
                            while off0 < ch.width {
                                if (avail >> off0) & 1 == 0 {
                                    off0 += 1;
                                    continue;
                                }
                                let mut w = 1u8;
                                while off0 + w < ch.width && ((avail >> (off0 + w)) & 1) != 0 {
                                    w += 1;
                                }
                                if u32::from(w) >= min_w {
                                    if 256 + self.offsets.len() + 2 >= max_codes {
                                        break;
                                    }
                                    // Mark head's own sub-interval claimed so a chain can't loop back into head
                                    let hbits = (((1u32 << w) - 1) << off0) as u16;
                                    self.claimed_mask[head] |= hbits;

                                    self.tun_word_buf.clear();
                                    self.tun_word_buf.push(ch.byte);
                                    let mut cur_j = head;
                                    let mut cur_off = off0;
                                    let mut cur_to = ch.to + u32::from(off0);
                                    let mut steps = 1usize;

                                    while let Some((nj, noff)) = find_containing(&self.cands, &self.page_dir, cur_to, w) {
                                        let nbits = (((1u32 << w) - 1) << noff) as u16;
                                        if (self.claimed_mask[nj] & nbits) != 0 {
                                            break;
                                        }
                                        self.claimed_mask[nj] |= nbits;
                                        let cnj = self.cands[nj];
                                        self.tun_word_buf.push(cnj.byte);
                                        elim_rows += usize::from(w);
                                        cur_j = nj;
                                        cur_off = noff;
                                        cur_to = cnj.to + u32::from(noff);
                                        steps += 1;
                                    }
                                    if steps >= 2 {
                                        self.tun_word_buf.reverse();
                                        let tun_code = (255 + self.offsets.len()) as u32;
                                        self.spelling.extend_from_slice(&self.tun_word_buf);
                                        self.offsets.push(self.spelling.len() as u32);
                                        self.tail_patches.push(TailPatch {
                                            tail_cand: cur_j as u32,
                                            tail_off: cur_off,
                                            width: w,
                                            head_ret: ch.ret_start + u32::from(off0),
                                            tun_code,
                                        });
                                    }
                                }
                                off0 += w;
                            }
                        }
                    }
                }
            }
            let t3 = Instant::now();

            // Stage 4: Exact unmodified Open `queue.wire`!
            if self.edges.len() < m as usize + 1 {
                self.edges.resize(m as usize + 1, 0);
            }
            self.extra.clear();
            if wide_base {
                self.extra.resize(m as usize, 0);
            }
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide_base { 0 } else { b.code << bits };
                    if wide_base {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b.end;
                }
            }
            let t4 = Instant::now();

            // Stage 5: Patch ONLY the tunnel tails into `edges[]` + `inline_words`!
            let num_tunnels = self.tail_patches.len();
            if !wide_base && num_tunnels > 0 {
                for pi in 0..num_tunnels {
                    let tp = self.tail_patches[pi];
                    let tc = self.cands[tp.tail_cand as usize];
                    let a = usize::from(tc.byte);
                    let mut z = thin_starts[a] + tc.thin_local + u32::from(tp.tail_off);
                    let z_end = z + u32::from(tp.width);
                    let mut head_r = tp.head_ret;

                    let b_start = cursors[a];
                    let b_end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let rel = self.bands[b_start..b_end].partition_point(|b| b.end <= z);
                    let mut bi = b_start + rel;

                    while z < z_end {
                        let b = self.bands[bi];
                        let span = z_end.min(b.end) - z;
                        let dst = z.wrapping_add(b.delta) as usize;
                        let final_code = if b.code < 256 {
                            tp.tun_code
                        } else {
                            // Merge thick prefix b.code[..len-1] + tun_code!
                            if 256 + self.offsets.len() > max_codes {
                                // Out of u32 codes: leave this band sub-span unpatched (exact!)
                                z += span;
                                head_r += span;
                                bi += 1;
                                continue;
                            }
                            let bw = (b.code - 256) as usize;
                            let b_lo = self.offsets[bw] as usize;
                            let b_hi = self.offsets[bw + 1] as usize;
                            let tw = (tp.tun_code - 256) as usize;
                            let t_lo = self.offsets[tw] as usize;
                            let t_hi = self.offsets[tw + 1] as usize;
                            let new_code = (255 + self.offsets.len()) as u32;
                            // Copy prefix (excluding trailing byte c) + full tunnel word (which starts with c!)
                            self.spelling.extend_from_within(b_lo..b_hi - 1);
                            self.spelling.extend_from_within(t_lo..t_hi);
                            self.offsets.push(self.spelling.len() as u32);
                            new_code
                        };
                        let pack = final_code << bits;
                        let mut val = pack | head_r;
                        for e in &mut self.edges[dst..dst + span as usize] {
                            *e = val;
                            val = val.wrapping_add(1);
                        }
                        z += span;
                        head_r += span;
                        bi += 1;
                    }
                }
            }
            inline_words(&self.spelling, &self.offsets, &mut self.inline16, &mut self.lens);
            let t5 = Instant::now();

            // Stage 6: 100% UNMODIFIED `emit_plain`!
            emit_plain(
                &self.edges[..stats.retained],
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
            let t6 = Instant::now();

            (
                stats,
                [
                    (t1 - t0).as_secs_f64(),
                    (t2 - t1).as_secs_f64(),
                    (t3 - t2).as_secs_f64(),
                    (t4 - t3).as_secs_f64(),
                    (t5 - t4).as_secs_f64(),
                    (t6 - t5).as_secs_f64(),
                ],
                elim_rows,
                num_tunnels,
            )
        }
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
        let need = self.arena_words(ns, m as usize).max(m as usize + 1);
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
        // Ensure strictly m < (1 << bits) so next < mask always
        let bits = (u32::BITS - m.leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize + 1 {
            self.edges.resize(m as usize + 1, 0);
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
}

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

#[allow(clippy::too_many_arguments)]
fn emit_plain(
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

#[allow(clippy::too_many_arguments)]
fn emit_lazy_tunnel(
    edges_buf: &mut [u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &mut Vec<u8>,
    tunnels: &mut Vec<(u32, u32)>,
    m: usize,
    n: usize,
    mark: u32,
    bits: u32,
    min_steps: u32,
    sub_tunnels: bool,
    out: &mut Vec<u8>,
) {
    let tunnel_base = lens.len();
    let max_codes = (u32::MAX >> bits).saturating_sub(2) as usize;
    if !extra.is_empty() || m < 64 || max_codes <= tunnel_base + 16 {
        emit_plain(
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

    tunnels.clear();
    out.resize(n + INLINE, 0);
    let edges = &mut edges_buf[..m + 1];
    edges[m] = SENTINEL;

    let mask = (1u32 << bits) - 1;
    let mark_idx = mark as usize;
    let mark_edge = edges[mark_idx];
    // Sentinel at mark guarantees no tunnel strip ever includes mark!
    edges[mark_idx] = SENTINEL;

    let mut at = 0usize;
    // Step 0: emit mark's word
    let mut j = {
        let code = (mark_edge >> bits) as usize;
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
        (mark_edge & mask) as usize
    };

    if j == mark_idx || at >= n {
        out.truncate(at.min(n));
        repeat(n, out);
        return;
    }

    let max_s: usize = 15;
    let mut rem = m - 1;

    'outer: while rem > 0 {
        rem -= 1;
        let u = j;
        let e = edges[u];
        let code = (e >> bits) as usize;
        let next = (e & mask) as usize;
        let len = lens[code];

        if len <= INLINE as u8 {
            let has_l = edges[u.saturating_sub(1)] == e.wrapping_sub(1);
            let has_r = edges[u + 1] == e.wrapping_add(1);
            let step_start_at = at;
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
            j = next;
            if j == mark_idx || at >= n {
                break 'outer;
            }
            if has_l | has_r {
                // Measure initial two-sided slack (bounded by next and m-1-next to prevent bit carry/borrow!)
                let max_l = max_s.min(u).min(next);
                let mut t_left = 0usize;
                if has_l {
                    t_left = 1;
                    while t_left < max_l && edges[u - t_left - 1] == e.wrapping_sub((t_left + 1) as u32) {
                        t_left += 1;
                    }
                }
                let max_r = max_s.min(m - 1 - u).min(m - 1 - next);
                let mut t_right = 0usize;
                if has_r {
                    t_right = 1;
                    while t_right < max_r && edges[u + t_right + 1] == e.wrapping_add((t_right + 1) as u32) {
                        t_right += 1;
                    }
                }
                if t_left + t_right > 0 {
                    let t_start_u = u;
                    let t_start_out = step_start_at;
                    let mut t_steps = 1u32;

                    // Inner tracking loop
                    while rem > 0 {
                        rem -= 1;
                        let cu = j;
                        let ce = edges[cu];
                        let ccode = (ce >> bits) as usize;
                        let cnext = (ce & mask) as usize;
                        let clen = lens[ccode];

                        if clen == TUNNEL {
                            // Record active tunnel ending at cu BEFORE taking shortcut!
                            if t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                                let tid_code = lens.len() as u32;
                                lens.push(TUNNEL);
                                tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                                let shifted = tid_code << bits;
                                for d in 1..=t_left {
                                    let src = t_start_u - d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu - d) as u32);
                                    }
                                }
                                for d in 1..=t_right {
                                    let src = t_start_u + d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu + d) as u32);
                                    }
                                }
                            }
                            let (out_pos, byte_len) = tunnels[ccode - tunnel_base];
                            let src = out_pos as usize;
                            let take = (byte_len as usize).min(n - at);
                            out.copy_within(src..src + take, at);
                            at += take;
                            j = cnext;
                            if j == mark_idx || at >= n {
                                break 'outer;
                            }
                            break;
                        }

                        // Measure slack of cu capped at t_left, t_right
                        let lim_l = t_left.min(cu).min(cnext);
                        let mut nl = 0usize;
                        while nl < lim_l && edges[cu - nl - 1] == ce.wrapping_sub((nl + 1) as u32) {
                            nl += 1;
                        }
                        let lim_r = t_right.min(m - 1 - cu).min(m - 1 - cnext);
                        let mut nr = 0usize;
                        while nr < lim_r && edges[cu + nr + 1] == ce.wrapping_add((nr + 1) as u32) {
                            nr += 1;
                        }

                        if nl + nr == 0 {
                            // Tunnel ended at cu (exit = cu)!
                            if t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                                let tid_code = lens.len() as u32;
                                lens.push(TUNNEL);
                                tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                                let shifted = tid_code << bits;
                                for d in 1..=t_left {
                                    let src = t_start_u - d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu - d) as u32);
                                    }
                                }
                                for d in 1..=t_right {
                                    let src = t_start_u + d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu + d) as u32);
                                    }
                                }
                            }
                            // Emit cu's normal word and return to fast outer loop
                            if clen <= INLINE as u8 {
                                out[at..at + INLINE].copy_from_slice(&inline16[ccode]);
                                at += usize::from(clen);
                            } else {
                                let w = ccode - 256;
                                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                                let take = word.len().min(n - at);
                                out[at..at + take].copy_from_slice(&word[..take]);
                                at += take;
                            }
                            j = cnext;
                            if j == mark_idx || at >= n {
                                break 'outer;
                            }
                            break;
                        } else {
                            if sub_tunnels && (nl < t_left || nr < t_right) && t_steps >= min_steps && lens.len() <= max_codes && at > t_start_out {
                                let tid_code = lens.len() as u32;
                                lens.push(TUNNEL);
                                tunnels.push((t_start_out as u32, (at - t_start_out) as u32));
                                let shifted = tid_code << bits;
                                for d in (nl + 1)..=t_left {
                                    let src = t_start_u - d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu - d) as u32);
                                    }
                                }
                                for d in (nr + 1)..=t_right {
                                    let src = t_start_u + d;
                                    if (edges[src] >> bits) < tunnel_base as u32 {
                                        edges[src] = shifted | ((cu + d) as u32);
                                    }
                                }
                            }
                            t_left = nl;
                            t_right = nr;
                            t_steps += 1;
                            if clen <= INLINE as u8 {
                                out[at..at + INLINE].copy_from_slice(&inline16[ccode]);
                                at += usize::from(clen);
                            } else {
                                let w = ccode - 256;
                                let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
                                let take = word.len().min(n - at);
                                out[at..at + take].copy_from_slice(&word[..take]);
                                at += take;
                            }
                            j = cnext;
                            if j == mark_idx || at >= n {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        } else if len == TUNNEL {
            let (out_pos, byte_len) = tunnels[code - tunnel_base];
            let src = out_pos as usize;
            let take = (byte_len as usize).min(n - at);
            out.copy_within(src..src + take, at);
            at += take;
            j = next;
            if j == mark_idx || at >= n {
                break 'outer;
            }
        } else {
            // LONG word
            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
            j = next;
            if j == mark_idx || at >= n {
                break 'outer;
            }
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
