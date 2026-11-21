# Carbon Kitchen Sink — Cycle 23: Gauss–Kuzmin Absence of Interior Spirals (`Ostrowski-Shift4`) & `OutQ16` Chunk-Scale Overflow Geometry

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Ostrowski / continued-fraction multiplicative Rauzy–Veech steps (`aw > (bw << 2)`) on multi-thousand-interval IETs, and L1D active-tail capacity vs `Spill::ovf` overflow geometry across `OutQ16` chunk scales (`CHUNK = 128, 256, 512, 1024`).

---

## 1. Mathematical & Architectural Questions

1. **Always-On Multiplication-Free Ostrowski Guard (`Ostrowski-Shift4`):** [`endpoint.rs`](../../../../src/open/endpoint.rs) specializes `eliminate::<false>` (`widest <= 32,768`) vs `eliminate::<true>` (`widest > 32,768`) because `(aw - 1) / bw >= 2` executes hardware integer division. Replacing that guard with the 1-cycle shift test `aw > (bw << 2)` (`q >= 4`) avoids hardware `div` unless $q \ge 4$ steps are guaranteed to collapse. How many $q \ge 4$ spirals actually occur inside normal `4,130–12,835`-node Wikidata IETs, and can unified `Ostrowski-Shift4` beat two-specialization `master`?
2. **`OutQ16` Chunk-Scale Trade-Off (`CHUNK` $\in \{128, 256, 512, 1024\}$):** With $\alpha \approx 90$ active bytes, `master`'s `CHUNK = 512` (`1 KiB`) keeps $\approx 90\text{ KiB}$ of open tail chunks in `Pass 1` (spilling `32–48 KiB` L1D). Does `CHUNK = 256` (`45 KiB` active tails, L1D-resident) win in `Pass 1`, or does `CHUNK = 1024` (`2 KiB`, halving `push_slow`/`wire_slow` calls) beat `CHUNK = 512` without triggering `Spill::ovf` heap overflows?

### Proposition 23.1 (2-Cycle Partner Scarcity on Large-Alphabet IETs & `OutQ` Overflow Trade-Off)
1. **Complete Absence of $q \ge 4$ Spirals on Normal Blocks (`0 / 79,349` steps):** A Rauzy–Veech spiral requires the top-right winner `x` and bottom-right winner `y` to form a **simultaneous 2-cycle** (`prev(y, 1) == x` or `prev(x, 0) == y`). On large-alphabet structured blocks ($N_{\mathrm{nodes}} \in [4{,}130, 12{,}835]$), random partner permutation suppresses 2-cycle incidence to $\approx 1 / N_{\mathrm{nodes}}$, yielding **exact `0` rule reductions (`+0` across all `79,349` steps on 6 Wikidata blocks + `21,471` steps on `dense-300k`)**, while the two untaken guards inside `Elim::eliminate` cost `+2.1%` end-to-end (`1.0213x`).
2. **Saddle Optimality of `CHUNK = 512`:** Smaller chunks (`128, 256`) multiply `push_slow`/`wire_slow` calls by `4x`/`2x` (`1.0923x`, `1.0301x`) and enlarge `Spill::next`, whereas `CHUNK = 1024` (`2 KiB`) doubles internal tail waste (`\alpha \cdot 2\text{ KiB} \approx 192\text{ KiB}`), exhausting `out[0..n)` and spilling into heap-allocated `Spill::ovf` on sub-maximal or dense blocks (`1.050x` on `dense-300k`).

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`55` Pairs/Block `1T`, `35` Passes `2T`)

All arms passed `8 / 8` byte-for-byte parity across the 6 authenticated blocks, `dense-300k`, and `spiral-200k`:

| Block / Suite | `A/A` (`Master2/Master1`) | `Ostrowski-Shift4` ($q \ge 4$) | `OutQ-128` (`256 B`) | `OutQ-256` (`512 B`) | `OutQ-1024` (`2 KiB`) |
|---|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10`) | `0.996` | `1.030` (`+0` rules saved) | `1.083` | `1.017` | `0.987` |
| `block-2.L` (`1T p10`) | `1.004` | `1.027` (`+0` rules saved) | `1.083` | `1.027` | `0.988` |
| `block-85.L` (`1T p10`) | `0.997` | `1.021` (`+0` rules saved) | `1.114` | `1.042` | `0.976` |
| `block-143.L` (`1T p10`) | `0.999` | `1.021` (`+0` rules saved) | `1.081` | `1.031` | `0.984` |
| `block-210.L` (`1T p10`) | `0.997` | `1.011` (`+0` rules saved) | `1.101` | `1.028` | `0.984` |
| `block-207.L` (`1T p10`) | `0.980` | `1.025` (`+0` rules saved) | `1.093` | `1.032` | `0.989` |
| **`1T` 6-Block Sum** | **`0.9956`** | `1.0213x` | `1.0923x` | `1.0301x` | `0.9846x` |
| **`dense-300k` (`0.216`)** | `0.999` | `1.059x` (`+0` rules saved) | `1.071x` | `1.027x` | **`1.050x` (`ovf` spill!)** |
| `spiral-200k` | `0.997` | `0.997x` | `1.002x` | `0.995x` | `1.004x` |
| **`2T` Concurrent (`p10`)** | **`1.0030`** | `1.0220x` | — | `1.0216x` | `0.9909x` |
| **6-Blk Scratch** | `1,172.7 KiB` | `1,172.7 KiB` | `1,178.7 KiB` | `1,174.7 KiB` | `1,172.7 KiB` (`+ovf` on `<500K`) |

---

## 3. Cycle 23 Verdicts (One-Line Kills)

- **Kill (`Ostrowski-Shift4` Always-On Spiral Guard):** Killed (`1.0213x` 1T, `1.0220x` 2T) because `0` of `79,349` steps on the 6 Wikidata blocks (`and 0 of 21,471 on dense-300k`) form a $q \ge 4$ 2-cycle spiral, confirming `master`'s `CERTIFIED_WIDTH = 32_768` two-loop split is strictly optimal.
- **Kill (`OutQ-128` / `OutQ-256` L1D-Resident Small Chunks):** Killed (`1.0923x` and `1.0301x`) because `4x`/`2x` more `push_slow` and `wire_slow` chunk transitions outweigh L1D active-tail locality and grow `Spill::next` (`+2–6 KiB`).
- **Kill (`OutQ-1024` `2 KiB` Large Chunks):** Killed because `~192 KiB` of partially-filled byte tails exhausts `out` on sub-maximal/dense blocks (`1.050x` regression on `dense-300k` via `Spill::ovf` heap allocations) for `< 1%` `2T` gain (`0.9909x` vs A/A `1.0030x`).
