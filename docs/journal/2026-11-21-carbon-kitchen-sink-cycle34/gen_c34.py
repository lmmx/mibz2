import os, shutil

ROOT = "/tmp/cks_bench"
BASE = f"{ROOT}/cand_c33_w16_d12"

def clone(name):
    dst = f"{ROOT}/{name}"
    if os.path.exists(dst):
        shutil.rmtree(dst)
    shutil.copytree(BASE, dst)
    return dst

# 1. Neon-C34-Diet: exact Neon-C33 with mu=1 serial emit (XOR_GATE = u32::MAX)
d = clone("cand_c34_diet")
mod_p = f"{d}/mod.rs"
with open(mod_p) as f:
    s = f.read()
assert "const XOR_GATE: u32 = 65_536;" in s
s = s.replace("const XOR_GATE: u32 = 65_536;", "const XOR_GATE: u32 = u32::MAX;", 1)
assert "if m >= 65_536 && self.extra.is_empty()" in s
s = s.replace("if m >= 65_536 && self.extra.is_empty()", "if false && self.extra.is_empty()", 1)
with open(mod_p, "w") as f:
    f.write(s)

# 2. Neon-C34-Bi0: exact Neon-C33 with XOR_GATE = 1024 (forced on for all blocks m >= 1024)
d = clone("cand_c34_bi0")
mod_p = f"{d}/mod.rs"
with open(mod_p) as f:
    s = f.read()
s = s.replace("const XOR_GATE: u32 = 65_536;", "const XOR_GATE: u32 = 1024;", 1)
s = s.replace("if m >= 65_536 && self.extra.is_empty()", "if m >= 1024 && self.extra.is_empty()", 1)
with open(mod_p, "w") as f:
    f.write(s)

# 3. Neon-C34-HoistP: Neon-C33 + Pass-1 mark check hoisted out of 99.999% of runs
d = clone("cand_c34_hoistp")
mod_p = f"{d}/mod.rs"
with open(mod_p) as f:
    s = f.read()

old_take = """    #[inline(always)]
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {"""
new_take = """    /// Fast unmarked run: guaranteed `p` is NOT in `[pos, pos + width)`!
    #[inline(always)]
    fn take_unmarked<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, limit: u32) {
        self.f.runs += 1;
        let a = usize::from(c);
        let f = &mut self.f;
        if width <= limit {
            if QUEUE {
                self.queues.push(a, f.m, width, &f.thin);
            }
            f.thin[a] += width;
            f.m += width;
        } else {
            if self.ns >= self.cap_strips {
                self.strip_over = true;
            } else {
                let idx = 3 * self.ns;
                if self.edges.len() < idx + 3 {
                    let grow_to = (idx + 3).max(2 * self.edges.len()).max(1024);
                    let capped = if self.cap_strips < (usize::MAX / 4) {
                        grow_to.min(3 * self.cap_strips + 3)
                    } else {
                        grow_to
                    };
                    self.edges.resize(capped.max(idx + 3), 0);
                }
                self.edges[idx] = pos;
                self.edges[idx + 1] = f.thin[a] + f.all[a];
                self.edges[idx + 2] = (width << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
            f.all[a] += width;
        }
    }

    #[inline(always)]
    fn take<const QUEUE: bool>(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {"""
assert old_take in s
s = s.replace(old_take, new_take, 1)

old_call = """        maximal(records, |c, pos, width| {
            pass.take::<QUEUE>(c, pos, width, p, limit)
        });"""
new_call = """        {
            let mut pos = 0u32;
            let mut i = 0usize;
            while i < records.len() {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    width += u32::from(records[i] & 255);
                    i += 1;
                }
                let end = pos + width;
                if end <= p {
                    pass.take_unmarked::<QUEUE>(c, pos, width, limit);
                    pos = end;
                } else {
                    pass.take::<QUEUE>(c, pos, width, p, limit);
                    pos = end;
                    break;
                }
            }
            while i < records.len() {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    width += u32::from(records[i] & 255);
                    i += 1;
                }
                pass.take_unmarked::<QUEUE>(c, pos, width, limit);
                pos += width;
            }
        }"""
assert old_call in s
s = s.replace(old_call, new_call, 1)

with open(mod_p, "w") as f:
    f.write(s)

print("Generated Cycle 34 candidates: cand_c34_diet, cand_c34_bi0, cand_c34_hoistp!")
