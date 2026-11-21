from pathlib import Path

base_mod = Path("/tmp/cks_bench/cand_mn16_direct/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# Clean duplicate #[inline(never)] in master_a1 and cand_mn16_direct if present
for p in [Path("/tmp/cks_bench/master_a1/mod.rs"), Path("/tmp/cks_bench/cand_mn16_direct/mod.rs")]:
    lines = p.read_text().splitlines()
    out_l = []
    for ln in lines:
        if ln.strip() == "#[inline(never)]" and out_l and out_l[-1].strip() == "#[inline(never)]":
            continue
        out_l.append(ln)
    p.write_text("\n".join(out_l))

base_mod = Path("/tmp/cks_bench/cand_mn16_direct/mod.rs").read_text()

# Now build `cand_operad/endpoint.rs` implementing Theorem 20.1:
# `RULE = 4` u32s (128 bits):
# - When `1 <= len <= 15`:
#   Bytes `0..len` hold the exact bytes (little-endian `u128`), bytes `len..15` are `0`, and byte `15` (`rule[3] >> 24`) holds `len as u32` (`1..=15`)!
# - When `len >= 16`:
#   `rule[0] = left`, `rule[1] = right`, `rule[2] = len`, `rule[3] = 0xFF00_0000` (unexpanded) or `0x8000_0000 | first` (expanded in `spelling`)!

old_elim_helpers = """    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

    #[inline(always)]
    fn len(&self, w: u32) -> u32 {
        if w <= 256 {
            u32::from(w != 0)
        } else {
            self.a[self.rule(w) + 2]
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
        let len = self.len(left) + self.len(right);
        self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        self.top += RULE;
        id
    }"""

new_elim_helpers = """    #[inline(always)]
    fn rule(&self, w: u32) -> usize {
        self.base + (w - 257) as usize * RULE
    }

    #[inline(always)]
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
    }

    /// Extract low-120-bit little-endian `u128` payload of `w` when `len(w) <= 15`
    #[inline(always)]
    fn short_u128(&self, w: u32) -> u128 {
        if w <= 256 {
            u128::from(w - 1)
        } else {
            let r = self.rule(w);
            let w0 = u128::from(self.a[r]);
            let w1 = u128::from(self.a[r + 1]) << 32;
            let w2 = u128::from(self.a[r + 2]) << 64;
            let w3 = u128::from(self.a[r + 3] & 0x00FF_FFFF) << 96;
            w0 | w1 | w2 | w3
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
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, 0xFF00_0000]);
        }
        self.top += RULE;
        id
    }"""

old_append = """    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let rule = &mut self.a[r..r + RULE];
                if rule[3] == u32::MAX {
                    rule[3] = dest.len() as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = rule[3] as usize;
                    dest.extend_from_within(start..start + rule[2] as usize);
                }
            }
        }
    }"""

new_append = """    fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let r = self.rule(id);
                let r3 = self.a[r + 3];
                let tag = r3 >> 24;
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
                }
            }
        }
    }"""

old_ep_destruct = """        let Kernel {
            endpoint,
            strips,
            bands,
            spelling,
            offsets,
            edges,
            ..
        } = self;"""
new_ep_destruct = """        let Kernel {
            endpoint,
            strips,
            bands,
            spelling,
            edges,
            inline16,
            lens,
            ..
        } = self;"""

old_bottom_loop = """        let mut cursors = [0; 256];
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
                // (the port's) every band takes a row at least, so the loop always ends; a wrong
                // construction panics here rather than looping (the mutation check's finding)
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;
                let code = if node_word == 0 {
                    a as u32
                } else {
                    let height = e.len(node_word) as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                    e.append(node_word, &mut endpoint.stack, spelling);
                    spelling.push(a as u8);
                    offsets.push(spelling.len() as u32);
                    254 + offsets.len() as u32
                };
                bands.push(Band {
                    end: k + width,
                    delta: dst.wrapping_sub(k),
                    code,
                });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        cursors"""

# Look at how clean Operad-MN16's bottom loop is:
# In dead `edges[ns + etop ..]`:
# - `pair_ht`: 1024 x 2 = 2048 u32s (8 KiB)
# - `str_ht`: 4096 u32s (16 KiB)
# - `u_rec`: 5 u32s per unique word (`[w0, w1, w2, w3, wlen_or_long]` -> 4096 * 5 = 20,480 u32s)!
#   Wait: `2048 + 4096 + 20480 = 26,624` u32s! On `block-0`, `ns + etop = 63,932`, `63,932 + 26,624 = 90,556 > 89,874` (`by 682 words`)!
#   Make `META_CAP = 3072` (`3072 * 5 = 15,360` -> `2048 + 4096 + 15360 = 21,504` u32s -> `63,932 + 21,504 = 85,436 <= 89,874` — FITS INSIDE `block-0` WITH ZERO `edges` GROWTH!!)!
operad_bottom = """        const PAIR_BITS: u32 = 10;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 2048 u32s (8 KiB)
        const STR_BITS: u32 = 12;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s (16 KiB)
        const META_CAP: usize = 3072; // 5 u32s per unique word = 15,360 u32s
        const SCRATCH_NEED: usize = PAIR_LEN + STR_LEN + 5 * META_CAP; // 21,504 u32s

        let (ebase, etop, enodes) = (e.base, e.top, e.nodes);
        if e.a.len() < etop + SCRATCH_NEED {
            // Tiny-m block fallback: direct operad emit into `inline16` without growing `edges`!
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
                    bands.push(Band { end: k + width, delta: dst.wrapping_sub(k), code });
                    in_node += width;
                    k += width;
                    if in_node == node_width { id = L::next(e.a, id, 0); in_node = 0; }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }

        let used_arena = ns + etop;
        let (live_prefix, ht_space) = edges.split_at_mut(used_arena);
        let mut e = Elim::<L> {
            a: &mut live_prefix[ns..],
            base: ebase,
            top: etop,
            nodes: enodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, rest_ht) = ht_space.split_at_mut(PAIR_LEN);
        let (str_ht, u_rec) = rest_ht.split_at_mut(STR_LEN);
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

        let mut unique_words = 0usize;
        let mut overflow_rec: Vec<([u8; 16], u8)> = Vec::new();
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
                            // ZERO-STACK, ZERO-SPELLING REGISTER PATH (>93% of bands)!
                            let wlen = h_len + 1;
                            let v = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));
                            let h = hash_u128(v, wlen);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            let w0 = v as u32;
                            let w1 = (v >> 32) as u32;
                            let w2 = (v >> 64) as u32;
                            let w3 = (v >> 96) as u32;
                            if (ent & 0xFF00_0000) == tag && {
                                let uidx = (ent & 0x00FF_FFFF) as usize;
                                let base_u = 5 * uidx;
                                u_rec[base_u + 4] == wlen
                                    && u_rec[base_u] == w0
                                    && u_rec[base_u + 1] == w1
                                    && u_rec[base_u + 2] == w2
                                    && u_rec[base_u + 3] == w3
                            } {
                                256 + (ent & 0x00FF_FFFF)
                            } else {
                                let uidx = unique_words;
                                unique_words += 1;
                                if uidx < META_CAP {
                                    str_ht[slot] = tag | (uidx as u32);
                                    let base_u = 5 * uidx;
                                    u_rec[base_u] = w0;
                                    u_rec[base_u + 1] = w1;
                                    u_rec[base_u + 2] = w2;
                                    u_rec[base_u + 3] = w3;
                                    u_rec[base_u + 4] = wlen;
                                } else {
                                    overflow_rec.push((v.to_le_bytes(), wlen as u8));
                                }
                                256 + uidx as u32
                            }
                        } else {
                            // Cold LONG path (wlen >= 17, <7% of bands): `spelling` holds ONLY >=16B expansions!
                            let already_memo = (e.a[e.rule(node_word) + 3] >> 24) != 0xFF;
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let wlen = end_s - start;
                            let h = hash_long(&spelling[start..end_s]);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && {
                                let uidx = (ent & 0x00FF_FFFF) as usize;
                                let base_u = 5 * uidx;
                                if u_rec[base_u + 4] == u32::from(u8::MAX) {
                                    let ps = u_rec[base_u] as usize;
                                    let pe = u_rec[base_u + 1] as usize;
                                    (pe - ps) == wlen && spelling[ps..pe] == spelling[start..end_s]
                                } else {
                                    false
                                }
                            } {
                                if already_memo {
                                    spelling.truncate(start);
                                } else {
                                    spelling.pop();
                                }
                                256 + (ent & 0x00FF_FFFF)
                            } else {
                                let uidx = unique_words;
                                unique_words += 1;
                                if uidx < META_CAP {
                                    str_ht[slot] = tag | (uidx as u32);
                                    let base_u = 5 * uidx;
                                    u_rec[base_u] = start as u32;
                                    u_rec[base_u + 1] = end_s as u32;
                                    u_rec[base_u + 2] = 0;
                                    u_rec[base_u + 3] = 0;
                                    u_rec[base_u + 4] = u32::from(u8::MAX);
                                } else {
                                    let mut v = [0u8; 16];
                                    v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                                    v[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                                    overflow_rec.push((v, u8::MAX));
                                }
                                256 + uidx as u32
                            }
                        };
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    }
                };
                bands.push(Band { end: k + width, delta: dst.wrapping_sub(k), code });
                in_node += width;
                k += width;
                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }
            }
        }
        assert_eq!(id, 0);
        stats.words = unique_words;
        inline16.reserve_exact(unique_words);
        lens.reserve_exact(unique_words);
        let in_arena = unique_words.min(META_CAP);
        for uidx in 0..in_arena {
            let b = 5 * uidx;
            let mut v = [0u8; 16];
            v[0..4].copy_from_slice(&u_rec[b].to_le_bytes());
            v[4..8].copy_from_slice(&u_rec[b + 1].to_le_bytes());
            v[8..12].copy_from_slice(&u_rec[b + 2].to_le_bytes());
            v[12..16].copy_from_slice(&u_rec[b + 3].to_le_bytes());
            inline16.push(v);
            lens.push(u_rec[b + 4] as u8);
        }
        for &(v, l) in &overflow_rec {
            inline16.push(v);
            lens.push(l);
        }
        cursors"""

assert old_elim_helpers in base_ep and old_append in base_ep and old_bottom_loop in base_ep
op_ep = (
    base_ep
    .replace(old_elim_helpers, new_elim_helpers)
    .replace(old_append, new_append)
    .replace(old_ep_destruct, new_ep_destruct)
    .replace(old_bottom_loop, operad_bottom)
)

d_op = Path("/tmp/cks_bench/cand_operad")
d_op.mkdir(parents=True, exist_ok=True)
(d_op / "mod.rs").write_text(base_mod)
(d_op / "endpoint.rs").write_text(op_ep)
(d_op / "outq.rs").write_text(base_oq)
print("Generated cand_operad successfully.")
