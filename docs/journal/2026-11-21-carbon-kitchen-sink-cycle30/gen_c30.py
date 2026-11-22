from pathlib import Path

# Cycle 30 (Neon): Attacking `walk` (3,901 us = 37.4% of total runtime!) on top of Neon-C29 (`ab_all_pos`)
# 1. `cand_w_clean`: Peel step 0 (remove `started`), remove dead `if lbu == 16`, remove dead inner `at_f + INLINE <= at_b` on non-LONG fast path
# 2. `cand_w_u64`:   `cand_w_clean` + `lb <= 8` single-`shl` 64-bit backward store (`raw64 << ((8 - lbu) * 8)`)
# 3. `cand_w_lit`:   `cand_w_clean` + `cf < 256` / `cb < 256` 1-byte literal bypass (zero `lens`/`inline16` load on literals)
# 4. `cand_w_u64_u2`: `cand_w_u64` + 2x unrolled dual-cursor main loop (`at_f + 64 <= at_b`)

base_mod = Path("/tmp/cks_bench/ab_all_pos/mod.rs").read_text()
base_ep = Path("/tmp/cks_bench/ab_all_pos/endpoint.rs").read_text()
base_oq = Path("/tmp/cks_bench/ab_all_pos/outq.rs").read_text()

old_emit_start = base_mod.index("#[allow(clippy::too_many_arguments)]\nfn emit_xor_bi_no_off(")
old_emit_end = base_mod.index("\nfn emit(")
old_emit = base_mod[old_emit_start:old_emit_end]

# Helper tail (shared across all 4 variants for exact parity on tail + odd + periodic!)
tail_block = """
    while rem_pairs > 0 && cur_f != cur_b && (!started || cur_f != next_b) && at_f < at_b {
        started = true;
        rem_pairs -= 1;
        let ef = edges[cur_f as usize];
        let eb = edges[cur_b as usize];
        let cf = (ef >> bits) as usize;
        let cb = (eb >> bits) as usize;
        let nf = (ef & mask) ^ prev_f;
        let pb = (eb & mask) ^ next_b;
        prev_f = cur_f;
        cur_f = nf;
        next_b = cur_b;
        cur_b = pb;

        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }

        let lb = lens[cb];
        if lb != LONG {
            let lbu = usize::from(lb);
            let take = lbu.min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
            at_b -= take;
        } else {
            let meta = &inline16[cb];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
            at_b -= take;
        }
    }

    if cur_f == cur_b && at_f < at_b {
        let cf = (edges[cur_f as usize] >> bits) as usize;
        let lf = lens[cf];
        if lf != LONG {
            let take = usize::from(lf).min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&inline16[cf][..take]);
            at_f += take;
        } else {
            let meta = &inline16[cf];
            let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
            let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
            let word = &spelling[s..e];
            let take = word.len().min(at_b - at_f);
            out[at_f..at_f + take].copy_from_slice(&word[..take]);
            at_f += take;
        }
    }

    if at_f < at_b {
        let right_len = n - at_b;
        if right_len > 0 {
            out.copy_within(at_b..n, at_f);
        }
        let period = at_f + right_len;
        out.truncate(period);
        repeat(n, out);
    } else {
        out.truncate(n);
    }
}
"""

header = """#[allow(clippy::too_many_arguments)]
fn emit_xor_bi_no_off(
    edges: &[u32],
    spelling: &[u8],
    inline16: &[[u8; INLINE]],
    lens: &[u8],
    n: usize,
    mark: u32,
    j_neg1: u32,
    bits: u32,
    out: &mut Vec<u8>,
) {
    out.resize(n + INLINE, 0);
    let mask = (1u32 << bits) - 1;
    let mut prev_f = j_neg1;
    let mut cur_f = mark;
    let mut next_b = mark;
    let mut cur_b = j_neg1;
    let mut at_f = 0usize;
    let mut at_b = n;

    let max_pairs = edges.len() / 2 + 1;
    let mut started = false;
    let mut rem_pairs = max_pairs;
"""

# 1. `cand_w_clean`: Peel step 0, drop dead `if lbu == 16`, drop dead `if at_f + INLINE <= at_b` on non-LONG path!
body_clean = """    if rem_pairs > 0 && cur_f != cur_b && at_f + 32 <= at_b {
        started = true;
        // Peel first step so inner loop needs NO `started` flag!
        loop {
            rem_pairs -= 1;
            let ef = edges[cur_f as usize];
            let eb = edges[cur_b as usize];

            let cf = (ef >> bits) as usize;
            let cb = (eb >> bits) as usize;

            let nf = (ef & mask) ^ prev_f;
            let pb = (eb & mask) ^ next_b;
            prev_f = cur_f;
            cur_f = nf;
            next_b = cur_b;
            cur_b = pb;

            let lf = lens[cf];
            let lb = lens[cb];
            if lf != LONG && lb != LONG {
                // Fast branch-minimized path (~98%): `at_f + 32 <= at_b` & `lf <= 16` => `at_f_new + 16 <= at_b` GUARANTEED!
                out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                at_f += usize::from(lf);
                let lbu = usize::from(lb);
                let shifted = u128::from_le_bytes(inline16[cb]) << ((16 - lbu) * 8);
                out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                at_b -= lbu;
            } else {
                if lf != LONG {
                    out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                    at_f += usize::from(lf);
                } else {
                    let meta = &inline16[cf];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_f..at_f + take].copy_from_slice(&word[..take]);
                    at_f += take;
                }
                if lb != LONG {
                    let lbu = usize::from(lb);
                    if at_f + INLINE <= at_b {
                        let shifted = u128::from_le_bytes(inline16[cb]) << ((16 - lbu) * 8);
                        out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                        at_b -= lbu;
                    } else {
                        let take = lbu.min(at_b - at_f);
                        out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                        at_b -= take;
                    }
                } else {
                    let meta = &inline16[cb];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
                    at_b -= take;
                }
            }
            if rem_pairs == 0 || cur_f == cur_b || cur_f == next_b || at_f + 32 > at_b {
                break;
            }
        }
    }
"""

# 2. `cand_w_u64`: Single-`shl` 64-bit backward store for `lb <= 8` (~96% of all words!), 128-bit only for `9..=16`!
body_u64 = """    if rem_pairs > 0 && cur_f != cur_b && at_f + 32 <= at_b {
        started = true;
        loop {
            rem_pairs -= 1;
            let ef = edges[cur_f as usize];
            let eb = edges[cur_b as usize];

            let cf = (ef >> bits) as usize;
            let cb = (eb >> bits) as usize;

            let nf = (ef & mask) ^ prev_f;
            let pb = (eb & mask) ^ next_b;
            prev_f = cur_f;
            cur_f = nf;
            next_b = cur_b;
            cur_b = pb;

            let lf = lens[cf];
            let lb = lens[cb];
            if lf != LONG && lb <= 8 {
                // Ultra-fast path (~95%+): forward 16B copy + backward SINGLE 64-bit `shl` + 8B store!
                out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                at_f += usize::from(lf);
                let lbu = usize::from(lb);
                let b8 = &inline16[cb];
                let raw64 = u64::from_le_bytes([b8[0], b8[1], b8[2], b8[3], b8[4], b8[5], b8[6], b8[7]]);
                let sh64 = raw64 << ((8 - lbu) * 8);
                out[at_b - 8..at_b].copy_from_slice(&sh64.to_le_bytes());
                at_b -= lbu;
            } else {
                if lf != LONG {
                    out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                    at_f += usize::from(lf);
                } else {
                    let meta = &inline16[cf];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_f..at_f + take].copy_from_slice(&word[..take]);
                    at_f += take;
                }
                if lb != LONG {
                    let lbu = usize::from(lb);
                    if at_f + INLINE <= at_b {
                        let shifted = u128::from_le_bytes(inline16[cb]) << ((16 - lbu) * 8);
                        out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                        at_b -= lbu;
                    } else {
                        let take = lbu.min(at_b - at_f);
                        out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                        at_b -= take;
                    }
                } else {
                    let meta = &inline16[cb];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
                    at_b -= take;
                }
            }
            if rem_pairs == 0 || cur_f == cur_b || cur_f == next_b || at_f + 32 > at_b {
                break;
            }
        }
    }
"""

# 3. `cand_w_lit`: Literal fast path (`cf < 256` / `cb < 256` bypasses `lens` + `inline16` + `u128` shift!)
body_lit = """    if rem_pairs > 0 && cur_f != cur_b && at_f + 32 <= at_b {
        started = true;
        loop {
            rem_pairs -= 1;
            let ef = edges[cur_f as usize];
            let eb = edges[cur_b as usize];

            let cf = (ef >> bits) as usize;
            let cb = (eb >> bits) as usize;

            let nf = (ef & mask) ^ prev_f;
            let pb = (eb & mask) ^ next_b;
            prev_f = cur_f;
            cur_f = nf;
            next_b = cur_b;
            cur_b = pb;

            if cf < 256 {
                out[at_f] = cf as u8;
                at_f += 1;
            } else {
                let lf = lens[cf];
                if lf != LONG {
                    out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                    at_f += usize::from(lf);
                } else {
                    let meta = &inline16[cf];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_f..at_f + take].copy_from_slice(&word[..take]);
                    at_f += take;
                }
            }

            if cb < 256 {
                at_b -= 1;
                out[at_b] = cb as u8;
            } else {
                let lb = lens[cb];
                if lb != LONG {
                    let lbu = usize::from(lb);
                    if at_f + INLINE <= at_b {
                        let shifted = u128::from_le_bytes(inline16[cb]) << ((16 - lbu) * 8);
                        out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                        at_b -= lbu;
                    } else {
                        let take = lbu.min(at_b - at_f);
                        out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                        at_b -= take;
                    }
                } else {
                    let meta = &inline16[cb];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
                    at_b -= take;
                }
            }
            if rem_pairs == 0 || cur_f == cur_b || cur_f == next_b || at_f + 32 > at_b {
                break;
            }
        }
    }
"""

# 4. `cand_w_u64_both`: BOTH forward `lf <= 8` (`u64` copy) AND backward `lb <= 8` (`u64 <<`)!
body_u64_both = """    if rem_pairs > 0 && cur_f != cur_b && at_f + 32 <= at_b {
        started = true;
        loop {
            rem_pairs -= 1;
            let ef = edges[cur_f as usize];
            let eb = edges[cur_b as usize];

            let cf = (ef >> bits) as usize;
            let cb = (eb >> bits) as usize;

            let nf = (ef & mask) ^ prev_f;
            let pb = (eb & mask) ^ next_b;
            prev_f = cur_f;
            cur_f = nf;
            next_b = cur_b;
            cur_b = pb;

            let lf = lens[cf];
            let lb = lens[cb];
            if lf <= 8 && lb <= 8 {
                let f8 = &inline16[cf];
                out[at_f..at_f + 8].copy_from_slice(&f8[..8]);
                at_f += usize::from(lf);
                let lbu = usize::from(lb);
                let b8 = &inline16[cb];
                let raw64 = u64::from_le_bytes([b8[0], b8[1], b8[2], b8[3], b8[4], b8[5], b8[6], b8[7]]);
                let sh64 = raw64 << ((8 - lbu) * 8);
                out[at_b - 8..at_b].copy_from_slice(&sh64.to_le_bytes());
                at_b -= lbu;
            } else {
                if lf != LONG {
                    out[at_f..at_f + INLINE].copy_from_slice(&inline16[cf]);
                    at_f += usize::from(lf);
                } else {
                    let meta = &inline16[cf];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_f..at_f + take].copy_from_slice(&word[..take]);
                    at_f += take;
                }
                if lb != LONG {
                    let lbu = usize::from(lb);
                    if at_f + INLINE <= at_b {
                        let shifted = u128::from_le_bytes(inline16[cb]) << ((16 - lbu) * 8);
                        out[at_b - INLINE..at_b].copy_from_slice(&shifted.to_le_bytes());
                        at_b -= lbu;
                    } else {
                        let take = lbu.min(at_b - at_f);
                        out[at_b - take..at_b].copy_from_slice(&inline16[cb][lbu - take..lbu]);
                        at_b -= take;
                    }
                } else {
                    let meta = &inline16[cb];
                    let s = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]) as usize;
                    let e = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]) as usize;
                    let word = &spelling[s..e];
                    let take = word.len().min(at_b - at_f);
                    out[at_b - take..at_b].copy_from_slice(&word[word.len() - take..]);
                    at_b -= take;
                }
            }
            if rem_pairs == 0 || cur_f == cur_b || cur_f == next_b || at_f + 32 > at_b {
                break;
            }
        }
    }
"""

for name, body in [
    ("cand_w_clean", body_clean),
    ("cand_w_u64", body_u64),
    ("cand_w_lit", body_lit),
    ("cand_w_u64_both", body_u64_both),
]:
    d = Path(f"/tmp/cks_bench/{name}")
    d.mkdir(exist_ok=True)
    new_mod = base_mod[:old_emit_start] + header + body + tail_block + base_mod[old_emit_end:]
    (d / "mod.rs").write_text(new_mod)
    (d / "endpoint.rs").write_text(base_ep)
    (d / "outq.rs").write_text(base_oq)

print("Generated cand_w_clean, cand_w_u64, cand_w_lit, cand_w_u64_both successfully.")
