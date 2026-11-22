# Carbon Kitchen Sink (Neon) — Cycle 29: Pass-1 Bit-Lattice Cursor Fusion (`pos = (tail << 10) | fill`) + Algebraic `all = thin + thick_sum` (`0.855x` 1T, `0.850x` 2T p50)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Attacking `pass1` (`2,271 µs = 21.8%` of `CKS-BiOperad`) via algebraic array elimination (`all[a] = thin[a] + thick_sum[a]`), bit-lattice fusion of `(tail, fill)` into a single zero-shift cursor `pos[a] = (tail << 10) | fill`, carry-over `maximal` scanning, and `w <= 4` / `w == 1` SWAR store narrowing (`Pass1-SWAR4` / `Pass1-W1`), cleanly ablated against `BiOp48` and `0.5.11` `master`.

---

## 1. Mathematical & Architectural Construction

### Theorem 29.1 (Bit-Lattice Zero-Shift Cursor Identity & Algebraic Count Decomposition)
In `Pass 1`, every thin run (`489,099` runs across the 6 Wikidata blocks, `73.0%` with `w == 1` and `92.9%` with `w <= 4`) touches **four** `[u32; 256]` arrays (`4 KiB`: `queues.tails`, `queues.fill`, `f.thin`, `f.all`). Two exact algebraic identities halve those hot array touches to **two** (`2 KiB`: `queues.pos`, `f.thin`) at **`+0 B` heap**:
1. **Bit-Lattice Cursor Fusion (`BYTES = 1024 = 2^{10}`, `CHUNK = 512`):** Encode `pos[a] = (tail << 10) | fill` (`u32::MAX` before first chunk, `in_out_lim = in_out << 10`). Because `fill in 0..=512 < 1024`, incrementing `pos[a] += w` never carries into bit `10` even at `fill == 512`, `pos[a] < in_out_lim` tests `tail < in_out` in one compare, and the byte offset in `out` satisfies the zero-shift, zero-multiply identity:
   $$\mathrm{at} \;=\; \mathrm{tail} \cdot 1024 + 2 \cdot \mathrm{fill} \;=\; p + (p \mathbin{\&} 1023).$$
2. **Algebraic Decomposition `all[a] = thin[a] + thick_sum[a]`:** Store only `thick_sum[a]` in `f.all[a]` during `Pass 1` (eliminating `~489K` L1D RMWs to `f.all[a]` on the `98%` thin-run path!), read `to = f.thin[a] + f.all[a]` on the `2%` thick-strip path, and reconstruct `all[a] += thin[a]` once in `finish` (`256` iterations, `~50 ns`).

---

## 2. Surgical Ablation & Native Safe-Rust Confrontation (`70` Pairs/Block `1T`, `45` Passes `2T`)

All arms passed `18,414` exhaustive unmerged-record binary BWTs (`1..=10` $\times$ `LIMIT` $\in \{1, 4, 16\}$) + `dense-300k` + `6 / 6` Wikidata blocks byte-for-byte.

### 2.1 Initial Bundled Test & 5-Factor Surgical Ablation (`50` Rounds/Block, `construct` Delta vs `BiOp48`)
| Factor Isolated on `BiOp48` | `6`-Block `construct` Delta vs `BiOp48` (`6,157.1 µs`) | Cause |
|---|---:|---|
| `+Max` (carry-over `for &next` `maximal`) | **`+887.9 µs`** (catastrophic!) | Flattening `while` merge creates a loop-carried dependency on `(c, width)` |
| `+SWAR4` (`w <= 4` 8-byte `u64` store) | `+15.0 µs` (`+549 µs` for `P1-W1`) | `7.1%` (`27.0%`) width-branch mispredicts beat AVX2/SSE 32-byte store savings |
| **`+Pos` (`(tail << 10) \| fill`)** | **`−28.8 µs`** | Removes `tails[256]` (`−1 KiB` stack) & `tail * BYTES + 2 * f` arithmetic |
| **`+All` (`all = thin + thick_sum`)** | **`−169.5 µs` (`6 / 6` blocks faster)** | Eliminates `489K` thin-run RMWs to `f.all[a]` |

### 2.2 Isolated Verification of `+All+Pos` (`BiOp48-Pass1`, `70` Pairs/Block `1T`, `45` Passes `2T`)
| Block / Suite | `A/A` (`M2/M1`) | `BiOp48` (Cycle 28) | **`BiOp48 + All + Pos` (`Neon-C29`)** |
|---|---:|---:|---:|
| `block-0.L` (`1T p10`) | `0.981` | `0.906x` | **`0.893x`** |
| `block-2.L` (`1T p10`) | `0.998` | `0.895x` | **`0.887x`** |
| `block-85.L` (`1T p10`) | `1.008` | **`0.869x`** | `0.893x` |
| `block-143.L` (`1T p10`) | `0.999` | `0.834x` | **`0.833x`** |
| `block-210.L` (`1T p10`) | `0.990` | `0.845x` | **`0.819x`** |
| `block-207.L` (`1T p10`) | `0.997` | `0.866x` | **`0.853x`** |
| **`1T` 6-Block Sum (`p10`)** | **`0.9959`** | `0.8625x` | **`0.8552x` (`6 / 6` `< 0.90x`)** |
| **`2T` Concurrent (`p10`)** | **`0.9775`** | `0.8696x` | **`0.8585x`** (`9,790 / 11,404 µs`) |
| **`2T` Concurrent (`p50`)** | **`0.9944`** | `0.8756x` | **`0.8499x`** (`10,189 / 11,988 µs`, **`−2.57%` vs `BiOp48`!**) |
| **6-Blk Worker Scratch** | `1,171.7 KiB` | `945.8 KiB` (`−19.28%`) | **`945.8 KiB` (`−19.28%`, `−1 KiB` stack)** |

---

## 3. Cycle 29 Verdicts (One-Line Kills & Promotion)

- **Kill (`Carry-Over maximal` Loop):** Killed (`+887.9 µs` `construct` regression) because replacing `master`'s `99.8%`-untaken inner `while` merge with a single `for` loop introduces a loop-carried dependency on `(c, width)`.
- **Kill (`Pass1-SWAR4` & `Pass1-W1` Narrow `OutQ16::push` Stores):** Killed (`+15.0 µs` and `+549 µs`) because branching on `w <= 4` (`7.1%` mispredict) or `w == 1` (`27.0%` mispredict) costs more than `OutQ16`'s branchless 32-byte iota store.
- **Promote (`All + Pos` Pass-1 Algebra — New Neon Champion `Neon-C29`):** Promoted (`945.8 KiB = −19.28%` WS, `−1 KiB` stack, **`0.8552x` `1T` across `6 / 6` blocks `< 0.90x`**, **`0.8585x` p10 / `0.8499x` p50 `2T`** vs `0.5.11` `master`).
