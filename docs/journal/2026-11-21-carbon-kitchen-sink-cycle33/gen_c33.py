import os, shutil

ROOT = "/tmp/cks_bench"
BASE = f"{ROOT}/cand_links_only"

def clone_base(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def patch_wire(dst, unroll=8, with_q=False):
    # 1. Patch mod.rs: add mark_byte to First, record on mark hit, pass to wire_xor
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()

    old_mark = """            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }"""
    new_mark = """            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }"""
    assert old_mark in s
    s = s.replace(old_mark, new_mark, 1)

    old_first = """struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    runs: usize,
}"""
    new_first = """struct First {
    all: [u32; 256],
    thin: [u32; 256],
    thick: [u32; 256],
    m: u32,
    mark: u32,
    mark_byte: u8,
    runs: usize,
}"""
    assert old_first in s
    s = s.replace(old_first, new_first, 1)

    old_def = """            m: 0,
            mark: 0,
            runs: 0,"""
    new_def = """            m: 0,
            mark: 0,
            mark_byte: 0,
            runs: 0,"""
    assert old_def in s
    s = s.replace(old_def, new_def, 1)

    old_destr = """        let First {
            mut all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;"""
    new_destr = """        let First {
            mut all,
            thin,
            thick,
            m,
            mark,
            mark_byte,
            runs,
        } = *f;"""
    assert old_destr in s
    s = s.replace(old_destr, new_destr, 1)

    if not with_q:
        old_wire_loop = """            if QUEUE {
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
            }"""
        new_wire_loop = """            if QUEUE {
                let mut k = 0;
                let edges_m = &mut self.edges[..m as usize];
                let mb = usize::from(mark_byte);
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                    if a != mb {
                        for &b in &self.bands[cursors[a]..end] {
                            let bend = b.end();
                            let span = (bend - k) as usize;
                            let dst = b.dst() as usize;
                            let pack = b.code() << bits;
                            queue.wire_xor_nomark(edges_m, dst, span, pack);
                            k = bend;
                        }
                    } else {
                        for &b in &self.bands[cursors[a]..end] {
                            let bend = b.end();
                            let span = (bend - k) as usize;
                            let dst = b.dst() as usize;
                            let pack = b.code() << bits;
                            queue.wire_xor_mark(edges_m, dst, span, pack, mark, &mut j_neg1);
                            k = bend;
                        }
                    }
                }
            }"""
        assert old_wire_loop in s
        s = s.replace(old_wire_loop, new_wire_loop, 1)
    else:
        # QuadEmit: also pick second anchor (q, q_neg1) from a byte != mark_byte!
        # Update construct_capped / finish return tuple to include q and q_neg1
        s = s.replace("-> Option<(Counts, u32, u32, u32)>", "-> Option<(Counts, u32, u32, u32, u32, u32)>")
        s = s.replace("-> (Counts, u32, u32, u32)", "-> (Counts, u32, u32, u32, u32, u32)")
        old_wire_hdr = """        let mut j_neg1 = mark;
        if xor_mode {
            self.edges[..m as usize].fill(0);
            if QUEUE {
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
            }"""
        new_wire_hdr = """        let mut j_neg1 = mark;
        let mut q_anchor = u32::MAX;
        let mut q_neg1 = u32::MAX;
        if xor_mode {
            self.edges[..m as usize].fill(0);
            if QUEUE {
                let mb = usize::from(mark_byte);
                // Pick a deterministic second anchor `q` in O(1) from a byte `qb != mb` with thin[qb] > 0:
                let mut qb = 256usize;
                let mut best_thin = 0u32;
                for a in 0..256 {
                    if a != mb && thin[a] > best_thin {
                        best_thin = thin[a];
                        qb = a;
                    }
                }
                if qb < 256 {
                    q_anchor = outq::Reader::peek_first(out, &self.spill, heads, qb);
                }
                let mut k = 0;
                let edges_m = &mut self.edges[..m as usize];
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                    if a != mb && a != qb {
                        for &b in &self.bands[cursors[a]..end] {
                            let bend = b.end();
                            let span = (bend - k) as usize;
                            let dst = b.dst() as usize;
                            let pack = b.code() << bits;
                            queue.wire_xor_nomark(edges_m, dst, span, pack);
                            k = bend;
                        }
                    } else if a == mb {
                        for &b in &self.bands[cursors[a]..end] {
                            let bend = b.end();
                            let span = (bend - k) as usize;
                            let dst = b.dst() as usize;
                            let pack = b.code() << bits;
                            queue.wire_xor_mark(edges_m, dst, span, pack, mark, &mut j_neg1);
                            k = bend;
                        }
                    } else {
                        for &b in &self.bands[cursors[a]..end] {
                            let bend = b.end();
                            let span = (bend - k) as usize;
                            let dst = b.dst() as usize;
                            let pack = b.code() << bits;
                            queue.wire_xor_mark(edges_m, dst, span, pack, q_anchor, &mut q_neg1);
                            k = bend;
                        }
                    }
                }
            }"""
        assert old_wire_hdr in s
        s = s.replace(old_wire_hdr, new_wire_hdr, 1)
        s = s.replace("(stats, mark, j_neg1, bits)", "(stats, mark, j_neg1, q_anchor, q_neg1, bits)")
        s = s.replace(
            "let (stats, mark, j_neg1, bits) =",
            "let (stats, mark, j_neg1, q_anchor, q_neg1, bits) =",
        )
        s = s.replace(
            "self.walk(n, stats.retained, mark, j_neg1, bits, out);",
            "self.walk(n, stats.retained, mark, j_neg1, q_anchor, q_neg1, bits, out);",
        )
        old_walk = """    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, bits: u32, out: &mut Vec<u8>) {
        if m >= 65_536 && self.extra.is_empty() {
            emit_xor_bi_no_off(
                &self.edges[..m],
                &self.spelling,
                &self.inline16,
                &self.lens,
                n,
                mark,
                j_neg1,
                bits,
                out,
            );
        } else {"""
        new_walk = """    #[allow(clippy::too_many_arguments)]
    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, q_anchor: u32, q_neg1: u32, bits: u32, out: &mut Vec<u8>) {
        if m >= 65_536 && self.extra.is_empty() {
            if q_anchor < m as u32 && q_neg1 < m as u32 && q_anchor != mark && q_anchor != j_neg1 && q_neg1 != mark && n >= 256 {
                emit_xor_quad_no_off(
                    &self.edges[..m],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    j_neg1,
                    q_anchor,
                    q_neg1,
                    bits,
                    out,
                );
            } else {
                emit_xor_bi_no_off(
                    &self.edges[..m],
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    mark,
                    j_neg1,
                    bits,
                    out,
                );
            }
        } else {"""
        assert old_walk in s
        s = s.replace(old_walk, new_walk, 1)

        quad_fn = """
#[allow(clippy::too_many_arguments)]
fn emit_xor_bi_from(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mut prev_f: u32,
    mut cur_f: u32,
    mut next_b: u32,
    mut cur_b: u32,
    mut at_f: usize,
    mut at_b: usize,
    mut started: bool,
    mut rem_pairs: usize,
    bits: u32,
    out: &mut Vec<u8>,
) {
    let mask = (1u32 << bits) - 1;
    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f + 32 <= at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];

        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;

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
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            if at_f + INLINE <= at_b {
                let raw = u128::from_le_bytes(inline16[cb]);
                let shifted = if lbu == 16 { raw } else { raw << ((16 - lbu) * 8) };
                out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                at_b -= lbu;
            } else {
                let take = lbu.min(at_b - at_f);
                out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                at_b -= take;
            }
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f < at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;
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
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
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
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    if cur_f == cur_b && at_f < at_b {
        let cf = (edges[cur_f as usize] >> bits) as usize;
        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }

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

#[allow(clippy::too_many_arguments)]
fn emit_xor_quad_no_off(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    q: u32,
    q_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mid = n / 2;

    let mut prev_f0 = j_neg1;
    let mut cur_f0 = mark;
    let mut next_b0 = mark;
    let mut cur_b0 = j_neg1;

    let mut prev_f1 = q_neg1;
    let mut cur_f1 = q;
    let mut next_b1 = q;
    let mut cur_b1 = q_neg1;

    let mut at_f0 = 0usize;
    let mut at_b1 = mid;
    let mut at_f1 = mid;
    let mut at_b0 = n;

    let mut rem_quads = edges.len() / 4 + 1;
    let mut started = false;

    while rem_quads > 0
        && cur_f0 != cur_b1 && cur_f0 != next_b1
        && cur_f1 != cur_b0 && cur_f1 != next_b0
        && cur_f0 != cur_b0 && (!started || cur_f0 != next_b0)
        && at_f0 + 32 <= at_b1
        && at_f1 + 32 <= at_b0
    {
        let ef0 = edges[cur_f0 as usize];
        let eb0 = edges[cur_b0 as usize];
        let ef1 = edges[cur_f1 as usize];
        let eb1 = edges[cur_b1 as usize];

        let cf0 = (ef0 >> bits) as usize;
        let cb0 = (eb0 >> bits) as usize;
        let cf1 = (ef1 >> bits) as usize;
        let cb1 = (eb1 >> bits) as usize;

        let lf0 = lens[cf0];
        let lb0 = lens[cb0];
        let lf1 = lens[cf1];
        let lb1 = lens[cb1];

        if (lf0 != LONG) & (lb0 != LONG) & (lf1 != LONG) & (lb1 != LONG) {
            started = true;
            rem_quads -= 1;

            let nf0 = (ef0 & mask) ^ prev_f0;
            let pb0 = (eb0 & mask) ^ next_b0;
            let nf1 = (ef1 & mask) ^ prev_f1;
            let pb1 = (eb1 & mask) ^ next_b1;
            prev_f0 = cur_f0; cur_f0 = nf0;
            next_b0 = cur_b0; cur_b0 = pb0;
            prev_f1 = cur_f1; cur_f1 = nf1;
            next_b1 = cur_b1; cur_b1 = pb1;

            out[at_f0..at_f0 + INLINE].copy_from_slice(&inline16[cf0]);
            at_f0 += usize::from(lf0);

            let lb0u = usize::from(lb0);
            let r0 = u128::from_le_bytes(inline16[cb0]);
            let s0 = if lb0u == 16 { r0 } else { r0 << ((16 - lb0u) * 8) };
            out[at_b0 - INLINE..at_b0].copy_from_slice(&s0.to_le_bytes());
            at_b0 -= lb0u;

            out[at_f1..at_f1 + INLINE].copy_from_slice(&inline16[cf1]);
            at_f1 += usize::from(lf1);

            let lb1u = usize::from(lb1);
            let r1 = u128::from_le_bytes(inline16[cb1]);
            let s1 = if lb1u == 16 { r1 } else { r1 << ((16 - lb1u) * 8) };
            out[at_b1 - INLINE..at_b1].copy_from_slice(&s1.to_le_bytes());
            at_b1 -= lb1u;
        } else {
            // Cold path: at least one of the 4 words is LONG (>16 bytes).
            // Pre-check that all 4 words fit cleanly in their gaps; if not, break to lossless 2-cursor fallback!
            let len_of = |c: usize, l: u8| -> usize {
                if l != LONG {
                    usize::from(l)
                } else {
                    let meta = &inline16[c];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    e - s
                }
            };
            let w_f0 = len_of(cf0, lf0);
            let w_b0 = len_of(cb0, lb0);
            let w_f1 = len_of(cf1, lf1);
            let w_b1 = len_of(cb1, lb1);
            if at_f0 + w_f0 + w_b1 > at_b1 || at_f1 + w_f1 + w_b0 > at_b0 {
                break;
            }
            started = true;
            rem_quads -= 1;

            let nf0 = (ef0 & mask) ^ prev_f0;
            let pb0 = (eb0 & mask) ^ next_b0;
            let nf1 = (ef1 & mask) ^ prev_f1;
            let pb1 = (eb1 & mask) ^ next_b1;
            prev_f0 = cur_f0; cur_f0 = nf0;
            next_b0 = cur_b0; cur_b0 = pb0;
            prev_f1 = cur_f1; cur_f1 = nf1;
            next_b1 = cur_b1; cur_b1 = pb1;

            if lf0 != LONG {
                out[at_f0..at_f0 + w_f0].copy_from_slice(&inline16[cf0][..w_f0]);
            } else {
                let meta = &inline16[cf0];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                out[at_f0..at_f0 + w_f0].copy_from_slice(&spelling[s..s + w_f0]);
            }
            at_f0 += w_f0;

            if lb0 != LONG {
                out[at_b0 - w_b0..at_b0].copy_from_slice(&inline16[cb0][..w_b0]);
            } else {
                let meta = &inline16[cb0];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                out[at_b0 - w_b0..at_b0].copy_from_slice(&spelling[s..s + w_b0]);
            }
            at_b0 -= w_b0;

            if lf1 != LONG {
                out[at_f1..at_f1 + w_f1].copy_from_slice(&inline16[cf1][..w_f1]);
            } else {
                let meta = &inline16[cf1];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                out[at_f1..at_f1 + w_f1].copy_from_slice(&spelling[s..s + w_f1]);
            }
            at_f1 += w_f1;

            if lb1 != LONG {
                out[at_b1 - w_b1..at_b1].copy_from_slice(&inline16[cb1][..w_b1]);
            } else {
                let meta = &inline16[cb1];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                out[at_b1 - w_b1..at_b1].copy_from_slice(&spelling[s..s + w_b1]);
            }
            at_b1 -= w_b1;
        }
    }

    let rem_bi = edges.len() / 2 + 1;
    if (cur_f0 == cur_b1 || cur_f0 == next_b1) && !(cur_f0 == cur_b0 || (started && cur_f0 == next_b0)) {
        // Arc 1 [mark, q) met! Verify optional middle word fits before stitching.
        let mut ok = true;
        if cur_f0 == cur_b1 {
            let cf = (edges[cur_f0 as usize] >> bits) as usize;
            let lf = lens[cf];
            if lf != LONG {
                let w = usize::from(lf);
                if at_f0 + w <= at_b1 {
                    out[at_f0..at_f0 + w].copy_from_slice(&inline16[cf][..w]);
                    at_f0 += w;
                } else {
                    ok = false;
                }
            } else {
                let meta = &inline16[cf];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                let w = e - s;
                if at_f0 + w <= at_b1 {
                    out[at_f0..at_f0 + w].copy_from_slice(&spelling[s..e]);
                    at_f0 += w;
                } else {
                    ok = false;
                }
            }
        }
        if ok {
            let b1_len = mid - at_b1;
            if b1_len > 0 && at_f0 != at_b1 {
                out.copy_within(at_b1..mid, at_f0);
            }
            let u_len = at_f0 + b1_len;
            let f1_len = at_f1 - mid;
            if f1_len > 0 && u_len != mid {
                out.copy_within(mid..at_f1, u_len);
            }
            let new_f = u_len + f1_len;
            emit_xor_bi_from(
                edges, spelling, inline16, lens, n,
                prev_f1, cur_f1, next_b0, cur_b0,
                new_f, at_b0, true, rem_bi, bits, out,
            );
            return;
        }
    } else if (cur_f1 == cur_b0 || cur_f1 == next_b0) && !(cur_f0 == cur_b0 || (started && cur_f0 == next_b0)) {
        // Arc 2 [q, mark) met!
        let mut ok = true;
        if cur_f1 == cur_b0 {
            let cf = (edges[cur_f1 as usize] >> bits) as usize;
            let lf = lens[cf];
            if lf != LONG {
                let w = usize::from(lf);
                if at_f1 + w <= at_b0 {
                    out[at_f1..at_f1 + w].copy_from_slice(&inline16[cf][..w]);
                    at_f1 += w;
                } else {
                    ok = false;
                }
            } else {
                let meta = &inline16[cf];
                let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                let w = e - s;
                if at_f1 + w <= at_b0 {
                    out[at_f1..at_f1 + w].copy_from_slice(&spelling[s..e]);
                    at_f1 += w;
                } else {
                    ok = false;
                }
            }
        }
        if ok {
            let f1_len = at_f1 - mid;
            let new_b0 = at_b0 - f1_len;
            if f1_len > 0 && new_b0 != mid {
                out.copy_within(mid..at_f1, new_b0);
            }
            let b1_len = mid - at_b1;
            let new_b1 = new_b0 - b1_len;
            if b1_len > 0 && new_b1 != at_b1 {
                out.copy_within(at_b1..mid, new_b1);
            }
            emit_xor_bi_from(
                edges, spelling, inline16, lens, n,
                prev_f0, cur_f0, next_b1, cur_b1,
                at_f0, new_b1, true, rem_bi, bits, out,
            );
            return;
        }
    }

    // Fallback (periodic different cycle or buffer-skew): f0 and b0 are 100% consistent!
    emit_xor_bi_from(
        edges, spelling, inline16, lens, n,
        prev_f0, cur_f0, next_b0, cur_b0,
        at_f0, at_b0, started, rem_bi, bits, out,
    );
}
"""
        s = s.replace("fn emit_xor_bi_no_off(", quad_fn + "\nfn emit_xor_bi_no_off(", 1)

    with open(mod_p, "w") as f:
        f.write(s)

    # 2. Patch outq.rs: add wire_xor_nomark, wire_xor_mark, and peek_first
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()

    if unroll == 4:
        nomark_body = """
    #[inline(always)]
    pub(super) fn peek_first(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> u32 {
        let id = heads.heads[a];
        if id == NONE {
            return u32::MAX;
        }
        let mut epoch = 0u32;
        while let Some(c) = spill.cuts.get(epoch as usize) {
            if c[a] == 0 {
                epoch += 1;
            } else {
                break;
            }
        }
        let chunk: &[u8] = if id < heads.in_out {
            let at = id as usize * BYTES;
            &out[at..at + BYTES]
        } else {
            &spill.ovf[(id - heads.in_out) as usize][..]
        };
        (epoch << 16) | u32::from(u16::from_le_bytes([chunk[0], chunk[1]]))
    }

    #[inline(always)]
    pub(super) fn wire_xor_nomark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            let mut i = 0usize;
            while i + 4 <= span {
                let q = u64::from_le_bytes([
                    bytes[2 * i], bytes[2 * i + 1], bytes[2 * i + 2], bytes[2 * i + 3],
                    bytes[2 * i + 4], bytes[2 * i + 5], bytes[2 * i + 6], bytes[2 * i + 7],
                ]);
                let v0 = hi | (q as u32 & 0xFFFF);
                let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 4];
                eu[0] ^= pack | v0;
                eu[1] ^= pack | v1;
                eu[2] ^= pack | v2;
                eu[3] ^= pack | v3;
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
            self.at += span;
            self.read += span as u32;
        } else {
            let mut dummy = 0u32;
            self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
        }
    }

    #[inline(always)]
    pub(super) fn wire_xor_mark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        self.wire_xor(edges, dst_start, span, pack, mark, j_neg1);
    }
"""
    else:
        nomark_body = """
    #[inline(always)]
    pub(super) fn peek_first(out: &'a [u8], spill: &'a Spill, heads: &Heads, a: usize) -> u32 {
        let id = heads.heads[a];
        if id == NONE {
            return u32::MAX;
        }
        let mut epoch = 0u32;
        while let Some(c) = spill.cuts.get(epoch as usize) {
            if c[a] == 0 {
                epoch += 1;
            } else {
                break;
            }
        }
        let chunk: &[u8] = if id < heads.in_out {
            let at = id as usize * BYTES;
            &out[at..at + BYTES]
        } else {
            &spill.ovf[(id - heads.in_out) as usize][..]
        };
        (epoch << 16) | u32::from(u16::from_le_bytes([chunk[0], chunk[1]]))
    }

    #[inline(always)]
    pub(super) fn wire_xor_nomark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            let mut i = 0usize;
            while i + 8 <= span {
                let b = &bytes[2 * i..2 * i + 16];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let v0 = hi | (q0 as u32 & 0xFFFF);
                let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q0 >> 48) as u32);
                let v4 = hi | (q1 as u32 & 0xFFFF);
                let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                let v7 = hi | ((q1 >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 8];
                eu[0] ^= pack | v0;
                eu[1] ^= pack | v1;
                eu[2] ^= pack | v2;
                eu[3] ^= pack | v3;
                eu[4] ^= pack | v4;
                eu[5] ^= pack | v5;
                eu[6] ^= pack | v6;
                eu[7] ^= pack | v7;
                edges[v0 as usize] ^= u0 as u32;
                edges[v1 as usize] ^= (u0 + 1) as u32;
                edges[v2 as usize] ^= (u0 + 2) as u32;
                edges[v3 as usize] ^= (u0 + 3) as u32;
                edges[v4 as usize] ^= (u0 + 4) as u32;
                edges[v5 as usize] ^= (u0 + 5) as u32;
                edges[v6 as usize] ^= (u0 + 6) as u32;
                edges[v7 as usize] ^= (u0 + 7) as u32;
                i += 8;
            }
            if i + 4 <= span {
                let b = &bytes[2 * i..2 * i + 8];
                let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let v0 = hi | (q as u32 & 0xFFFF);
                let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 4];
                eu[0] ^= pack | v0;
                eu[1] ^= pack | v1;
                eu[2] ^= pack | v2;
                eu[3] ^= pack | v3;
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
            self.at += span;
            self.read += span as u32;
        } else {
            let mut dummy = 0u32;
            self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
        }
    }

    #[inline(always)]
    pub(super) fn wire_xor_mark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
        mark: u32,
        j_neg1: &mut u32,
    ) {
        self.wire_xor(edges, dst_start, span, pack, mark, j_neg1);
    }
"""
    anchor = "    #[inline(always)]\n    pub(super) fn wire_xor("
    assert anchor in o
    o = o.replace(anchor, nomark_body + "\n" + anchor, 1)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_dedup(dst, pair_bits=10, str_bits=12):
    ep = f"{dst}/endpoint.rs"
    with open(ep) as f:
        s = f.read()

    # 1. Fused short_word in join() and dedup
    old_join = """    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let ll = self.len(left);
        let rl = self.len(right);
        let len = ll + rl;
        if len <= 15 {
            // Operadic evaluation in truncated free monoid Sigma^{<=15}:
            let vl = self.short_u128(left);
            let vr = self.short_u128(right);
            let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
            self.a[self.top] = v as u32;
            self.a[self.top + 1] = (v >> 32) as u32;
            self.a[self.top + 2] = (v >> 64) as u32;
            self.a[self.top + 3] = (v >> 96) as u32;
        } else {
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        }
        self.top += RULE;
        id
    }"""

    new_join = """    /// Fused operad fetch: returns `(payload_u128, len)` when `len <= 15` (with `len` in `1..=15`),
    /// or `(0, actual_len)` when `len >= 16`. Single `w <= 256` check and single rule read!
    #[inline(always)]
    fn fetch_word(&self, w: u32) -> (u128, u32) {
        if w <= 256 {
            (u128::from(w - 1), 1)
        } else {
            let r = self.rule(w);
            let sl = &self.a[r..r + 4];
            let r3 = sl[3];
            if (r3 >> 28) == 0 {
                let v = u128::from(sl[0])
                    | (u128::from(sl[1]) << 32)
                    | (u128::from(sl[2]) << 64)
                    | (u128::from(r3 & 0x00FF_FFFF) << 96);
                (v, r3 >> 24)
            } else {
                (0, sl[2])
            }
        }
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (vl, ll) = self.fetch_word(left);
        let (vr, rl) = self.fetch_word(right);
        let len = ll + rl;
        if len <= 15 {
            let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
            let out = &mut self.a[self.top..self.top + 4];
            out[0] = v as u32;
            out[1] = (v >> 32) as u32;
            out[2] = (v >> 64) as u32;
            out[3] = (v >> 96) as u32;
        } else {
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        }
        self.top += RULE;
        id
    }"""
    assert old_join in s
    s = s.replace(old_join, new_join, 1)

    # 2. HT sizing + 1-mul hash_u128 + per-byte last_word cache + fused fetch_word in dedup
    old_ht = """        const PAIR_BITS: u32 = 10;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 2048 u32s (8 KiB)
        const STR_BITS: u32 = 12;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s (16 KiB)
        const HT_WORDS: usize = PAIR_LEN + STR_LEN; // 6144 u32s (24 KiB total, zero u_rec!)"""

    new_ht = f"""        const PAIR_BITS: u32 = {pair_bits};
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1);
        const STR_BITS: u32 = {str_bits};
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS;
        const HT_WORDS: usize = PAIR_LEN + STR_LEN;"""
    assert old_ht in s
    s = s.replace(old_ht, new_ht, 1)

    old_hash = """        #[inline(always)]
        fn hash_u128(v: u128, wlen: u32) -> u64 {
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ (u64::from(wlen)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^ (h >> 30)
        }"""
    new_hash = """        #[inline(always)]
        fn hash_u128(v: u128, wlen: u32) -> u64 {
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(wlen) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            h ^ (h >> 30)
        }"""
    assert old_hash in s
    s = s.replace(old_hash, new_hash, 1)

    old_dedup_loop = """        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    let pkey = (node_word << 8) | (a as u32);
                    let pslot = ((pkey.wrapping_mul(0x9E37_79B9)) >> (32 - PAIR_BITS)) as usize * 2;
                    if pair_ht[pslot] == pkey {
                        pair_ht[pslot + 1]
                    } else {
                        let h_len = e.len(node_word);
                        let found_code = if h_len <= 15 {
                            // ZERO-STACK, ZERO-SPELLING, DIRECT-INLINE16 REGISTER PATH (>93% of bands)!
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let v = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));"""

    new_dedup_loop = """        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let pkey = (node_word << 8) | (a as u32);
                    let pslot = ((pkey.wrapping_mul(0x9E37_79B9)) >> (32 - PAIR_BITS)) as usize * 2;
                    let found_c = if pair_ht[pslot] == pkey {
                        pair_ht[pslot + 1]
                    } else {
                        let (base_v, h_len) = e.fetch_word(node_word);
                        let found_code = if h_len <= 15 {
                            // ZERO-STACK, ZERO-SPELLING, DIRECT-INLINE16 REGISTER PATH (>93% of bands)!
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let v = base_v | (u128::from(a as u8) << (8 * h_len));"""
    assert old_dedup_loop in s
    s = s.replace(old_dedup_loop, new_dedup_loop, 1)

    old_dedup_end = """                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    }
                };"""
    new_dedup_end = """                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    };
                    last_w = node_word;
                    last_c = found_c;
                    found_c
                };"""
    assert old_dedup_end in s
    s = s.replace(old_dedup_end, new_dedup_end, 1)

    with open(ep, "w") as f:
        f.write(s)

# Build all Cycle 33 candidates:
d = clone_base("cand_c33_wire4");   patch_wire(d, unroll=4, with_q=False)
d = clone_base("cand_c33_wire8");   patch_wire(d, unroll=8, with_q=False)
d = clone_base("cand_c33_dedup24"); patch_dedup(d, pair_bits=10, str_bits=12)
d = clone_base("cand_c33_dedup12"); patch_dedup(d, pair_bits=9,  str_bits=11)
d = clone_base("cand_c33_w8_d24");  patch_wire(d, unroll=8, with_q=False); patch_dedup(d, pair_bits=10, str_bits=12)
d = clone_base("cand_c33_quad");    patch_wire(d, unroll=8, with_q=True);  patch_dedup(d, pair_bits=10, str_bits=12)

print("Generated 6 Cycle 33 candidates successfully!")
