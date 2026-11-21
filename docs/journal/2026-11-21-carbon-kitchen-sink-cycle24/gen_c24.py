from pathlib import Path

# 1. `cand_xor_u8`: Unrolls `wire_xor`'s fast path by 8 (`chunks_exact(16)` = 8 u16 ranks at once)
#    and eliminates `jn` branch inside the 8-wide chunk whenever `(mark ^ hi)` is outside `[0..65536)`!
d_u8 = Path("/tmp/cks_bench/cand_xor_u8")
d_u8.mkdir(parents=True, exist_ok=True)
(d_u8 / "mod.rs").write_text(Path("/tmp/cks_bench/cand_xor/mod.rs").read_text())
(d_u8 / "endpoint.rs").write_text(Path("/tmp/cks_bench/cand_xor/endpoint.rs").read_text())

oq_src = Path("/tmp/cks_bench/cand_xor/outq.rs").read_text()
old_wx = """        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
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
            self.read += span as u32;"""

new_wx = """        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            if (mark & !0xFFFF) != hi {
                // Epoch mismatch: `v == mark` is IMPOSSIBLE in this entire chunk! Zero `mark` branches + 4x unrolled!
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
            self.read += span as u32;"""

assert old_wx in oq_src
(d_u8 / "outq.rs").write_text(oq_src.replace(old_wx, new_wx))

# 2. `cand_xor4`: 4-Cursor Bidirectional Meet-in-the-Middle (`mu = 4`, 2 punctures `{mark, q0 = (mark + m/2) % m}`)
#    Confronts Theorem 24.1 natively: measures whether `mu = 4` beats `mu = 2` or loses to the mandatory `n`-scale staging buffer (`+900 KiB` WS) + `memcpy`!
d_x4 = Path("/tmp/cks_bench/cand_xor4")
d_x4.mkdir(parents=True, exist_ok=True)
(d_x4 / "endpoint.rs").write_text(Path("/tmp/cks_bench/cand_xor/endpoint.rs").read_text())

oq4 = oq_src.replace(
    "        mark: u32,\n        j_neg1: &mut u32,",
    "        mark: u32,\n        j_neg1: &mut u32,\n        q0: u32,\n        q_neg1: &mut u32,"
).replace(
    "self.wire_xor_slow(edges, dst_start, span, pack, mark, j_neg1);",
    "self.wire_xor_slow(edges, dst_start, span, pack, mark, j_neg1, q0, q_neg1);"
).replace(
    "                if v == mark {\n                    *j_neg1 = u as u32;\n                }",
    "                if v == mark { *j_neg1 = u as u32; }\n                if v == q0 { *q_neg1 = u as u32; }"
)
(d_x4 / "outq.rs").write_text(oq4)

m4 = Path("/tmp/cks_bench/cand_xor/mod.rs").read_text()
m4 = m4.replace(
    "    lens: Vec<u8>,\n}",
    "    lens: Vec<u8>,\n    stage_buf: Vec<u8>,\n}"
).replace(
    "        let lens = self.lens.capacity();",
    "        let lens = self.lens.capacity() + self.stage_buf.capacity();"
).replace(
    "    ) -> Option<(Counts, u32, u32, u32)> {",
    "    ) -> Option<(Counts, u32, u32, u32, u32, u32)> {"
).replace(
    "    ) -> (Counts, u32, u32, u32) {",
    "    ) -> (Counts, u32, u32, u32, u32, u32) {"
).replace(
    "        let mut j_neg1 = mark;",
    "        let q0 = if m >= 4 { (mark + m / 2) % m } else { mark };\n        let mut j_neg1 = mark;\n        let mut q_neg1 = q0;"
).replace(
    "queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);",
    "queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1, q0, &mut q_neg1);"
).replace(
    "                        if v == mark {\n                            j_neg1 = u as u32;\n                        }",
    "                        if v == mark { j_neg1 = u as u32; }\n                        if v == q0 { q_neg1 = u as u32; }"
).replace(
    "        (stats, mark, j_neg1, bits)",
    "        (stats, mark, j_neg1, q0, q_neg1, bits)"
).replace(
    "        let (stats, mark, j_neg1, bits) =\n            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;\n        self.walk(n, stats.retained, mark, j_neg1, bits, out);",
    "        let (stats, mark, j_neg1, q0, q_neg1, bits) =\n            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;\n        self.walk(n, stats.retained, mark, j_neg1, q0, q_neg1, bits, out);"
).replace(
    "        let (stats, mark, j_neg1, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();\n        let tc = t0.elapsed().as_secs_f64() * 1e6;\n        let t1 = std::time::Instant::now();\n        self.walk(n, stats.retained, mark, j_neg1, bits, out);",
    "        let (stats, mark, j_neg1, q0, q_neg1, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();\n        let tc = t0.elapsed().as_secs_f64() * 1e6;\n        let t1 = std::time::Instant::now();\n        self.walk(n, stats.retained, mark, j_neg1, q0, q_neg1, bits, out);"
).replace(
    "    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, bits: u32, out: &mut Vec<u8>) {",
    "    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, q0: u32, q_neg1: u32, bits: u32, out: &mut Vec<u8>) {"
).replace(
    "        emit_xor_bi(\n            &self.edges[..m],\n            &self.extra,\n            &self.spelling,\n            &self.offsets,\n            &self.inline16,\n            &self.lens,\n            n,\n            mark,\n            j_neg1,\n            bits,\n            out,\n        );",
    "        emit_xor_bi4(\n            &self.edges[..m],\n            &self.extra,\n            &self.spelling,\n            &self.offsets,\n            &self.inline16,\n            &self.lens,\n            n,\n            mark,\n            j_neg1,\n            q0,\n            q_neg1,\n            bits,\n            out,\n            &mut self.stage_buf,\n        );"
)

emit4_fn = """
#[allow(clippy::too_many_arguments)]
fn emit_xor_bi4(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    q0: u32,
    q_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
    stage: &mut Vec<u8>,
) {
    if edges.len() < 65_536 || !extra.is_empty() || q0 == mark || q0 == j_neg1 {
        emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
        return;
    }
    // Lemma 24.2: On any primitive block, len(F0) + len(B1) + len(F1) + len(B0) = n with len(F0), len(B0) >= 1,
    // so len(F1) + len(B1) <= n - 2 < n: F1 growing right from stage[0..] and B1 growing left from stage[..n]
    // fit inside a single n-byte staging buffer (+900 KiB WS) without ever crossing!
    out.resize(n + INLINE, 0);
    stage.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;

    // Arc A: F0 (from mark, prev=j_neg1, out[at_f0..]) <-> B1 (from q_neg1, next=q0, stage[..st_b1])
    let (mut pf0, mut cf0) = (j_neg1, mark);
    let (mut nb1, mut cb1) = (q0, q_neg1);
    let (mut at_f0, mut st_b1) = (0usize, n);
    let mut start_a = false;

    // Arc B: F1 (from q0, prev=q_neg1, stage[st_f1..]) <-> B0 (from j_neg1, next=mark, out[..at_b0])
    let (mut pf1, mut cf1) = (q_neg1, q0);
    let (mut nb0, mut cb0) = (mark, j_neg1);
    let (mut st_f1, mut at_b0) = (0usize, n);
    let mut start_b = false;

    let mut rem = edges.len();
    while rem > 0
        && cf0 != cb1 && (!start_a || cf0 != nb1)
        && cf1 != cb0 && (!start_b || cf1 != nb0)
        && at_f0 + 32 <= at_b0 && st_f1 + 32 <= st_b1
    {
        if start_a && (cf0 == cb0 || cf0 == nb0 || cf0 == mark) {
            // Obstruction B: Periodic block (d > 1) where q0 lies on a disjoint cycle from mark!
            emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
            return;
        }
        rem -= 1;
        start_a = true;
        start_b = true;
        let ef0 = edges[cf0 as usize];
        let eb1 = edges[cb1 as usize];
        let ef1 = edges[cf1 as usize];
        let eb0 = edges[cb0 as usize];

        let c_f0 = (ef0 >> bits) as usize;
        let c_b1 = (eb1 >> bits) as usize;
        let c_f1 = (ef1 >> bits) as usize;
        let c_b0 = (eb0 >> bits) as usize;

        let nf0 = (ef0 & mask) ^ pf0; pf0 = cf0; cf0 = nf0;
        let pb1 = (eb1 & mask) ^ nb1; nb1 = cb1; cb1 = pb1;
        let nf1 = (ef1 & mask) ^ pf1; pf1 = cf1; cf1 = nf1;
        let pb0 = (eb0 & mask) ^ nb0; nb0 = cb0; cb0 = pb0;

        // Emit F0 -> out[at_f0..]
        let l = lens[c_f0];
        if l != LONG {
            out[at_f0..at_f0 + INLINE].copy_from_slice(&inline16[c_f0]);
            at_f0 += usize::from(l);
        } else {
            let w = c_f0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b0 - at_f0);
            out[at_f0..at_f0 + take].copy_from_slice(&word[..take]);
            at_f0 += take;
        }
        // Emit B0 -> out[..at_b0]
        let l = lens[c_b0];
        if l != LONG {
            let lu = usize::from(l);
            if at_f0 + INLINE <= at_b0 {
                let raw = u128::from_le_bytes(inline16[c_b0]);
                let sh = if lu == 16 { raw } else { raw << ((16 - lu) * 8) };
                out[at_b0 - INLINE..at_b0].copy_from_slice(&sh.to_le_bytes());
                at_b0 -= lu;
            } else {
                let take = lu.min(at_b0 - at_f0);
                out[at_b0 - take..at_b0].copy_from_slice(&inline16[c_b0][lu - take..lu]);
                at_b0 -= take;
            }
        } else {
            let w = c_b0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b0 - at_f0);
            out[at_b0 - take..at_b0].copy_from_slice(&word[word.len() - take..]);
            at_b0 -= take;
        }
        // Emit F1 -> stage[st_f1..]
        let l = lens[c_f1];
        if l != LONG {
            stage[st_f1..st_f1 + INLINE].copy_from_slice(&inline16[c_f1]);
            st_f1 += usize::from(l);
        } else {
            let w = c_f1 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(st_b1 - st_f1);
            stage[st_f1..st_f1 + take].copy_from_slice(&word[..take]);
            st_f1 += take;
        }
        // Emit B1 -> stage[..st_b1]
        let l = lens[c_b1];
        if l != LONG {
            let lu = usize::from(l);
            if st_f1 + INLINE <= st_b1 {
                let raw = u128::from_le_bytes(inline16[c_b1]);
                let sh = if lu == 16 { raw } else { raw << ((16 - lu) * 8) };
                stage[st_b1 - INLINE..st_b1].copy_from_slice(&sh.to_le_bytes());
                st_b1 -= lu;
            } else {
                let take = lu.min(st_b1 - st_f1);
                stage[st_b1 - take..st_b1].copy_from_slice(&inline16[c_b1][lu - take..lu]);
                st_b1 -= take;
            }
        } else {
            let w = c_b1 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(st_b1 - st_f1);
            stage[st_b1 - take..st_b1].copy_from_slice(&word[word.len() - take..]);
            st_b1 -= take;
        }
    }

    // Finish remaining steps of Arc A (`F0` <-> `B1`)
    let mut rem_a = rem;
    while rem_a > 0 && cf0 != cb1 && (!start_a || cf0 != nb1) && at_f0 < at_b0 && st_f1 < st_b1 {
        if start_a && (cf0 == mark || cf0 == j_neg1) {
            emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
            return;
        }
        rem_a -= 1;
        start_a = true;
        let ef0 = edges[cf0 as usize];
        let eb1 = edges[cb1 as usize];
        let c_f0 = (ef0 >> bits) as usize;
        let c_b1 = (eb1 >> bits) as usize;
        let nf0 = (ef0 & mask) ^ pf0; pf0 = cf0; cf0 = nf0;
        let pb1 = (eb1 & mask) ^ nb1; nb1 = cb1; cb1 = pb1;

        let l = lens[c_f0];
        if l != LONG {
            let lu = usize::from(l).min(at_b0 - at_f0);
            out[at_f0..at_f0 + lu].copy_from_slice(&inline16[c_f0][..lu]);
            at_f0 += lu;
        } else {
            let w = c_f0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b0 - at_f0);
            out[at_f0..at_f0 + take].copy_from_slice(&word[..take]);
            at_f0 += take;
        }
        let l = lens[c_b1];
        if l != LONG {
            let lbu = usize::from(l);
            let take = lbu.min(st_b1 - st_f1);
            stage[st_b1 - take..st_b1].copy_from_slice(&inline16[c_b1][lbu - take..lbu]);
            st_b1 -= take;
        } else {
            let w = c_b1 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(st_b1 - st_f1);
            stage[st_b1 - take..st_b1].copy_from_slice(&word[word.len() - take..]);
            st_b1 -= take;
        }
    }
    if cf0 == cb1 && at_f0 < at_b0 {
        let c = (edges[cf0 as usize] >> bits) as usize;
        let l = lens[c];
        if l != LONG {
            let lu = usize::from(l).min(at_b0 - at_f0);
            out[at_f0..at_f0 + lu].copy_from_slice(&inline16[c][..lu]);
            at_f0 += lu;
        } else {
            let w = c - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b0 - at_f0);
            out[at_f0..at_f0 + take].copy_from_slice(&word[..take]);
            at_f0 += take;
        }
    } else if cf0 != nb1 {
        emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
        return;
    }

    // Finish remaining steps of Arc B (`F1` <-> `B0`)
    let mut rem_b = rem;
    while rem_b > 0 && cf1 != cb0 && (!start_b || cf1 != nb0) && at_f0 < at_b0 && st_f1 < st_b1 {
        if start_b && (cf1 == q0 || cb0 == mark) {
            emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
            return;
        }
        rem_b -= 1;
        start_b = true;
        let ef1 = edges[cf1 as usize];
        let eb0 = edges[cb0 as usize];
        let c_f1 = (ef1 >> bits) as usize;
        let c_b0 = (eb0 >> bits) as usize;
        let nf1 = (ef1 & mask) ^ pf1; pf1 = cf1; cf1 = nf1;
        let pb0 = (eb0 & mask) ^ nb0; nb0 = cb0; cb0 = pb0;

        let l = lens[c_f1];
        if l != LONG {
            let lu = usize::from(l).min(st_b1 - st_f1);
            stage[st_f1..st_f1 + lu].copy_from_slice(&inline16[c_f1][..lu]);
            st_f1 += lu;
        } else {
            let w = c_f1 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(st_b1 - st_f1);
            stage[st_f1..st_f1 + take].copy_from_slice(&word[..take]);
            st_f1 += take;
        }
        let l = lens[c_b0];
        if l != LONG {
            let lbu = usize::from(l);
            let take = lbu.min(at_b0 - at_f0);
            out[at_b0 - take..at_b0].copy_from_slice(&inline16[c_b0][lbu - take..lbu]);
            at_b0 -= take;
        } else {
            let w = c_b0 - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(at_b0 - at_f0);
            out[at_b0 - take..at_b0].copy_from_slice(&word[word.len() - take..]);
            at_b0 -= take;
        }
    }
    if cf1 == cb0 && st_f1 < st_b1 {
        let c = (edges[cf1 as usize] >> bits) as usize;
        let l = lens[c];
        if l != LONG {
            let lu = usize::from(l).min(st_b1 - st_f1);
            stage[st_f1..st_f1 + lu].copy_from_slice(&inline16[c][..lu]);
            st_f1 += lu;
        } else {
            let w = c - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];
            let take = word.len().min(st_b1 - st_f1);
            stage[st_f1..st_f1 + take].copy_from_slice(&word[..take]);
            st_f1 += take;
        }
    } else if cf1 != nb0 {
        emit_xor_bi(edges, extra, spelling, offsets, inline16, lens, n, mark, j_neg1, bits, out);
        return;
    }

    // Stitch `B1` (`stage[st_b1..n]`) and `F1` (`stage[0..st_f1]`) into `out[at_f0 .. at_b0]`!
    let b1_len = n - st_b1;
    let total_period = at_f0 + b1_len + st_f1 + (n - at_b0);
    if total_period != n {
        // Periodic block where q0 happened to lie on mark's cycle (spelled period < n)!
        let right_len = n - at_b0;
        out[at_f0..at_f0 + b1_len].copy_from_slice(&stage[st_b1..n]);
        let mut p_at = at_f0 + b1_len;
        out[p_at..p_at + st_f1].copy_from_slice(&stage[0..st_f1]);
        p_at += st_f1;
        if right_len > 0 {
            out.copy_within(at_b0..n, p_at);
            p_at += right_len;
        }
        out.truncate(p_at);
        repeat(n, out);
        return;
    }
    out[at_f0..at_f0 + b1_len].copy_from_slice(&stage[st_b1..n]);
    at_f0 += b1_len;
    out[at_f0..at_f0 + st_f1].copy_from_slice(&stage[0..st_f1]);
    at_f0 += st_f1;
    debug_assert_eq!(at_f0, at_b0);
    out.truncate(n);
}
"""

m4 = m4.replace("fn emit_xor_bi(", emit4_fn + "\nfn emit_xor_bi(")
(d_x4 / "mod.rs").write_text(m4)
print("Generated cand_xor_u8 and cand_xor4 successfully.")
