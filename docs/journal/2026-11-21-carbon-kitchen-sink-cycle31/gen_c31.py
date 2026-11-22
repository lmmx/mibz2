from pathlib import Path

# Cycle 31 (Neon): Z/2Z Top/Bottom Side-Exchange Symmetry (`CMOV-Elim`)
# Eliminates the 50/50 unpredictable `if aw <= bw` branch inside `Elim::eliminate::<false>` (`1,898 us = 18.2%` of runtime!),
# plus `cut_tail` (exploiting `next(narrow, side) == 0`) and single-store `set_links(narrow, side, wide, next)`.
# Also builds `cand_cmov_diet` (mu=1 serial + All+Pos + 48KiB spelling + CMOV-Elim) to track our mu=1 contention-free flagship too!

base_mod = Path("/tmp/cks_bench/ab_all_pos/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/ab_all_pos/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/ab_all_pos/outq.rs").read_text()

# Add `set_links` to `Links` trait (`Narrow`: 1 store instead of 2 RMWs; `Wide`: 2 direct stores)
old_trait = """    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
}"""
new_trait = """    fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32);
}"""
assert old_trait in base_ep

ep_cmov = base_ep.replace(old_trait, new_trait).replace(
    """    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
}""",
    """    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        let w = &mut a[id as usize * 4 + side];
        *w = (*w & 0xFFFF) | (to << 16);
    }
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 4 + side] = prev | (next << 16);
    }
}""",
).replace(
    """    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
}""",
    """    #[inline(always)]
    fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
        a[id as usize * 6 + 2 * side + 1] = to;
    }
    #[inline(always)]
    fn set_links(a: &mut [u32], id: u32, side: usize, prev: u32, next: u32) {
        a[id as usize * 6 + 2 * side] = prev;
        a[id as usize * 6 + 2 * side + 1] = next;
    }
}""",
)

# Add `cut_tail` and update `insert` to use `set_links`
old_ci = """    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_prev(self.a, id, side, at);
        L::set_next(self.a, id, side, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }"""
new_ci = """    #[inline(always)]
    fn cut_tail(&mut self, id: u32, side: usize) {
        // `id == prev(0, side)` => `next(id, side) == 0` identically!
        let p = L::prev(self.a, id, side);
        L::set_next(self.a, p, side, 0);
        L::set_prev(self.a, 0, side, p);
    }

    #[inline(always)]
    fn insert(&mut self, at: u32, id: u32, side: usize) {
        let next = L::next(self.a, at, side);
        L::set_links(self.a, id, side, at, next);
        L::set_next(self.a, at, side, id);
        L::set_prev(self.a, next, side, id);
    }"""
assert old_ci in ep_cmov
ep_cmov = ep_cmov.replace(old_ci, new_ci)

# Replace the `if aw <= bw` branch inside `Elim::eliminate` with Z/2Z branchless side-exchange!
old_elim_step = """            let w = aw.min(bw);
            assert!(w > 0 && w <= left);
            left -= w;
            taken += 1;
            if x == y {
                self.cut(x, 0);
                self.cut(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.cut(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.cut(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.cut(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }"""

new_elim_cmov = """            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.cut_tail(x, 0);
                self.cut_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            // Z/2Z Top/Bottom Side-Exchange Symmetry: ZERO `if aw <= bw` branch!
            let gt = aw > bw;
            let side = usize::from(gt);
            let narrow = if gt { y } else { x };
            let wide = if gt { x } else { y };
            let diff = aw.abs_diff(bw);
            self.cut_tail(narrow, side);
            self.insert(wide, narrow, side);
            self.set_word(narrow, word);
            if diff == 0 {
                self.cut(wide, 0);
                self.cut(wide, 1);
            } else {
                self.set_width(wide, diff);
            }"""

assert old_elim_step in ep_cmov
ep_cmov_only = ep_cmov.replace(old_elim_step, new_elim_cmov)

# Variant 2: `cand_cmov_unr`: `CMOV-Elim` PLUS `cut_tail` + `set_links` + `branchless join` check (inline fast-path when `left == 0 || right == 0`?)
# Actually let's test `cand_links_only` (ONLY `cut_tail` + `set_links`, keeping `if aw <= bw`) vs `cand_cmov` (`cut_tail` + `set_links` + Z/2Z `CMOV`)
# so we isolate the exact effect of removing the 50/50 `aw <= bw` branch!
new_elim_links_only = """            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if x == y {
                self.cut_tail(x, 0);
                self.cut_tail(x, 1);
                continue;
            }
            let word = self.join(self.word(x), self.word(y));
            if aw <= bw {
                self.cut_tail(x, 0);
                self.insert(y, x, 0);
                self.set_word(x, word);
                if aw == bw {
                    self.cut(y, 0);
                    self.cut(y, 1);
                } else {
                    self.set_width(y, bw - w);
                }
            } else {
                self.cut_tail(y, 1);
                self.insert(x, y, 1);
                self.set_word(y, word);
                self.set_width(x, aw - w);
            }"""
ep_links_only = ep_cmov.replace(old_elim_step, new_elim_links_only)

# Variant 3: `cand_cmov_nodiff`: What if we ALSO make `diff == 0` branch-hinted via `#[cold]` helper `cut_both` so hot loop stays ultra-compact?
ep_cmov_cold = ep_cmov_only.replace(
    """            if diff == 0 {
                self.cut(wide, 0);
                self.cut(wide, 1);
            } else {
                self.set_width(wide, diff);
            }""",
    """            if diff != 0 {
                self.set_width(wide, diff);
            } else {
                self.cut(wide, 0);
                self.cut(wide, 1);
            }""",
)

# Variant 4: `cand_diet_cmov` (mu=1 serial flagship with `48 KiB` spelling + `All+Pos` + `CMOV-Elim`!)
diet_mod = Path("/tmp/cks_bench/cand_cks_diet/mod.rs").read_text()
# Apply `48 KiB` spelling + `All` from `ab_all_pos` cleanly:
diet_mod = diet_mod.replace(
    "        self.bands.clear();\n        self.spelling.clear();",
    "        self.bands.clear();\n        self.spelling.clear();\n        if n >= 65_536 && self.spelling.capacity() < 49_152 {\n            self.spelling.reserve_exact(49_152);\n        }",
).replace(
    """                self.edges[idx] = x;
                self.edges[idx + 1] = f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
        }
        f.all[a] += w;""",
    """                self.edges[idx] = x;
                self.edges[idx + 1] = f.thin[a] + f.all[a];
                self.edges[idx + 2] = (w << 8) | u32::from(c);
                self.ns += 1;
            }
            f.thick[a] += 1;
            f.all[a] += w;
        }""",
).replace(
    """        let First {
            all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;""",
    """        let First {
            mut all,
            thin,
            thick,
            m,
            mark,
            runs,
        } = *f;
        for a in 0..256 {
            all[a] += thin[a];
        }""",
)

for name, mod_s, ep_s in [
    ("cand_links_only", base_mod, ep_links_only),
    ("cand_cmov", base_mod, ep_cmov_only),
    ("cand_cmov_cold", base_mod, ep_cmov_cold),
    ("cand_diet_cmov", diet_mod, ep_cmov_only),
]:
    d = Path(f"/tmp/cks_bench/{name}")
    d.mkdir(exist_ok=True)
    (d / "mod.rs").write_text(mod_s)
    (d / "endpoint.rs").write_text(ep_s)
    (d / "outq.rs").write_text(base_oq)

print("Generated cand_links_only, cand_cmov, cand_cmov_cold, cand_diet_cmov successfully.")
