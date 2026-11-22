import os, shutil

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_combo"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def patch_cap127(dst):
    """Canonical runs.rs invariant: a record with (rec & 255) < 127 never merges with records[i+1]."""
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    old_call = """        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });"""
    new_call = """        {
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
                    pass.f.runs += 1;
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
        }"""
    assert old_call in s
    s = s.replace(old_call, new_call, 1)
    with open(mod_p, "w") as f:
        f.write(s)

def patch_coldmerge(dst):
    """100% general (works even on unmerged records): hoist continuation while-loop into #[cold] helper."""
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    old_max = """fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
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
}"""
    new_max = """#[cold]
#[inline(never)]
fn merge_rest(records: &[u16], i: &mut usize, c: u8, mut width: u32) -> u32 {
    let mut idx = *i;
    while idx < records.len() && (records[idx] >> 8) as u8 == c {
        width += u32::from(records[idx] & 255);
        idx += 1;
    }
    *i = idx;
    width
}

fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    let mut pos = 0;
    let mut i = 0;
    let len = records.len();
    while i < len {
        let rec = records[i];
        let c = (rec >> 8) as u8;
        let mut width = u32::from(rec & 255);
        i += 1;
        if i < len && (records[i] >> 8) as u8 == c {
            width = merge_rest(records, &mut i, c, width);
        }
        visit(c, pos, width);
        pos += width;
    }
}"""
    assert old_max in s
    s = s.replace(old_max, new_max, 1)
    with open(mod_p, "w") as f:
        f.write(s)

def patch_pt64(dst):
    """Wormhole CleanPt64: single [u64; 256] packing (thin[a] << 32) | (tail << 10) | fill."""
    mod_p = f"{dst}/mod.rs"
    outq_p = f"{dst}/outq.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(outq_p) as f:
        o = f.read()

    # In outq.rs: replace tails+fill with pt: [u64; 256] (and in_out_lim = in_out << 10)
    old_q = """pub(super) struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    /// chunks `out` holds
    in_out: u32,
    /// chunks taken
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    /// ranks in each byte's last chunk (`CHUNK` before its first)
    fill: [u32; 256],
    /// the retained ranks the block may have (`Cap::m`)
    limit: u32,
    /// (sticky) a push went past `limit`: the block is refused, and no chunk is taken after
    over: bool,
}"""
    new_q = """pub(super) struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    in_out_lim: u32,
    chunks: u32,
    heads: [u32; 256],
    /// Packed `(thin[a] << 32) | (tail << 10) | fill` (`u32::MAX` in low 32 before first chunk)
    pt: [u64; 256],
    limit: u32,
    over: bool,
}"""
    assert old_q in o
    o = o.replace(old_q, new_q, 1)

    old_new = """        Queues {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
        }"""
    new_new = """        let in_out = (out.len() / BYTES) as u32;
        Queues {
            in_out,
            in_out_lim: in_out << 10,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            pt: [u32::MAX as u64; 256],
            limit,
            over: false,
        }"""
    assert old_new in o
    o = o.replace(old_new, new_new, 1)

    old_push_blk = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
    new_push_blk = """    #[inline(always)]
    pub(super) fn thin_of(&self, a: usize) -> u32 {
        (self.pt[a] >> 32) as u32
    }

    pub(super) fn extract_thin(&self) -> [u32; 256] {
        std::array::from_fn(|a| (self.pt[a] >> 32) as u32)
    }

    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, x) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&x.to_le_bytes());
            }
            let wu = u64::from(w);
            self.pt[a] = v + ((wu << 32) | wu);
        } else {
            self.push_slow_pt(a, m, w);
        }
    }

    #[cold]
    #[inline(never)]
    fn push_slow_pt(&mut self, a: usize, mut m: u32, w: u32) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;
        while left > 0 {
            let p = self.pt[a] as u32;
            if p == u32::MAX || (p & 1023) as usize == CHUNK {
                self.take_chunk_pt(a);
            }
            let v = self.pt[a];
            let p = v as u32;
            let f = (p & 1023) as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = self.chunk_mut(p >> 10);
            for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                .chunks_exact_mut(2)
                .enumerate()
            {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            let tu = u64::from(take);
            self.pt[a] = v + ((tu << 32) | tu);
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                let cut: [u32; 256] = std::array::from_fn(|b| (self.pt[b] >> 32) as u32);
                self.spill.cuts.push(cut);
            }
        }
    }

    fn take_chunk_pt(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE);
        }
        self.spill.next[id as usize] = NONE;
        let hi = self.pt[a] & !0xFFFF_FFFFu64;
        if self.heads[a] == NONE {
            self.heads[a] = id;
        } else {
            self.spill.next[((self.pt[a] as u32) >> 10) as usize] = id;
        }
        self.pt[a] = hi | u64::from(id << 10);
    }"""
    assert old_push_blk in o
    # Remove old push_slow & take_chunk since we replaced them
    i_ps = o.index("    fn push_slow(")
    i_cm = o.index("    fn chunk_mut(")
    o = o[:i_ps] + o[i_cm:]
    o = o.replace(old_push_blk, new_push_blk, 1)

    # Patch mod.rs Pass::piece to use push_thin when QUEUE is true
    old_piece = """        if keep {
            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
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
                    let need = (at + 3)
                        .max(2 * self.edges.len())
                        .max(1024)
                        .min(self.cap_strips.saturating_mul(3));
                    self.edges.reserve_exact(need - self.edges.len());
                    self.edges.resize(need, 0);
                }
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }"""
    new_piece = """        if keep {
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
            }"""
    assert old_piece in s
    s = s.replace(old_piece, new_piece, 1)

    old_fin_f = "        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());"
    new_fin_f = """        let mut f = pass.f;
        if QUEUE {
            f.thin = pass.queues.extract_thin();
        }
        let (heads, over) = (pass.queues.heads(), pass.queues.over());"""
    assert old_fin_f in s
    s = s.replace(old_fin_f, new_fin_f, 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_wc(dst, slot_bytes=128):
    """L1D/DTLB Write-Combining Stage (`256 * slot_bytes` on stack: 16 KiB = 4 pages or 32 KiB = 8 pages, +0 B persistent scratch!)."""
    assert slot_bytes in (64, 128)
    slot_ranks = slot_bytes // 2
    slot_shift = 6 if slot_bytes == 64 else 7
    mod_p = f"{dst}/mod.rs"
    outq_p = f"{dst}/outq.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(outq_p) as f:
        o = f.read()

    old_q = """pub(super) struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    /// chunks `out` holds
    in_out: u32,
    /// chunks taken
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    /// ranks in each byte's last chunk (`CHUNK` before its first)
    fill: [u32; 256],
    /// the retained ranks the block may have (`Cap::m`)
    limit: u32,
    /// (sticky) a push went past `limit`: the block is refused, and no chunk is taken after
    over: bool,
}"""
    new_q = f"""const WC_RANKS: usize = {slot_ranks};
const WC_BYTES: usize = {slot_bytes};

pub(super) struct Queues<'a> {{
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    chunks: u32,
    heads: [u32; 256],
    tails: [u32; 256],
    fill: [u32; 256],
    /// Contiguous L1D write-combining stage (`256 * {slot_bytes} B` = {256*slot_bytes//1024} KiB = {256*slot_bytes//4096} DTLB pages!)
    wc: Box<[u8; 256 * WC_BYTES]>,
    wf: [u8; 256],
    limit: u32,
    over: bool,
}}"""
    # Wait: don't Box-allocate per block! Put `[u8; 256 * WC_BYTES]` on the stack directly inside Queues (zero heap alloc, zero persistent scratch)!
    new_q = new_q.replace("Box<[u8; 256 * WC_BYTES]>", "[u8; 256 * WC_BYTES]")
    assert old_q in o
    o = o.replace(old_q, new_q, 1)

    old_new = """        Queues {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
            limit,
            over: false,
        }"""
    new_new = """        Queues {
            in_out: (out.len() / BYTES) as u32,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            tails: [NONE; 256],
            fill: [CHUNK as u32; 256],
            wc: [0u8; 256 * WC_BYTES],
            wf: [0u8; 256],
            limit,
            over: false,
        }"""
    assert old_new in o
    o = o.replace(old_new, new_new, 1)

    old_heads = """    pub(super) fn heads(&self) -> Heads {
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }"""
    new_heads = """    pub(super) fn flush_all_and_heads(&mut self) -> Heads {
        if !self.over {
            for a in 0..256 {
                if self.wf[a] > 0 {
                    self.flush_wc(a);
                }
            }
        }
        Heads {
            heads: self.heads,
            in_out: self.in_out,
        }
    }"""
    assert old_heads in o
    o = o.replace(old_heads, new_heads, 1)

    old_push = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }

    /// `push` across a chunk's end or an epoch's, into an overflow chunk, or of more than 16
    /// ranks (a limit above 16, in the tests): rank by rank. Past the queues' `limit` it sets
    /// `over` and writes nothing: every chunk is taken here, so the queues take none once the
    /// block is refused, and `push` itself pays nothing for the limit (only a chunk's end
    /// brings it here, and its fast case writes within a chunk already taken).
    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
        if self.over || m + w > self.limit {
            self.over = true;
            return;
        }
        let mut left = w;"""

    new_push = f"""    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {{
        let wf = usize::from(self.wf[a]);
        let lo = m & (EPOCH - 1);
        if w <= 16 && wf + 16 <= WC_RANKS && lo + w < EPOCH && m + w <= self.limit {{
            let at = (a << {slot_shift}) + 2 * wf;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.wc[at..at + 32].chunks_exact_mut(2).zip(iota) {{
                d.copy_from_slice(&v.to_le_bytes());
            }}
            self.wf[a] = (wf + w as usize) as u8;
        }} else {{
            self.push_slow(a, m, w, thin);
        }}
    }}

    #[cold]
    #[inline(never)]
    fn flush_wc(&mut self, a: usize) {{
        let mut left = usize::from(self.wf[a]);
        let mut src = a << {slot_shift};
        while left > 0 {{
            if self.fill[a] as usize == CHUNK {{
                self.take_chunk(a);
            }}
            let f = self.fill[a] as usize;
            let take = left.min(CHUNK - f);
            let id = self.tails[a];
            let chunk: &mut [u8] = if id < self.in_out {{
                let at = id as usize * BYTES;
                &mut self.out[at..at + BYTES]
            }} else {{
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }};
            chunk[2 * f..2 * (f + take)].copy_from_slice(&self.wc[src..src + 2 * take]);
            self.fill[a] += take as u32;
            src += 2 * take;
            left -= take;
        }}
        self.wf[a] = 0;
    }}

    #[cold]
    #[inline(never)]
    fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {{
        if self.over || m + w > self.limit {{
            self.over = true;
            return;
        }}
        if self.wf[a] > 0 {{
            self.flush_wc(a);
        }}
        let mut left = w;"""
    assert old_push in o
    o = o.replace(old_push, new_push, 1)

    s = s.replace("pass.queues.heads()", "pass.queues.flush_all_and_heads()", 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(outq_p, "w") as f:
        f.write(o)

d = clone("cand_c37_cap127");    patch_cap127(d)
d = clone("cand_c37_coldmerge"); patch_coldmerge(d)
d = clone("cand_c37_pt64");      patch_pt64(d)
d = clone("cand_c37_wc64");      patch_wc(d, slot_bytes=64)
d = clone("cand_c37_wc128");     patch_wc(d, slot_bytes=128)
d = clone("cand_c37_cap_pt64");  patch_cap127(d); patch_pt64(d)
print("Generated 6 Cycle 37 S1 candidates!")
