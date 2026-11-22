import os, shutil

ROOT = "/tmp/cks_bench"
BASE = f"{ROOT}/cand_c33_w16_d12"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def clean_endpoint_warnings(dst):
    ep = f"{dst}/endpoint.rs"
    with open(ep) as f:
        s = f.read()
    # Allow dead_code only on test-only / fallback helpers if needed, or remove unused len()/short_u128()
    # Check if len() or short_u128() are used in fallback (!can_dedup): yes, lines 651, 654!
    # Replace lines 651-654 in !can_dedup with fetch_word so len() and short_u128() can be removed cleanly!
    old_fb = """                        let h_len = e.len(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));"""
    new_fb = """                        let (base_v, h_len) = e.fetch_word(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = base_v | (u128::from(a as u8) << (8 * h_len));"""
    assert old_fb in s
    s = s.replace(old_fb, new_fb, 1)

    # Remove now-unused len() and short_u128() methods
    old_unused = """    #[inline(always)]
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
    }"""
    assert old_unused in s
    s = s.replace(old_unused, "", 1)

    # Suppress warning on CERTIFIED_WIDTH if only used internally
    s = s.replace("pub(super) const CERTIFIED_WIDTH: u32 = 32_768;", "const CERTIFIED_WIDTH: u32 = 32_768;", 1)

    with open(ep, "w") as f:
        f.write(s)

# 1. cand_neon_bi: spotless zero-warning Neon-Bi (mu=2)
bi = clone("cand_neon_bi")
clean_endpoint_warnings(bi)

# Clean unused constants/methods in mod.rs and outq.rs of cand_neon_bi
with open(f"{bi}/mod.rs") as f:
    s = f.read()
s = s.replace("const DENSE: (usize, usize) = (7, 25);", "#[allow(dead_code)]\nconst DENSE: (usize, usize) = (7, 25);", 1)
s = s.replace("fn each_run(", "#[allow(dead_code)]\nfn each_run(", 1)
s = s.replace("pub fn spelling_stats(", "#[allow(dead_code)]\n    pub fn spelling_stats(", 1)
# Keep public run_with_limit helper for exhaustive limit testing
extra_api = """    #[allow(dead_code)]
    pub fn run_limit(&mut self, records: &[u16], n: usize, p: u32, limit: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, limit, cap, out)
    }
"""
s = s.replace("    pub fn run_entry(", extra_api + "    pub fn run_entry(", 1)
with open(f"{bi}/mod.rs", "w") as f:
    f.write(s)

with open(f"{bi}/outq.rs") as f:
    o = f.read()
# Remove unused peek_first in cand_neon_bi (was added for QuadEmit)
peek_block = """    #[inline(always)]
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
    }"""
assert peek_block in o
o = o.replace(peek_block, "", 1)
with open(f"{bi}/outq.rs", "w") as f:
    f.write(o)

# 2. cand_neon_diet: spotless zero-warning Neon-Diet (mu=1)
diet = clone("cand_neon_diet")
clean_endpoint_warnings(diet)

with open(f"{diet}/mod.rs") as f:
    s = f.read()
s = s.replace("const DENSE: (usize, usize) = (7, 25);", "#[allow(dead_code)]\nconst DENSE: (usize, usize) = (7, 25);", 1)
s = s.replace("fn each_run(", "#[allow(dead_code)]\nfn each_run(", 1)
s = s.replace("pub fn spelling_stats(", "#[allow(dead_code)]\n    pub fn spelling_stats(", 1)
s = s.replace("    pub fn run_entry(", extra_api + "    pub fn run_entry(", 1)
# Disable XOR wiring cleanly without dead-code warnings
s = s.replace("const XOR_GATE: u32 = 65_536;", "const XOR_GATE: u32 = u32::MAX;", 1)
s = s.replace("if m >= 65_536 && self.extra.is_empty()", "if false && self.extra.is_empty()", 1)
s = s.replace("fn emit_xor_bi_no_off(", "#[allow(dead_code)]\nfn emit_xor_bi_no_off(", 1)
with open(f"{diet}/mod.rs", "w") as f:
    f.write(s)

with open(f"{diet}/outq.rs") as f:
    o = f.read()
assert peek_block in o
o = o.replace(peek_block, "", 1)
o = o.replace("    #[inline(always)]\n    pub(super) fn wire_xor_nomark(", "    #[allow(dead_code)]\n    #[inline(always)]\n    pub(super) fn wire_xor_nomark(", 1)
o = o.replace("    #[inline(always)]\n    pub(super) fn wire_xor_mark(", "    #[allow(dead_code)]\n    #[inline(always)]\n    pub(super) fn wire_xor_mark(", 1)
o = o.replace("    #[inline(always)]\n    pub(super) fn wire_xor(", "    #[allow(dead_code)]\n    #[inline(always)]\n    pub(super) fn wire_xor(", 1)
o = o.replace("    #[cold]\n    #[inline(never)]\n    fn wire_xor_slow(", "    #[allow(dead_code)]\n    #[cold]\n    #[inline(never)]\n    fn wire_xor_slow(", 1)
with open(f"{diet}/outq.rs", "w") as f:
    f.write(o)

print("Generated spotless zero-warning cand_neon_diet and cand_neon_bi!")
