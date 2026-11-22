import os, shutil
from gen_c36 import add_6stage_profiler
from gen_c43 import patch_s2b_narrow_slice, patch_wire_serial16

ROOT = "/tmp/cks_c36"
BASE_HEAD = f"{ROOT}/cand_c43_base"
BASE_NOBI = f"{ROOT}/cand_c43_nb42"

def clone_from(src, name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)
    return dst

def patch_s2a_tunnel(dst):
    """
    Theorem 44.1 (S2aTunnel — Dead-Link Tunneling of `to` into `L::BOTTOM`):
    - Pass 1 becomes READ-ONLY on `strips_raw` (no `sw[1] += first[a]` dirty writebacks!)
      and stores `to = sw[1] + first[a]` into `sid`'s dead side-1 slot `L::BOTTOM`
      (already hot & dirty in L1d alongside `width` and `word`!).
    - Pass 2 reads `(to, w) = (e.bottom(sid), e.width(sid))` directly from node `sid`
      (zero random reads of the 288 KiB `strips_raw` array!) and clears `L::BOTTOM`
      before `append_to(&mut order[1], sid, 1)`.
    """
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    old_s2a = """        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            sw[1] += first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
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

    new_s2a = """        for i in 0..ns {
            let sw = &strips_raw[3 * i..3 * i + 3];
            let from = sw[0];
            let wbyte = sw[2];
            let byte = (wbyte & 0xFF) as u8;
            let width = wbyte >> 8;
            let a = usize::from(byte);
            let to = sw[1] + first[a];
            let dst_idx = write[a] as usize;
            image_order[dst_idx] = i as u32;
            write[a] = dst_idx as u32 + 1;
            let sid = i as u32 + 1;
            e.set_bottom(sid, to);
            e.set_width(sid, width);
            e.set_word(sid, literal(byte));
            widest = widest.max(width);
            if x < from {
                let gw = from - x;
                widest = widest.max(gw);
                let gid = e.node(gw, 0);
                e.append_to(&mut order[0], gid, 0);
                e.append_to(&mut terminals[1], gid, 1);
            }
            e.append_to(&mut order[0], sid, 0);
            x = from + width;
        }
        if x < n {
            widest = widest.max(n - x);
            let id = e.node(n - x, 0);
            e.append_to(&mut order[0], id, 0);
            e.append_to(&mut terminals[1], id, 1);
        }
        x = 0;
        for &i in image_order.iter() {
            let sid = i + 1;
            let to = e.bottom(sid);
            let width = e.width(sid);
            e.set_bottom(sid, 0);
            if x < to {
                widest = widest.max(to - x);
                let id = e.node(to - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            e.append_to(&mut order[1], sid, 1);
            x = to + width;
        }"""

    assert old_s2a in e, "old_s2a not found"
    e = e.replace(old_s2a, new_s2a, 1)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_s2b_noinline(dst):
    """Apply S2b-NarrowSlice with explicit #[inline(never)] for deterministic 32B-aligned codegen."""
    patch_s2b_narrow_slice(dst)
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()
    old_fn = "    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {"
    new_fn = "    #[inline(never)]\n    fn eliminate_narrow_plain(&mut self, remaining: &mut u32) -> bool {"
    assert old_fn in e
    e = e.replace(old_fn, new_fn, 1)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_s3a_compact8(dst):
    """Test 8-bit pairs HT (2 KiB pairs + 8 KiB contents = 10 KiB HT) + bands.reserve_exact(8192) on n >= 65_536."""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()
    assert "pairs: &'a mut [u32; 1024]," in e
    e = e.replace("pairs: &'a mut [u32; 1024],", "pairs: &'a mut [u32; 512],", 1)
    e = e.replace("key.wrapping_mul(0x9e37_79b9) >> 23", "key.wrapping_mul(0x9e37_79b9) >> 24", 1)
    e = e.replace("const PAIR_WORDS: usize = 1024;", "const PAIR_WORDS: usize = 512;", 1)
    e = e.replace("let pairs: &mut [u32; 1024] = p_sl.try_into().unwrap();", "let pairs: &mut [u32; 512] = p_sl.try_into().unwrap();", 1)
    with open(ep_p, "w") as f:
        f.write(e)
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        m = f.read()
    old_bc = "        self.bands.clear();"
    new_bc = "        self.bands.clear();\n        if n >= 65_536 && self.bands.capacity() < 8192 { self.bands.reserve_exact(8192); }"
    assert old_bc in m
    m = m.replace(old_bc, new_bc, 1)
    with open(mod_p, "w") as f:
        f.write(m)

def add_c44_profiler(dst):
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

# Build Cycle 44 candidates:
d = clone_from(BASE_HEAD, "cand_c44_tunnel"); patch_s2a_tunnel(d)
d = clone_from(BASE_HEAD, "cand_c44_s2bni");  patch_s2b_noinline(d)
d = clone_from(BASE_HEAD, "cand_c44_s3c8");   patch_s3a_compact8(d)
d = clone_from(BASE_HEAD, "cand_c44_head");   patch_s2a_tunnel(d); patch_s2b_noinline(d)
d = clone_from(BASE_NOBI, "cand_c44_nobi");   patch_wire_serial16(d); patch_s2a_tunnel(d); patch_s2b_noinline(d)

# Build 6-stage profiled clones for exact stage-by-stage gap audit!
p = clone_from(BASE_HEAD, "prof_c44_base");               add_c44_profiler(p)
p = clone_from(f"{ROOT}/cand_c44_head", "prof_c44_head"); add_c44_profiler(p)
p = clone_from(f"{ROOT}/cand_c44_nobi", "prof_c44_nobi"); add_c44_profiler(p)

print("Generated Cycle 44 candidates + 6-stage profilers successfully!")
