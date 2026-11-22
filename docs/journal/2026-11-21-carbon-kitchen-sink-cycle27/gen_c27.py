from pathlib import Path

# Cycle 27 (Neon):
# 1. `cand_i15_bi` (`Inline15-BiOperad`, mu=2): Self-describing 128-bit operad slots (`byte 15 = len in 1..=15, 0 = LONG`),
#    ZERO `lens: Vec<u8>`, branchless backward u128 shift `<< ((16 - lb) * 8)`, and `48 KiB` spelling pre-reserve (`944 KiB = -19.5%` WS!).
# 2. `cand_i15_diet` (`Inline15-DietOperad`, mu=1): Same self-describing slots on the serial mu=1 flagship!
# 3. Also add a 6-phase profiler `profile_6phases()` to `cand_cks_bi` and `master_a1` (separate cold method, never called in timing loops!).

def build_inline15(src_dir: str, dst_dir: str, is_bi: bool):
    m = Path(f"/tmp/cks_bench/{src_dir}/mod.rs").read_text()
    ep = Path(f"/tmp/cks_bench/{src_dir}/endpoint.rs").read_text()
    oq = Path(f"/tmp/cks_bench/{src_dir}/outq.rs").read_text()

    # Remove `lens: Vec<u8>,` from Kernel struct
    m = m.replace("    lens: Vec<u8>,\n", "")
    m = m.replace("        let lens = self.lens.capacity();\n", "        let lens = 0usize;\n")
    m = m.replace("let dict_l1 = 16 * self.inline16.len() + self.lens.len();", "let dict_l1 = 16 * self.inline16.len();")

    # Update `trace` (`INDUCE=false`) to store length in `v[15]` (`1..=15`, `0 = LONG`)
    old_tr = """                if wlen <= INLINE {
                    v[..wlen].copy_from_slice(&self.spelling[start..start + wlen]);
                    self.lens.push(wlen as u8);
                } else {
                    v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                    v[4..8].copy_from_slice(&(self.spelling.len() as u32).to_le_bytes());
                    self.lens.push(LONG);
                }"""
    new_tr = """                if wlen <= 15 {
                    v[..wlen].copy_from_slice(&self.spelling[start..start + wlen]);
                    v[15] = wlen as u8;
                } else {
                    v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                    v[4..8].copy_from_slice(&(self.spelling.len() as u32).to_le_bytes());
                    v[15] = 0;
                }"""
    assert old_tr in m
    m = m.replace(old_tr, new_tr)

    # Update `inline16` literal init + `spelling` 48 KiB exact pre-reserve (Proposition 26.2!)
    old_init = """        self.bands.clear();
        self.spelling.clear();
        if self.inline16.len() < 256 {
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
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);"""
    new_init = """        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 && self.spelling.capacity() < 49_152 {
            self.spelling.reserve_exact(49_152);
        }
        if self.inline16.len() < 256 {
            let cap_hint = if n >= 65_536 { 1792 } else { 256 };
            self.inline16.clear();
            self.inline16.reserve_exact(cap_hint);
            self.inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v[15] = 1;
                v
            }));
        }
        self.inline16.truncate(256);"""
    assert old_init in m
    m = m.replace(old_init, new_init)

    # Update `walk` calls to omit `&self.lens`
    m = m.replace("            &self.lens,\n", "")
    m = m.replace("    lens: &[u8],\n", "")

    # Update `emit` (serial mu=1) to read `slot[15]` directly!
    old_emit_body = """        let len = lens[code];
        if len != LONG {
            // `at < n` here, so the copy ends within `n + 16`
            out[at..at + INLINE].copy_from_slice(&inline16[code]);
            at += usize::from(len);
        } else {
            let meta = &inline16[code];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }"""
    new_emit_body = """        let slot = inline16[code];
        let len = slot[15];
        if len != 0 {
            out[at..at + INLINE].copy_from_slice(&slot);
            at += usize::from(len);
        } else {
            let s = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
            let e = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(n - at);
            out[at..at + take].copy_from_slice(&word[..take]);
            at += take;
        }"""
    assert old_emit_body in m
    m = m.replace(old_emit_body, new_emit_body)

    if is_bi:
        # Update `emit_xor_bi_no_off` fast loop, tail loop, and odd-middle step!
        old_bi_fast = """        let lf = lens[cf];
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
        }"""
        new_bi_fast = """        let sf = inline16[cf];
        let lf = sf[15];
        if lf != 0 {
            out[at_f..at_f + INLINE].copy_from_slice(&sf);
            at_f += usize::from(lf);
        } else {
            let s = u32::from_le_bytes([sf[0], sf[1], sf[2], sf[3]]) as usize;
            let e = u32::from_le_bytes([sf[4], sf[5], sf[6], sf[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let sb = inline16[cb];
        let lb = sb[15];
        if lb != 0 {
            let lbu = usize::from(lb);
            if at_f + INLINE <= at_b {
                // `lbu in 1..=15` => `(16 - lbu) * 8 in 8..=120`: branchless AND shifts byte 15 (`lb`) completely out!
                let shifted = u128::from_le_bytes(sb) << ((16 - lbu) * 8);
                out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                at_b -= lbu;
            } else {
                let take = lbu.min(at_b - at_f);
                out[at_b - take..at_b].copy_from_slice(&sb[lbu - take..lbu]);
                at_b -= take;
            }
        } else {
            let s = u32::from_le_bytes([sb[0], sb[1], sb[2], sb[3]]) as usize;
            let e = u32::from_le_bytes([sb[4], sb[5], sb[6], sb[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }"""
        assert old_bi_fast in m
        m = m.replace(old_bi_fast, new_bi_fast)

        old_bi_tail = """        let lf = lens[cf];
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
    }"""
        new_bi_tail = """        let sf = inline16[cf];
        let lf = sf[15];
        if lf != 0 {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&sf[..take]);
            at_f += take;
        } else {
            let s = u32::from_le_bytes([sf[0], sf[1], sf[2], sf[3]]) as usize;
            let e = u32::from_le_bytes([sf[4], sf[5], sf[6], sf[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let sb = inline16[cb];
        let lb = sb[15];
        if lb != 0 {
            let lbu = usize::from(lb);
            let take = lbu.min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&sb[lbu - take..lbu]);
            at_b -= take;
        } else {
            let s = u32::from_le_bytes([sb[0], sb[1], sb[2], sb[3]]) as usize;
            let e = u32::from_le_bytes([sb[4], sb[5], sb[6], sb[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    if cur_f == cur_b && at_f < at_b {
        let cf = (edges[cur_f as usize] >> bits) as usize;
        let sf = inline16[cf];
        let lf = sf[15];
        if lf != 0 {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&sf[..take]);
            at_f += take;
        } else {
            let s = u32::from_le_bytes([sf[0], sf[1], sf[2], sf[3]]) as usize;
            let e = u32::from_le_bytes([sf[4], sf[5], sf[6], sf[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }"""
        assert old_bi_tail in m
        m = m.replace(old_bi_tail, new_bi_tail)

    # Update endpoint.rs: remove `lens` destructure, encode `len` in byte 15 for `h_len <= 14` (`wlen <= 15`), `0` for `wlen >= 16`!
    ep = ep.replace("            lens,\n", "")
    old_ep_fallback = """                        if h_len <= 15 {
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
                        }"""
    new_ep_fallback = """                        if h_len <= 14 {
                            let vp = e.short_u128(node_word)
                                | (u128::from(a as u8) << (8 * h_len))
                                | (u128::from(h_len + 1) << 120);
                            inline16.push(vp.to_le_bytes());
                        } else {
                            let start = spelling.len();
                            e.append(node_word, &mut endpoint.stack, spelling);
                            spelling.push(a as u8);
                            let end_s = spelling.len();
                            let mut v = [0u8; 16];
                            v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                            v[4..8].copy_from_slice(&(end_s as u32).to_le_bytes());
                            inline16.push(v);
                        }"""
    assert old_ep_fallback in ep
    ep = ep.replace(old_ep_fallback, new_ep_fallback)

    old_ep_main = """                        let found_code = if h_len <= 15 {
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
                        };"""
    new_ep_main = """                        let found_code = if h_len <= 14 {
                            // Self-describing operad word in M_{<=15}: byte 15 = wlen (1..=15)!
                            // Single 16-byte compare (`inline16[c] == v_bytes`) verifies BOTH payload AND length!
                            let wlen = h_len + 1;
                            let v = e.short_u128(node_word)
                                | (u128::from(a as u8) << (8 * h_len))
                                | (u128::from(wlen) << 120);
                            let v_bytes = v.to_le_bytes();
                            let h = hash_u128(v, wlen);
                            let tag = (((h >> 56) as u32) | 0x80) << 24;
                            let slot = (h as usize) & STR_MASK;
                            let ent = str_ht[slot];
                            if (ent & 0xFF00_0000) == tag && inline16[(ent & 0x00FF_FFFF) as usize] == v_bytes {
                                ent & 0x00FF_FFFF
                            } else {
                                let c = inline16.len() as u32;
                                inline16.push(v_bytes);
                                str_ht[slot] = tag | c;
                                c
                            }
                        } else {
                            // Cold LONG path (wlen >= 16): `slot[15] == 0`!
                            let r3 = e.a[e.rule(node_word) + 3];
                            let already_memo = (r3 >> 28) == 0 || r3 != u32::MAX;
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
                                let meta = &inline16[c];
                                if meta[15] == 0 {
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
                                str_ht[slot] = tag | c;
                                c
                            }
                        };"""
    assert old_ep_main in ep
    ep = ep.replace(old_ep_main, new_ep_main)

    d = Path(f"/tmp/cks_bench/{dst_dir}")
    d.mkdir(parents=True, exist_ok=True)
    (d / "mod.rs").write_text(m)
    (d / "endpoint.rs").write_text(ep)
    (d / "outq.rs").write_text(oq)

build_inline15("cand_cks_bi", "cand_i15_bi", True)
build_inline15("cand_cks_diet", "cand_i15_diet", False)

# Also add `profile_6phases` to a separate profiling module `prof_bi` so we never touch `cand_cks_bi` hot code!
p_dir = Path("/tmp/cks_bench/prof_bi")
p_dir.mkdir(parents=True, exist_ok=True)
pm = Path("/tmp/cks_bench/cand_cks_bi/mod.rs").read_text()
pep = Path("/tmp/cks_bench/cand_cks_bi/endpoint.rs").read_text()
poq = Path("/tmp/cks_bench/cand_cks_bi/outq.rs").read_text()

# Instrument `prof_bi`: return (t_pass1, t_finish, t_elim, t_dedup, t_wire, t_walk, len16_steps, total_steps)
pep = pep.replace(
    "    pub(super) fn induced(\n        &mut self,\n        ns: usize,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n    ) -> [usize; 256] {",
    "    pub(super) fn induced_timed(\n        &mut self,\n        ns: usize,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n    ) -> ([usize; 256], f64, f64) {\n        let most = Self::nodes_most(ns);\n        if self.wide(ns) {\n            self.induced_with_timed::<Wide>(ns, n, thin, stats, most)\n        } else {\n            self.induced_with_timed::<Narrow>(ns, n, thin, stats, most)\n        }\n    }\n    pub(super) fn induced(\n        &mut self,\n        ns: usize,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n    ) -> [usize; 256] {",
)
# Clone induced_with as induced_with_timed with Instant around elim vs dedup
iw_start = pep.index("    fn induced_with<L: Links>(")
iw_end = pep.rindex("impl Endpoint {")
iw_body = pep[iw_start:iw_end]
iw_timed = iw_body.replace(
    "    fn induced_with<L: Links>(\n        &mut self,\n        ns: usize,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n        most: usize,\n    ) -> [usize; 256] {",
    "    fn induced_with_timed<L: Links>(\n        &mut self,\n        ns: usize,\n        n: u32,\n        thin: &[u32; 256],\n        stats: &mut Counts,\n        most: usize,\n    ) -> ([usize; 256], f64, f64) {\n        let t_el0 = std::time::Instant::now();"
).replace(
    "        assert_eq!(x as usize, stats.retained);",
    "        assert_eq!(x as usize, stats.retained);\n        let t_el = t_el0.elapsed().as_secs_f64() * 1e6;\n        let t_dd0 = std::time::Instant::now();"
).replace(
    "            return cursors;",
    "            return (cursors, t_el, t_dd0.elapsed().as_secs_f64() * 1e6);"
).replace(
    "        cursors\n    }\n}",
    "        (cursors, t_el, t_dd0.elapsed().as_secs_f64() * 1e6)\n    }\n}"
)
pep = pep[:iw_end] + "impl Kernel {\n" + iw_timed + "\n" + pep[iw_end:]

# Add `profile_6phases` method to prof_bi/mod.rs
prof_method = """
impl Kernel {
    pub fn profile_6phases(&mut self, records: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> ([f64; 6], usize, usize) {
        out.resize(n + INLINE, 0);
        let t0 = std::time::Instant::now();
        let mut pass = self.begin(&mut out[..], u32::MAX, usize::MAX);
        maximal(records, |c, pos, width| {
            pass.take::<true>(c, pos, width, p, LIMIT);
        });
        let (f, heads, ns) = (pass.f, pass.queues.heads(), pass.ns);
        let t_p1 = t0.elapsed().as_secs_f64() * 1e6;

        let t1 = std::time::Instant::now();
        let (all, thin, thick, m, mark, runs) = (f.all, f.thin, f.thick, f.m, f.mark, f.runs);
        let first = starts(all);
        let mut stats = Counts { n, runs, retained: m as usize, thick: ns, ..Counts::default() };
        let words = self.arena_words(ns, m as usize);
        if self.edges.len() < words { self.edges.resize(words, 0); }
        let mut write = starts(thick);
        let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
        let image_order = &mut rest_edges[..ns];
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let a = usize::from(byte);
            sw[1] += first[a];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
        }
        self.bands.clear();
        self.spelling.clear();
        if self.inline16.len() < 256 {
            self.inline16.clear();
            self.inline16.reserve_exact(1792);
            self.inline16.extend((0..=255u8).map(|c| { let mut v = [0; INLINE]; v[0] = c; v }));
            self.lens.clear();
            self.lens.reserve_exact(1792);
            self.lens.resize(256, 1);
        }
        self.inline16.truncate(256);
        self.lens.truncate(256);
        let t_fin = t1.elapsed().as_secs_f64() * 1e6;

        let (cursors, t_elim, t_dedup) = self.induced_timed(ns, n as u32, &thin, &mut stats);

        let t2 = std::time::Instant::now();
        let bits = (u32::BITS - (m - 1).leading_zeros()).max(1);
        let wide = (255 + stats.words as u32) > (u32::MAX >> bits);
        if self.edges.len() < m as usize { self.edges.resize(m as usize, 0); }
        self.extra.clear();
        if wide { self.extra.resize(m as usize, 0); }
        let mut j_neg1 = mark;
        let xor_mode = m >= 65_536 && !wide;
        if xor_mode {
            self.edges[..m as usize].fill(0);
            let mut k = 0;
            let edges_m = &mut self.edges[..m as usize];
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, &heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let bend = b.end();
                    let span = (bend - k) as usize;
                    let dst = b.dst() as usize;
                    let pack = b.code() << bits;
                    queue.wire_xor(edges_m, dst, span, pack, mark, &mut j_neg1);
                    k = bend;
                }
            }
        }
        let t_wire = t2.elapsed().as_secs_f64() * 1e6;

        // Count len==16 steps across edges[0..m] (done outside timed walk!)
        let mut len16_cnt = 0usize;
        for &ev in &self.edges[..m as usize] {
            let c = (ev >> bits) as usize;
            if self.lens[c] == 16 { len16_cnt += 1; }
        }

        let t3 = std::time::Instant::now();
        self.walk(n, m as usize, mark, j_neg1, bits, out);
        let t_walk = t3.elapsed().as_secs_f64() * 1e6;
        ([t_p1, t_fin, t_elim, t_dedup, t_wire, t_walk], len16_cnt, m as usize)
    }
}
"""
pm += prof_method
(p_dir / "mod.rs").write_text(pm)
(p_dir / "endpoint.rs").write_text(pep)
(p_dir / "outq.rs").write_text(poq)

print("Generated cand_i15_bi, cand_i15_diet, and prof_bi successfully.")
