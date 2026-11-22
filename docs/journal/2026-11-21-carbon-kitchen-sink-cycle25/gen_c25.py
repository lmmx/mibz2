from pathlib import Path

# Generate the two Cycle-25 Grand Synthesis Kernels:
# 1. `cand_cks_diet` (`CKS-DietOperad`, mu = 1 serial walk):
#    - Cycle 20 128-bit Truncated Free-Monoid Operad (`Operad-MN16`) with 28-bit memo offset
#    - Direct-`inline16` Zero-`u_rec` 1-probe Myhill-Nerode Quotient in dead `edges[0..4*ns]` / `edges[4*ns+etop..]`
#    - Cycle 21 `MuxStrips` (`strips` in `edges[0..3*ns]`, `strips = 0 B`) + `Band64` (`[end:20 | dst:20 | code:24]`)
#    - Zero `offsets` (`LONG` `(start, end)` in `inline16[code][0..8]`)
# 2. `cand_cks_bi` (`CKS-BiOperad`, mu = 2 L2-gated antipodal walk):
#    - Everything in `CKS-DietOperad` PLUS Cycle 17 + Cycle 24 L2-gated (`m >= 65_536 && !wide`)
#      `Unrolled4-Epoch-XOR` (`OutQ16::wire_xor` + `emit_xor_bi`, `+0 B` scratch, `0 B` staging)!

diet2_mod = Path("/tmp/cks_bench/cand_diet2/mod.rs").read_text()
diet2_ep = Path("/tmp/cks_bench/cand_diet2/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/cand_diet2/outq.rs").read_text()
u8_oq = Path("/tmp/cks_bench/cand_xor_u8/outq.rs").read_text()

# Upgrade `diet2_mod`: adaptive `cap_hint` for `inline16` + `lens` after Pass 1 admission
old_init_i16 = """        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.resize(256, 1);
        }"""

new_init_i16 = """        if self.inline16.len() < 256 {
            let cap_hint = if n >= 65_536 { 1792 } else { 256 };
            self.inline16.clear();
            self.inline16.reserve_exact(cap_hint);
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            self.lens.clear();
            self.lens.reserve_exact(cap_hint);
            self.lens.resize(256, 1);
        }"""

assert old_init_i16 in diet2_mod
cks_diet_mod = diet2_mod.replace(old_init_i16, new_init_i16)

# Upgrade `diet2_ep`:
# 1. 28-bit memo offset (`u32::MAX` sentinel, `(r3 >> 28) == 0` for short operad rules)
# 2. Direct-`inline16` Zero-`u_rec` 1-probe quotient placed in dead `edges[0..4*ns]` (when `4*ns >= 6144`)
#    or dead tail `edges[4*ns+etop..]` (when `edges.len() >= 4*ns + etop + 6144`)!
old_ep_len_append = """    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule(w);
            let tag = self.a[r + 3] >> 24;
            if tag < 16 {
                tag
            } else {
                self.a[r + 2]
            }
        }
    }"""

new_ep_len_append = """    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            let r = self.rule(w);
            let r3 = self.a[r + 3];
            if (r3 >> 28) == 0 {
                r3 >> 24
            } else {
                self.a[r + 2]
            }
        }
    }"""

assert old_ep_len_append in diet2_ep
cks_ep = diet2_ep.replace(old_ep_len_append, new_ep_len_append).replace(
    "self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, 0xFF00_0000]);",
    "self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);"
).replace(
    """                let tag = r3 >> 24;
                if tag < 16 {
                    // Stop-leaf! Emit up to 15 pre-assembled operad bytes in one slice copy!
                    let v = u128::from(self.a[r])
                        | (u128::from(self.a[r + 1]) << 32)
                        | (u128::from(self.a[r + 2]) << 64)
                        | (u128::from(r3 & 0x00FF_FFFF) << 96);
                    let bytes = v.to_le_bytes();
                    dest.extend_from_slice(&bytes[..tag as usize]);
                } else if r3 == 0xFF00_0000 {
                    self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                    let right = self.a[r + 1];
                    let left = self.a[r];
                    stack.push(right);
                    stack.push(left);
                } else {
                    let start = (r3 & 0x00FF_FFFF) as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }""",
    """                if (r3 >> 28) == 0 {
                    // Stop-leaf! Emit up to 15 pre-assembled operad bytes in one slice copy!
                    let tag = (r3 >> 24) as usize;
                    let v = u128::from(self.a[r])
                        | (u128::from(self.a[r + 1]) << 32)
                        | (u128::from(self.a[r + 2]) << 64)
                        | (u128::from(r3 & 0x00FF_FFFF) << 96);
                    let bytes = v.to_le_bytes();
                    dest.extend_from_slice(&bytes[..tag]);
                } else if r3 == u32::MAX {
                    self.a[r + 3] = 0x8000_0000 | (dest.len() as u32);
                    let right = self.a[r + 1];
                    let left = self.a[r];
                    stack.push(right);
                    stack.push(left);
                } else {
                    let start = (r3 & 0x0FFF_FFFF) as usize;
                    let len = self.a[r + 2] as usize;
                    dest.extend_from_within(start..start + len);
                }"""
)

start_dedup = cks_ep.index("        const PAIR_BITS: u32 = 10;")
end_dedup = cks_ep.rindex("impl Endpoint {")
assert end_dedup > start_dedup + 1000
old_dedup_tail = cks_ep[start_dedup:end_dedup]
new_dedup_tail = """        const PAIR_BITS: u32 = 10;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 2048 u32s (8 KiB)
        const STR_BITS: u32 = 12;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s (16 KiB)
        const HT_WORDS: usize = PAIR_LEN + STR_LEN; // 6144 u32s (24 KiB total, zero u_rec!)

        let (ebase, etop, enodes) = (e.base, e.top, e.nodes);
        let can_dedup = 4 * ns >= HT_WORDS || e.a.len() >= etop + HT_WORDS;
        if !can_dedup {
            // Tiny-block fallback (both 4*ns < 6144 and tail < 6144): direct operad emit into `inline16`!
            let mut cursors = [0; 256];
            id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
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
                        stats.words += 1;
                        let h_len = e.len(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));
                            inline16.push(vp.to_le_bytes());
                            lens.push((h_len + 1) as u8);
                        } else {
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let mut v = [0u8; 16];
                            v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                            v[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                            inline16.push(v);
                            lens.push(u8::MAX);
                        }
                        c
                    };
                    bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }

        // Reuse dead `edges[0..4*ns]` (hits 6/6 Wikidata blocks!) or dead tail `edges[4*ns+etop..]`:
        let (e_slice, ht_buf) = if 4 * ns >= HT_WORDS {
            let (dead_hdr, rest) = edges.split_at_mut(4 * ns);
            (&mut rest[..etop], &mut dead_hdr[..HT_WORDS])
        } else {
            let (prefix, tail) = edges.split_at_mut(4 * ns + etop);
            (&mut prefix[4 * ns..], &mut tail[..HT_WORDS])
        };
        let mut e = Elim::<L> {
            a: e_slice,
            base: ebase,
            top: etop,
            nodes: enodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, str_ht) = ht_buf.split_at_mut(PAIR_LEN);
        pair_ht.fill(0);
        str_ht.fill(0);

        #[inline(always)]
        fn hash_u128(v: u128, wlen: u32) -> u64 {
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ (u64::from(wlen)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^ (h >> 30)
        }

        #[inline(always)]
        fn hash_long(w: &[u8]) -> u64 {
            let n = w.len();
            let lo = u64::from_le_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
            let hi = u64::from_le_bytes([w[n - 8], w[n - 7], w[n - 6], w[n - 5], w[n - 4], w[n - 3], w[n - 2], w[n - 1]]);
            let mid = u64::from_le_bytes([w[n / 2 - 4], w[n / 2 - 3], w[n / 2 - 2], w[n / 2 - 1], w[n / 2], w[n / 2 + 1], w[n / 2 + 2], w[n / 2 + 3]]);
            let h = lo.wrapping_mul(0xBF58_476D_1CE4_E5B9)
                ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ mid.wrapping_mul(0xD6E8_FEB8_6659_FD93)
                ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            h ^ (h >> 30)
        }

        let mut cursors = [0; 256];
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
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
                            let v = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));
                            let v_bytes = v.to_le_bytes();
                            let h = hash_u128(v, wlen);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                lens[c] == wlen_u8 && inline16[c] == v_bytes
                            } {
                                ent & 0x00FF_FFFF
                            } else {
                                let c = inline16.len() as u32;
                                inline16.push(v_bytes);
                                lens.push(wlen_u8);
                                str_ht[slot] = tag | c;
                                c
                            }
                        } else {
                            // Cold LONG path (wlen >= 17, <7% of bands): `spelling` holds ONLY deduplicated >=16B expansions!
                            let already_memo = e.a[e.rule(node_word) + 3] != u32::MAX;
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let wlen_long = end_s - start;
                            let h = hash_long(&spelling[start..end_s]);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let c = (ent & 0x00FF_FFFF) as usize;
                                if lens[c] == u8::MAX {
                                    let meta = &inline16[c];
                                    let ps = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                                    let pe = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                                    (pe - ps) == wlen_long && spelling[ps..pe] == spelling[start..end_s]
                                } else {
                                    false
                                }
                            } {
                                if already_memo {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                                ent & 0x00FF_FFFF
                            } else {
                                let c = inline16.len() as u32;
                                let mut meta = [0u8; 16];
                                meta[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                meta[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                inline16.push(meta);
                                lens.push(u8::MAX);
                                str_ht[slot] = tag | c;
                                c
                            }
                        };
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    }
                };
                bands.push(Band::new(k + width, dst, code));
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        stats.words = inline16.len() - 256;
        cursors
    }
}

"""

cks_ep = cks_ep.replace(old_dedup_tail, new_dedup_tail)

# Write `cand_cks_diet`
d_diet = Path("/tmp/cks_bench/cand_cks_diet")
d_diet.mkdir(parents=True, exist_ok=True)
(d_diet / "mod.rs").write_text(cks_diet_mod)
(d_diet / "endpoint.rs").write_text(cks_ep)
(d_diet / "outq.rs").write_text(base_oq)

# Now build `cand_cks_bi` (`CKS-BiOperad` = `CKS-DietOperad` + L2-gated `Unrolled4-Epoch-XOR` + `emit_xor_bi`!)
# Clean up the duplicate `#[inline(always)]` warning in `u8_oq` too!
u8_oq_clean = u8_oq.replace("    /// The next `span` ranks, each with `pack` added, into `dst`\n    #[inline(always)]\n\n", "")

cks_bi_mod = cks_diet_mod.replace(
    "    ) -> Option<(Counts, u32, u32)> {",
    "    ) -> Option<(Counts, u32, u32, u32)> {"
).replace(
    "    ) -> (Counts, u32, u32) {",
    "    ) -> (Counts, u32, u32, u32) {"
)

old_bi_wire = """        if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
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
        }
        (stats, mark, bits)"""

new_bi_wire = """        const XOR_GATE: u32 = 65_536;
        let xor_mode = m >= XOR_GATE && !wide;
        let mut j_neg1 = mark;
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
            } else {
                let mut stream = starts(thin);
                let mut j = 0;
                let (bands_s, edges_m) = (&self.bands[..], &mut self.edges[..m as usize]);
                each_run(records, p, limit, |c, _, w, keep| {
                    if !keep {
                        return;
                    }
                    let a = usize::from(c);
                    let mut z = stream[a];
                    let end = z + w;
                    while z < end {
                        let ci = cursors[a];
                        let b = bands_s[ci];
                        let bend = b.end();
                        let bstart = if ci == 0 { 0 } else { bands_s[ci - 1].end() };
                        let span = (end.min(bend) - z) as usize;
                        let dest = (b.dst() + (z - bstart)) as usize;
                        let pack = b.code() << bits;
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
                        cursors[a] += usize::from(z == bend);
                    }
                    stream[a] = end;
                });
            }
        } else if QUEUE {
            let mut k = 0;
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
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
        }
        (stats, mark, j_neg1, bits)"""

assert old_bi_wire in cks_bi_mod
cks_bi_mod = cks_bi_mod.replace(old_bi_wire, new_bi_wire).replace(
    """        let (stats, mark, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, bits, out);""",
    """        let (stats, mark, j_neg1, bits) =
            self.construct_capped::<INDUCE, QUEUE>(records, n as u32, p, limit, cap, out)?;
        self.walk(n, stats.retained, mark, j_neg1, bits, out);"""
).replace(
    """        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);""",
    """        let (stats, mark, j_neg1, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let tc = t0.elapsed().as_secs_f64() * 1e6;
        let t1 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, j_neg1, bits, out);"""
).replace(
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
    }""",
    """    fn walk(&mut self, n: usize, m: usize, mark: u32, j_neg1: u32, bits: u32, out: &mut Vec<u8>) {
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
        } else {
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
        }
    }"""
)

emit_xor_bi_no_off_src = """
#[allow(clippy::too_many_arguments)]
fn emit_xor_bi_no_off(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut prev_f = j_neg1;
    let mut cur_f = mark;
    let mut next_b = mark;
    let mut cur_b = j_neg1;
    let mut at_f = 0usize;
    let mut at_b = n;

    let max_pairs = edges.len() / 2 + 1;
    let mut started = false;
    let mut rem_pairs = max_pairs;
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
"""

cks_bi_mod = cks_bi_mod.replace("fn emit(", emit_xor_bi_no_off_src + "\nfn emit(")

d_bi = Path("/tmp/cks_bench/cand_cks_bi")
d_bi.mkdir(parents=True, exist_ok=True)
(d_bi / "mod.rs").write_text(cks_bi_mod)
(d_bi / "endpoint.rs").write_text(cks_ep)
(d_bi / "outq.rs").write_text(u8_oq_clean)

print("Generated cand_cks_diet (CKS-DietOperad) and cand_cks_bi (CKS-BiOperad) successfully.")
