import os, shutil

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c41_combo"
BASE_NOBI = f"{ROOT}/cand_c41_nobi41"

def clone_from(src, name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    return dst

def patch_pair2(dst):
    """
    Theorem 42.1 (Adjacent-Run Distinct-Byte Invariant):
    In canonical CAP=127 records, consecutive thin runs (`w0 <= limit && w1 <= limit`) ALWAYS satisfy `c0 != c1`!
    Therefore `pt[a0]` and `pt[a1]` cannot alias, and their `out` chunks are disjoint -> 2-run pipelined fast path!
    """
    mod_p = f"{dst}/mod.rs"
    outq_p = f"{dst}/outq.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(outq_p) as f:
        o = f.read()

    pair_fn = """    #[inline(always)]
    pub(super) fn push_thin_pair(&mut self, a0: usize, w0: u32, a1: usize, w1: u32, m: u32) {
        let v0 = self.pt[a0];
        let v1 = self.pt[a1];
        let p0 = v0 as u32;
        let p1 = v1 as u32;
        let f0 = (p0 & 1023) as usize;
        let f1 = (p1 & 1023) as usize;
        let m1 = m + w0;
        if (w0 <= 16) & (w1 <= 16) & (f0 <= CHUNK - 16) & (f1 <= CHUNK - 16) & ((m as u16) < 65_504) {
            let at0 = p0 as usize + f0;
            let at1 = p1 as usize + f1;
            let out_len = self.out.len();
            if (at0 + 32 <= out_len) & (at1 + 32 <= out_len) {
                let lo0 = m as u16;
                let iota0: [u16; 16] = std::array::from_fn(|i| lo0.wrapping_add(i as u16));
                for (d, x) in self.out[at0..at0 + 32].chunks_exact_mut(2).zip(iota0) {
                    d.copy_from_slice(&x.to_le_bytes());
                }
                let lo1 = m1 as u16;
                let iota1: [u16; 16] = std::array::from_fn(|i| lo1.wrapping_add(i as u16));
                for (d, x) in self.out[at1..at1 + 32].chunks_exact_mut(2).zip(iota1) {
                    d.copy_from_slice(&x.to_le_bytes());
                }
                let wu0 = u64::from(w0);
                let wu1 = u64::from(w1);
                self.pt[a0] = v0 + ((wu0 << 32) | wu0);
                self.pt[a1] = v1 + ((wu1 << 32) | wu1);
                return;
            }
        }
        self.push_thin(a0, m, w0);
        self.push_thin(a1, m1, w1);
    }
"""
    anchor = "    #[inline(always)]\n    pub(super) fn push_thin("
    assert anchor in o
    o = o.replace(anchor, pair_fn + "\n" + anchor, 1)

    old_loop = """        {
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= limit {
                    // Canonical runs.rs invariant (CAP = 127 > LIMIT = 16): no continuation possible!
                    pass.piece::<QUEUE>(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take::<QUEUE>(c, pos, width, p, limit);
                    pos += width;
                }
            }
        }"""
    new_loop = """        {
            let mut pos = 0u32;
            let mut i = 0usize;
            let nrec = records.len();
            while i + 1 < nrec {
                let r0 = records[i];
                let r1 = records[i + 1];
                let w0 = u32::from(r0 & 255);
                let w1 = u32::from(r1 & 255);
                if (w0 <= limit) & (w1 <= limit) {
                    let c0 = (r0 >> 8) as u8;
                    let c1 = (r1 >> 8) as u8;
                    if c0 != c1 {
                        let wtot = w0 + w1;
                        if p.wrapping_sub(pos) < wtot {
                            let dp = p - pos;
                            if dp < w0 {
                                pass.f.mark = pass.f.m + dp;
                                pass.f.mark_byte = c0;
                            } else {
                                pass.f.mark = pass.f.m + dp;
                                pass.f.mark_byte = c1;
                            }
                        }
                        if QUEUE {
                            pass.queues.push_thin_pair(usize::from(c0), w0, usize::from(c1), w1, pass.f.m);
                        } else {
                            pass.f.thin[usize::from(c0)] += w0;
                            pass.f.thin[usize::from(c1)] += w1;
                        }
                        pass.f.m += wtot;
                        pos += wtot;
                        i += 2;
                        continue;
                    }
                }
                let c = (r0 >> 8) as u8;
                let mut width = w0;
                i += 1;
                if width <= limit {
                    pass.piece::<QUEUE>(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take::<QUEUE>(c, pos, width, p, limit);
                    pos += width;
                }
            }
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                if width <= limit {
                    pass.piece::<QUEUE>(c, pos, width, true, p);
                    pos += width;
                } else {
                    if width == 127 {
                        while i < nrec && (records[i] >> 8) as u8 == c {
                            width += u32::from(records[i] & 255);
                            i += 1;
                        }
                    }
                    pass.take::<QUEUE>(c, pos, width, p, limit);
                    pos += width;
                }
            }
        }"""
    assert old_loop in s
    s = s.replace(old_loop, new_loop, 1)
    with open(mod_p, "w") as f:
        f.write(s)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_s3_arr(dst):
    """Zero-bounds-check fixed-size `&mut [u32; 1024]` / `&mut [u32; 2048]` Dictionary with #[cold] tiny-block fallback."""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    old_dict = """struct Dictionary<'a> {
    pairs: &'a mut [u32],
    contents: &'a mut [u32],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let uid = id + (u32::from(g == 2) << 8);
        let key = (uid << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 23) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            if self.contents.is_empty() {
                return self.insert(entry, len);
            }
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = h as usize & (self.contents.len() - 1);
            let ent = self.contents[index];
            let code = if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                ent & 0x00FF_FFFF
            } else {
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                c
            };
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            return code;
        }
        // Cold LONG path (height >= 16)
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = super::long_slot(start as u32, self.spelling.len() as u32);
        if self.contents.is_empty() {
            return self.insert(entry, LONG);
        }
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = h as usize & (self.contents.len() - 1);
        let ent = self.contents[index];
        let code = if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && super::long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            ent & 0x00FF_FFFF
        } else {
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}"""

    new_dict = """struct Dictionary<'a> {
    pairs: &'a mut [u32; 1024],
    contents: &'a mut [u32; 2048],
    inline16: &'a mut Vec<[u8; INLINE]>,
    lens: &'a mut Vec<u8>,
    spelling: &'a mut Vec<u8>,
    stack: &'a mut Vec<u32>,
}

impl Dictionary<'_> {
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        let g = handle >> ID_BITS;
        let id = handle & ID_MASK;
        let uid = id + (u32::from(g == 2) << 8);
        let key = (uid << 8) | u32::from(byte);
        let bucket = (key.wrapping_mul(0x9e37_79b9) >> 23) as usize;
        let slot = bucket * 2;
        if self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }
        self.code_miss(e, handle, id, byte, key, slot)
    }

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {
            let v = e.short_payload_h(handle) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = (h as usize) & 2047;
            let ent = self.contents[index];
            let code = if (ent & 0xFF00_0000) == tag && {
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            } {
                ent & 0x00FF_FFFF
            } else {
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                c
            };
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            return code;
        }
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(handle, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = super::long_slot(start as u32, self.spelling.len() as u32);
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = (h as usize) & 2047;
        let ent = self.contents[index];
        let code = if (ent & 0xFF00_0000) == tag && {
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && super::long_word(&self.inline16[old], self.spelling) == bytes
        } {
            if memoized {
                self.spelling.truncate(start);
            } else {
                self.spelling.pop();
            }
            ent & 0x00FF_FFFF
        } else {
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}"""
    assert old_dict in e
    e = e.replace(old_dict, new_dict, 1)

    old_setup = """        let (pairs, contents) = if cache.len() >= CACHE_WORDS {
            cache[..CACHE_WORDS].fill(0);
            cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
        } else {
            (&mut [][..], &mut [][..])
        };
        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16,
            lens,
            spelling,
            stack: &mut endpoint.stack,
        };"""

    new_setup = """        let mut e = Elim::<L> {
            a: live,
            base,
            top,
            nodes,
            _links: std::marker::PhantomData,
        };
        begin_inline(if n >= 65_536 { 1536 } else { 0 }, inline16, lens);
        if cache.len() < CACHE_WORDS {
            let mut cursors = [0; 256];
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = bands.len();
                let end = k + thin[a];
                while k < end {
                    let (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    let width = (end - k).min(node_width - in_node);
                    assert!(width > 0);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        let (entry, len) = e.inline_word(node_word, a as u8, &mut endpoint.stack, spelling);
                        let c = lens.len() as u32;
                        inline16.push(entry);
                        lens.push(len);
                        c
                    };
                    bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = next_id;
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            return cursors;
        }
        cache[..CACHE_WORDS].fill(0);
        let (p_sl, c_sl) = cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS);
        let pairs: &mut [u32; 1024] = p_sl.try_into().unwrap();
        let contents: &mut [u32; 2048] = c_sl.try_into().unwrap();
        let mut dictionary = Dictionary {
            pairs,
            contents,
            inline16,
            lens,
            spelling,
            stack: &mut endpoint.stack,
        };"""
    assert old_setup in e
    e = e.replace(old_setup, new_setup, 1)

    with open(ep_p, "w") as f:
        f.write(e)

def patch_band_width(dst):
    """Store `span` (width) directly in low 20 bits of Band(u64) for QUEUE=true, eliminating `bend - k` in S4!"""
    mod_p = f"{dst}/mod.rs"
    ep_p = f"{dst}/endpoint.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(ep_p) as f:
        e = f.read()

    # In endpoint.rs: push Band::new(width, dst, code) instead of Band::new(k + width, dst, code)
    e = e.replace("bands.push(Band::new(k + width, dst, code));", "bands.push(Band::new(width, dst, code));")
    # In mod.rs S4 (QUEUE=true branches): use band.end() directly as span (no k tracking!)
    old_xor = """            let mut k = 0;
            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        let bend = band.end();
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            (bend - k) as usize,
                            band.code() << bits,
                        );
                        k = bend;
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        let bend = band.end();
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            (bend - k) as usize,
                            band.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                        k = bend;
                    }
                }
            }"""
    new_xor = """            let mb = usize::from(mark_byte);
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                if a != mb {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor_nomark(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                        );
                    }
                } else {
                    for &band in &self.bands[cursors[a]..end] {
                        queue.wire_xor(
                            edges,
                            band.dst() as usize,
                            band.end() as usize,
                            band.code() << bits,
                            mark,
                            &mut predecessor,
                        );
                    }
                }
            }"""
    assert old_xor in s
    s = s.replace(old_xor, new_xor, 1)

    old_ser = """        } else if QUEUE {
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
            }"""
    new_ser = """        } else if QUEUE {
            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                for &b in &self.bands[cursors[a]..end] {
                    let span = b.end() as usize;
                    let dst = b.dst() as usize;
                    let bcode = b.code();
                    let pack = if wide { 0 } else { bcode << bits };
                    if wide {
                        self.extra[dst..dst + span].fill(bcode);
                    }
                    queue.wire(&mut self.edges[dst..dst + span], pack);
                }
            }"""
    assert old_ser in s
    s = s.replace(old_ser, new_ser, 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(ep_p, "w") as f:
        f.write(e)

d = clone_from(BASE, "cand_c42_pair2");  patch_pair2(d)
d = clone_from(BASE, "cand_c42_s3arr");  patch_s3_arr(d)
d = clone_from(BASE, "cand_c42_bandw");  patch_band_width(d)
d = clone_from(BASE, "cand_c42_combo");  patch_pair2(d); patch_s3_arr(d); patch_band_width(d)
d = clone_from(BASE_NOBI, "cand_c42_nobi"); patch_pair2(d); patch_s3_arr(d); patch_band_width(d)

print("Generated 5 Cycle 42 candidates successfully!")
