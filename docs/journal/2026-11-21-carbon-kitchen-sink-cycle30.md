# Carbon Kitchen Sink (Neon) — Cycle 30: Branch-Entropy Dominance Over Store Width in Past-L2 Dual-Cursor `emit` (`Walk-Clean`, `Walk-BwdU64`, `Walk-LitFast`, `Walk-BothU64`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Testing four microarchitectural transformations of `emit_xor_bi_no_off` (`3,694 µs = 37.4%` of `Neon-C29`): peeling step 0 + eliminating redundant guards (`Walk-Clean`), single-`shl` 64-bit backward stores for `lb <= 8` (`Walk-BwdU64`, `Walk-BothU64`), and 1-byte literal bypass (`Walk-LitFast`, `c < 256`).

---

## 1. Theorem 30.1 (Branch-Entropy ROB-Flush Obstruction Inside Past-L2 Dual-Cursor `emit`)

In `Neon-C29` (`ab_all_pos`), `emit_xor_bi_no_off` achieves **`0.6355x`** of `0.5.11` `master`'s `walk` (`0.600x` on `block-210.L`, `0.604x` on `block-143.L`), operating at `~16 ns` per dual-cursor iteration (`~8 ns/vertex`) — right at the past-L2 memory-latency floor of two concurrent `edges[cur_f]` / `edges[cur_b]` loads.
- Inside that `16 ns` memory-latency shadow, the unconditional 16-byte forward copy and `u128` left-shifted backward copy execute completely in parallel with the next iteration's in-flight `edges` loads.
- Adding ANY data-dependent length/literal branch inside the hot loop (`cb < 256` with `~28%` word rate / `H = 0.86` bits, or `lb <= 8` with `~4%` 9–16 B word rate / `H = 0.24` bits) triggers ROB flushes that **cancel the speculative `edges[nf]` / `edges[pb]` loads already in flight**, turning a `4%–28%` branch into **`+419 µs` (`Walk-BwdU64`) to `+835 µs` (`Walk-LitFast`)** of serialized memory-latency stalls!
- Even fusing the two independent `~99.5%`-taken guards `if lf != LONG` and `if lb != LONG` into a single `if lf != LONG && lb != LONG` (`Walk-Clean`) doubles the combined rare-branch miss probability (`+30.2 µs`).

---

## 2. Native Safe-Rust Confrontation Against `0.5.11` `master` (`70` Pairs/Block `1T`, `45` Passes `2T`)

All arms passed byte-for-byte parity on all `6 / 6` Wikidata blocks and `dense-300k`:

| Block / Metric | `A/A` (`M2/M1`) | **`Neon-C29`** (Baseline) | `Walk-Clean` | `Walk-BwdU64` (`lb<=8`) | `Walk-BothU64` | `Walk-LitFast` (`c<256`) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` (`1T p10` / `walk`) | `0.987` | **`0.895x`** (`376.1 µs`) | `0.893x` (`+0.7`) | `0.928x` (`+31.2`) | `0.941x` (`+60.3`) | `0.941x` (`+67.7`) |
| `block-2.L` (`1T p10` / `walk`) | `1.001` | **`0.893x`** (`469.2 µs`) | `0.889x` (`+2.7`) | `0.917x` (`+50.8`) | `0.941x` (`+88.1`) | `0.967x` (`+121.5`) |
| `block-85.L` (`1T p10` / `walk`) | `1.004` | **`0.873x`** (`610.1 µs`) | `0.874x` (`+6.8`) | `0.885x` (`+66.9`) | `0.938x` (`+98.3`) | `0.889x` (`+73.6`) |
| `block-143.L` (`1T p10` / `walk`) | `1.000` | **`0.827x`** (`839.9 µs`) | `0.826x` (`+12.0`) | `0.885x` (`+111.4`) | `0.897x` (`+195.3`) | `0.923x` (`+285.2`) |
| `block-210.L` (`1T p10` / `walk`) | `0.999` | **`0.810x`** (`842.2 µs`) | `0.824x` (`+7.6`) | `0.867x` (`+99.9`) | `0.888x` (`+168.6`) | `0.883x` (`+156.4`) |
| `block-207.L` (`1T p10` / `walk`) | `1.001` | **`0.857x`** (`556.2 µs`) | `0.853x` (`+0.4`) | `0.885x` (`+59.1`) | `0.909x` (`+101.4`) | `0.919x` (`+130.9`) |
| **6-Blk `walk` Sum** | `5,812.3 µs` | **`3,693.7 µs` (`0.6355x`)** | `3,723.9` (`+30.2`) | `4,113.0` (`+419.3`) | `4,405.8` (`+712.1`) | `4,529.0` (`+835.3`) |
| **`1T` 6-Block Sum (`p10`)** | **`0.9993`** | **`0.8508x`** | `0.8523x` | `0.8900x` | `0.9133x` | `0.9169x` |
| **`dense-300k`** | `0.990` | **`0.740x`** | `0.766x` | `0.806x` | `0.797x` | `0.768x` |
| **`2T` Concurrent (`p10 / p50`)** | **`1.0044 / 0.9929`** | **`0.8409 / 0.8526`** | `0.8357 / 0.8621` | `0.8908 / 0.8884` | `0.9285 / 0.9149` | `0.9212 / 0.9321` |

---

## 3. Cycle 30 Verdicts (One-Line Kills)

- **Kill (`Walk-BwdU64` / `Walk-BothU64` 64-Bit `len <= 8` Store Fast-Path):** Killed (`walk` `+419.3 µs` / `+712.1 µs`, `1T` `0.8900x` / `0.9133x` vs `Neon-C29`'s `0.8508x`) because `~4%` 9–16 B length mispredicts flush the ROB and serialize in-flight `edges` loads.
- **Kill (`Walk-LitFast` `c < 256` Literal Bypass):** Killed (`walk` `+835.3 µs`, `1T` `0.9169x`, `2T` `0.9212x`) because `H(0.28)=0.86`-bit literal-vs-word branch mispredicts destroy dual-cursor memory-level parallelism.
- **Kill (`Walk-Clean` Combined `lf != LONG && lb != LONG` Guard):** Killed (`walk` `+30.2 µs`, `1T` `0.8523x` vs `0.8508x`, `2T p50` `0.8621x` vs `0.8526x`) because fusing the two rare `LONG` guards degrades branch prediction vs independent guards; `Neon-C29` remains champion (`0.8508x` 1T, `0.8409x/0.8526x` 2T, `0.740x` dense).
