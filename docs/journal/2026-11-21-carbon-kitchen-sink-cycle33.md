# Carbon Kitchen Sink (Neon) — Cycle 33: Unique-Symbol `mark_byte` MSHR-Burst Wiring, Fused-Word `12 KiB` Dedup, & In-Place 4-Cursor `XOR-QuadEmit`

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`mibz2 0.5.11`, tagged `v0.5.11`, `src/` identical to `90e671966`)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical Framing

Cycle 27's phase profile left three targets beyond `Pass 1` and `Elim` (upgraded in Cycles 29 and 31):
1. **`wire_xor` (`1,595 µs`, `15.3%`):** In `Neon-C31`, `OutQ16::wire_xor` guarded `v == mark` by epoch (`(mark & !0xFFFF) != hi`), which still sent `25–50%` of the entire block (every byte in `mark`'s epoch) into the unrolled-1× fallback, and exposed only `4` scatter RMWs `edges[v0..v3]` per aliasing barrier against `edges[u0..u0+4]`.
2. **`dedup` (`738 µs`, `7.1%`) + `join` (`elim`):** `e.len(w)` and `e.short_u128(w)` performed two redundant `w <= 256` branches and read `a[r + 3]` twice per rule; `hash_u128` used three 64-bit multiplies; and the `24 KiB` (`10/12`-bit) quotient table occupied `75%` of the `32 KiB` L1D alongside `inline16` and `e.a`.
3. **`walk` (`3,999 µs`, `39%`):** Can an **in-place, zero-scratch, `1.00m`-load** 4-cursor bidirectional XOR-torus walk (`XOR-QuadEmit`, $\mu = 4 \to 2$) inside `out[0..n]` escape Cycle 24's staging-buffer obstruction (`XOR-BiEmit4`) and beat `BiEmit` ($\mu = 2$) on real text?

### Theorem 33.1 (Unique-Symbol Mark Isolation & MSHR-Burst Coalescing in `OutQ16::wire_xor`)
Each byte $a \in \{0,\dots,255\}$ queues a pairwise-disjoint subset $S_a \subset [0, m)$ of retained ranks. Because `mark` is the unique rank of the primary row $p$ with $L[p] = a^* = \texttt{f.mark\_byte}$ (already visited in Pass 1), $\texttt{mark} \notin S_a$ for all `255` bytes $a \neq a^*$.
- Hoisting `a != mark_byte` outside the band loop routes **`>99%`** of all ranks through a branch-free `wire_xor_nomark`.
- Unpacking `u64::from_le_bytes` (`4` ranks/load) and coalescing sequential updates into a single slice `&mut edges[u0..u0 + K]` (`1` bounds check) followed by `K` scattered RMWs `edges[v_k] ^= (u0 + k)` exposes `K` simultaneous independent past-L2 MSHR requests per store-aliasing epoch (`K = 8` or `16` vs `K = 4`).

### Theorem 33.2 (Fused Operad Extraction & Compact `12 KiB` Direct-Mapped Quotient)
Fusing `len` and `short_u128` into `fetch_word(w) -> (u128, u32)` performs one `w <= 256` branch and one 16-byte rule load in both `join()` and `dedup`. Adding a 1-entry per-byte register cache `(last_w, last_c)`, collapsing `hash_u128` to a single 64-bit multiply `(lo ^ hi.rotate_left(29) ^ (wlen << 56)).wrapping_mul(C)`, and halving the arena-hosted quotient table from `24 KiB` (`PAIR_BITS = 10, STR_BITS = 12`) to `12 KiB` (`PAIR_BITS = 9, STR_BITS = 11`) costs only `+7` words (`+0.12%`, `5,687 -> 5,694` across `6` blocks) while halving L1D pressure and `fill(0)` cost.

### Theorem 33.3 (In-Place `XOR-QuadEmit` Feasibility & Branch-Amplification / Stitch Obstruction)
Picking a second anchor $q$ in $O(1)$ from the first queued rank of the heaviest byte $a_q \neq a^*$ splits the cycle into two arcs of lengths $U$ and $n - U$ ($U \sim \mathrm{Uniform}(0, n)$). Placing `(f0, b1, f1, b0)` at `(0, n/2, n/2, n)` inside `out[0..n]` gives an exact, **`+0 B`-scratch**, **`1.00m`-load** 4-cursor walk with expected serial makespan $\frac{1}{2}\mathbb{E}[\max(U, n-U)] = 0.375\,m$ (`−25%` vs `BiEmit`'s `0.500 m`) and lossless 2-cursor fallback on periodic disjoint cycles or buffer skew.
- **Obstruction on real text:** In lockstep 4-cursor traversal, the combined short-word guard `(lf0 != LONG) & (lb0 != LONG) & (lf1 != LONG) & (lb1 != LONG)` amplifies per-cursor long-word frequency $p$ to $1 - (1 - p)^4 \approx 15\text{--}28\%$ on Wikidata, triggering Theorem 30.1 ROB flushes, while in-place `copy_within` stitching moves `~250–450 KiB` on `900 KiB` blocks. Conversely, on `0`-long `dense-300k` ($p = 0$), `XOR-QuadEmit` genuinely wins (`0.736×` vs `Neon-C31`'s `0.769×`).

---

## 2. Verification & Native Safe-Rust A/B Confrontation

All candidates (`#![forbid(unsafe_code)]`) were compiled with `rustc -C opt-level=3 -C target-cpu=native -C codegen-units=1` in the same binary as verbatim `0.5.11` `master` (`MasterA1` / `MasterA2` A/A control).

- **Parity:** `100%` byte-exact on all `6/6` Wikidata blocks, `dense-300k`, `periodic-150k-d2`, `periodic-105k-d3` (`3` disjoint cycles!), and `30` malformed non-BWT inputs at `n = 70,000 >= 65,536`.
- **Scratch (6-block cumulative):** `Master = 1,171.7 KiB`, `W8+D24 / W16+D12 / Quad = 945.8 KiB` (**`−19.28%`**, `+0 B` vs `Neon-C31`).
- **Unique words (6-block sum):** `Neon-C31 = 5,687`, `Dedup24 = 5,692`, `Dedup12 = 5,694`.

### 2.1 Exploratory Sweep (`run_c33.rs`: `40` rounds/block `1T`, `30` passes `2T`)

| Candidate | `construct` (`µs`) | `walk` (`µs`) | `1T` Sum (`µs`) | `1T` vs `M1` | `dense-300k` | `2T` p10 / p50 vs `M1` |
|---|---:|---:|---:|---:|---:|---:|
| `MasterA1` (`0.5.11`) | `6,021.2` | `6,160.2` | `12,226.6` | `1.0000×` | `1.000×` | `1.0000× / 1.0000×` |
| `MasterA2` (A/A) | — | — | `12,118.5` | `0.9912×` | — | `0.9897× / 0.9929×` |
| `Neon-C31` (prev champ) | `6,260.0` | `3,999.6` | `10,269.8` | `0.8400×` | `0.769×` | `0.8493× / 0.8366×` |
| `Wire4` (`K=4`) | `6,267.6` (`+7.6`) | `4,012.1` | `10,283.2` | `0.8411×` | — | — |
| `Wire8` (`K=8`) | `6,087.9` (`−172.1`) | `3,990.0` | `10,067.0` | `0.8234×` | `0.770×` | — |
| `Dedup24` (`10/12`) | `6,204.7` (`−55.2`) | `3,990.2` | `10,252.9` | `0.8386×` | — | — |
| `Dedup12` (`9/11`) | — | — | `10,224.3` | `0.8362×` | — | — |
| `W8+D24` | `6,082.3` (`−177.6`) | `4,020.0` | `10,147.2` | `0.8299×` | `0.786×` | **`0.8230× / 0.8215×`** |
| `Quad` ($\mu=4\to 2$) | `6,042.8` (`−217.1`) | `4,482.4` (`+482.8`) | `10,580.5` | `0.8654×` | **`0.736×`** | `0.8528× / 0.8645×` |

### 2.2 High-Precision Confirmation (`run_c33b.rs`: `60` rounds/block `1T`, `40` passes `2T`)

| Candidate | `construct` (`µs`) | `walk` (`µs`) | `1T` Sum (`µs`) | `1T` vs `M1` | `Δ` vs `C31` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `MasterA1` (`0.5.11`) | `6,102.3` | `6,326.7` | `12,372.4` | `1.0000×` | `+1,934.4` | `12,376.5 (1.0000×)` | `12,764.0 (1.0000×)` |
| `MasterA2` (A/A) | `6,068.3` | `6,327.2` | `12,385.6` | `1.0011×` | `+1,947.6` | `12,534.9 (1.0128×)` | `12,797.4 (1.0026×)` |
| `Neon-C31` | `6,366.5` | `4,059.3` | `10,438.0` | `0.8437×` | `0.0` | `10,607.6 (0.8571×)` | `10,916.2 (0.8552×)` |
| `Wire8` | `6,189.8` | `4,059.2` | `10,271.2` | `0.8302×` | `−166.9` | `10,420.3 (0.8419×)` | `10,649.6 (0.8344×)` |
| `Wire16` | `6,291.3` | `4,067.5` | `10,358.8` | `0.8373×` | `−79.2` | `10,520.0 (0.8500×)` | `10,762.6 (0.8432×)` |
| `W8+D12` | `6,165.3` | `4,105.4` | `10,296.7` | `0.8322×` | `−141.4` | `10,508.9 (0.8491×)` | `10,744.6 (0.8418×)` |
| **`W16+D12` (`Neon-C33`)** | **`6,144.8`** | **`4,068.2`** | **`10,205.5`** | **`0.8249×`** | **`−232.6`** | **`10,326.4 (0.8344×)`** | **`10,625.9 (0.8325×)`** |

Per-block `1T` ratios for **`Neon-C33` (`W16+D12`)** vs `MasterA1` (A/A in parens):
`block-0: 0.858× (0.998)`, `block-2: 0.839× (0.991)`, `block-85: 0.858× (1.001)`, `block-143: 0.800× (1.007)`, `block-210: 0.809× (1.001)`, `block-207: 0.821× (1.002)`.
Notably, `Neon-C33`'s `construct` (`6,144.8 µs`) is now within **`+42.5 µs` (`+0.7%`)** of `0.5.11` `master`'s non-XOR `construct` (`6,102.3 µs`) across all 6 blocks combined — **only `7 µs`/block** to build the entire bidirectional XOR graph!

---

## 3. One-Line Kills & Promotions

- **KILL `Wire4` (`K = 4`):** `+13.4 µs` vs `Neon-C31` (`0.8411×`); 4 scatter RMWs per store-aliasing barrier fail to saturate past-L2 MSHRs.
- **KILL `XOR-QuadEmit` (`Quad`, $\mu = 4 \to 2$):** Despite winning `0`-long `dense-300k` (`0.736×` vs `0.769×`), it loses on Wikidata by `+482.8 µs` in `walk` (`0.8654×` 1T, `0.8528×/0.8645×` 2T) via $1-(1-p)^4$ LONG-word branch amplification and `~450 KiB` `copy_within` stitch overhead (Theorem 33.3).
- **PROMOTE `Neon-C33` (`W16+D12` — `mark_byte` isolation + 16/8/4-wide `u64` slice-checked `wire_xor_nomark` + fused `fetch_word` + `last_word` cache + 1-mul `hash_u128` + `12 KiB` HT):** Beats `Neon-C31` on `6/6` blocks (`−232.6 µs` `1T`, `−281.2 / −290.3 µs` `2T`), reaching **`0.8249×` `1T`** (vs A/A `1.0011×`) and **`0.8344×` p10 / `0.8325×` p50 `2T`** (vs A/A `1.0128×/1.0026×`) at **`945.8 KiB` (`−19.28%` WS)**.

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle33/` (`gen_c33.py`, `gen_c33b.py`, `run_c33.rs`, `run_c33b.rs`).
- **Cycle 34 (Neon):** Pass-1 `mark` check hoisting (`p.wrapping_sub(x) < w` eliminated from `99.999%` of runs), adaptive `XOR_GATE` calibration (`32K / 48K / 64K / 96K`) across `m = 20K..250K`, and `1T / 2T / 4T` L3 contention scaling comparing **`Neon-Bi` ($\mu = 2$)** vs **`Neon-Diet` ($\mu = 1$)** vs `0.5.11` `master`.
