from pathlib import Path

# Read master_a1 as base
base_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# --- ARM 1: cand_mn16 (Two-tier Arena MN16 + offsets-in-inline16 + Truncate-Safe Tape) ---
# Let's build cand_mn16 cleanly:
# 1. Kernel drops `offsets` usage in `induced` (keeps empty `offsets` only for dead `trace` INDUCE=false path or eliminates `offsets` completely!)
# Let's eliminate `offsets` completely from Kernel!

mn16_mod = base_mod.replace("    offsets: Vec<u32>,\n", "")
# Fix non-INDUCE `trace` path (unused in `<true, true>`, keep compiling cleanly):
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
                    let s = (start as u32).to_le_bytes();
                    let e = (self.spelling.len() as u32).to_le_bytes();
                    v[0..4].copy_from_slice(&s);
                    v[4..8].copy_from_slice(&e);
                    self.lens.push(LONG);
                }
                self.inline16.push(v);
                code"""
mn16_mod = mn16_mod.replace(old_trace_code, new_trace_code)

old_clear = """        self.bands.clear();
        self.spelling.clear();
        self.offsets.clear();
        self.offsets.push(0);"""
new_clear = """        self.bands.clear();
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
mn16_mod = mn16_mod.replace(old_clear, new_clear)

old_walk = """    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
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
new_walk = """    fn walk(&mut self, n: usize, m: usize, mark: u32, bits: u32, out: &mut Vec<u8>) {
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
mn16_mod = mn16_mod.replace(old_walk, new_walk)

old_emit_sig = """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    offsets: &[u32],
    inline16: &[[u8; INLINE]],"""
new_emit_sig = """fn emit(
    edges: &[u32],
    extra: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],"""
mn16_mod = mn16_mod.replace(old_emit_sig, new_emit_sig)

old_long_branch = """            let w = code - 256;
            let word = &spelling[offsets[w] as usize..offsets[w + 1] as usize];"""
new_long_branch = """            let meta = &inline16[code];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];"""
mn16_mod = mn16_mod.replace(old_long_branch, new_long_branch)
mn16_mod = mn16_mod.replace("        let offsets = 4 * self.offsets.capacity();\n", "        let offsets = 0usize;\n")

# Remove dead `inline_words` warning by adding #[allow(dead_code)]
mn16_mod = mn16_mod.replace("fn inline_words(", "#[allow(dead_code)]\nfn inline_words(")

# Now build cand_mn16 endpoint.rs!
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

new_bottom_loop = """        // Arena-hosted two-tier Myhill-Nerode quotient (MN16):
        // Slice dead `edges` after `ns + e.top` for:
        // - pair_ht: 2048 x 2 u32s (16 KiB) direct-mapped `(node_word << 8) | a` -> `code`
        // - str_ht:  4096 u32s (16 KiB) open-addressed `(tag8 << 24) | u_idx`
        // - u_meta:  up to 4096 x 2 u32s `[start, len]` for unique words before `reserve_exact`
        const PAIR_BITS: u32 = 11;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 4096 u32s
        const STR_BITS: u32 = 12;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s
        const MAX_DEDUP: usize = 3072; // 75% load factor bound -> O(1) guaranteed probe
        const SCRATCH_NEED: usize = PAIR_LEN + STR_LEN + 2 * MAX_DEDUP; // 14,336 u32s (56 KiB)

        let used_arena = ns + e.top;
        if edges.len() < used_arena + SCRATCH_NEED {
            // Only possible on tiny blocks where m < 75K; still capped cleanly
            edges.resize(used_arena + SCRATCH_NEED, 0);
        }
        let (live_prefix, ht_space) = edges.split_at_mut(used_arena);
        let mut e = Elim::<L> {
            a: &mut live_prefix[ns..],
            base: e.base,
            top: e.top,
            nodes: e.nodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, rest_ht) = ht_space.split_at_mut(PAIR_LEN);
        let (str_ht, u_meta) = rest_ht.split_at_mut(STR_LEN);
        pair_ht.fill(0);
        str_ht.fill(0);

        #[inline(always)]
        fn hash_word(w: &[u8]) -> u64 {
            let n = w.len();
            let mut h = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            if n >= 8 {
                let lo = u64::from_le_bytes([w[0], w[1], w[2], w[3], w[4], w[5], w[6], w[7]]);
                let hi = u64::from_le_bytes([w[n - 8], w[n - 7], w[n - 6], w[n - 5], w[n - 4], w[n - 3], w[n - 2], w[n - 1]]);
                h ^= lo.wrapping_mul(0xBF58_476D_1CE4_E5B9) ^ hi.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB);
                if n > 16 {
                    let mid = u64::from_le_bytes([w[n / 2 - 4], w[n / 2 - 3], w[n / 2 - 2], w[n / 2 - 1], w[n / 2], w[n / 2 + 1], w[n / 2 + 2], w[n / 2 + 3]]);
                    h ^= mid.wrapping_mul(0xD6E8_FEB8_6659_FD93);
                }
            } else if n >= 4 {
                let lo = u32::from_le_bytes([w[0], w[1], w[2], w[3]]) as u64;
                let hi = u32::from_le_bytes([w[n - 4], w[n - 3], w[n - 2], w[n - 1]]) as u64;
                h ^= ((lo << 32) | hi).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            } else {
                let mut v = 0u64;
                for &b in w {
                    v = (v << 8) | u64::from(b);
                }
                h ^= v.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            }
            h ^ (h >> 30)
        }

        let mut unique_words = 0usize;
        let mut overflow_meta: Vec<(u32, u32)> = Vec::new();
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
                        let already_memo = node_word <= 256 || e.a[e.rule(node_word) + 3] != u32::MAX;
                        let start = spelling.len();
                        e.append(node_word, &mut endpoint.stack, spelling);
                        spelling.push(a as u8);
                        let wlen = spelling.len() - start;
                        let h = hash_word(&spelling[start..]);
                        let tag = (((h >> 56) as u32) | 0x80) << 24;
                        let mut slot = (h as usize) & STR_MASK;
                        let found_code = if unique_words < MAX_DEDUP {
                            loop {
                                let ent = str_ht[slot];
                                if ent == 0 {
                                    let uidx = unique_words;
                                    unique_words += 1;
                                    str_ht[slot] = tag | (uidx as u32);
                                    u_meta[2 * uidx] = start as u32;
                                    u_meta[2 * uidx + 1] = wlen as u32;
                                    break 256 + uidx as u32;
                                }
                                if (ent & 0xFF00_0000) == tag {
                                    let uidx = (ent & 0x00FF_FFFF) as usize;
                                    let pstart = u_meta[2 * uidx] as usize;
                                    let plen = u_meta[2 * uidx + 1] as usize;
                                    if plen == wlen && spelling[pstart..pstart + plen] == spelling[start..start + wlen] {
                                        if already_memo {
                                            spelling.truncate(start);
                                        } else {
                                            spelling.pop();
                                        }
                                        break 256 + uidx as u32;
                                    }
                                }
                                slot = (slot + 1) & STR_MASK;
                            }
                        } else {
                            let uidx = unique_words;
                            unique_words += 1;
                            overflow_meta.push((start as u32, wlen as u32));
                            256 + uidx as u32
                        };
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = found_code;
                        found_code
                    }
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
        stats.words = unique_words;
        inline16.reserve_exact(unique_words);
        lens.reserve_exact(unique_words);
        let in_arena = unique_words.min(MAX_DEDUP);
        for uidx in 0..in_arena {
            let s = u_meta[2 * uidx] as usize;
            let wlen = u_meta[2 * uidx + 1] as usize;
            let mut v = [0u8; 16];
            if wlen <= 16 {
                v[..wlen].copy_from_slice(&spelling[s..s + wlen]);
                lens.push(wlen as u8);
            } else {
                v[0..4].copy_from_slice(&(s as u32).to_le_bytes());
                v[4..8].copy_from_slice(&((s + wlen) as u32).to_le_bytes());
                lens.push(u8::MAX);
            }
            inline16.push(v);
        }
        for &(s32, wlen32) in &overflow_meta {
            let (s, wlen) = (s32 as usize, wlen32 as usize);
            let mut v = [0u8; 16];
            if wlen <= 16 {
                v[..wlen].copy_from_slice(&spelling[s..s + wlen]);
                lens.push(wlen as u8);
            } else {
                v[0..4].copy_from_slice(&s32.to_le_bytes());
                v[4..8].copy_from_slice(&(s32 + wlen32).to_le_bytes());
                lens.push(u8::MAX);
            }
            inline16.push(v);
        }
        cursors"""

mn16_ep = base_ep.replace(old_ep_destruct, new_ep_destruct).replace(old_bottom_loop, new_bottom_loop)
# Also prevent tiny-block `edges.resize` from exceeding `m` unnecessarily on full blocks:
# Notice on all full blocks with m >= 89,874, `used_arena + 14,336` <= 121,203 (on block-143 m=196,841; on block-0 63,932 + 14,336 = 78,268 <= 89,874!).
# So `edges.len() >= m >= used_arena + 14,336` on ALL 6 blocks -> ZERO extra bytes in `edges`!

d_mn16 = Path("/tmp/cks_bench/cand_mn16")
d_mn16.mkdir(parents=True, exist_ok=True)
(d_mn16 / "mod.rs").write_text(mn16_mod)
(d_mn16 / "endpoint.rs").write_text(mn16_ep)
(d_mn16 / "outq.rs").write_text(base_oq)

# --- ARM 2: cand_pair (ONLY direct-mapped `(node_word, a)` pair cache, NO string hashing!) ---
# Let's test if `(node_word, a)` alone captures enough or if full string quotient beats it!
pair_bottom_loop = """        const PAIR_BITS: u32 = 11;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 4096 u32s (16 KiB)
        const MAX_META: usize = 8192;
        const SCRATCH_NEED: usize = PAIR_LEN + 2 * MAX_META;

        let used_arena = ns + e.top;
        if edges.len() < used_arena + SCRATCH_NEED {
            edges.resize(used_arena + SCRATCH_NEED, 0);
        }
        let (live_prefix, ht_space) = edges.split_at_mut(used_arena);
        let mut e = Elim::<L> {
            a: &mut live_prefix[ns..],
            base: e.base,
            top: e.top,
            nodes: e.nodes,
            _links: std::marker::PhantomData,
        };
        let (pair_ht, u_meta) = ht_space.split_at_mut(PAIR_LEN);
        pair_ht.fill(0);

        let mut unique_words = 0usize;
        let mut overflow_meta: Vec<(u32, u32)> = Vec::new();
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
                        let start = spelling.len();
                        e.append(node_word, &mut endpoint.stack, spelling);
                        spelling.push(a as u8);
                        let wlen = spelling.len() - start;
                        let uidx = unique_words;
                        unique_words += 1;
                        if uidx < MAX_META {
                            u_meta[2 * uidx] = start as u32;
                            u_meta[2 * uidx + 1] = wlen as u32;
                        } else {
                            overflow_meta.push((start as u32, wlen as u32));
                        }
                        let c = 256 + uidx as u32;
                        pair_ht[pslot] = pkey;
                        pair_ht[pslot + 1] = c;
                        c
                    }
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
        stats.words = unique_words;
        inline16.reserve_exact(unique_words);
        lens.reserve_exact(unique_words);
        let in_arena = unique_words.min(MAX_META);
        for uidx in 0..in_arena {
            let s = u_meta[2 * uidx] as usize;
            let wlen = u_meta[2 * uidx + 1] as usize;
            let mut v = [0u8; 16];
            if wlen <= 16 {
                v[..wlen].copy_from_slice(&spelling[s..s + wlen]);
                lens.push(wlen as u8);
            } else {
                v[0..4].copy_from_slice(&(s as u32).to_le_bytes());
                v[4..8].copy_from_slice(&((s + wlen) as u32).to_le_bytes());
                lens.push(u8::MAX);
            }
            inline16.push(v);
        }
        for &(s32, wlen32) in &overflow_meta {
            let (s, wlen) = (s32 as usize, wlen32 as usize);
            let mut v = [0u8; 16];
            if wlen <= 16 {
                v[..wlen].copy_from_slice(&spelling[s..s + wlen]);
                lens.push(wlen as u8);
            } else {
                v[0..4].copy_from_slice(&s32.to_le_bytes());
                v[4..8].copy_from_slice(&(s32 + wlen32).to_le_bytes());
                lens.push(u8::MAX);
            }
            inline16.push(v);
        }
        cursors"""

pair_ep = base_ep.replace(old_ep_destruct, new_ep_destruct).replace(old_bottom_loop, pair_bottom_loop)
d_pair = Path("/tmp/cks_bench/cand_pair")
d_pair.mkdir(parents=True, exist_ok=True)
(d_pair / "mod.rs").write_text(mn16_mod)
(d_pair / "endpoint.rs").write_text(pair_ep)
(d_pair / "outq.rs").write_text(base_oq)

# --- ARM 3: cand_mn16_diet (MN16 + `out`-tail temporary tape so `spelling` holds ONLY LONG unique words!) ---
# In `construct_capped`: `out` has `n + 16` bytes, and `Pass 1` took `chunks` chunks (`chunks * 1024` bytes).
# So `&mut out[chunks * 1024 ..]` is completely unused during `finish`!
diet_mod = mn16_mod.replace(
    "let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());",
    "let (f, heads, over, q_chunks) = (pass.f, pass.queues.heads(), pass.queues.over(), pass.queues.chunks_used());"
).replace(
    "Some(self.finish::<INDUCE, QUEUE>(&f, records, n, p, limit, &out[..], &heads))",
    "Some(self.finish::<INDUCE, QUEUE>(&f, records, n, p, limit, &mut out[..], &heads, q_chunks))"
).replace(
    "        out: &[u8],\n        heads: &outq::Heads,\n    ) -> (Counts, u32, u32) {",
    "        out: &mut [u8],\n        heads: &outq::Heads,\n        q_chunks: u32,\n    ) -> (Counts, u32, u32) {"
).replace(
    "        let mut cursors = if INDUCE {\n            self.induced(n, &thin, &mut stats)",
    "        let q_bytes = (q_chunks as usize * 1024).min(out.len());\n        let (out_q, out_tape) = out.split_at_mut(q_bytes);\n        let mut cursors = if INDUCE {\n            self.induced(n, &thin, &mut stats, out_tape)"
).replace(
    "let mut queue = outq::Reader::new(out, &self.spill, heads, a);",
    "let mut queue = outq::Reader::new(out_q, &self.spill, heads, a);"
)

diet_oq = base_oq.replace(
    "    pub(super) fn over(&self) -> bool {\n        self.over\n    }",
    "    pub(super) fn over(&self) -> bool {\n        self.over\n    }\n    pub(super) fn chunks_used(&self) -> u32 {\n        self.chunks\n    }"
)

# In `diet_ep`, `e.append_slice` writes into `out_tape` when `out_tape.len() >= 131072` (always true on 900KB blocks!),
# and at the end ONLY unique LONG words (`wlen > 16`) are copied into `spelling` with `reserve_exact`!
diet_ep = mn16_ep.replace(
    "    pub(super) fn induced(\n        &mut self,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n    ) -> [usize; 256] {\n        let most = Self::nodes_most(self.strips.len());\n        if self.wide(self.strips.len()) {\n            self.induced_with::<Wide>(n, thin, stats, most)\n        } else {\n            self.induced_with::<Narrow>(n, thin, stats, most)\n        }\n    }",
    "    pub(super) fn induced(\n        &mut self,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n        out_tape: &mut [u8],\n    ) -> [usize; 256] {\n        let most = Self::nodes_most(self.strips.len());\n        if self.wide(self.strips.len()) {\n            self.induced_with::<Wide>(n, thin, stats, most, out_tape)\n        } else {\n            self.induced_with::<Narrow>(n, thin, stats, most, out_tape)\n        }\n    }"
).replace(
    "    fn induced_with<L: Links>(\n        &mut self,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n        most: usize,\n    ) -> [usize; 256] {",
    "    fn induced_with<L: Links>(\n        &mut self,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n        most: usize,\n        out_tape: &mut [u8],\n    ) -> [usize; 256] {"
)

# Add `append_buf` to `Elim` that writes to `out_tape` with fallback to `spelling`:
append_buf_fn = """    fn append_tape(&mut self, word: u32, stack: &mut Vec<u32>, tape: &mut [u8], tlen: &mut usize) -> bool {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                if *tlen >= tape.len() {
                    return false;
                }
                tape[*tlen] = (id - 1) as u8;
                *tlen += 1;
            } else {
                let r = self.rule(id);
                let rule = &mut self.a[r..r + RULE];
                if rule[3] == u32::MAX {
                    rule[3] = *tlen as u32;
                    stack.push(rule[1]);
                    stack.push(rule[0]);
                } else {
                    let start = rule[3] as usize;
                    let len = rule[2] as usize;
                    if *tlen + len > tape.len() {
                        return false;
                    }
                    tape.copy_within(start..start + len, *tlen);
                    *tlen += len;
                }
            }
        }
        true
    }
    fn append(&mut self,"""

diet_ep = diet_ep.replace("    fn append(&mut self,", append_buf_fn)

# Replace the bottom scan in diet_ep to use `out_tape` when `out_tape.len() >= 98304` (and `total_height` fits):
old_mn16_scan_head = """        let mut unique_words = 0usize;
        let mut overflow_meta: Vec<(u32, u32)> = Vec::new();"""
# Simpler: in diet_ep, check `use_out_tape = out_tape.len() >= 98_304;` and if `append_tape` ever returns false (impossible on Wikidata where unique rule bytes <= 42 KiB << 130+ KiB), fall back cleanly!
# Let's write the exact loop replacement for diet_ep:
diet_scan_body = """        let use_tape = out_tape.len() >= 98_304;
        let mut tlen = 0usize;
        let mut tape_ok = use_tape;
        let mut unique_words = 0usize;
        let mut overflow_meta: Vec<(u32, u32)> = Vec::new();
        let mut cursors = [0; 256];
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        if tape_ok {
            'outer: for a in 0..256 {
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
                            let already_memo = node_word <= 256 || e.a[e.rule(node_word) + 3] != u32::MAX;
                            let start = tlen;
                            if !e.append_tape(node_word, &mut endpoint.stack, out_tape, &mut tlen) || tlen >= out_tape.len() {
                                tape_ok = false;
                                break 'outer;
                            }
                            out_tape[tlen] = a as u8;
                            tlen += 1;
                            let wlen = tlen - start;
                            let h = hash_word(&out_tape[start..tlen]);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let mut slot = (h as usize) & STR_MASK;
                            let found_code = if unique_words < MAX_DEDUP {
                                loop {
                                    let ent = str_ht[slot];
                                    if ent == 0 {
                                        let uidx = unique_words;
                                        unique_words += 1;
                                        str_ht[slot] = tag | (uidx as u32);
                                        u_meta[2 * uidx] = start as u32;
                                        u_meta[2 * uidx + 1] = wlen as u32;
                                        break 256 + uidx as u32;
                                    }
                                    if (ent & 0xFF00_0000) == tag {
                                        let uidx = (ent & 0x00FF_FFFF) as usize;
                                        let pstart = u_meta[2 * uidx] as usize;
                                        let plen = u_meta[2 * uidx + 1] as usize;
                                        if plen == wlen && out_tape[pstart..pstart + plen] == out_tape[start..start + wlen] {
                                            if already_memo {
                                                tlen = start;
                                            } else {
                                                tlen -= 1;
                                            }
                                            break 256 + uidx as u32;
                                        }
                                    }
                                    slot = (slot + 1) & STR_MASK;
                                }
                            } else {
                                let uidx = unique_words;
                                unique_words += 1;
                                overflow_meta.push((start as u32, wlen as u32));
                                256 + uidx as u32
                            };
                            pair_ht[pslot] = pkey;
                            pair_ht[pslot + 1] = found_code;
                            found_code
                        }
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
        }
        if tape_ok {
            assert_eq!(id, 0);
            stats.words = unique_words;
            inline16.reserve_exact(unique_words);
            lens.reserve_exact(unique_words);
            let in_arena = unique_words.min(MAX_DEDUP);
            let mut long_bytes = 0usize;
            for uidx in 0..in_arena {
                let wlen = u_meta[2 * uidx + 1] as usize;
                if wlen > 16 {
                    long_bytes += wlen;
                }
            }
            for &(_, wlen32) in &overflow_meta {
                if wlen32 as usize > 16 {
                    long_bytes += wlen32 as usize;
                }
            }
            spelling.reserve_exact(long_bytes);
            for uidx in 0..in_arena {
                let s = u_meta[2 * uidx] as usize;
                let wlen = u_meta[2 * uidx + 1] as usize;
                let mut v = [0u8; 16];
                if wlen <= 16 {
                    v[..wlen].copy_from_slice(&out_tape[s..s + wlen]);
                    lens.push(wlen as u8);
                } else {
                    let ls = spelling.len() as u32;
                    spelling.extend_from_slice(&out_tape[s..s + wlen]);
                    let le = spelling.len() as u32;
                    v[0..4].copy_from_slice(&ls.to_le_bytes());
                    v[4..8].copy_from_slice(&le.to_le_bytes());
                    lens.push(u8::MAX);
                }
                inline16.push(v);
            }
            for &(s32, wlen32) in &overflow_meta {
                let (s, wlen) = (s32 as usize, wlen32 as usize);
                let mut v = [0u8; 16];
                if wlen <= 16 {
                    v[..wlen].copy_from_slice(&out_tape[s..s + wlen]);
                    lens.push(wlen as u8);
                } else {
                    let ls = spelling.len() as u32;
                    spelling.extend_from_slice(&out_tape[s..s + wlen]);
                    let le = spelling.len() as u32;
                    v[0..4].copy_from_slice(&ls.to_le_bytes());
                    v[4..8].copy_from_slice(&le.to_le_bytes());
                    lens.push(u8::MAX);
                }
                inline16.push(v);
            }
            return cursors;
        }
        // Fallback if tiny block or out_tape overflowed: reset rule[3], bands, ht, spelling
        for r in (e.base..e.top).step_by(RULE) {
            e.a[r + 3] = u32::MAX;
        }
        bands.clear();
        pair_ht.fill(0);
        str_ht.fill(0);
        unique_words = 0;
        overflow_meta.clear();
        id = L::next(e.a, 0, 0);
        in_node = 0;
        k = 0;"""

diet_ep = diet_ep.replace(
    """        let mut unique_words = 0usize;
        let mut overflow_meta: Vec<(u32, u32)> = Vec::new();
        let mut cursors = [0; 256];
        id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;""",
    diet_scan_body
)

d_diet = Path("/tmp/cks_bench/cand_mn16_diet")
d_diet.mkdir(parents=True, exist_ok=True)
(d_diet / "mod.rs").write_text(diet_mod)
(d_diet / "endpoint.rs").write_text(diet_ep)
(d_diet / "outq.rs").write_text(diet_oq)

print("Generated cand_pair, cand_mn16, cand_mn16_diet successfully.")
