import os, shutil, subprocess

ROOT = "/tmp/cks_c36"

def extract_master(dst):
    if os.path.exists(dst):
        shutil.rmtree(dst)
    os.makedirs(dst)
    for src, fn in [
        ("origin/master:src/open.rs", "mod.rs"),
        ("origin/master:src/open/endpoint.rs", "endpoint.rs"),
        ("origin/master:src/open/outq.rs", "outq.rs"),
    ]:
        data = subprocess.check_output(
            ["git", "show", src], cwd="/home/antigravity/carbon-kitchen-sink"
        ).decode()
        with open(f"{dst}/{fn}", "w") as f:
            f.write(data)

    # Strip crate-only imports & OpenCells wrapper from mod.rs WITHOUT touching any Kernel/Pass/emit line!
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    s = s.replace("use crate::bwt::{InverseBwt, LazyCells};\nuse crate::{runs::Runs, Kernels};\n", "")
    # Remove OpenCells / OpenCellsOf / InverseBwt impl block (lines between Cap impl and `const INLINE`)
    i0 = s.index("/// The workers' kernel wherever production ran")
    i1 = s.index("/// The bytes of a code's word kept inline")
    s = s[:i0] + s[i1:]
    # Remove #[cfg(test)] mod tests / mod b5 etc at bottom, keep `mod endpoint; mod outq;`
    i_mod = s.index("mod endpoint;")
    s = s[:i_mod] + """mod endpoint;
mod outq;

impl Kernel {
    #[inline(never)]
    pub fn run_entry(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> Option<Counts> {
        self.try_run::<true, true>(records, n, p, LIMIT, cap, out)
    }
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
    pub fn scratch_report(&self) -> (usize, usize, String) {
        use std::mem::size_of;
        let q = self.spill.scratch_bytes();
        let tiles_dir = size_of::<Tile>() * self.tiles.capacity() + 4 * self.directory.capacity();
        let bands = size_of::<Band>() * self.bands.capacity();
        let spelling = self.spelling.capacity();
        let offsets = 4 * self.offsets.capacity();
        let edges = 4 * self.edges.capacity();
        let extra = 4 * self.extra.capacity();
        let inline16 = 16 * self.inline16.capacity();
        let lens = self.lens.capacity();
        let ep = self.endpoint.scratch_bytes();
        let tot = q + tiles_dir + bands + spelling + offsets + edges + extra + inline16 + lens + ep;
        let dict_l1 = 16 * self.inline16.len() + self.lens.len();
        let s = format!("queues={q} bands={bands} spelling={spelling} offsets={offsets} edges={edges} extra={extra} inline16={inline16} lens={lens} endpoint={ep}");
        (tot, dict_l1, s)
    }
}
"""
    s = s.replace("pub(crate) const LIMIT", "pub const LIMIT")
    s = s.replace("pub(crate) struct Cap", "pub struct Cap")
    s = s.replace("pub(crate) m:", "pub m:")
    s = s.replace("pub(crate) strips:", "pub strips:")
    s = s.replace("pub(crate) const NONE:", "pub const NONE:")
    s = s.replace("pub(crate) fn dense(", "pub fn dense(")
    with open(mod_p, "w") as f:
        f.write(s)

    # Add scratch_bytes() helpers to outq.rs and endpoint.rs (non-test)
    with open(f"{dst}/outq.rs", "a") as f:
        f.write("\nimpl Spill { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.next.capacity() + BYTES * self.ovf.capacity() + 1024 * self.cuts.capacity() } }\n")
    with open(f"{dst}/endpoint.rs", "a") as f:
        f.write("\nimpl Endpoint { pub(super) fn scratch_bytes(&self) -> usize { 4 * self.stack.capacity() } }\n")

def make_nobi(dst):
    mod_p = f"{dst}/mod.rs"
    with open(mod_p) as f:
        s = f.read()
    assert "if QUEUE && !wide && m >= 65_536 {" in s
    s = s.replace("if QUEUE && !wide && m >= 65_536 {", "if false && QUEUE && !wide && m >= 65_536 {", 1)
    with open(mod_p, "w") as f:
        f.write(s)

def add_6stage_profiler(dst):
    """Instrument S1, S2a, S2b, S3a, S4, S5 in a separate profiler copy."""
    mod_p = f"{dst}/mod.rs"
    ep_p = f"{dst}/endpoint.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(ep_p) as f:
        e = f.read()

    # Add stage_us: [f64; 6] field to Kernel
    s = s.replace("lens: Vec<u8>,\n}", "lens: Vec<u8>,\n    pub stage_us: [f64; 6],\n}", 1)
    # Time S1 inside construct_capped
    old_s1 = """        out.resize(n as usize + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m, cap.strips);
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let ns = pass.ns;
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());"""
    new_s1 = """        let t_s1 = std::time::Instant::now();
        out.resize(n as usize + INLINE, 0);
        let mut pass = self.begin(&mut out[..], cap.m, cap.strips);
        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });
        let ns = pass.ns;
        let (f, heads, over) = (pass.f, pass.queues.heads(), pass.queues.over());
        self.stage_us[0] = t_s1.elapsed().as_secs_f64() * 1e6;"""
    assert old_s1 in s
    s = s.replace(old_s1, new_s1, 1)

    # Time S2a start at finish entry, S4 around wiring
    old_fin_start = "        let mut stats = Counts {"
    new_fin_start = "        let t_s2a = std::time::Instant::now();\n        let mut stats = Counts {"
    assert old_fin_start in s
    s = s.replace(old_fin_start, new_fin_start, 1)

    # Record t_s2a elapsed just before induced() and accumulate inside endpoint.rs
    old_ind = "        let mut cursors = if INDUCE {"
    new_ind = "        self.stage_us[1] = t_s2a.elapsed().as_secs_f64() * 1e6;\n        let mut cursors = if INDUCE {"
    assert old_ind in s
    s = s.replace(old_ind, new_ind, 1)

    old_s4 = "        self.predecessor = None;"
    new_s4 = "        let t_s4 = std::time::Instant::now();\n        self.predecessor = None;"
    assert old_s4 in s
    s = s.replace(old_s4, new_s4, 1)

    old_ret = "        (stats, mark, bits)"
    new_ret = "        self.stage_us[4] = t_s4.elapsed().as_secs_f64() * 1e6;\n        (stats, mark, bits)"
    assert old_ret in s
    s = s.replace(old_ret, new_ret, 1)

    # Add run_profile6 method
    prof_api = """    #[inline(never)]
    pub fn run_profile6(&mut self, records: &[u16], n: usize, p: u32, cap: Cap, out: &mut Vec<u8>) -> [f64; 6] {
        let (stats, mark, bits) = self.construct_capped::<true, true>(records, n as u32, p, LIMIT, cap, out).unwrap();
        let t5 = std::time::Instant::now();
        self.walk(n, stats.retained, mark, bits, out);
        self.stage_us[5] = t5.elapsed().as_secs_f64() * 1e6;
        self.stage_us
    }
"""
    s = s.replace("    pub fn run_entry(", prof_api + "    pub fn run_entry(", 1)

    # In endpoint.rs: time S2a (nodes), S2b (eliminate), S3a (bottom + dedup + bands)
    old_ep_destr = """            inline16,
            lens,
            ..
        } = self;"""
    new_ep_destr = """            inline16,
            lens,
            stage_us,
            ..
        } = self;
        let t_s2a_ep = std::time::Instant::now();"""
    assert old_ep_destr in e
    e = e.replace(old_ep_destr, new_ep_destr, 1)

    old_s2b = "        let (mut remaining, mut steps) = (n, 0);"
    new_s2b = "        stage_us[1] += t_s2a_ep.elapsed().as_secs_f64() * 1e6;\n        let t_s2b = std::time::Instant::now();\n        let (mut remaining, mut steps) = (n, 0);"
    assert old_s2b in e
    e = e.replace(old_s2b, new_s2b, 1)

    old_s3a = "        e.assign_bottom(stats.retained as u32);"
    new_s3a = "        stage_us[2] = t_s2b.elapsed().as_secs_f64() * 1e6;\n        let t_s3a = std::time::Instant::now();\n        e.assign_bottom(stats.retained as u32);"
    assert old_s3a in e
    e = e.replace(old_s3a, new_s3a, 1)

    old_s3a_end = "        assert_eq!(id, 0);\n        cursors"
    new_s3a_end = "        assert_eq!(id, 0);\n        stage_us[3] = t_s3a.elapsed().as_secs_f64() * 1e6;\n        cursors"
    assert old_s3a_end in e
    e = e.replace(old_s3a_end, new_s3a_end, 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_wire16(dst):
    """Port Cycle 33 mark_byte isolation + Wire16/8/4 u64 MSHR-burst onto origin/master."""
    mod_p = f"{dst}/mod.rs"
    outq_p = f"{dst}/outq.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(outq_p) as f:
        o = f.read()

    old_mark = """            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
            }"""
    new_mark = """            if p.wrapping_sub(x) < w {
                f.mark = f.m + (p - x);
                f.mark_byte = c;
            }"""
    assert old_mark in s
    s = s.replace(old_mark, new_mark, 1)
    s = s.replace("    mark: u32,\n    runs: usize,", "    mark: u32,\n    mark_byte: u8,\n    runs: usize,", 1)
    s = s.replace("            mark: 0,\n            runs: 0,", "            mark: 0,\n            mark_byte: 0,\n            runs: 0,", 1)
    s = s.replace("            mark,\n            runs,\n        } = *f;", "            mark,\n            mark_byte,\n            runs,\n        } = *f;", 1)

    old_wire = """            for a in 0..256 {
                let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                let mut queue = outq::Reader::new(out, &self.spill, heads, a);
                for &band in &self.bands[cursors[a]..end] {
                    queue.wire_xor(
                        edges,
                        band.dst() as usize,
                        (band.end() - k) as usize,
                        band.code() << bits,
                        mark,
                        &mut predecessor,
                    );
                    k = band.end();
                }
            }"""
    new_wire = """            let mb = usize::from(mark_byte);
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
    assert old_wire in s
    s = s.replace(old_wire, new_wire, 1)

    nomark_fn = """
    #[inline(always)]
    pub(super) fn wire_xor_nomark(
        &mut self,
        edges: &mut [u32],
        dst_start: usize,
        span: usize,
        pack: u32,
    ) {
        if self.at + span <= CHUNK && self.read + span as u32 <= self.cut && self.id < self.in_out {
            let hi = self.epoch << 16;
            let at = self.id as usize * BYTES + 2 * self.at;
            let bytes = &self.out[at..at + 2 * span];
            let mut i = 0usize;
            while i + 16 <= span {
                let b = &bytes[2 * i..2 * i + 32];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                let v0 = hi | (q0 as u32 & 0xFFFF);
                let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q0 >> 48) as u32);
                let v4 = hi | (q1 as u32 & 0xFFFF);
                let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                let v7 = hi | ((q1 >> 48) as u32);
                let v8 = hi | (q2 as u32 & 0xFFFF);
                let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                let v11 = hi | ((q2 >> 48) as u32);
                let v12 = hi | (q3 as u32 & 0xFFFF);
                let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                let v15 = hi | ((q3 >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 16];
                eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                i += 16;
            }
            if i + 8 <= span {
                let b = &bytes[2 * i..2 * i + 16];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let v0 = hi | (q0 as u32 & 0xFFFF);
                let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q0 >> 48) as u32);
                let v4 = hi | (q1 as u32 & 0xFFFF);
                let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                let v7 = hi | ((q1 >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 8];
                eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                eu[4] ^= pack | v4; eu[5] ^= pack | v5; eu[6] ^= pack | v6; eu[7] ^= pack | v7;
                edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                edges[v4 as usize] ^= (u0 + 4) as u32; edges[v5 as usize] ^= (u0 + 5) as u32;
                edges[v6 as usize] ^= (u0 + 6) as u32; edges[v7 as usize] ^= (u0 + 7) as u32;
                i += 8;
            }
            if i + 4 <= span {
                let b = &bytes[2 * i..2 * i + 8];
                let q = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let v0 = hi | (q as u32 & 0xFFFF);
                let v1 = hi | ((q >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 4];
                eu[0] ^= pack | v0; eu[1] ^= pack | v1; eu[2] ^= pack | v2; eu[3] ^= pack | v3;
                edges[v0 as usize] ^= u0 as u32;       edges[v1 as usize] ^= (u0 + 1) as u32;
                edges[v2 as usize] ^= (u0 + 2) as u32; edges[v3 as usize] ^= (u0 + 3) as u32;
                i += 4;
            }
            while i < span {
                let u = dst_start + i;
                let v = hi | u32::from(u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]));
                edges[u] ^= pack | v;
                edges[v as usize] ^= u as u32;
                i += 1;
            }
            self.at += span;
            self.read += span as u32;
        } else {
            let mut dummy = 0u32;
            self.wire_xor_slow(edges, dst_start, span, pack, u32::MAX, &mut dummy);
        }
    }
"""
    anchor = "    #[inline(always)]\n    pub(super) fn wire_xor("
    assert anchor in o
    o = o.replace(anchor, nomark_fn + "\n" + anchor, 1)
    with open(mod_p, "w") as f:
        f.write(s)
    with open(outq_p, "w") as f:
        f.write(o)

def patch_s2a_fuse(dst):
    """Fuse S2a's 3 sequential passes over strips_raw into ONE single pass!"""
    mod_p = f"{dst}/mod.rs"
    ep_p = f"{dst}/endpoint.rs"
    with open(mod_p) as f:
        s = f.read()
    with open(ep_p) as f:
        e = f.read()

    old_mod_loop = """        let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
        let image_order = &mut rest_edges[..ns];
        self.tiles.clear();
        let (mut last_x, mut retained_x) = (0, 0u32);
        for i in 0..ns {
            let sw = &mut strips_raw[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let width = sw[2] >> 8;
            let from = sw[0];
            let a = usize::from(byte);
            sw[1] += first[a];
            let to = sw[1];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if !INDUCE {
                if from > last_x {
                    self.tiles.push(Tile {
                        end: from,
                        delta: retained_x.wrapping_sub(last_x),
                        symbol: TERMINAL,
                    });
                    retained_x += from - last_x;
                }
                last_x = from + width;
                self.tiles.push(Tile {
                    end: last_x,
                    delta: to.wrapping_sub(from),
                    symbol: byte.into(),
                });
            }
        }"""
    new_mod_loop = """        if !INDUCE {
            let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
            let image_order = &mut rest_edges[..ns];
            self.tiles.clear();
            let (mut last_x, mut retained_x) = (0, 0u32);
            for i in 0..ns {
                let sw = &mut strips_raw[3 * i..3 * i + 3];
                let byte = (sw[2] & 0xFF) as u8;
                let width = sw[2] >> 8;
                let from = sw[0];
                let a = usize::from(byte);
                sw[1] += first[a];
                let to = sw[1];
                image_order[write[a] as usize] = i as u32;
                write[a] += 1;
                if from > last_x {
                    self.tiles.push(Tile {
                        end: from,
                        delta: retained_x.wrapping_sub(last_x),
                        symbol: TERMINAL,
                    });
                    retained_x += from - last_x;
                }
                last_x = from + width;
                self.tiles.push(Tile {
                    end: last_x,
                    delta: to.wrapping_sub(from),
                    symbol: byte.into(),
                });
            }
            if last_x < n {
                self.tiles.push(Tile {
                    end: n,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
            }
            self.directory.clear();
            let mut ti = 0;
            for x in (0..n).step_by(PAGE) {
                while self.tiles[ti].end <= x {
                    ti += 1;
                }
                self.directory.push(ti as u32);
            }
        }"""
    # Remove the trailing `if !INDUCE { if last_x < n ... }` block too
    old_tail = """        if !INDUCE {
            if last_x < n {
                self.tiles.push(Tile {
                    end: n,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
            }
            self.directory.clear();
            let mut ti = 0;
            for x in (0..n).step_by(PAGE) {
                while self.tiles[ti].end <= x {
                    ti += 1;
                }
                self.directory.push(ti as u32);
            }
        }"""
    assert old_mod_loop in s and old_tail in s
    s = s.replace(old_mod_loop + "\n" + old_tail, new_mod_loop, 1)
    s = s.replace("self.induced(ns, n, &thin, &mut stats)", "self.induced(ns, n, &thin, &first, &mut write, &mut stats)", 1)

    old_sig1 = """    pub(super) fn induced(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, stats, most)
        }
    }

    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {"""
    new_sig1 = """    pub(super) fn induced(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut Counts,
    ) -> [usize; 256] {
        let most = Self::nodes_most(ns);
        if self.wide(ns) {
            self.induced_with::<Wide>(ns, n, thin, first, write, stats, most)
        } else {
            self.induced_with::<Narrow>(ns, n, thin, first, write, stats, most)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn induced_with<L: Links>(
        &mut self,
        ns: usize,
        n: u32,
        thin: &[u32; 256],
        first: &[u32; 256],
        write: &mut [u32; 256],
        stats: &mut Counts,
        most: usize,
    ) -> [usize; 256] {"""
    assert old_sig1 in e
    e = e.replace(old_sig1, new_sig1, 1)

    old_passes = """        e.node(0, 0);
        // the widest node, for the width certificate (`CERTIFIED_WIDTH`)
        let mut widest = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            e.node(s.width, literal(s.byte));
            widest = widest.max(s.width);
        }
        // the lists, as the nodes are made: on side 0 the image gaps (its terminals) then the
        // domain order, on side 1 the domain gaps then the image order
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0;
        for i in 0..ns {
            let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
            if x < s.from {
                widest = widest.max(s.from - x);
                let id = e.node(s.from - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            e.append_to(&mut order[0], i as u32 + 1, 0);
            x = s.from + s.width;
        }"""
    new_passes = """        e.node(0, 0);
        e.nodes = ns as u32 + 1;
        let mut widest = 0u32;
        let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
        let mut x = 0u32;
        for i in 0..ns {
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
        }"""
    assert old_passes in e
    e = e.replace(old_passes, new_passes, 1)

    with open(mod_p, "w") as f:
        f.write(s)
    with open(ep_p, "w") as f:
        f.write(e)

def patch_s3_tag(dst, compact12=True):
    """Upgrade S3a Dictionary: pure-register u128 shift-OR, 8-bit tag in contents, last_word cache, 1-mul short hash."""
    ep_p = f"{dst}/endpoint.rs"
    with open(ep_p) as f:
        e = f.read()

    # 1. Pure-register u128 shift-OR in inline_word
    old_inl = """        if height < INLINE {
            let mut bytes = self.short_payload(id).to_le_bytes();
            bytes[height] = byte;
            (bytes, (height + 1) as u8)
        }"""
    new_inl = """        if height < INLINE {
            let v = self.short_payload(id) | (u128::from(byte) << (8 * height));
            (v.to_le_bytes(), (height + 1) as u8)
        }"""
    assert old_inl in e
    e = e.replace(old_inl, new_inl, 1)

    # 2. Fast tagged Dictionary + 1-mul short hash + optional 12 KiB HT
    pair_shift = 23 if compact12 else 22
    pair_words = 1024 if compact12 else 2048
    str_words = 2048 if compact12 else 4096

    old_dict = """impl Dictionary<'_> {
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
        if handle == 0 {
            return u32::from(byte);
        }
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {
            return self.pairs[slot + 1];
        }

        let start = self.spelling.len();
        let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
        let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
        if self.contents.is_empty() {
            return self.insert(entry, len);
        }
        let bytes = if len == LONG {
            &self.spelling[start..]
        } else {
            &entry[..len as usize]
        };
        let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
        let old = self.contents[index] as usize;
        let same = old != 0
            && self.lens[old] == len
            && if len == LONG {
                super::long_word(&self.inline16[old], self.spelling) == bytes
            } else {
                self.inline16[old] == entry
            };
        let code = if same {
            // Fresh child memo entries may point into this expansion. Preserve
            // those bytes until the arena is retired.
            if len == LONG {
                if memoized {
                    self.spelling.truncate(start);
                } else {
                    self.spelling.pop();
                }
            }
            old as u32
        } else {
            let code = self.insert(entry, len);
            self.contents[index] = code;
            code
        };
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }

    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }
}

/// A cache collision only evicts a candidate. Sharing requires equal length and bytes.
fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
    let (low, high, middle) = if len == LONG {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
    } else {
        let value = u128::from_le_bytes(*entry);
        (value as u64, (value >> 64) as u64, 0)
    };
    let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}"""

    new_dict = f"""impl Dictionary<'_> {{
    #[inline(always)]
    fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {{
        if handle == 0 {{
            return u32::from(byte);
        }}
        let id = handle & ID_MASK;
        let key = (id << 8) | u32::from(byte);
        let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> {pair_shift}) as usize;
        if !self.pairs.is_empty() && self.pairs[slot] == key {{
            return self.pairs[slot + 1];
        }}
        self.code_miss(e, handle, id, byte, key, slot)
    }}

    fn code_miss<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, id: u32, byte: u8, key: u32, slot: usize) -> u32 {{
        let height = (handle >> ID_BITS) as usize;
        if height < INLINE {{
            let v = e.short_payload(id) | (u128::from(byte) << (8 * height));
            let entry = v.to_le_bytes();
            let len = (height + 1) as u8;
            if self.contents.is_empty() {{
                return self.insert(entry, len);
            }}
            let lo = v as u64;
            let hi = (v >> 64) as u64;
            let h = (lo ^ hi.rotate_left(29) ^ (u64::from(len) << 56))
                .wrapping_mul(0xBF58_476D_1CE4_E5B9);
            let h = h ^ (h >> 30);
            let tag = (((h >> 56) as u32) | 0x80) << 24;
            let index = h as usize & (self.contents.len() - 1);
            let ent = self.contents[index];
            let code = if (ent & 0xFF00_0000) == tag && {{
                let old = (ent & 0x00FF_FFFF) as usize;
                self.lens[old] == len && self.inline16[old] == entry
            }} {{
                ent & 0x00FF_FFFF
            }} else {{
                let c = self.insert(entry, len);
                self.contents[index] = tag | c;
                c
            }};
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            return code;
        }}
        // Cold LONG path (height >= 16)
        let start = self.spelling.len();
        let memoized = e.a[e.rule(id) + 3] != u32::MAX;
        e.append(id, self.stack, self.spelling);
        self.spelling.push(byte);
        let entry = super::long_slot(start as u32, self.spelling.len() as u32);
        if self.contents.is_empty() {{
            return self.insert(entry, LONG);
        }}
        let bytes = &self.spelling[start..];
        let h = spelling_hash_long(bytes);
        let tag = (((h >> 56) as u32) | 0x80) << 24;
        let index = h as usize & (self.contents.len() - 1);
        let ent = self.contents[index];
        let code = if (ent & 0xFF00_0000) == tag && {{
            let old = (ent & 0x00FF_FFFF) as usize;
            self.lens[old] == LONG && super::long_word(&self.inline16[old], self.spelling) == bytes
        }} {{
            if memoized {{
                self.spelling.truncate(start);
            }} else {{
                self.spelling.pop();
            }}
            ent & 0x00FF_FFFF
        }} else {{
            let c = self.insert(entry, LONG);
            self.contents[index] = tag | c;
            c
        }};
        self.pairs[slot] = key;
        self.pairs[slot + 1] = code;
        code
    }}

    #[inline(always)]
    fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {{
        let code = self.lens.len() as u32;
        self.inline16.push(entry);
        self.lens.push(len);
        code
    }}
}}

fn spelling_hash_long(bytes: &[u8]) -> u64 {{
    let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    let h = part(0).wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ part(bytes.len() - 8).rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
        ^ part(bytes.len() / 2 - 4).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^ (h >> 30)
}}"""
    assert old_dict in e
    e = e.replace(old_dict, new_dict, 1)

    old_ht = """        const PAIR_WORDS: usize = 2 * 1024;
        const CACHE_WORDS: usize = PAIR_WORDS + 4096;"""
    new_ht = f"""        const PAIR_WORDS: usize = {pair_words};
        const CACHE_WORDS: usize = PAIR_WORDS + {str_words};"""
    assert old_ht in e
    e = e.replace(old_ht, new_ht, 1)

    # 3. Per-byte last_word register cache in the band loop!
    old_band_loop = """        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            while k < end {"""
    new_band_loop = """        for a in 0..256 {
            cursors[a] = bands.len();
            let end = k + thin[a];
            let mut last_w = u32::MAX;
            let mut last_c = 0u32;
            while k < end {"""
    assert old_band_loop in e
    e = e.replace(old_band_loop, new_band_loop, 1)

    old_code_call = "                let code = dictionary.code(&mut e, node_word, a as u8);"
    new_code_call = """                let code = if node_word == 0 {
                    a as u32
                } else if node_word == last_w {
                    last_c
                } else {
                    let c = dictionary.code(&mut e, node_word, a as u8);
                    last_w = node_word;
                    last_c = c;
                    c
                };"""
    assert old_code_call in e
    e = e.replace(old_code_call, new_code_call, 1)

    with open(ep_p, "w") as f:
        f.write(e)

# Build all Cycle 36 modules
extract_master(f"{ROOT}/master_a1")
extract_master(f"{ROOT}/master_a2")
extract_master(f"{ROOT}/master_nobi");  make_nobi(f"{ROOT}/master_nobi")
extract_master(f"{ROOT}/prof_master");  add_6stage_profiler(f"{ROOT}/prof_master")
extract_master(f"{ROOT}/prof_nobi");    make_nobi(f"{ROOT}/prof_nobi"); add_6stage_profiler(f"{ROOT}/prof_nobi")

extract_master(f"{ROOT}/cand_wire16");  patch_wire16(f"{ROOT}/cand_wire16")
extract_master(f"{ROOT}/cand_s2afuse"); patch_s2a_fuse(f"{ROOT}/cand_s2afuse")
extract_master(f"{ROOT}/cand_s3tag12"); patch_s3_tag(f"{ROOT}/cand_s3tag12", compact12=True)
extract_master(f"{ROOT}/cand_s3tag24"); patch_s3_tag(f"{ROOT}/cand_s3tag24", compact12=False)

extract_master(f"{ROOT}/cand_combo");   patch_wire16(f"{ROOT}/cand_combo"); patch_s2a_fuse(f"{ROOT}/cand_combo"); patch_s3_tag(f"{ROOT}/cand_combo", compact12=True)
extract_master(f"{ROOT}/prof_combo");   patch_wire16(f"{ROOT}/prof_combo"); patch_s2a_fuse(f"{ROOT}/prof_combo"); patch_s3_tag(f"{ROOT}/prof_combo", compact12=True); add_6stage_profiler(f"{ROOT}/prof_combo")

extract_master(f"{ROOT}/cand_nobi_combo"); make_nobi(f"{ROOT}/cand_nobi_combo"); patch_s2a_fuse(f"{ROOT}/cand_nobi_combo"); patch_s3_tag(f"{ROOT}/cand_nobi_combo", compact12=True)
extract_master(f"{ROOT}/prof_nobi_combo"); make_nobi(f"{ROOT}/prof_nobi_combo"); patch_s2a_fuse(f"{ROOT}/prof_nobi_combo"); patch_s3_tag(f"{ROOT}/prof_nobi_combo", compact12=True); add_6stage_profiler(f"{ROOT}/prof_nobi_combo")

print("Generated all Cycle 36 re-baseline & candidate modules successfully!")
