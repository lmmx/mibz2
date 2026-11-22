from pathlib import Path

# Cycle 29 (Neon): Attacking `pass1` (2,271 us = 21.8% of total BiOp48 runtime!)
# 1. `cand_p1_core`: `BiOp48` + Algebraic `all = thin + thick_sum` (0 `all[a]` RMWs on 98% thin runs!)
#    + Fused `pos = tail * 512 + fill` (eliminates `tails[a]` array + `tail * BYTES + 2 * f` arithmetic!)
#    + Carry-over 1-load-per-record `maximal` loop (still 100% general for adjacent same-byte records!)
# 2. `cand_p1_swar4`: `cand_p1_core` + `w <= 4` single 8-byte `u64` SWAR store in `OutQ16::push`
# 3. `cand_p1_w1`:    `cand_p1_core` + `w == 1` single 2-byte `u16` store in `OutQ16::push`

mod48 = Path("/tmp/cks_bench/cand_biop48/mod.rs").read_text()
ep48 = Path("/tmp/cks_bench/cand_biop48/endpoint.rs").read_text()
oq48 = Path("/tmp/cks_bench/cand_biop48/outq.rs").read_text()

def build_p1_variant(dst: str, push_mode: str):
    # --- Patch outq.rs: fuse `tails[256]` + `fill[256]` into single `pos: [u32; 256]` (`tail * CHUNK + fill`) ---
    # Invariant:
    # - Before first chunk for byte `a`: `pos[a] = u32::MAX` (`>= in_out_ranks`, and `pos[a] & (CHUNK-1) == CHUNK-1` -> falls into `push_slow`!).
    #   Wait: even cleaner: `pos[a] = NONE_POS` where `NONE_POS = u32::MAX & !511` + `512`? Let's just use `pos[a] = u32::MAX` (`tail = u32::MAX`, `fill = CHUNK` when `pos == u32::MAX` or `pos >= in_out_ranks`!).
    #   Wait: for overflow chunks (`tail >= in_out`), `pos[a] = tail * CHUNK + fill` still works for `tail = pos >> 9` and `fill = pos & 511` — EXCEPT when `fill == CHUNK` (`512`), `pos` has wrapped to `(tail + 1) * CHUNK`!
    #   AHA! Watch that! If `pos` increments from `tail * 512 + 511` by `1` to `(tail + 1) * 512`, then `pos >> 9` would become `tail + 1` (WRONG tail!!)!
    #   SUPER CLEAN FIX: Store `pos` as `tail * 512 + fill` ONLY while `fill < 512`, OR store `tail` in bits `10..32` (`tail << 10 | fill`, where `fill in 0..=512`)!
    #   Check `(tail << 10) | fill`:
    #   - `fill = p & 1023` (valid for `0..=512` — NEVER carries into `tail` even at `fill == 512`!!)
    #   - `tail = p >> 10` (exact even when `fill == 512`!)
    #   - Byte offset in `out`: `((p >> 10) * 1024 + 2 * (p & 1023))` = `((p & !1023) + 2 * (p & 1023))` = `p + (p & 1023)`!!
    #   WAIT A SECOND!! STOP AND ADMIRE THAT BIT-LATTICE IDENTITY!!
    #   Since `BYTES = 1024 = 1 << 10`:
    #     `(tail << 10) + 2 * fill = (p & !1023) + 2 * (p & 1023) = p + (p & 1023)`!!
    #   ZERO shifts, ZERO multiplies, ZERO carry at `fill == 512`!!
    #   AND initialize `pos = [CHUNK as u32 | (0x3F_FFFF << 10); 256]` (actually `NONE_POS = (NONE << 10)`? Wait: `in_out_limit = in_out << 10`, so `p < in_out_limit` checks `tail < in_out` in ONE comparison, and `NONE_POS = u32::MAX` satisfies `p >= in_out_limit` AND `f = p & 1023 >= CHUNK`!)
    #   Let's check `take_chunk`: `self.pos[a] = id << 10;` (`fill = 0`, `tail = id`, so `p < in_out_limit` iff `id < in_out`!).
    #   Let's check `p += w`: since `f + w <= 512 < 1024`, `p += w` NEVER overflows bit 9 into bit 10!
    #   OH THAT IS PURE ART!!

    oq = oq48
    old_q_struct = """pub(super) struct Queues<'a> {
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
    new_q_struct = """pub(super) struct Queues<'a> {
    out: &'a mut [u8],
    spill: &'a mut Spill,
    in_out: u32,
    in_out_lim: u32,
    chunks: u32,
    heads: [u32; 256],
    /// Bit-lattice fused `(tail << 10) | fill` (`fill in 0..=512`; `u32::MAX` before first chunk).
    /// Byte offset in `out` is simply `p + (p & 1023)` (since `BYTES = 1024 = 1 << 10`)!
    pos: [u32; 256],
    limit: u32,
    over: bool,
}"""
    assert old_q_struct in oq
    oq = oq.replace(old_q_struct, new_q_struct)

    old_q_new = """        Queues {
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
    new_q_new = """        let in_out = (out.len() / BYTES) as u32;
        Queues {
            in_out,
            in_out_lim: in_out << 10,
            out,
            spill,
            chunks: 0,
            heads: [NONE; 256],
            pos: [u32::MAX; 256],
            limit,
            over: false,
        }"""
    assert old_q_new in oq
    oq = oq.replace(old_q_new, new_q_new)

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
    }"""

    if push_mode == "core":
        new_push = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let p = self.pos[a];
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.pos[a] = p + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
    elif push_mode == "swar4":
        new_push = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let p = self.pos[a];
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w <= 4 && f + 4 <= CHUNK && lo < EPOCH - 4 && p < self.in_out_lim {
            // 8-byte SWAR iota store for ~92% of all thin runs (`w <= 4`): 1 u64 write, 4x fewer chunk tail spills!
            let at = p as usize + f;
            let packed = (u64::from(lo as u16).wrapping_mul(0x0001_0001_0001_0001u64))
                .wrapping_add(0x0003_0002_0001_0000u64);
            self.out[at..at + 8].copy_from_slice(&packed.to_le_bytes());
            self.pos[a] = p + w;
        } else if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.pos[a] = p + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
    elif push_mode == "w1":
        new_push = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let p = self.pos[a];
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w == 1 && f < CHUNK && lo + 1 < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            self.out[at..at + 2].copy_from_slice(&(lo as u16).to_le_bytes());
            self.pos[a] = p + 1;
        } else if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.pos[a] = p + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
    assert old_push in oq
    oq = oq.replace(old_push, new_push)

    old_pslow_and_tc = """        while left > 0 {
            if self.fill[a] as usize == CHUNK {
                self.take_chunk(a);
            }
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
            let chunk = self.chunk_mut(self.tails[a]);
            for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                .chunks_exact_mut(2)
                .enumerate()
            {
                d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
            }
            self.fill[a] += take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                // the epoch's end: every queue's length, this byte's with the ranks so far
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }

    /// A new last chunk for byte `a`
    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE);
        }
        self.spill.next[id as usize] = NONE;
        if self.heads[a] == NONE {
            self.heads[a] = id;
        } else {
            self.spill.next[self.tails[a] as usize] = id;
        }
        self.tails[a] = id;
        self.fill[a] = 0;
    }"""
    new_pslow_and_tc = """        while left > 0 {
            if self.pos[a] == u32::MAX || (self.pos[a] & 1023) as usize == CHUNK {
                self.take_chunk(a);
            }
            let p = self.pos[a];
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
            self.pos[a] = p + take;
            m += take;
            left -= take;
            if m % EPOCH == 0 {
                let mut cut = *thin;
                cut[a] += w - left;
                self.spill.cuts.push(cut);
            }
        }
    }

    /// A new last chunk for byte `a`
    fn take_chunk(&mut self, a: usize) {
        let id = self.chunks;
        self.chunks += 1;
        if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
            self.spill.ovf.push([0; BYTES]);
        }
        if self.spill.next.len() <= id as usize {
            self.spill.next.resize(id as usize + 1, NONE);
        }
        self.spill.next[id as usize] = NONE;
        if self.heads[a] == NONE {
            self.heads[a] = id;
        } else {
            self.spill.next[(self.pos[a] >> 10) as usize] = id;
        }
        self.pos[a] = id << 10;
    }"""
    assert old_pslow_and_tc in oq
    oq = oq.replace(old_pslow_and_tc, new_pslow_and_tc)

    # --- Patch mod.rs: carry-over 1-load `maximal` + algebraic `all = thin + thick_sum` ---
    m = mod48
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
    new_max = """#[inline(always)]
fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
    if records.is_empty() {
        return;
    }
    let mut pos = 0u32;
    let mut rec = records[0];
    let mut c = (rec >> 8) as u8;
    let mut width = u32::from(rec & 255);
    for &next in &records[1..] {
        let nc = (next >> 8) as u8;
        let nw = u32::from(next & 255);
        if nc == c {
            width += nw;
        } else {
            visit(c, pos, width);
            pos += width;
            c = nc;
            width = nw;
        }
    }
    visit(c, pos, width);
    let _ = rec;
}"""
    assert old_max in m
    m = m.replace(old_max, new_max)

    # Algebraic `all[a] = thin[a] + thick_sum[a]` so thin runs NEVER touch `all[a]` (`thick_sum[a]`)!
    old_piece = """    #[inline(always)]
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
    }"""
    new_piece = """    #[inline(always)]
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
                self.edges[idx + 1] = f.thin[a] + f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
            f.all[a] += w; // `f.all` holds `thick_sum` during Pass 1!
        }
    }"""
    assert old_piece in m
    m = m.replace(old_piece, new_piece)

    # Reconstruct `all[a] = thin[a] + thick_sum[a]` once at the start of `finish` (256 iterations = ~50 ns!)
    old_fin_all = """        let First {
            all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;"""
    new_fin_all = """        let First {
            mut all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;
        for a in 0..256 {
            all[a] += thin[a];
        }"""
    assert old_fin_all in m
    m = m.replace(old_fin_all, new_fin_all)

    d = Path(f"/tmp/cks_bench/{dst}")
    d.mkdir(parents=True, exist_ok=True)
    (d / "mod.rs").write_text(m)
    (d / "endpoint.rs").write_text(ep48)
    (d / "outq.rs").write_text(oq)

build_p1_variant("cand_p1_core", "core")
build_p1_variant("cand_p1_swar4", "swar4")
build_p1_variant("cand_p1_w1", "w1")
print("Generated cand_p1_core, cand_p1_swar4, cand_p1_w1 successfully.")
