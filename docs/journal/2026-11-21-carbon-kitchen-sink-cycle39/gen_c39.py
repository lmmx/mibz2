import os, shutil

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c38_inl2"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

def make_nobi(dst):
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    assert "if QUEUE && !wide && m >= 65_536 {" in s
    s = s.replace("if QUEUE && !wide && m >= 65_536 {", "if false && QUEUE && !wide && m >= 65_536 {", 1)
    with open(mod_p, "w") as f:
        f.write(s)

def patch_nostats(dst):
    """Add `const STATS: bool` specialization: `false` in production hot path (OpenCells discards Counts!), `true` for tests."""
    mod_p = f"{dst}/mod.rs"
    ep_p = f"{dst}/endpoint.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(ep_p) as f:
        e = f.read()

    # In S1 (Cap127 thin path): skip pass.f.runs += 1 when !STATS (keep in Pass::take for thick runs, negligible 4.7%)
    s = s.replace(
        "pass.f.runs += 1;\n                    pass.piece::<QUEUE>(c, pos, width, true, p);",
        "pass.piece::<QUEUE>(c, pos, width, true, p);",
        1,
    )
    # In S2b (eliminate): skip `taken += 1` in non-spiral loop (keep `stats.steps` only when STATS=true — here gated by const STATS=false!)
    e = e.replace("            taken += 1;\n", "")
    # In S3a (induced_with): remove the 6 per-band Counts updates + width*height multiply + SATURATED branch!
    old_stats = """                if node_word != 0 {
                    let grade = node_word >> ID_BITS;
                    let height = if grade == SATURATED {
                        e.len_handle(node_word)
                    } else {
                        grade
                    } as usize;
                    stats.words += 1;
                    stats.copies += width as usize;
                    stats.traced += height;
                    stats.thick_area += width as usize * height;
                    stats.unit_work += usize::from(width == 1) * height;
                    stats.longest = stats.longest.max(height);
                }"""
    assert old_stats in e
    e = e.replace(old_stats, "", 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_s3fast(dst):
    """
    1. Branchless Inline2 `uid`: offset rule_id by 65_792 so `uid = id + (u32::from(g == 2) << 8)` (0 branches!).
    2. Single-slice node fetch (`Links::read_band_node`) + hoist node fields across byte boundaries in S3a!
    """
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    # 1. Offset rule_id by 65_792 so rule_id never overlaps 0..65_792!
    old_rule = """    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + id as usize * RULE
    }"""
    new_rule = """    #[inline(always)]
    fn rule(&self, id: u32) -> usize {
        self.base + (id - 65_792) as usize * RULE
    }"""
    assert old_rule in e
    e = e.replace(old_rule, new_rule, 1)

    old_id = "let id = ((self.top - self.base) / RULE) as u32;"
    new_id = "let id = ((self.top - self.base) / RULE) as u32 + 65_792;"
    assert old_id in e
    e = e.replace(old_id, new_id, 1)

    old_uid = "let uid = if g == 1 { id } else if g == 2 { 256 + id } else { 65_792 + id };"
    new_uid = "let uid = id + (u32::from(g == 2) << 8);"
    assert old_uid in e
    e = e.replace(old_uid, new_uid, 1)

    # Fix unused variable warning in inline_word too
    e = e.replace("        let id = word & ID_MASK;\n        let height = (word >> ID_BITS) as usize;", "        let height = (word >> ID_BITS) as usize;", 1)

    # 2. Add Links::band_node(a, id) -> (next0, bottom, width, word) with ONE slice load per node,
    #    and cache the current node's fields across byte boundaries in S3a!
    e = e.replace(
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);",
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);\n    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32);",
        1,
    )
    e = e.replace(
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }""",
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 4..id as usize * 4 + 4];
        (sl[0] >> 16, sl[1], sl[2], sl[3])
    }""",
        1,
    )
    e = e.replace(
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }""",
        """    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn band_node(a: &[u32], id: u32) -> (u32, u32, u32, u32) {
        let sl = &a[id as usize * 6..id as usize * 6 + 6];
        (sl[1], sl[2], sl[4], sl[5])
    }""",
        1,
    )

    old_s3_loop = """        let mut id = L::next(e.a, 0, 0);
        let mut in_node = 0;
        let mut k = 0;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let node_width = e.width(id);
                let node_word = e.word(id);
                let width = (end - k).min(node_width - in_node);
                // (the port's) every band takes a row at least, so the loop always ends; a wrong
                // construction panics here rather than looping (the mutation check's finding)
                assert!(width > 0);
                let dst = e.bottom(id) + in_node;"""

    new_s3_loop = """        let mut id = L::next(e.a, 0, 0);
        let (mut next_id, mut node_bottom, mut node_width, mut node_word) = if id != 0 {
            L::band_node(e.a, id)
        } else {
            (0, 0, 0, 0)
        };
        let mut in_node = 0u32;
        let mut k = 0u32;
        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {
                let width = (end - k).min(node_width - in_node);
                assert!(width > 0);
                let dst = node_bottom + in_node;"""
    assert old_s3_loop in e
    e = e.replace(old_s3_loop, new_s3_loop, 1)

    old_s3_step = """                if in_node == node_width {
                    id = L::next(e.a, id, 0);
                    in_node = 0;
                }"""
    new_s3_step = """                if in_node == node_width {
                    id = next_id;
                    in_node = 0;
                    if id != 0 {
                        (next_id, node_bottom, node_width, node_word) = L::band_node(e.a, id);
                    }
                }"""
    assert old_s3_step in e
    e = e.replace(old_s3_step, new_s3_step, 1)

    with open(ep_p, "w") as f:
        f.write(e)

def patch_wire_serial_u64(dst):
    """8-wide u64-unpacked serial OutQ16::wire for the nobi (mu=1) regime."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
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
    with open(outq_p, "w") as f:
        f.write(o)

d = clone("cand_c39_nostats");    patch_nostats(d)
d = clone("cand_c39_s3fast");     patch_s3fast(d)
d = clone("cand_c39_combo");      patch_nostats(d); patch_s3fast(d)
d = clone("cand_c39_nobi_c38");   make_nobi(d)
d = clone("cand_c39_nobi_combo"); make_nobi(d); patch_nostats(d); patch_s3fast(d)
d = clone("cand_c39_nobi_u64w");  make_nobi(d); patch_nostats(d); patch_s3fast(d); patch_wire_serial_u64(d)

print("Generated 6 Cycle 39 candidates successfully!")
