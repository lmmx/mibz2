from pathlib import Path

base_mod = Path("/tmp/cks_bench/cand_mn16/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/master_a1/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/master_a1/outq.rs").read_text()

# Add stage timing helper `run_timed` to BOTH master_a1 and cand_mn16_direct for exact stage attribution!
timed_fn_master = """
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
"""

m1_mod = Path("/tmp/cks_bench/master_a1/mod.rs").read_text()
if "pub fn run_stages" not in m1_mod:
    m1_mod = m1_mod.replace("    pub fn scratch_report(", timed_fn_master + "\n    pub fn scratch_report(")
    Path("/tmp/cks_bench/master_a1/mod.rs").write_text(m1_mod)

direct_mod = base_mod.replace("    pub fn scratch_report(", timed_fn_master + "\n    pub fn scratch_report(")

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

direct_bottom_loop = """        // MN16-Direct: Zero-allocation, O(1) 1-probe direct-mapped two-tier quotient in dead `edges`.
        // Tier 1: 1024-entry direct-mapped `(node_word << 8) | a` -> `code` (8 KiB in `edges`)
        // Tier 2: 4096-entry direct-mapped 8-bit-tagged word cache (16 KiB in `edges`)
        // Plus `u_meta` `[start, len]` in `edges` for exact `reserve_exact(unique_words)`.
        // If `edges.len() < used_arena + SCRATCH_NEED` (only on tiny-m blocks where Dict << L1D),
        // skips hash tables completely so `edges` NEVER grows a single byte!
        const PAIR_BITS: u32 = 10;
        const PAIR_LEN: usize = 1 << (PAIR_BITS as usize + 1); // 2048 u32s (8 KiB)
        const STR_BITS: u32 = 12;
        const STR_MASK: usize = (1 << STR_BITS) - 1;
        const STR_LEN: usize = 1 << STR_BITS; // 4096 u32s (16 KiB)
        const META_CAP: usize = 4096; // 8192 u32s
        const SCRATCH_NEED: usize = PAIR_LEN + STR_LEN + 2 * META_CAP; // 14,336 u32s

        let (ebase, etop, enodes) = (e.base, e.top, e.nodes);
        let used_arena = ns + etop;
        if edges.len() < used_arena + SCRATCH_NEED {
            // Tiny-m block: zero extra allocation, direct inline16 push
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
                        let start = spelling.len();
                        e.append(node_word, &mut endpoint.stack, spelling);
                        spelling.push(a as u8);
                        let wlen = spelling.len() - start;
                        let c = inline16.len() as u32;
                        let mut v = [0u8; 16];
                        if wlen <= 16 {
                            v[..wlen].copy_from_slice(&spelling[start..start + wlen]);
                            lens.push(wlen as u8);
                        } else {
                            v[0..4].copy_from_slice(&(start as u32).to_le_bytes());
                            v[4..8].copy_from_slice(&((start + wlen) as u32).to_le_bytes());
                            lens.push(u8::MAX);
                        }
                        inline16.push(v);
                        c
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
            return cursors;
        }

        let (live_prefix, ht_space) = edges.split_at_mut(used_arena);
        let mut e = Elim::<L> {
            a: &mut live_prefix[ns..],
            base: ebase,
            top: etop,
            nodes: enodes,
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
                        let slot = (h as usize) & STR_MASK;
                        let ent = str_ht[slot];
                        let found_code = if (ent & 0xFF00_0000) == tag && {
                            let uidx = (ent & 0x00FF_FFFF) as usize;
                            let pstart = u_meta[2 * uidx] as usize;
                            let plen = u_meta[2 * uidx + 1] as usize;
                            plen == wlen && spelling[pstart..pstart + plen] == spelling[start..start + wlen]
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
                                u_meta[2 * uidx] = start as u32;
                                u_meta[2 * uidx + 1] = wlen as u32;
                            } else {
                                overflow_meta.push((start as u32, wlen as u32));
                            }
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
        let in_arena = unique_words.min(META_CAP);
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

direct_ep = base_ep.replace(old_ep_destruct, new_ep_destruct).replace(old_bottom_loop, direct_bottom_loop)
d_dir = Path("/tmp/cks_bench/cand_mn16_direct")
d_dir.mkdir(parents=True, exist_ok=True)
(d_dir / "mod.rs").write_text(direct_mod)
(d_dir / "endpoint.rs").write_text(direct_ep)
(d_dir / "outq.rs").write_text(base_oq)
print("Generated cand_mn16_direct.")
