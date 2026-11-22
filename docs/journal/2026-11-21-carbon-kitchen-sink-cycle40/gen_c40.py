import os, shutil
from gen_c36 import add_6stage_profiler

ROOT = "/tmp/cks_c36"
BASE = f"{ROOT}/cand_c39_combo"
BASE_NOBI = f"{ROOT}/cand_c39_nobi_u64w"

def clone_from(src, name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    return dst

def patch_s1_trim(dst):
    """Exact-trimmed `out` slice + single `get_mut(at..at+32)` check (eliminating `in_out_lim` + redundant bounds check) + `(m ^ m_next) < EPOCH`."""
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()

    o = o.replace("    in_out: u32,\n    in_out_lim: u32,\n", "    in_out: u32,\n", 1)
    old_new = """        let in_out = (out.len() / BYTES) as u32;
        Queues {
            in_out,
            in_out_lim: in_out << 10,
            out,"""
    new_new = """        let in_out = (out.len() / BYTES) as u32;
        let out = &mut out[..in_out as usize * BYTES];
        Queues {
            in_out,
            out,"""
    assert old_new in o
    o = o.replace(old_new, new_new, 1)

    old_push = """    #[inline(always)]
    pub(super) fn push_thin(&mut self, a: usize, m: u32, w: u32) {
        let v = self.pt[a];
        let p = v as u32;
        let f = (p & 1023) as usize;
        let lo = m & (EPOCH - 1);
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, x) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&x.to_le_bytes());
            }
            let wu = u64::from(w);
            self.pt[a] = v + ((wu << 32) | wu);
        } else {
            self.push_slow_pt(a, m, w);
        }
    }"""
    new_push = """    #[inline(always)]
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
    assert old_push in o
    o = o.replace(old_push, new_push, 1)

    with open(outq_p, "w") as f:
        f.write(o)

def patch_cold_thick(dst):
    """Outline the entire 4.7% thick-run + width==127 continuation path into #[cold] #[inline(never)] to keep S1 hot loop at ~9 live GPRs!"""
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()

    helper = """    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn take_thick_cold<const QUEUE: bool>(
        &mut self,
        records: &[u16],
        i: &mut usize,
        c: u8,
        pos: u32,
        mut width: u32,
        p: u32,
        limit: u32,
    ) -> u32 {
        if width == 127 {
            let mut idx = *i;
            let nrec = records.len();
            while idx < nrec && (records[idx] >> 8) as u8 == c {
                width += u32::from(records[idx] & 255);
                idx += 1;
            }
            *i = idx;
        }
        if p.wrapping_sub(pos) >= width {
            self.piece::<QUEUE>(c, pos, width, false, p);
        } else {
            self.take_marked::<QUEUE>(c, pos, width, p, limit);
        }
        pos + width
    }
"""
    anchor = "    /// `take` on the thick run holding the marked row"
    assert anchor in s
    s = s.replace(anchor, helper + "\n" + anchor, 1)

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
            while i < nrec {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let width = u32::from(rec & 255);
                i += 1;
                if width <= limit {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        pass.f.mark = pass.f.m + (p - pos);
                        pass.f.mark_byte = c;
                    }
                    if QUEUE {
                        pass.queues.push_thin(a, pass.f.m, width);
                    } else {
                        pass.f.thin[a] += width;
                    }
                    pass.f.m += width;
                    pos += width;
                } else {
                    pos = pass.take_thick_cold::<QUEUE>(records, &mut i, c, pos, width, p, limit);
                }
            }
        }"""
    assert old_loop in s
    s = s.replace(old_loop, new_loop, 1)

    with open(mod_p, "w") as f:
        f.write(s)

def patch_s2a_slim(dst):
    """Slim S2a Pass 2 (read only to & width) + single-store Narrow::set_links in Elim::insert."""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    old_p2 = """        for &i in image_order.iter() {
            let idx = i as usize;
            let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
            if x < s.to {
                widest = widest.max(s.to - x);
                let id = e.node(s.to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = s.to + s.width;
        }"""
    new_p2 = """        for &i in image_order.iter() {
            let idx = i as usize * 3;
            let to = strips_raw[idx + 1];
            let width = strips_raw[idx + 2] >> 8;
            if x < to {
                let gw = to - x;
                widest = widest.max(gw);
                let id = e.node(gw, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], i + 1, 1);
            x = to + width;
        }"""
    assert old_p2 in e
    e = e.replace(old_p2, new_p2, 1)

    e = e.replace(
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);",
        "fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);\n    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32);",
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
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 4 + side] = prev | (next << 16);
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
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 6 + 2 * side] = prev;
        a[id as usize * 6 + 2 * side + 1] = next;
    }""",
        1,
    )
    e = e.replace(
        """        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);""",
        "        L::set_links(self.a, id, side, at, next);",
        1,
    )
    with open(ep_p, "w") as f:
        f.write(e)

def add_c40_profiler(dst):
    """Add 6-stage profiler hooks compatible with Cap127/ColdThick/S2aFuse."""
    mod_p = f"{dst}/mod.rs"
    ep_p = f"{dst}/endpoint.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(ep_p) as f:
        e = f.read()

    s = s.replace("lens: Vec<u8>,\n}", "lens: Vec<u8>,\n    pub stage_us: [f64; 6],\n}", 1)
    s = s.replace("        out.resize(n as usize + INLINE, 0);", "        let t_s1 = std::time::Instant::now();\n        out.resize(n as usize + INLINE, 0);", 1)
    s = s.replace("        let (heads, over) = (pass.queues.heads(), pass.queues.over());", "        let (heads, over) = (pass.queues.heads(), pass.queues.over());\n        self.stage_us[0] = t_s1.elapsed().as_secs_f64() * 1e6;", 1)
    s = s.replace("        let mut stats = Counts {", "        let t_s2a = std::time::Instant::now();\n        let mut stats = Counts {", 1)
    s = s.replace("        let mut cursors = if INDUCE {", "        self.stage_us[1] = t_s2a.elapsed().as_secs_f64() * 1e6;\n        let mut cursors = if INDUCE {", 1)
    s = s.replace("        self.predecessor = None;", "        let t_s4 = std::time::Instant::now();\n        self.predecessor = None;", 1)
    s = s.replace("        (stats, mark, bits)", "        self.stage_us[4] = t_s4.elapsed().as_secs_f64() * 1e6;\n        (stats, mark, bits)", 1)
    prof_api = """    #[inline(never)]
    pub fn run_profile6(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> [f64; 6] {
        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let t5 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);
        self.stage_us[5] = t5.elapsed().as_secs_f64() * 1e6;
        self.stage_us
    }
"""
    s = s.replace("    #[inline(never)]\n    pub fn run_entry(", prof_api + "    #[inline(never)]\n    pub fn run_entry(", 1)

    e = e.replace("            inline16,\n            lens,\n            ..\n        } = self;", "            inline16,\n            lens,\n            stage_us,\n            ..\n        } = self;\n        let t_s2a_ep = std::time::Instant::now();", 1)
    e = e.replace("        let (mut remaining, mut steps) = (n, 0);", "        stage_us[1] += t_s2a_ep.elapsed().as_secs_f64() * 1e6;\n        let t_s2b = std::time::Instant::now();\n        let (mut remaining, mut steps) = (n, 0);", 1)
    e = e.replace("        e.assign_bottom(stats.retained as u32);", "        stage_us[2] = t_s2b.elapsed().as_secs_f64() * 1e6;\n        let t_s3a = std::time::Instant::now();\n        e.assign_bottom(stats.retained as u32);", 1)
    e = e.replace("        assert_eq!(id, 0);\n        cursors", "        assert_eq!(id, 0);\n        stage_us[3] = t_s3a.elapsed().as_secs_f64() * 1e6;\n        cursors", 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(ep_p, "w") as f:
        f.write(e)

d = clone_from(BASE, "prof_c39");       add_c40_profiler(d)
d = clone_from(BASE_NOBI, "prof_c39_nobi"); add_c40_profiler(d)
d = clone_from(BASE, "cand_c40_trim");  patch_s1_trim(d)
d = clone_from(BASE, "cand_c40_cold");  patch_cold_thick(d)
d = clone_from(BASE, "cand_c40_s2a");   patch_s2a_slim(d)
d = clone_from(BASE, "cand_c40_combo"); patch_s1_trim(d); patch_cold_thick(d); patch_s2a_slim(d)
d = clone_from(BASE, "prof_c40");       patch_s1_trim(d); patch_cold_thick(d); patch_s2a_slim(d); add_c40_profiler(d)
d = clone_from(BASE_NOBI, "cand_c40_nobi"); patch_s1_trim(d); patch_cold_thick(d); patch_s2a_slim(d)
d = clone_from(BASE_NOBI, "prof_c40_nobi"); patch_s1_trim(d); patch_cold_thick(d); patch_s2a_slim(d); add_c40_profiler(d)

print("Generated all Cycle 40 candidates and 6-stage profilers!")
