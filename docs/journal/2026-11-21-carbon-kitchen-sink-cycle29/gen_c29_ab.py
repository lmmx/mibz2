from pathlib import Path

# Quick 10-second surgical ablation of Cycle 29's 4 independent factors on top of `BiOp48`:
# - `ab_max`:   ONLY carry-over `maximal`
# - `ab_all`:   ONLY algebraic `all = thin + thick_sum`
# - `ab_pos`:   ONLY bit-lattice `pos = (tail << 10) | fill`
# - `ab_swar4`: ONLY 8-byte `u64` SWAR4 store on `w <= 4` (keeping original `BiOp48` `maximal`, `all`, `tails/fill`!)
# - `ab_swar4_all`: `ab_swar4` + `ab_all`

mod48 = Path("/tmp/cks_bench/cand_biop48/mod.rs").read_text()
ep48 = Path("/tmp/cks_bench/cand_biop48/endpoint.rs").read_text()
oq48 = Path("/tmp/cks_bench/cand_biop48/outq.rs").read_text()

mod_p1 = Path("/tmp/cks_bench/cand_p1_core/mod.rs").read_text()
oq_pos = Path("/tmp/cks_bench/cand_p1_core/outq.rs").read_text()

# 1. `ab_max`: ONLY carry-over `maximal`
old_max = mod48[mod48.index("fn maximal("):mod48.index("// A maximal run")]
new_max = mod_p1[mod_p1.index("#[inline(always)]\nfn maximal("):mod_p1.index("// A maximal run")].replace("let mut rec", "let rec")

d = Path("/tmp/cks_bench/ab_max"); d.mkdir(exist_ok=True)
(d / "mod.rs").write_text(mod48.replace(old_max, new_max))
(d / "endpoint.rs").write_text(ep48)
(d / "outq.rs").write_text(oq48)

# 2. `ab_all`: ONLY algebraic `all = thin + thick_sum`
m_all = mod_p1.replace(new_max.replace("let rec", "let mut rec"), old_max)
d = Path("/tmp/cks_bench/ab_all"); d.mkdir(exist_ok=True)
(d / "mod.rs").write_text(m_all)
(d / "endpoint.rs").write_text(ep48)
(d / "outq.rs").write_text(oq48)

# 3. `ab_pos`: ONLY bit-lattice `pos` in outq.rs
d = Path("/tmp/cks_bench/ab_pos"); d.mkdir(exist_ok=True)
(d / "mod.rs").write_text(mod48)
(d / "endpoint.rs").write_text(ep48)
(d / "outq.rs").write_text(oq_pos)

# 4. `ab_swar4`: ONLY 8-byte `u64` SWAR4 on `w <= 4` in original `outq.rs`!
old_push = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
new_swar4_orig = """    #[inline(always)]
    pub(super) fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
        let f = self.fill[a] as usize;
        let lo = m & (EPOCH - 1);
        let tail = self.tails[a];
        if w <= 4 && f + 4 <= CHUNK && lo < EPOCH - 4 && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let packed = (u64::from(lo as u16).wrapping_mul(0x0001_0001_0001_0001u64))
                .wrapping_add(0x0003_0002_0001_0000u64);
            self.out[at..at + 8].copy_from_slice(&packed.to_le_bytes());
            self.fill[a] = (f as u32) + w;
        } else if w <= 16 && f + 16 <= CHUNK && lo + w < EPOCH && tail < self.in_out {
            let at = tail as usize * BYTES + 2 * f;
            let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.fill[a] = (f as u32) + w;
        } else {
            self.push_slow(a, m, w, thin);
        }
    }"""
assert old_push in oq48
oq_swar4 = oq48.replace(old_push, new_swar4_orig)

d = Path("/tmp/cks_bench/ab_swar4"); d.mkdir(exist_ok=True)
(d / "mod.rs").write_text(mod48)
(d / "endpoint.rs").write_text(ep48)
(d / "outq.rs").write_text(oq_swar4)

# 5. `ab_swar4_all`: `ab_swar4` + `ab_all`
d = Path("/tmp/cks_bench/ab_swar4_all"); d.mkdir(exist_ok=True)
(d / "mod.rs").write_text(m_all)
(d / "endpoint.rs").write_text(ep48)
(d / "outq.rs").write_text(oq_swar4)

print("Generated 5 ablation arms.")
