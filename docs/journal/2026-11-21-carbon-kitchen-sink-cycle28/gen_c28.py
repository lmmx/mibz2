from pathlib import Path

# Cycle 28 (Neon):
# 1. `cand_biop48`: `CKS-BiOperad` + `48 KiB` `spelling` exact-fit (`945.8 KiB = -19.28%` WS, Proposition 26.2)
# 2. `cand_fused_op`: `cand_biop48` + fused `len_and_val` (single load of rule words in `join` & `dedup`)
# 3. `cand_birv`: `cand_fused_op` + Church-Rosser Bidirectional Rauzy-Veech Induction (`BiRV`, mu_RV = 2!)
# 4. `cand_leftrv`: `cand_fused_op` + pure Left-End Rauzy-Veech Induction (ablation vs Right-End & BiRV)

bi_mod = Path("/tmp/cks_bench/cand_cks_bi/mod.rs").read_text()
bi_ep = Path("/tmp/cks_bench/cand_cks_bi/endpoint.rs").read_text()
bi_oq = Path("/tmp/cks_bench/cand_cks_bi/outq.rs").read_text()

# 1. `cand_biop48`: add `48 KiB` spelling exact-fit
old_sp = "        self.bands.clear();\n        self.spelling.clear();"
new_sp = """        self.bands.clear();
        self.spelling.clear();
        if n >= 65_536 && self.spelling.capacity() < 49_152 {
            self.spelling.reserve_exact(49_152);
        }"""
assert old_sp in bi_mod
mod48 = bi_mod.replace(old_sp, new_sp)

d_48 = Path("/tmp/cks_bench/cand_biop48")
d_48.mkdir(parents=True, exist_ok=True)
(d_48 / "mod.rs").write_text(mod48)
(d_48 / "endpoint.rs").write_text(bi_ep)
(d_48 / "outq.rs").write_text(bi_oq)

# 2. `cand_fused_op`: single-load `len_and_val(w) -> (u32, u128)` in `Elim::join` and `dedup`!
old_join_block = """    #[inline(always)]
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
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let ll = self.len(left);
        let rl = self.len(right);
        let len = ll + rl;
        if len <= 15 {
            // Operadic evaluation in truncated free monoid Sigma^{<=15}:
            let vl = self.short_u128(left);
            let vr = self.short_u128(right);
            let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
            self.a[self.top] = v as u32;
            self.a[self.top + 1] = (v >> 32) as u32;
            self.a[self.top + 2] = (v >> 64) as u32;
            self.a[self.top + 3] = (v >> 96) as u32;
        } else {
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        }
        self.top += RULE;
        id
    }"""

new_join_block = """    /// Single-load extraction of `(len, low_120_payload)` for any non-zero word `w`:
    /// when `len <= 15`, returns exact payload in `u128`; when `len >= 16`, returns `(len, 0)`.
    #[inline(always)]
    fn len_and_val(&self, w: u32) -> (u32, u128) {
        if w <= 256 {
            (1, u128::from(w - 1))
        } else {
            let r = self.rule(w);
            let r3 = self.a[r + 3];
            if (r3 >> 28) == 0 {
                let len = r3 >> 24;
                let v = u128::from(self.a[r])
                    | (u128::from(self.a[r + 1]) << 32)
                    | (u128::from(self.a[r + 2]) << 64)
                    | (u128::from(r3 & 0x00FF_FFFF) << 96);
                (len, v)
            } else {
                (self.a[r + 2], 0)
            }
        }
    }

    #[inline(always)]
    fn join(&mut self, left: u32, right: u32) -> u32 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let id = ((self.top - self.base) / RULE) as u32 + 257;
        let (ll, vl) = self.len_and_val(left);
        let (rl, vr) = self.len_and_val(right);
        let len = ll + rl;
        if len <= 15 {
            let v = vl | (vr << (8 * ll)) | ((len as u128) << 120);
            self.a[self.top] = v as u32;
            self.a[self.top + 1] = (v >> 32) as u32;
            self.a[self.top + 2] = (v >> 64) as u32;
            self.a[self.top + 3] = (v >> 96) as u32;
        } else {
            self.a[self.top..self.top + RULE].copy_from_slice(&[left, right, len, u32::MAX]);
        }
        self.top += RULE;
        id
    }"""

assert old_join_block in bi_ep
ep_fused = bi_ep.replace(old_join_block, new_join_block).replace(
    """                        let h_len = e.len(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));""",
    """                        let (h_len, v_pre) = e.len_and_val(node_word);
                        let c = inline16.len() as u32;
                        if h_len <= 15 {
                            let vp = v_pre | (u128::from(a as u8) << (8 * h_len));""",
).replace(
    """                        let h_len = e.len(node_word);
                        let found_code = if h_len <= 15 {
                            // ZERO-STACK, ZERO-SPELLING, DIRECT-INLINE16 REGISTER PATH (>93% of bands)!
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let v = e.short_u128(node_word) | (u128::from(a as u8) << (8 * h_len));""",
    """                        let (h_len, v_pre) = e.len_and_val(node_word);
                        let found_code = if h_len <= 15 {
                            let wlen = h_len + 1;
                            let wlen_u8 = wlen as u8;
                            let v = v_pre | (u128::from(a as u8) << (8 * h_len));""",
)

d_fop = Path("/tmp/cks_bench/cand_fused_op")
d_fop.mkdir(parents=True, exist_ok=True)
(d_fop / "mod.rs").write_text(mod48)
(d_fop / "endpoint.rs").write_text(ep_fused)
(d_fop / "outq.rs").write_text(bi_oq)

# 3. `cand_birv` & 4. `cand_leftrv`:
#    Add `MID` sentinel between `terminals[side]` and `order[side]` on both sides (`nodes_most = 3 * ns + 4`),
#    allowing Left-End (`next(mid, 0), next(mid, 1)`), Right-End (`prev(0, 0), prev(0, 1)`), or Simultaneous BiRV!
def build_rv_variant(dst: str, mode: str):
    ep = ep_fused
    ep = ep.replace("3 * ns + 3", "3 * ns + 4")

    # Replace `close` to insert `mid` sentinel between `terminals[side]` and `order[side]`:
    # 0 <-> terminals[side] <-> mid <-> order[side] <-> 0
    old_close = """    fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }"""
    new_close = """    fn close_with_mid(&mut self, side: usize, mid: u32, first: (u32, u32), second: (u32, u32)) {
        let mut prev = 0;
        for (head, tail) in [first, (mid, mid), second] {
            if tail == 0 {
                continue;
            }
            L::set_next(self.a, prev, side, head);
            L::set_prev(self.a, head, side, prev);
            prev = tail;
        }
        L::set_next(self.a, prev, side, 0);
        L::set_prev(self.a, 0, side, prev);
    }"""
    assert old_close in ep
    ep = ep.replace(old_close, new_close)

    # Add `eliminate_birv` or `eliminate_left` right before `fn eliminate<const SPIRALS: bool>`
    if mode == "birv":
        extra_elim = """    /// Church-Rosser Bidirectional Rauzy-Veech Induction (`BiRV`, mu_RV = 2):
    /// Eliminates simultaneously from LEFT boundary (`next(mid, 0/1)`) and RIGHT boundary (`prev(0, 0/1)`)
    /// whenever the four frontier nodes are pairwise disjoint!
    fn eliminate_birv(&mut self, mid: u32, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + 2 * RULE > self.a.len() {
                break false;
            }
            let (xl, yl) = (L::next(self.a, mid, 0), L::next(self.a, mid, 1));
            let (xr, yr) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
            if xl != xr && xl != yr && yl != xr && yl != yr {
                // Disjoint frontiers! Issue BOTH widths & words before mutating links (2x ILP/MLP!)
                let (awl, bwl) = (self.width(xl), self.width(yl));
                let (awr, bwr) = (self.width(xr), self.width(yr));
                let wl = awl.min(bwl);
                let wr = awr.min(bwr);
                left -= wl + wr;
                taken += 2;

                // Execute LEFT step (at `mid` boundary, replacing LEFT prefix via `insert(prev(..))`)
                if xl == yl {
                    self.cut(xl, 0);
                    self.cut(xl, 1);
                } else {
                    let word_l = self.join(self.word(xl), self.word(yl));
                    if awl <= bwl {
                        self.cut(xl, 0);
                        let before_yl = L::prev(self.a, yl, 0);
                        self.insert(before_yl, xl, 0);
                        self.set_word(xl, word_l);
                        if awl == bwl {
                            self.cut(yl, 0);
                            self.cut(yl, 1);
                        } else {
                            self.set_width(yl, bwl - wl);
                        }
                    } else {
                        self.cut(yl, 1);
                        let before_xl = L::prev(self.a, xl, 1);
                        self.insert(before_xl, yl, 1);
                        self.set_word(yl, word_l);
                        self.set_width(xl, awl - wl);
                    }
                }

                // Execute RIGHT step (at `0` boundary, replacing RIGHT suffix via `insert(..)`)
                if xr == yr {
                    self.cut(xr, 0);
                    self.cut(xr, 1);
                } else {
                    let word_r = self.join(self.word(xr), self.word(yr));
                    if awr <= bwr {
                        self.cut(xr, 0);
                        self.insert(yr, xr, 0);
                        self.set_word(xr, word_r);
                        if awr == bwr {
                            self.cut(yr, 0);
                            self.cut(yr, 1);
                        } else {
                            self.set_width(yr, bwr - wr);
                        }
                    } else {
                        self.cut(yr, 1);
                        self.insert(xr, yr, 1);
                        self.set_word(yr, word_r);
                        self.set_width(xr, awr - wr);
                    }
                }
            } else {
                // Frontier collision (only on final ~1-3 steps or shared node): single RIGHT step!
                let (aw, bw) = (self.width(xr), self.width(yr));
                let w = aw.min(bw);
                left -= w;
                taken += 1;
                if xr == yr {
                    self.cut(xr, 0);
                    self.cut(xr, 1);
                } else {
                    let word = self.join(self.word(xr), self.word(yr));
                    if aw <= bw {
                        self.cut(xr, 0);
                        self.insert(yr, xr, 0);
                        self.set_word(xr, word);
                        if aw == bw {
                            self.cut(yr, 0);
                            self.cut(yr, 1);
                        } else {
                            self.set_width(yr, bw - w);
                        }
                    } else {
                        self.cut(yr, 1);
                        self.insert(xr, yr, 1);
                        self.set_word(yr, word);
                        self.set_width(xr, aw - w);
                    }
                }
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }
"""
    else:
        extra_elim = """    /// Pure Left-End Rauzy-Veech Induction (ablation vs Right-End & BiRV)
    fn eliminate_birv(&mut self, mid: u32, remaining: &mut u32, steps: &mut usize) -> bool {
        let (mut left, mut taken) = (*remaining, *steps);
        let done = loop {
            if left == 0 {
                break true;
            }
            if self.top + RULE > self.a.len() {
                break false;
            }
            let (xl, yl) = (L::next(self.a, mid, 0), L::next(self.a, mid, 1));
            let (aw, bw) = (self.width(xl), self.width(yl));
            let w = aw.min(bw);
            left -= w;
            taken += 1;
            if xl == yl {
                self.cut(xl, 0);
                self.cut(xl, 1);
                continue;
            }
            let word = self.join(self.word(xl), self.word(yl));
            if aw <= bw {
                self.cut(xl, 0);
                let before_yl = L::prev(self.a, yl, 0);
                self.insert(before_yl, xl, 0);
                self.set_word(xl, word);
                if aw == bw {
                    self.cut(yl, 0);
                    self.cut(yl, 1);
                } else {
                    self.set_width(yl, bw - w);
                }
            } else {
                self.cut(yl, 1);
                let before_xl = L::prev(self.a, xl, 1);
                self.insert(before_xl, yl, 1);
                self.set_word(yl, word);
                self.set_width(xl, aw - w);
            }
        };
        (*remaining, *steps) = (left, taken);
        done
    }
"""
    ep = ep.replace("    fn eliminate<const SPIRALS: bool>(", extra_elim + "\n    fn eliminate<const SPIRALS: bool>(")

    old_call = """        stats.open_nodes = e.nodes as usize - 1;
        for side in 0..2 {
            e.close(side, terminals[side], order[side]);
        }
        // the rules follow the nodes made
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let (mut remaining, mut steps) = (n, 0);
        #[cfg(test)]
        let spirals = endpoint.spirals.unwrap_or(widest > CERTIFIED_WIDTH);
        #[cfg(not(test))]
        let spirals = widest > CERTIFIED_WIDTH;
        stats.spirals = spirals;
        while !if spirals {
            e.eliminate::<true>(&mut remaining, &mut steps)
        } else {
            e.eliminate::<false>(&mut remaining, &mut steps)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, 4 * ns + top + RULE);
            e = Elim {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }"""
    new_call = """        let mid = e.node(0, 0);
        stats.open_nodes = e.nodes as usize - 2;
        for side in 0..2 {
            e.close_with_mid(side, mid, terminals[side], order[side]);
        }
        e.base = L::STRIDE * e.nodes as usize;
        e.top = e.base;
        let (mut remaining, mut steps) = (n, 0);
        let spirals = widest > CERTIFIED_WIDTH;
        stats.spirals = spirals;
        while !if spirals {
            e.eliminate::<true>(&mut remaining, &mut steps)
        } else {
            e.eliminate_birv(mid, &mut remaining, &mut steps)
        } {
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            grow(edges, 4 * ns + top + 2 * RULE);
            e = Elim {
                a: &mut edges[4 * ns..],
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
        }
        e.cut(mid, 0);
        e.cut(mid, 1);"""
    assert old_call in ep
    ep = ep.replace(old_call, new_call)

    d = Path(f"/tmp/cks_bench/{dst}")
    d.mkdir(parents=True, exist_ok=True)
    (d / "mod.rs").write_text(mod48)
    (d / "endpoint.rs").write_text(ep)
    (d / "outq.rs").write_text(bi_oq)

build_rv_variant("cand_birv", "birv")
build_rv_variant("cand_leftrv", "left")
print("Generated cand_biop48, cand_fused_op, cand_birv, cand_leftrv successfully.")
