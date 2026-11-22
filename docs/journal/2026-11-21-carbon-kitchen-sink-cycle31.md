# Carbon Kitchen Sink (Neon) — Cycle 31: $\mathbb{Z}/2\mathbb{Z}$ Side-Exchange Symmetry (`CMOV-Elim`) vs Static-Side Tail-Cut & Single-Store Link Packing (`Neon-C31`: `0.844x` 1T, `0.837x` 2T, `−19.3%` WS)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Testing $\mathbb{Z}/2\mathbb{Z}$ top/bottom side-exchange symmetry (`side = usize::from(aw > bw)`, branchless `CMOV-Elim`) against its clean ablation `Links-Only` (`cut_tail` exploiting `next(tail, side) == 0` + single-store `Narrow::set_links` eliminating dual RMW hazards) on top of `Neon-C29` and `0.5.11` `master`.

---

## 1. Theorem 31.1 (Autocorrelated Width Peeling & Static-Side Store-Forwarding in `Narrow` `Elim`)

Inside `Elim::eliminate::<false>` (`79,349` steps across the 6 Wikidata blocks), each step operates on `(x, y) = (prev(0, 0), prev(0, 1))`:
1. **Tail-Sentinel Invariant (`cut_tail`) & Single-Store Link Packing (`set_links`):**
   - Because `x = prev(0, 0)` and `y = prev(0, 1)` are always immediately left of sentinel `0`, `next(x, 0) == 0` and `next(y, 1) == 0` identically, eliminating `next` load/shift on every cut.
   - In `Narrow`, `set_prev` followed by `set_next` executes two read-modify-writes on the **same `u32` word** `a[id * 4 + side]`; replacing them with a single direct store `L::set_links(a, id, side, at, next)` (`a[id * 4 + side] = at | (next << 16)`) eliminates a 4-cycle store-to-load forwarding stall per step (`−179.7 µs` across 6 blocks, winning on `6 / 6` blocks!).
2. **Why Static-Side `if aw <= bw` Beats Dynamic-Side `CMOV-Elim` (`−344.4 µs`):**
   - Width peeling in BWT IETs is **locally autocorrelated** (a wide node peels multiple narrow partners sequentially), making `if aw <= bw` TAGE-predictable (`~75–80%`).
   - Collapsing both sides via runtime `side = usize::from(aw > bw)` chains `gt -> cmov(narrow, wide) -> dynamic index (+ side)` onto the critical pointer-chasing path and forces `diff == 0` on both sides (`+164.7 µs` vs `Neon-C29`, `+344.4 µs` vs `Links-Only`).

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`70` Pairs/Block `1T`, `45` Passes `2T`)

Passed `12,276` exhaustive binary BWTs (`1..=10` $\times$ `LIMIT` $\in \{1, 4, 16\}$) + `dense-300k` + `6 / 6` Wikidata blocks byte-for-byte:

| Block / Metric | `A/A` (`M2/M1`) | `Neon-C29` | **`Neon-C31` (`Links-Only`)** | `CMOV-Bi` ($\mathbb{Z}/2\mathbb{Z}$) | `Diet-CMOV` ($\mu=1$) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10` / `c`) | `0.986` | `0.895x` (`756.8`) | **`0.866x`** (`720.8`, **`−36.0`**) | `0.902x` (`+6.1`) | `0.931x` (`678.7`) |
| `block-2.L` (`1T p10` / `c`) | `0.993` | `0.881x` (`891.2`) | **`0.860x`** (`864.6`, **`−26.6`**) | `0.882x` (`+41.4`) | `0.918x` (`810.8`) |
| `block-85.L` (`1T p10` / `c`) | `0.992` | `0.871x` (`743.6`) | **`0.864x`** (`731.1`, **`−12.5`**) | `0.873x` (`+15.9`) | `0.920x` (`620.6`) |
| `block-143.L` (`1T p10` / `c`) | `0.987` | `0.825x` (`1439.3`) | **`0.821x`** (`1373.8`, **`−65.4`**) | `0.826x` (`+63.3`) | `0.902x` (`1298.6`) |
| `block-210.L` (`1T p10` / `c`) | `0.992` | `0.829x` (`1150.0`) | **`0.836x`** (`1133.9`, **`−16.0`**) | `0.837x` (`+34.9`) | `0.925x` (`962.3`) |
| `block-207.L` (`1T p10` / `c`) | `1.006` | `0.850x` (`1009.7`) | **`0.840x`** (`986.5`, **`−23.2`**) | `0.854x` (`+3.0`) | `0.920x` (`888.7`) |
| **6-Blk `construct` Sum** | `5,622.3 µs` | `5,990.4 µs` | **`5,810.7 µs` (`−179.7 µs`)** | `6,155.1 µs` (`+164.7`) | **`5,259.7 µs` (`0.9355x`)** |
| **`1T` 6-Block Sum (`p10`)** | **`0.9926`** | `0.8516x` | **`0.8435x` (`−15.65%`)** | `0.8556x` | **`0.9175x`** |
| **`dense-300k`** | `1.009` | `0.779x` | **`0.771x`** | `0.782x` | `0.971x` |
| **`2T` Concurrent (`p10 / p50`)** | **`0.9970 / 0.9929`** | `0.8473 / 0.8483` | **`0.8373 / 0.8426` (`−16.27%`)** | `0.8643 / 0.8689` | **`0.9293 / 0.9235`** |
| **6-Blk Worker Scratch** | `1,171.7 KiB` | `945.8 KiB` (`−19.28%`) | **`945.8 KiB` (`−19.28%`)** | `945.8 KiB` | **`945.8 KiB` (`−19.28%`)** |

*(Note: on `block-0.L` and `block-143.L`, `Neon-C31`'s `construct` is now **faster than `0.5.11` `master`'s `construct` even after paying the entire `wire_xor` cost**!)*

---

## 3. Cycle 31 Verdicts (One-Line Kill & Promotion)

- **Kill (`CMOV-Elim` Dynamic-Side $\mathbb{Z}/2\mathbb{Z}$ Branchless Induction):** Killed (`construct` `+164.7 µs` vs `Neon-C29` and `+344.4 µs` vs `Links-Only`; `2T` `0.8643x` vs `0.8373x`) because autocorrelated width peeling is TAGE-predictable while dynamic `side` + `cmov` serializes the critical link-address path.
- **Promote (`cut_tail` + Single-Store `set_links` — New Neon Champion `Neon-C31`):** Promoted (`−179.7 µs` `construct` on `6 / 6` blocks, **`0.8435x` `1T`**, **`0.8373x` p10 / `0.8426x` p50 `2T`** vs A/A `0.9970`, **`945.8 KiB = −19.28%` WS**).
