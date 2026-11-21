from pathlib import Path

# Build `cand_xor` directly from verbatim `master_a1` so we isolate `XOR-BiEmit` cleanly against `master_a1` (and also build `cand_xor_mn16` combining `MN16-Direct + XOR-BiEmit`)!
base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# Let's check `outq.rs`: add `wire_xor` to `Reader`!
# In `wire_xor(&mut self, edges: &mut [u32], dst_start: usize, span: usize, pack: u32, mark: u32, j_neg1: &mut u32)`:
# Note: in `edges`, `dst` slice `dst_start..dst_start + span` AND scattered `v` indices `0..m` both live in `edges`!
# Passing `edges: &mut [u32]` to `wire_xor` lets safe Rust index `edges[u] ^= pack | v` and `edges[v as usize] ^= u as u32` with 100% bounds-checked safe Rust (`#![forbid(unsafe_code)]`)!

xor_oq_add = """
    /// Wire XOR-bijective doubly-linked permutation edges:
    /// For each offset `i in 0..span`, `u = dst_start + i` transitions to `v` popped from queue:
    ///   edges[u] ^= pack | v;
    ///   edges[v as usize] ^= u as u32;
    /// Records `*j_neg1 = u as u32` when `v == mark`.
    #[inline(always)]
    pub(super) fn wire_xor(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            for (i, s) in self.out[at..at + 2 * span].chunks_exact(2).enumerate() {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark {
                    *j_neg1 = u as u32;
                }
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
            }
            self.at += span;
            self.read += span as u32;
        } else {
            self.wire_xor_slow(edges, dst_start, span, pack, mark, j_neg1);
        }
    }

    #[cold]
    #[inline(never)]
    fn wire_xor_slow(
        &mut self,
        edges: &mut [u32],
        mut dst_start: usize,
        mut left: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        while left > 0 {
            while self.read == self.cut {
                self.epoch += 1;
                self.cut = self
                    .spill
                    .cuts
                    .get(self.epoch as usize)
                    .map_or(u32::MAX, |c| c[self.a]);
            }
            if self.at == CHUNK {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = left
                .min(CHUNK - self.at)
                .min((self.cut - self.read) as usize);
            let chunk: &[u8] = if self.id < self.in_out {
                let at = self.id as usize * BYTES;
                &self.out[at..at + BYTES]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][..]
            };
            let hi = self.epoch << 16;
            for (i, s) in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2).enumerate() {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark {
                    *j_neg1 = u as u32;
                }
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
            }
            self.at += take;
            self.read += take as u32;
            dst_start += take;
            left -= take;
        }
    }
"""

xor_oq = base_oq.replace(
    "    pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {",
    xor_oq_add + "\n    #[allow(dead_code)]\n    pub(super) fn wire(&mut self, dst: &mut [u32], pack: u32) {"
)

# Now update `mod.rs` for `cand_xor`:
# `construct_capped` and `finish` return `(Counts, u32, u32, u32)` -> `(stats, mark, j_neg1, bits)`!
xor_mod = base_mod.replace(
    "    ) -> Option<(Counts, u32, u32)> {",
    "    ) -> Option<(Counts, u32, u32, u32)> {"
).replace(
    "    ) -> (Counts, u32, u32) {",
    "    ) -> (Counts, u32, u32, u32) {"
)

old_wire_block = """        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        if QUEUE {
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
        (stats, mark, bits)"""

new_wire_block = """        if self.edges.len() < m as usize {
            self.edges.resize(m as usize, 0);
        }
        self.edges[..m as usize].fill(0);
        self.extra.clear();
        if wide {
            self.extra.resize(m as usize, 0);
        }
        stats.table_bytes = 4 * (m as usize + self.extra.len());
        let mut j_neg1 = mark;
        if QUEUE {
            let mut k = 0;
            let edges_m = &mut self.edges[..m as usize];
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
                    queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);
                    k = b.end;
                }
            }
        } else {
            let mut stream = starts(thin);
            let mut j = 0;
            let edges_m = &mut self.edges[..m as usize];
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
                    for i in 0..span {
                        let u = dest + i;
                        let v = j + i as u32;
                        if v == mark {
                            j_neg1 = u as u32;
                        }
                        edges_m[u] ^= pack | v;
                        edges_m[v as usize] ^= u as u32;
                    }
                    j += span as u32;
                    z += span as u32;
                    cursors[a] += usize::from(z == b.end);
                }
                stream[a] = end;
            });
        }
        (stats, mark, j_neg1, bits)"""

assert old_wire_block in xor_mod
xor_mod = xor_mod.replace(old_wire_block, new_wire_block)

old_try_run = """        let (stats, mark, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, bits, out);"""
new_try_run = """        let (stats, mark, j_neg1, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, j_neg1, bits, out);"""
xor_mod = xor_mod.replace(old_try_run, new_try_run)

old_run_stages = """        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);"""
new_run_stages = """        let (stats, mark, j_neg1, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, j_neg1, bits, out);"""
xor_mod = xor_mod.replace(old_run_stages, new_run_stages)

old_walk_emit = """    /// The words inlined and the walk from the marked row: the block's bytes into `out`
    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
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
    }"""

new_walk_emit = """    /// The words inlined and the bidirectional XOR-meet-in-the-middle walk from `(mark, j_neg1)`
    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, bits: u32, out: &mut Vec<u8>) {
        inline_words(
            &self.spelling,
            &self.offsets,
            &mut self.inline16,
            &mut self.lens,
        );
        emit_xor_bi(
            &self.edges[..m],
            &self.extra,
            &self.spelling,
            &self.offsets,
            &self.inline16,
            &self.lens,
            n,
            mark,
            j_neg1,
            bits,
            out,
        );
    }"""
assert old_walk_emit in xor_mod
xor_mod = xor_mod.replace(old_walk_emit, new_walk_emit)

bi_emit_fn = """
#[allow(clippy::too_many_arguments)]
fn emit_xor_bi(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    // Layout: reserve 16 bytes of left overrun (if needed? No! Backward cursor shifts u128 left
    // so padding lands in `[at_b - 16 .. at_b - len_b]`, which is >= at_f + 16 while `at_b - at_f >= 32`!
    // And at `at_b == n`, `at_b - 16 == n - 16 >= at_f + 16` whenever `n >= 32`!)
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut prev_f = j_neg1;
    let mut cur_f = mark;
    let mut next_b = mark;
    let mut cur_b = j_neg1;
    let mut at_f = 0usize;
    let mut at_b = n;

    let max_pairs = edges.len() / 2 + 1;

    // Fast dual-cursor loop while middle gap >= 32 bytes (guarantees both 16-byte stores
    // `out[at_f..at_f+16]` and `out[at_b-16..at_b]` stay strictly inside unwritten `[at_f..at_b)`)
    let mut rem_pairs = max_pairs;
    while rem_pairs > 0 && cur_f != cur_b && cur_f != next_b && at_f + 32 <= at_b {
        rem_pairs -= 1;
        // Issue BOTH independent loads in parallel (mu = 2 MLP!)
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];

        let cf = if extra.is_empty() { (ef >> bits) as usize } else { extra[cur_f as usize] as usize };
        let cb = if extra.is_empty() { (eb >> bits) as usize } else { extra[cur_b as usize] as usize };

        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
            at_f += usize::from(lf);
        } else {
            let w = cf - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            // Right-align via single register shift `u128 << (8 * (16 - lb))` so padding lands in `[at_b - 16 .. at_b - lbu)`!
            let raw = u128::from_le_bytes(inline16[cb]);
            let shifted = if lbu == 16 { raw } else { raw << ((16 - lbu) * 8) };
            out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
            at_b -= lbu;
        } else {
            let w = cb - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    // Exact tail loop when middle gap < 32 bytes (at most 2-4 steps!)
    while rem_pairs > 0 && cur_f != cur_b && cur_f != next_b && at_f < at_b {
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = if extra.is_empty() { (ef >> bits) as usize } else { extra[cur_f as usize] as usize };
        let cb = if extra.is_empty() { (eb >> bits) as usize } else { extra[cur_b as usize] as usize };
        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let w = cf - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            let take = lbu.min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
            at_b -= take;
        } else {
            let w = cb - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    // Odd cycle length L = 2k + 1: one middle vertex `cur_f == cur_b` remains
    if cur_f == cur_b && at_f < at_b {
        let ef = edges[cur_f as usize];
        let cf = if extra.is_empty() { (ef >> bits) as usize } else { extra[cur_f as usize] as usize };
        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let w = cf - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }

    // If periodic block (cycle spelled `at_f + (n - at_b) < n` bytes), compact right half onto `at_f` and repeat!
    // On every primitive block (`d == 1`), `at_f == at_b` and this branch is skipped completely!
    if at_f < at_b {
        let right_len = n - at_b;
        if right_len > 0 {
            out.copy_within(at_b..n, at_f);
        }
        let period = at_f + right_len;
        out.truncate(period);
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}
"""

xor_mod = xor_mod.replace("#[allow(clippy::too_many_arguments)]\nfn emit(", bi_emit_fn + "\n#[allow(dead_code)]\n#[allow(clippy::too_many_arguments)]\nfn emit(")

d_xor = Path("/tmp/cks_bench/cand_xor")
d_xor.mkdir(parents=True, exist_ok=True)
(d_xor / "mod.rs").write_text(xor_mod)
(d_xor / "endpoint.rs").write_text(base_ep)
(d_xor / "outq.rs").write_text(xor_oq)
print("Generated cand_xor successfully.")
