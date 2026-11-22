from pathlib import Path

# Cycle 32 (Neon): Shifted Schur-Complement Equilibrium (`LIMIT` Pareto Frontier `14, 16, 20, 24, 32`)
# Tests whether `Neon-C31`'s `0.635x` walk + faster `elim` shifts the optimal `LIMIT` away from 16,
# and maps the exact `(1T speed, 2T speed, 6-block scratch)` Pareto frontier!

base_mod = Path("/tmp/cks_bench/cand_links_only/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/cand_links_only/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/cand_links_only/outq.rs").read_text()

# Add fast inline 2x16-wide store for `17 <= w <= 32` in `OutQ16::push` so LIMIT > 16 never falls into cold `push_slow`!
old_push_else = """        } else {
            self.push_slow(a, m, w, thin);
        }"""
new_push_32 = """        } else if w <= 32 && f + 32 <= CHUNK && lo + w < EPOCH && p < self.in_out_lim {
            let at = p as usize + f;
            let iota0: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
            for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota0) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            let lo16 = (lo + 16) as u16;
            let iota1: [u16; 16] = std::array::from_fn(|i| lo16.wrapping_add(i as u16));
            for (d, v) in self.out[at + 32..at + 64].chunks_exact_mut(2).zip(iota1) {
                d.copy_from_slice(&v.to_le_bytes());
            }
            self.pos[a] = p + w;
        } else {
            self.push_slow(a, m, w, thin);
        }"""
assert old_push_else in base_oq
oq_32 = base_oq.replace(old_push_else, new_push_32)

for lim in [14, 20, 24, 32]:
    d = Path(f"/tmp/cks_bench/cand_l{lim}")
    d.mkdir(exist_ok=True)
    m_lim = base_mod.replace("pub const LIMIT: u32 = 16;", f"pub const LIMIT: u32 = {lim};")
    assert f"pub const LIMIT: u32 = {lim};" in m_lim
    (d / "mod.rs").write_text(m_lim)
    (d / "endpoint.rs").write_text(base_ep)
    (d / "outq.rs").write_text(oq_32 if lim > 16 else base_oq)

print("Generated cand_l14, cand_l20, cand_l24, cand_l32 successfully.")
