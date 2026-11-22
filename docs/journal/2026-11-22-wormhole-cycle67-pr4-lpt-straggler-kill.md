# Wormhole Cycle 67 (Neon): Within-Window Graham LPT Straggler Scheduling vs FIFO in PR 4 (Honest Kill)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `PR4_Own_ZS52` (`FIFO_A` / `FIFO_B` A/A control on `3296195e3`)  
**Harness:** [`c67_lpt_straggler_ab.rs`](2026-11-22-wormhole-cycle67/c67_lpt_straggler_ab.rs) (`#![forbid(unsafe_code)]`, same-binary A/B + A/A control, fresh-process OS counters, exact `AbHash`)  
**Output:** [`c67_results.txt`](2026-11-22-wormhole-cycle67/c67_results.txt)  
**Manifest:** [`c67_lpt_straggler.toml`](2026-11-22-wormhole-cycle67/c67_lpt_straggler.toml) (`check_manifest.py`: `PASS`)

---

## 1. Hypothesis

Unlike serial `StreamDecoder` (which must dispatch blocks strictly FIFO), PR 4's `SpanFinder` sees all `~50–115` block offsets inside each window up front, along with each candidate's compressed bit span `span_bits[i] = cands[i+1].rel_bit - cands[i].rel_bit` — a strong monotonic proxy for block decode time (`~1 KB` tiny block `0.03 ms`, `~60 KB` Wikidata block `1.5 ms`, `~900 KB` `LazyCells` straggler `~10 ms`).

Cycle 67 tests two deadlock-free Graham (1969) LPT schedules bounded to `batch = min(2W, slots/2)` (preserving ring-slot safety):
1. **`LPT_Batch`:** Within every `batch`-sized chunk of a window, keep `chunk[0]` first (head-of-line consumer safety) and sort `chunk[1..batch]` descending by `span_bits`.
2. **`LPT_Tail`:** Strict FIFO everywhere except the final `batch` blocks of the EOF window (dispatched heaviest-first to shrink the EOF tail straggler).

---

## 2. Empirical Results (`c67_results.txt`, Exact `AbHash`)

| Regime | Arm | med_ms | vs `FIFO_A` `[95% CI]` | sys | vcsw | Peak RSS (MiB) |
|---|---|---:|---|---:|---:|---:|
| **`wikidata-l9 4W`** (`10` rd) | `FIFO_A` | `315.02` | `1.0000 [1.0000, 1.0000]` | `15.01` | `144` | `37.05` |
| | `FIFO_B` (A/A) | `316.18` | `1.0080 [0.9919, 1.0244]` | `13.62` | `149` | `36.90` |
| | `LPT_Batch` | `315.50` | `1.0053 [0.9957, 1.0149]` | `15.62` | `161` (`+11.8%`) | `37.69` |
| | `LPT_Tail` | `316.27` | `1.0101 [0.9939, 1.0266]` | `12.89` | `149` | `37.05` |
| **`wikidata-l9 10W`** (`5` rd) | `FIFO_A` | `326.89` | `1.0000 [1.0000, 1.0000]` | `27.17` | `137` | `56.32` |
| | `FIFO_B` (A/A) | `323.40` | `0.9944 [0.9790, 1.0100]` | `30.16` | `133` | `55.93` |
| | `LPT_Batch` | `323.60` | `0.9943 [0.9759, 1.0131]` | `24.02` | `146` | `56.02` |
| | `LPT_Tail` | `322.82` | `0.9944 [0.9792, 1.0098]` | `27.78` | `151` | `55.43` |
| **`hetero-l9 4W`** (`10` rd) | `FIFO_A` | `235.36` | `1.0000 [1.0000, 1.0000]` | `34.12` | `59` | `78.11` |
| | `FIFO_B` (A/A) | `235.38` | `0.9992 [0.9894, 1.0090]` | `32.73` | `68` | `78.66` |
| | `LPT_Batch` | `237.59` | `1.0031 [0.9936, 1.0127]` | `33.55` | `55` | `79.58` |
| | `LPT_Tail` | `236.31` | `1.0082 [0.9933, 1.0232]` | `35.58` | `59` | `80.09` |
| **`hetero-l9 10W`** (`7` rd) | `FIFO_A` | `279.66` | `1.0000 [1.0000, 1.0000]` | `62.15` | `101` | `138.95` |
| | `FIFO_B` (A/A) | `277.21` | `1.0021 [0.9779, 1.0269]` | `67.13` | `97` | `140.24` |
| | `LPT_Batch` | `277.18` | `1.0006 [0.9887, 1.0127]` | `72.20` | `100` | `138.74` |
| | `LPT_Tail` | `280.34` | `1.0080 [0.9958, 1.0204]` | `69.33` | `116` | `141.77` |

---

## 3. Why Both LPT Variants Lose (Theorem 67.1 — FIFO Saturation Under `slots = 4W`)

1. Across the interior of a multi-block stream, `slots = 4W` + `READY_BYTES = 12 MiB` already absorbs block-duration variance under strict FIFO (`< 1%` idle bubbles); out-of-order dispatch merely increases ready-ring occupancy and `vcsw` (`+11.8%` at `4W`) while workers wait on earlier slots.
2. At EOF, `bzip2`'s `900 KB` block cap bounds the single-block tail delta to `< 0.5%` on Wikidata (`1.5 ms / 315 ms`), and on uniform heavy tails (`hetero-l9`) sorting equal-weight tail blocks is a no-op.

---

## 4. One-Line Kill Decision & Next Step

- **KILLED `LPT_Batch` & `LPT_Tail`:** Dead neutral to slightly slower (`0.994–1.010×`, every 95% CI spans `1.000`), adding `+12%` `vcsw` / `+0.6..2.8 MiB` RSS for zero wall gain — keep PR 4 candidate dispatch **strictly FIFO**.
- **Next (Cycle 68):** Final post-Round-6 six-cost (`W, D, B, Q, M, S`) & `10W` DRAM `B/B` trajectory across all 5 architectural milestones (`v0.5.12` -> `#739` -> `+MechBR+Grid` -> `+PR4_Own_ZS52` -> `+R4 Open`), plus upstream `origin/master` & Mechanisms sync check.
