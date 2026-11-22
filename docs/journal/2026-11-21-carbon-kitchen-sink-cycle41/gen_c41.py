import os, shutil

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c40_trim"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def make_nobi_w8(dst):
    mod_p = f"{dst}/mod.rs"
    outq_p = f"{dst}/outq.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(outq_p) as f:
        o = f.read()
    assert "if QUEUE && !wide && m >= 65_536 {" in s
    s = s.replace("if QUEUE && !wide && m >= 65_536 {", "if false && QUEUE && !wide && m >= 65_536 {", 1)
    old_w = """            for (e, s) in dst
                .iter_mut()
                .zip(self.out[at..at + 2 * span].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }"""
    new_w = """            let bytes = &self.out[at..at + 2 * span];
            let mut i = 0usize;
            while i + 8 <= span {
                let b = &bytes[2 * i..2 * i + 16];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let d = &mut dst[i..i + 8];
                d[0] = hi | (q0 as u32 & 0xFFFF);
                d[1] = hi | ((q0 >> 16) as u32 & 0xFFFF);
                d[2] = hi | ((q0 >> 32) as u32 & 0xFFFF);
                d[3] = hi | ((q0 >> 48) as u32);
                d[4] = hi | (q1 as u32 & 0xFFFF);
                d[5] = hi | ((q1 >> 16) as u32 & 0xFFFF);
                d[6] = hi | ((q1 >> 32) as u32 & 0xFFFF);
                d[7] = hi | ((q1 >> 48) as u32);
                i += 8;
            }
            while i < span {
                dst[i] = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                i += 1;
            }"""
    assert old_w in o
    o = o.replace(old_w, new_w, 1)
    with open(mod_p, "w") as f:
        f.write(s)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_iota8(dst):
    """Single-path 16-byte XMM iota8 (`w <= 8`, ~97.6% fast path, half write traffic & half cache-line splits!)."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_push = """    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let m_next = m + w;
        if w <= 16 && f + 16 <= CHUNK && (m ^ m_next) < EPOCH {
            let at = p as usize + f;
            if let Some(dst) = self.out.get_mut(at..at + 32) {
                let lo = m as u16;
                let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&x.to_le_bytes());
                }
                let wu = u64::from(w);
                self.pt[a] = v + ((wu << 32) | wu);
                return;
            }
        }
        self.push_slow_pt(a, m, w);
    }"""
    new_push = """    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let m_next = m + w;
        if w <= 8 && f + 8 <= CHUNK && (m ^ m_next) < EPOCH {
            let at = p as usize + f;
            if let Some(dst) = self.out.get_mut(at..at + 16) {
                let lo = m as u16;
                let iota: [u16; 8] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&x.to_le_bytes());
                }
                let wu = u64::from(w);
                self.pt[a] = v + ((wu << 32) | wu);
                return;
            }
        }
        self.push_slow_pt(a, m, w);
    }"""
    assert old_push in o
    o = o.replace(old_push, new_push, 1)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_iota4(dst):
    """Single-path 8-byte GPR SWAR iota4 (`w <= 4`, ~92.9% fast path, ZERO SIMD registers!)."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_push = """    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let m_next = m + w;
        if w <= 16 && f + 16 <= CHUNK && (m ^ m_next) < EPOCH {
            let at = p as usize + f;
            if let Some(dst) = self.out.get_mut(at..at + 32) {
                let lo = m as u16;
                let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                for (d, x) in dst.chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&x.to_le_bytes());
                }
                let wu = u64::from(w);
                self.pt[a] = v + ((wu << 32) | wu);
                return;
            }
        }
        self.push_slow_pt(a, m, w);
    }"""
    new_push = """    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let lo = m as u16;
        if w <= 4 && f + 4 <= CHUNK && lo <= 65_531 {
            let at = p as usize + f;
            if let Some(dst) = self.out.get_mut(at..at + 8) {
                let pack = u64::from(lo)
                    .wrapping_mul(0x0001_0001_0001_0001u64)
                    .wrapping_add(0x0003_0002_0001_0000u64);
                dst.copy_from_slice(&pack.to_le_bytes());
                let wu = u64::from(w);
                self.pt[a] = v + ((wu << 32) | wu);
                return;
            }
        }
        self.push_slow_pt(a, m, w);
    }"""
    assert old_push in o
    o = o.replace(old_push, new_push, 1)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_onebr(dst):
    """Combine `f + 16 <= CHUNK` and `(m as u16) < 65_520` into one single branch in push_thin."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_cond = "        let m_next = m + w;\n        if w <= 16 && f + 16 <= CHUNK && (m ^ m_next) < EPOCH {"
    new_cond = "        if (w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520) {"
    assert old_cond in o
    o = o.replace(old_cond, new_cond, 1)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_fastjoin(dst):
    """Algebraic single-check `join`: `sum < 16` skips SATURATED checks; `sum <= SATURATED` replaces 2 SATURATED checks!"""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()
    old_join = """    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
        let len = if left_len < SATURATED && right_len < SATURATED {
            left_len + right_len
        } else {
            self.len_handle(left) + self.len_handle(right)
        };
        if len == 2 {
            // Zero rule allocation: pack 2-byte bigram directly into 22-bit handle payload!
            return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
        }
        let id = ((self.top - self.base) / RULE) as u32 + 65_792;
        let rule = if len < INLINE as u32 {
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(len) << 120);
            [
                bytes as u32,
                (bytes >> 32) as u32,
                (bytes >> 64) as u32,
                (bytes >> 96) as u32,
            ]
        } else {
            [left, right, len, u32::MAX]
        };
        self.a[self.top..self.top + RULE].copy_from_slice(&rule);
        self.top += RULE;
        (len.min(SATURATED) << ID_BITS) | id
    }"""
    new_join = """    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let left_len = left >> ID_BITS;
        let sum = left_len + (right >> ID_BITS);
        if sum < INLINE as u32 {
            if sum == 2 {
                return (2 << ID_BITS) | (left & 0xFF) | ((right & 0xFF) << 8);
            }
            let id = ((self.top - self.base) / RULE) as u32 + 65_792;
            let bytes = self.short_payload_h(left)
                | (self.short_payload_h(right) << (8 * left_len))
                | (u128::from(sum) << 120);
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = bytes as u32;
            out[1] = (bytes >> 32) as u32;
            out[2] = (bytes >> 64) as u32;
            out[3] = (bytes >> 96) as u32;
            self.top += RULE;
            (sum << ID_BITS) | id
        } else {
            // Since left_len, right_len >= 1, `sum <= SATURATED` implies BOTH `< SATURATED`!
            let len = if sum <= SATURATED {
                sum
            } else {
                self.len_handle(left) + self.len_handle(right)
            };
            let id = ((self.top - self.base) / RULE) as u32 + 65_792;
            let out = &mut self.a[self.top..self.top + RULE];
            out[0] = left;
            out[1] = right;
            out[2] = len;
            out[3] = u32::MAX;
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
    }"""
    assert old_join in e
    e = e.replace(old_join, new_join, 1)
    with open(ep_p, "w") as f:
        f.write(e)

d = clone("cand_c41_iota8");    patch_iota8(d)
d = clone("cand_c41_iota4");    patch_iota4(d)
d = clone("cand_c41_onebr");    patch_onebr(d)
d = clone("cand_c41_fastjoin"); patch_fastjoin(d)
d = clone("cand_c41_combo");    patch_onebr(d); patch_fastjoin(d)
d = clone("cand_c41_nobi40");   make_nobi_w8(d)
d = clone("cand_c41_nobi41");   make_nobi_w8(d); patch_onebr(d); patch_fastjoin(d)

print("Generated 7 Cycle 41 candidates successfully!")
