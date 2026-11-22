from pathlib import Path

# 1. Patch `master_a1` and `master_a2` cleanly (verbatim 0.5.11 kernel + benchmark entrypoints)
def patch_master(d: Path):
    m = Path("src/open.rs").read_text()
    m = m.replace("use crate::bwt::{InverseBwt, LazyCells};\nuse crate::{runs::Runs, Kernels};\n", "")
    cut_oc = m.index("/// The workers' kernel wherever production ran `LazyCells`")
    cut_after = m.index("/// The bytes of a code's word kept inline")
    m = m[:cut_oc] + m[cut_after:]
    m = m.replace("pub(crate) ", "pub ")
    m = m.replace("#[cfg(test)]\nmod checks;\n", "")
    if "#[cfg(test)]\nmod tests" in m:
        m = m[:m.index("#[cfg(test)]\nmod tests")]
    m += """
impl Kernel {
    #[inline(never)]
    pub fn run_entry(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, LIMIT, cap, out)
    }
    #[inline(never)]
    pub fn run_stages(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> (f64, f64) {
        let t0 = std::time::Instant::now();
        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);
        let tw = t1.elapsed().as_secs_f64() * 1e6;
        (tc, tw)
    }
    pub fn spelling_stats(&self) -> (usize, usize) {
        (self.spelling.len(), self.spelling.capacity())
    }
    pub fn scratch_report(&self) -> (usize, usize, String) {
        use std::mem::size_of;
        let q = self.spill.scratch_bytes();
        let strips = size_of::<Strip>() * self.strips.capacity();
        let tiles_dir = size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
        let bands = size_of::<Band>() * self.bands.capacity();
        let spelling = self.spelling.capacity();
        let offsets = 4 * self.offsets.capacity();
        let edges = 4 * self.edges.capacity();
        let extra = 4 * self.extra.capacity();
        let inline16 = 16 * self.inline16.capacity();
        let lens = self.lens.capacity();
        let ep = self.endpoint.scratch_bytes();
        let tot = q + strips + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
        let dict_l1 = 16 * self.inline16.len() + self.lens.len();
        let s = format!(
            "queues={q} strips={strips} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}"
        );
        (tot, dict_l1, s)
    }
}
"""
    (d / "mod.rs").write_text(m)
    ep = Path("src/open/endpoint.rs").read_text()
    ep += "\nimpl Endpoint {\n    pub(super) fn scratch_bytes(&self) -> usize { 4 * self.stack.capacity() }\n}\n"
    (d / "endpoint.rs").write_text(ep)
    oq = Path("src/open/outq.rs").read_text()
    oq += "\nimpl Spill {\n    pub(super) fn scratch_bytes(&self) -> usize { 2 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity() }\n}\n"
    (d / "outq.rs").write_text(oq)

patch_master(Path("/tmp/cks_bench/master_a1"))
patch_master(Path("/tmp/cks_bench/master_a2"))

# Add spelling_stats and 6-phase profiler helper to cand_cks_bi & cand_cks_diet
for cname in ["cand_cks_diet", "cand_cks_bi"]:
    p = Path(f"/tmp/cks_bench/{cname}/mod.rs")
    s = p.read_text()
    if "fn spelling_stats" not in s:
        s = s.replace(
            "    pub fn scratch_report(&self)",
            "    pub fn spelling_stats(&self) -> (usize, usize) { (self.spelling.len(), self.spelling.capacity()) }\n    pub fn scratch_report(&self)",
        )
        p.write_text(s)

bi_mod = Path("/tmp/cks_bench/cand_cks_bi/mod.rs").read_text()
bi_ep = Path("/tmp/cks_bench/cand_cks_bi/endpoint.rs").read_text()
bi_oq = Path("/tmp/cks_bench/cand_cks_bi/outq.rs").read_text()

extra_oq = """
    #[inline(always)]
    pub(super) fn read_count(&self) -> u32 {
        self.read
    }

    #[inline(always)]
    pub(super) fn set_epoch(&mut self, ep: u32, next_cut: u32) {
        self.epoch = ep;
        self.cut = next_cut;
    }

    #[inline(always)]
    pub(super) fn peek_lo(&mut self) -> u16 {
        if self.at == CHUNK {
            self.id = self.spill.next[self.id as usize];
            self.at = 0;
        }
        let s = if self.id < self.in_out {
            let at = self.id as usize * BYTES + 2 * self.at;
            &self.out[at..at + 2]
        } else {
            &self.spill.ovf[(self.id - self.in_out) as usize][2 * self.at..2 * self.at + 2]
        };
        u16::from_le_bytes([s[0], s[1]])
    }

    #[inline(always)]
    pub(super) fn wire_xor_in_epoch(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        let hi = self.epoch << 16;
        if self.at + span <= CHUNK && self.id < self.in_out {
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            if (mark & !0xFFFF) != hi {
                let mut i = 0usize;
                while i + 4 <= span {
                    let b = &bytes[2 * i..2 * i + 8];
                    let v0 = hi | u32::from(u16::from_le_bytes([b[0], b[1]]));
                    let v1 = hi | u32::from(u16::from_le_bytes([b[2], b[3]]));
                    let v2 = hi | u32::from(u16::from_le_bytes([b[4], b[5]]));
                    let v3 = hi | u32::from(u16::from_le_bytes([b[6], b[7]]));
                    let u0 = dst_start + i;
                    edges[u0] ^= pack | v0;
                    edges[u0 + 1] ^= pack | v1;
                    edges[u0 + 2] ^= pack | v2;
                    edges[u0 + 3] ^= pack | v3;
                    edges[v0 as usize] ^= u0 as u32;
                    edges[v1 as usize] ^= (u0 + 1) as u32;
                    edges[v2 as usize] ^= (u0 + 2) as u32;
                    edges[v3 as usize] ^= (u0 + 3) as u32;
                    i += 4;
                }
                while i < span {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
            } else {
                let mut jn = *j_neg1;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark { jn = u as u32; }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *j_neg1 = jn;
            }
            self.at += span;
            self.read += span as u32;
        } else {
            self.wire_xor_in_epoch_slow(edges, dst_start, span, pack, mark, j_neg1);
        }
    }

    #[cold]
    #[inline(never)]
    fn wire_xor_in_epoch_slow(
        &mut self,
        edges: &mut [u32],
        mut dst_start: usize,
        mut left: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        let hi = self.epoch << 16;
        while left > 0 {
            if self.at == CHUNK {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let take = left.min(CHUNK - self.at);
            let bytes = if self.id < self.in_out {
                let at = self.id as usize * BYTES + 2 * self.at;
                &self.out[at..at + 2 * take]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][2 * self.at..2 * (self.at + take)]
            };
            let mut jn = *j_neg1;
            for (i, s) in bytes.chunks_exact(2).enumerate() {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                if v == mark { jn = u as u32; }
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
            }
            *j_neg1 = jn;
            self.at += take;
            self.read += take as u32;
            dst_start += take;
            left -= take;
        }
    }

    #[inline(always)]
    pub(super) fn wire_xor_subtile(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        max_span: usize,
        lo_bound: u32,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) -> usize {
        let hi = self.epoch << 16;
        let mut done = 0usize;
        let mut left = max_span;
        let mut dst = dst_start;
        let check_mark = (mark & !0xFFFF) == hi;
        while left > 0 {
            if self.at == CHUNK {
                self.id = self.spill.next[self.id as usize];
                self.at = 0;
            }
            let avail = left.min(CHUNK - self.at);
            let bytes = if self.id < self.in_out {
                let at = self.id as usize * BYTES + 2 * self.at;
                &self.out[at..at + 2 * avail]
            } else {
                &self.spill.ovf[(self.id - self.in_out) as usize][2 * self.at..2 * (self.at + avail)]
            };
            let last_lo = u32::from(u16::from_le_bytes([bytes[2 * avail - 2], bytes[2 * avail - 1]]));
            if last_lo < lo_bound {
                if !check_mark {
                    let mut i = 0usize;
                    while i + 4 <= avail {
                        let b = &bytes[2 * i..2 * i + 8];
                        let v0 = hi | u32::from(u16::from_le_bytes([b[0], b[1]]));
                        let v1 = hi | u32::from(u16::from_le_bytes([b[2], b[3]]));
                        let v2 = hi | u32::from(u16::from_le_bytes([b[4], b[5]]));
                        let v3 = hi | u32::from(u16::from_le_bytes([b[6], b[7]]));
                        let u0 = dst + i;
                        edges[u0] ^= pack | v0;
                        edges[u0 + 1] ^= pack | v1;
                        edges[u0 + 2] ^= pack | v2;
                        edges[u0 + 3] ^= pack | v3;
                        edges[v0 as usize] ^= u0 as u32;
                        edges[v1 as usize] ^= (u0 + 1) as u32;
                        edges[v2 as usize] ^= (u0 + 2) as u32;
                        edges[v3 as usize] ^= (u0 + 3) as u32;
                        i += 4;
                    }
                    while i < avail {
                        let u = dst + i;
                        let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                        edges[u] ^= pack | v;
                        edges[v as usize] ^= u as u32;
                        i += 1;
                    }
                } else {
                    let mut jn = *j_neg1;
                    for (i, s) in bytes.chunks_exact(2).enumerate() {
                        let u = dst + i;
                        let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                        if v == mark { jn = u as u32; }
                        edges[u] ^= pack | v;
                        edges[v as usize] ^= u as u32;
                    }
                    *j_neg1 = jn;
                }
                self.at += avail;
                self.read += avail as u32;
                done += avail;
                dst += avail;
                left -= avail;
            } else {
                let mut i = 0usize;
                let mut jn = *j_neg1;
                while i < avail {
                    let lo = u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                    if lo >= lo_bound {
                        break;
                    }
                    let u = dst + i;
                    let v = hi | lo;
                    if check_mark && v == mark { jn = u as u32; }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    i += 1;
                }
                *j_neg1 = jn;
                self.at += i;
                self.read += i as u32;
                done += i;
                break;
            }
        }
        done
    }
"""

# Insert extra_oq into Reader impl in outq.rs
marker_oq = "    /// Wire XOR-bijective doubly-linked permutation edges:"
assert marker_oq in bi_oq
ep_oq = bi_oq.replace(marker_oq, extra_oq + "\n" + marker_oq)
# Expose spill.cuts pub(super)
ep_oq = ep_oq.replace("    cuts: Vec<[u32; 256]>,", "    pub(super) cuts: Vec<[u32; 256]>,")

# Patch `cand_epoch64` mod.rs: Epoch-Major 64K-row (256 KiB L2) transposed `wire_xor`!
old_xor_q = """            if QUEUE {
                let mut k = 0;
                let edges_m = &mut self.edges[..m as usize];
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                    for &b in &self.bands[cursors[a]..end] {
                        let bend = b.end();
                        let span = (bend - k) as usize;
                        let dst = b.dst() as usize;
                        let pack = b.code() << bits;
                        queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);
                        k = bend;
                    }
                }
            } else {"""

new_epoch64_q = """            if QUEUE {
                let (spill, bands, edges_m) = (&self.spill, &self.bands, &mut self.edges[..m as usize]);
                let mut active = [0u8; 256];
                let mut n_act = 0usize;
                let mut b_idx = [0usize; 256];
                let mut b_k = [0u32; 256];
                let mut b_rem = [0u32; 256];
                let mut b_dst = [0usize; 256];
                let mut b_pack = [0u32; 256];
                let mut k_acc = 0u32;
                for a in 0..256 {
                    if thin[a] > 0 {
                        active[n_act] = a as u8;
                        n_act += 1;
                        b_idx[a] = cursors[a];
                        b_k[a] = k_acc;
                    }
                    k_acc += thin[a];
                }
                let mut readers: [outq::Reader<'_>; 256] =
                    std::array::from_fn(|a| outq::Reader::new(out, spill, heads, a));
                let num_epochs = spill.cuts.len() + 1;
                for ep in 0..num_epochs {
                    let next_cut_arr = spill.cuts.get(ep);
                    for &au in &active[..n_act] {
                        let a = usize::from(au);
                        let target = next_cut_arr.map_or(thin[a], |c| c[a]);
                        let q = &mut readers[a];
                        q.set_epoch(ep as u32, target);
                        let mut todo = target - q.read_count();
                        let mut bi = b_idx[a];
                        let mut bk = b_k[a];
                        let mut rem = b_rem[a];
                        let mut dst = b_dst[a];
                        let mut pack = b_pack[a];
                        while todo > 0 {
                            if rem == 0 {
                                let b = bands[bi];
                                bi += 1;
                                let bend = b.end();
                                rem = bend - bk;
                                bk = bend;
                                dst = b.dst() as usize;
                                pack = b.code() << bits;
                            }
                            let step = todo.min(rem);
                            q.wire_xor_in_epoch(edges_m, dst, step as usize, pack, mark, &mut j_neg1);
                            dst += step as usize;
                            rem -= step;
                            todo -= step;
                        }
                        b_idx[a] = bi;
                        b_k[a] = bk;
                        b_rem[a] = rem;
                        b_dst[a] = dst;
                        b_pack[a] = pack;
                    }
                }
            } else {"""

assert old_xor_q in bi_mod
e64_mod = bi_mod.replace(old_xor_q, new_epoch64_q)

d_e64 = Path("/tmp/cks_bench/cand_epoch64")
d_e64.mkdir(parents=True, exist_ok=True)
(d_e64 / "mod.rs").write_text(e64_mod)
(d_e64 / "endpoint.rs").write_text(bi_ep)
(d_e64 / "outq.rs").write_text(ep_oq)

# 3. `cand_subtile32`: 32K-row (128 KiB L2 half-tile) transposed `wire_xor`!
new_sub32_q = """            if QUEUE {
                let (spill, bands, edges_m) = (&self.spill, &self.bands, &mut self.edges[..m as usize]);
                let mut active = [0u8; 256];
                let mut n_act = 0usize;
                let mut b_idx = [0usize; 256];
                let mut b_k = [0u32; 256];
                let mut b_rem = [0u32; 256];
                let mut b_dst = [0usize; 256];
                let mut b_pack = [0u32; 256];
                let mut k_acc = 0u32;
                for a in 0..256 {
                    if thin[a] > 0 {
                        active[n_act] = a as u8;
                        n_act += 1;
                        b_idx[a] = cursors[a];
                        b_k[a] = k_acc;
                    }
                    k_acc += thin[a];
                }
                let mut readers: [outq::Reader<'_>; 256] =
                    std::array::from_fn(|a| outq::Reader::new(out, spill, heads, a));
                let num_epochs = spill.cuts.len() + 1;
                for ep in 0..num_epochs {
                    let next_cut_arr = spill.cuts.get(ep);
                    for &lo_bound in &[32_768u32, 65_536u32] {
                        for &au in &active[..n_act] {
                            let a = usize::from(au);
                            let target = next_cut_arr.map_or(thin[a], |c| c[a]);
                            let q = &mut readers[a];
                            q.set_epoch(ep as u32, target);
                            let mut todo = target - q.read_count();
                            if todo == 0 {
                                continue;
                            }
                            if lo_bound == 32_768 && u32::from(q.peek_lo()) >= 32_768 {
                                continue;
                            }
                            let mut bi = b_idx[a];
                            let mut bk = b_k[a];
                            let mut rem = b_rem[a];
                            let mut dst = b_dst[a];
                            let mut pack = b_pack[a];
                            while todo > 0 {
                                if rem == 0 {
                                    let b = bands[bi];
                                    bi += 1;
                                    let bend = b.end();
                                    rem = bend - bk;
                                    bk = bend;
                                    dst = b.dst() as usize;
                                    pack = b.code() << bits;
                                }
                                let step = todo.min(rem) as usize;
                                let wired = if lo_bound == 65_536 {
                                    q.wire_xor_in_epoch(edges_m, dst, step, pack, mark, &mut j_neg1);
                                    step
                                } else {
                                    q.wire_xor_subtile(edges_m, dst, step, lo_bound, pack, mark, &mut j_neg1)
                                };
                                dst += wired;
                                rem -= wired as u32;
                                todo -= wired as u32;
                                if wired < step {
                                    break;
                                }
                            }
                            b_idx[a] = bi;
                            b_k[a] = bk;
                            b_rem[a] = rem;
                            b_dst[a] = dst;
                            b_pack[a] = pack;
                        }
                    }
                }
            } else {"""

s32_mod = bi_mod.replace(old_xor_q, new_sub32_q)
d_s32 = Path("/tmp/cks_bench/cand_subtile32")
d_s32.mkdir(parents=True, exist_ok=True)
(d_s32 / "mod.rs").write_text(s32_mod)
(d_s32 / "endpoint.rs").write_text(bi_ep)
(d_s32 / "outq.rs").write_text(ep_oq)

# 4. `cand_noqueue`: QUEUE = false sequential-`v` `records` `wire_xor` (0 B queues)!
nq_mod = bi_mod.replace(
    "self.try_run::<true, true>(records, n, p, LIMIT, cap, out)",
    "self.try_run::<true, false>(records, n, p, LIMIT, cap, out)",
).replace(
    "self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out)",
    "self.construct_capped::<true, false>(records, n as u32, p, LIMIT, cap, out)",
)
# Also skip reserving `out` in `begin` when QUEUE=false? Wait: `begin` only allocates `out` if `out.len() < n`, which is fine (`out` is caller's buffer!).
# Wait: `outq::Queues::new` calls `spill.reset(...)` which reserves `next`! Let's check if `QUEUE=false` skips `queues` completely!
# Even cleaner: in `cand_noqueue`, add fast 6-phase profiler to `cand_cks_bi` too!
d_nq = Path("/tmp/cks_bench/cand_noqueue")
d_nq.mkdir(parents=True, exist_ok=True)
(d_nq / "mod.rs").write_text(nq_mod)
(d_nq / "endpoint.rs").write_text(bi_ep)
(d_nq / "outq.rs").write_text(bi_oq)

print("Generated master_a1, master_a2, cand_epoch64, cand_subtile32, cand_noqueue successfully.")
