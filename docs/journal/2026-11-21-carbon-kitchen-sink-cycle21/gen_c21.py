from pathlib import Path

base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

mb_mod = base_mod

old_sb = """#[derive(Clone, Copy)]
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
    spill: outq::Spill,
    strips: Vec<Strip>,
    tiles: Vec<Tile>,
    directory: Vec<u32>,
    bands: Vec<Band>,"""

new_sb = """#[derive(Clone, Copy)]
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
    bands: Vec<Band>,"""

assert old_sb in mb_mod
mb_mod = mb_mod.replace(old_sb, new_sb)

old_pass = """struct Pass<'a> {
    queues: outq::Queues<'a>,
    strips: &'a mut Vec<Strip>,
    f: First,
}"""
new_pass = """struct Pass<'a> {
    queues: outq::Queues<'a>,
    edges: &'a mut Vec<u32>,
    ns: usize,
    cap_strips: usize,
    strip_over: bool,
    f: First,
}"""
assert old_pass in mb_mod
mb_mod = mb_mod.replace(old_pass, new_pass)

old_push_strip = """        } else {
            self.strips.push(Strip {
                from: x,
                to: f.all[a],
                width: w,
                byte: c,
            });
            f.thick[a] += 1;
        }"""
new_push_strip = """        } else {
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
        }"""
assert old_push_strip in mb_mod
mb_mod = mb_mod.replace(old_push_strip, new_push_strip)

old_trace_band = """            self.bands.push(Band {
                end: *k + width,
                delta: exit.wrapping_sub(*k),
                code,
            });"""
new_trace_band = """            self.bands.push(Band::new(*k + width, exit, code));"""
assert old_trace_band in mb_mod
mb_mod = mb_mod.replace(old_trace_band, new_trace_band)

old_begin_cc = """    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32) -> Pass<'a> {
        self.strips.clear();
        Pass {
            queues: outq::Queues::new(out, &mut self.spill, limit),
            strips: &mut self.strips,
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
        let mut pass = self.begin(&mut out[..], cap.m);
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        #[cfg(test)]
        {
            self.spill.taken = pass.queues.chunks();
        }
        if over || f.m > cap.m || self.strips.len() > cap.strips {
            // refused: what the pass kept is let go to the cap's size, so that a refused
            // block leaves no more scratch than an admitted one may
            self.strips.clear();
            self.strips.shrink_to(cap.strips);
            return None;
        }
        Some(self.finish::<INDUCE, QUEUE>(&f, records, n, p, limit, &out[..], &heads))
    }"""

new_begin_cc = """    fn begin<'a>(&'a mut self, out: &'a mut [u8], limit: u32, cap_strips: usize) -> Pass<'a> {
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
    ) -> Option<(Counts, u32, u32)> {
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
    }"""

assert old_begin_cc in mb_mod
mb_mod = mb_mod.replace(old_begin_cc, new_begin_cc)

old_finish_head = """    fn finish<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        f: &First,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        out: &[u8],
        heads: &outq::Heads,
    ) -> (Counts, u32, u32) {"""
new_finish_head = """    fn finish<const INDUCE: bool, const QUEUE: bool>(
        &mut self,
        f: &First,
        ns: usize,
        records: &[u16],
        n: u32,
        p: u32,
        limit: u32,
        out: &[u8],
        heads: &outq::Heads,
    ) -> (Counts, u32, u32) {"""
assert old_finish_head in mb_mod
mb_mod = mb_mod.replace(old_finish_head, new_finish_head)

old_finish_strips = """        stats.retained = m as usize;
        stats.thick = self.strips.len();
        let first = starts(all);
        let mut write = starts(thick);
        // (EdgesMux, the Mechanisms line's cycle 19, Theorem 19.1) the strips in image order
        // are kept in `edges`' first `ns` words, dead until the wiring below writes them, and
        // the elimination's arena follows them there (`induced`): `edges` is sized for both
        // now, once, so that neither is moved
        let ns = self.strips.len();
        let need = if INDUCE {
            self.arena_words(ns, m as usize)
        } else {
            ns
        };
        if self.edges.len() < need {
            // (cold: the first blocks) exact, not doubled; nothing in it is kept
            self.edges.clear();
            self.edges.reserve_exact(need);
            self.edges.resize(need, 0);
        }
        let image_order = &mut self.edges[..ns];
        self.tiles.clear();
        let (mut last_x, mut retained_x) = (0, 0u32);
        for (i, s) in self.strips.iter_mut().enumerate() {
            let a = usize::from(s.byte);
            s.to += first[a];
            image_order[write[a] as usize] = i as u32;
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
        }"""

new_finish_strips = """        stats.retained = m as usize;
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
        }"""

assert old_finish_strips in mb_mod
mb_mod = mb_mod.replace(old_finish_strips, new_finish_strips)

mb_mod = mb_mod.replace(
    "self.induced(n, &thin, &mut stats)",
    "self.induced(ns, n, &thin, &mut stats)"
).replace(
    "let s = self.strips[self.edges[si] as usize];",
    "let s_idx = self.edges[3 * ns + si] as usize;\n                    let s = Strip::unpack(&self.edges[3 * s_idx..3 * s_idx + 3]);"
)

old_wire_b = """                for b in &self.bands[cursors[a]..end] {
                    let span = (b.end - k) as usize;
                    let dst = k.wrapping_add(b.delta) as usize;
                    let pack = if wide { 0 } else { b.code << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(b.code);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                    k = b.end;
                }"""
new_wire_b = """                for &b in &self.bands[cursors[a]..end] {
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
                }"""
assert old_wire_b in mb_mod
mb_mod = mb_mod.replace(old_wire_b, new_wire_b)

old_nq_whole = """        } else {
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
        }"""
new_nq_whole = """        } else {
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
        }"""
assert old_nq_whole in mb_mod
mb_mod = mb_mod.replace(old_nq_whole, new_nq_whole)
mb_mod = mb_mod.replace("let strips = size_of::<Strip>() * self.strips.capacity();", "let strips = 0usize;")

mb_ep = base_ep.replace(
    "use super::{Band, Counts, Kernel};",
    "use super::{Band, Counts, Kernel, Strip};"
).replace(
    "(ns + (stride + RULE) * Self::nodes_most(ns)).max(m)",
    "(4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)"
).replace(
    """    pub(super) fn induced(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(self.strips.len());
        if self.wide(self.strips.len()) {
            self.induced_with::<Wide>(n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            strips,
            bands,
            spelling,
            offsets,
            edges,
            ..
        } = self;
        // `edges` is written only once the elimination is done: its buffer holds the strips'
        // image order (its first `ns` words, `finish`), then the arena, sized by `finish`
        // (`arena_words`)
        let ns = strips.len();
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= ns + nodes_words);
        #[cfg(test)]
        if endpoint.tight {
            edges.truncate(ns + nodes_words);
        }
        let (image_order, arena) = edges.split_at_mut(ns);""",
    """    pub(super) fn induced(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            bands,
            spelling,
            offsets,
            edges,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= 4 * ns + nodes_words);
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);"""
).replace(
    """        for s in strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }""",
    """        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }"""
).replace(
    """        for (i, s) in strips.iter().enumerate() {""",
    """        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);"""
).replace(
    """        for &i in image_order.iter() {
            let s = strips[i as usize];""",
    """        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);"""
).replace(
    """            grow(edges, ns + top + RULE);
            e = Elim {
                a: &mut edges[ns..],""",
    """            grow(edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut edges[4 * ns..],"""
).replace(
    """                bands.push(Band {
                    end: k + width,
                    delta: dst.wrapping_sub(k),
                    code,
                });""",
    """                bands.push(Band::new(k + width, dst, code));"""
)

d_mb = Path("/tmp/cks_bench/cand_muxband")
d_mb.mkdir(parents=True, exist_ok=True)
(d_mb / "mod.rs").write_text(mb_mod)
(d_mb / "endpoint.rs").write_text(mb_ep)
(d_mb / "outq.rs").write_text(base_oq)

# Build `cand_diet2` (`MuxStrips + Band64 + Operad-MN16` with `META_CAP = 1536`)
op_ep = Path("/tmp/cks_bench/cand_operad/endpoint.rs").read_text()

diet2_mod = mb_mod.replace("    offsets: Vec<u32>,\n", "")
old_trace_code = """                self.spelling[start..].reverse();
                self.spelling.push(c);
                self.offsets.push(self.spelling.len() as u32);
                254 + self.offsets.len() as u32"""
new_trace_code = """                self.spelling[start..].reverse();
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
                code"""
diet2_mod = diet2_mod.replace(old_trace_code, new_trace_code).replace(
    """        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);""",
    """        self.bands.clear();
        self.spelling.clear();
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);"""
).replace(
    """    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        emit(
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
    }""",
    """    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
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
    }"""
).replace(
    "fn inline_words(",
    "#[allow(dead_code)]\nfn inline_words("
).replace(
    """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],""",
    """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],"""
).replace(
    """            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];""",
    """            let meta = &inline16[code];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];"""
).replace(
    "        let offsets = 4 * self.offsets.capacity();\n",
    "        let offsets = 0usize;\n"
)

diet2_ep = op_ep.replace(
    "use super::{Band, Counts, Kernel};",
    "use super::{Band, Counts, Kernel, Strip};"
).replace(
    "(ns + (stride + RULE) * Self::nodes_most(ns)).max(m)",
    "(4 * ns + (stride + RULE) * Self::nodes_most(ns)).max(m)"
).replace(
    """    pub(super) fn induced(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(self.strips.len());
        if self.wide(self.strips.len()) {
            self.induced_with::<Wide>(n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            strips,
            bands,
            spelling,
            edges,
            inline16,
            lens,
            ..
        } = self;
        // `edges` is written only once the elimination is done: its buffer holds the strips'
        // image order (its first `ns` words, `finish`), then the arena, sized by `finish`
        // (`arena_words`)
        let ns = strips.len();
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= ns + nodes_words);
        #[cfg(test)]
        if endpoint.tight {
            edges.truncate(ns + nodes_words);
        }
        let (image_order, arena) = edges.split_at_mut(ns);""",
    """    pub(super) fn induced(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {
        let Kernel {
            endpoint,
            bands,
            spelling,
            edges,
            inline16,
            lens,
            ..
        } = self;
        let nodes_words = L::STRIDE * most;
        assert!(edges.len() >= 4 * ns + nodes_words);
        let (hdr, arena) = edges.split_at_mut(4 * ns);
        let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);"""
).replace(
    """        for s in strips.iter() {
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }""",
    """        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, u32::from(s.byte) + 1);
            widest = widest.max(s.width);
        }"""
).replace(
    """        for (i, s) in strips.iter().enumerate() {""",
    """        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);"""
).replace(
    """        for &i in image_order.iter() {
            let s = strips[i as usize];""",
    """        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);"""
).replace(
    """            grow(edges, ns + top + RULE);
            e = Elim {
                a: &mut edges[ns..],""",
    """            grow(edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut edges[4 * ns..],"""
).replace(
    "const META_CAP: usize = 3072;",
    "const META_CAP: usize = 1536;"
).replace(
    "let used_arena = ns + etop;",
    "let used_arena = 4 * ns + etop;"
).replace(
    "&mut live_prefix[ns..]",
    "&mut live_prefix[4 * ns..]"
).replace(
    "Band { end: k + width, delta: dst.wrapping_sub(k), code }",
    "Band::new(k + width, dst, code)"
)

d_d2 = Path("/tmp/cks_bench/cand_diet2")
d_d2.mkdir(parents=True, exist_ok=True)
(d_d2 / "mod.rs").write_text(diet2_mod)
(d_d2 / "endpoint.rs").write_text(diet2_ep)
(d_d2 / "outq.rs").write_text(base_oq)
print("Generated cand_muxband and cand_diet2 successfully.")
