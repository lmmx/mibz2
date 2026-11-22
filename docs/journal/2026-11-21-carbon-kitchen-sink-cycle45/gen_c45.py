import os, shutil
from gen_c44 import patch_s2b_noinline, patch_s3a_compact8, add_c44_profiler
from gen_c43 import patch_wire_serial16

ROOT = "/tmp/cks_c36"
BASE_HEAD = f"{ROOT}/cand_c43_base"
BASE_NOBI = f"{ROOT}/cand_c43_nb42"

def clone_from(src, name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    return dst

def patch_wire_serial_simd(dst):
    """Restore origin/master's auto-vectorized `chunks_exact(2)` serial wire (keeping BandW!)."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_block = """            let bytes = &self.out[at..at + 2 * span];
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
    new_block = """            for (e, s) in dst
                .iter_mut()
                .zip(self.out[at..at + 2 * span].chunks_exact(2))
            {
                *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
            }"""
    assert old_block in o, "old_block not found in outq.rs"
    o = o.replace(old_block, new_block, 1)
    with open(outq_p, "w") as f:
        f.write(o)

def fix_duplicate_inline_attr(dst):
    """Ensure zero warnings under -D warnings on prof_master / prof_nobi."""
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    s = s.replace("    #[inline(never)]\n    #[inline(never)]\n", "    #[inline(never)]\n")
    with open(mod_p, "w") as f:
        f.write(s)

def patch_fix_inline2_zero_key(dst):
    """
    Theorem 45.1 (Zero-Branch Injective Inline2 Key Space):
    Fix 0x00-byte key==0 collision in `Dictionary::code` caught by the Cycle 45
    n=70,000 arbitrary-byte audit:
    - `g == 1` (literal b in 0..=255): uid = id + 1 in 1..=256 (strictly non-zero!)
    - `g == 2` (Inline2 (b0,b1) in 0..=65535): uid = id + 257 in 257..=65792
    - `g >= 3` (rules): base offset 65_793 -> uid = id + 1 >= 65_794
    Guarantees `key = (uid << 8) | byte >= 256 > 0` (never collides with `pairs`'s 0 empty sentinel)
    and 100% injective across all 256 bytes 0x00..=0xFF!
    """
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()
    old_uid = "        let uid = id + (u32::from(g == 2) << 8);"
    new_uid = "        let uid = id + 1 + (u32::from(g == 2) << 8);"
    assert old_uid in e, "old_uid not found"
    e = e.replace(old_uid, new_uid, 1)
    assert e.count("65_792") == 3, f"expected 3 occurrences of 65_792, found {e.count('65_792')}"
    e = e.replace("65_792", "65_793")
    with open(ep_p, "w") as f:
        f.write(e)

# 1. Clean prof_master and prof_nobi of duplicate #[inline(never)] warnings
fix_duplicate_inline_attr(f"{ROOT}/prof_master")
fix_duplicate_inline_attr(f"{ROOT}/prof_nobi")

# 2. Assemble final Round-5 drop-in champions (all promoted winners, NO S2aTunnel, NO Pair2!):
#    S1: Cap127 + Pt64 + S1Trim + OneBr
#    S2a: S2aFuse
#    S2b: Inline2 (Theorem 45.1 injective key fix) + FastJoin + S2b-NI (#[inline(never)] eliminate_narrow_plain) + NoStats
#    S3a: S3Tag12 + S3Fast + S3Arr + S3a-C8 (10 KiB HT + bands.reserve_exact(8192)) + NoStats
#    S4: BandW + Wire16 (head) / W16 vs SIMD chunks_exact(2) (nobi)
d = clone_from(BASE_HEAD, "cand_r5_head")
patch_s2b_noinline(d)
patch_s3a_compact8(d)
patch_fix_inline2_zero_key(d)

d = clone_from(BASE_NOBI, "cand_r5_nobi_w16")
patch_wire_serial16(d)
patch_s2b_noinline(d)
patch_s3a_compact8(d)
patch_fix_inline2_zero_key(d)

d = clone_from(BASE_NOBI, "cand_r5_nobi_simd")
patch_wire_serial_simd(d)
patch_s2b_noinline(d)
patch_s3a_compact8(d)
patch_fix_inline2_zero_key(d)

# 3. 6-stage profiled clones for final Round-5 gap-closure table:
p = clone_from(f"{ROOT}/cand_r5_head", "prof_r5_head");           add_c44_profiler(p)
p = clone_from(f"{ROOT}/cand_r5_nobi_w16", "prof_r5_nobi_w16");   add_c44_profiler(p)
p = clone_from(f"{ROOT}/cand_r5_nobi_simd", "prof_r5_nobi_simd"); add_c44_profiler(p)

print("Generated Round-5 Grand Synthesis modules + 6-stage profilers successfully!")
