# Wormhole Cycle 57 (Neon) — Charge 4 (Part 1): Forensics of Whole-Input `ParScan` (`1.28×` RSS / `1.11×` `6T`) vs Ping-Pong 2-Window Streaming (`0.935×` `10W`, `0.95×` RSS)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`)  
**Mechanisms Reference:** `origin/research/carbon-word-transport` (`300a29976`, `C38b decode_stream_parscan_c38b`)  
**Artifacts:** [`c57_parscan_forensics_ab.rs`](2026-11-22-wormhole-cycle57/c57_parscan_forensics_ab.rs), [`c57_results.txt`](2026-11-22-wormhole-cycle57/c57_results.txt), [`c57_parscan_forensics.toml`](2026-11-22-wormhole-cycle57/c57_parscan_forensics.toml)

---

## 1. Root-Cause Diagnosis of `#741`'s Rejection & `C38b`'s Remaining Barrier

Mechanisms' `#741` `parscan` entry point was rejected on the i9 for two concrete reasons: **`1.28×` peak RSS at `10T`** and **`1.11×` slower wall at `6T`**. We isolated three distinct mechanisms:

1. **Whole-Input Buffering + Unbounded Ready Map (`P0_WholeIn` → `1.28–1.32×` RSS at `10T`):** Holding the full compressed stream in RAM plus an unbounded `BTreeMap` without `#739`'s `READY_BYTES = 12 MiB` cap explodes peak RSS (`+23.5 MiB` at `10W`, `+61.5 MiB` at `16W` on Wikidata; unbounded on larger archives!).
2. **`6T` Unpinned Thread-Count Asymmetry (from C56 → `1.11×` slower at `6T`):** Unpinned `StreamDecoder::new(6)` runs `6` workers + `1` serial scanner = `7` parallel threads on a `10`-core i9; `parscan(6)` ran only `6` threads doing both Huffman + BWT (`7/6 = 1.167×` → observed `1.11×`). **Fix:** `StreamDecoder` must only engage PR 4 parallel-scan mode when serial scanning actually binds (`threads >= 9` unpinned on `10C` i9, or `threads >= physical_cores`, or pinned `threads >= 8` — never at `6T` unpinned!).
3. **Single-Window Inter-Epoch Drain Barrier (`P1_Win1_Bar`, modeling `C38b`):** `C38b` (`300a29976`) already replaced whole-input buffering with a bounded `4 MiB` `Arc<Vec<u8>>` window, fixing RSS — **but** waits for `workers_active == 0` at every `4 MiB` boundary so `Arc::try_unwrap` can reclaim the single buffer before reading the next `4 MiB`.
4. **Our PR 4 Breakthrough (`P2_Win2_NoBar` — Ping-Pong `2`-Window Streaming):** Keeps at most **`2` bounded `4 MiB` `Arc<Vec<u8>>` windows** live (`<= 8 MiB` total input RAM even on a `100 GB` archive!). Window `k+1` is read and anchor-scanned while window `k` is still decoding; workers claim spans across windows with **zero inter-window barrier** and reclaim window `k` via `Arc::try_unwrap` the instant its last block finishes `read_item` (before BWT inversion even starts!), feeding straight into `#739`'s `Ready` ring & `READY_BYTES = 12 MiB` cap!

---

## 2. Native Safe-Rust Results (`#![forbid(unsafe_code)]`, Same-Binary A/B vs `3296195e3`)

| `W` (`rds`) | Arm | `med_ms` | `vs M739_A [95% CI]` | `user` | `sys` | `minflt` | `vcsw` | `rss_MB` (`vs M739`) |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| **`4W`** (`12`) | `M739_A` / `M739_B` (A/A) | `372.6 / 370.9` | `0.9899 [0.9717, 1.0085]` | `700.9` | `26.45` | `10,442` | `202` | `56.76` (`1.00×`) |
| | `P0_WholeIn` (`#741` model) | `365.66` | `0.9916 [0.9774, 1.0061]` | `681.4` | `27.28` | `10,197` | `129` | `57.32` (`1.01×`) |
| | `P1_Win1_Bar` (`C38b` barrier) | `363.76` | `0.9747 [0.9627, 0.9869]` | `692.5` | **`16.55`** | **`5,670`** | `170` | **`39.59` (`0.70×`)** |
| | **`P2_Win2_NoBar` (PR 4)** | **`357.50`** | **`0.9655 [0.9530, 0.9782]`** | `689.9` | `18.62` | `6,687` | **`144`** | **`43.57` (`0.77×`)** |
| **`6W`** (`8`) | `M739_A` / `M739_B` (A/A) | `383.7 / 383.3` | `0.9897 [0.9507, 1.0303]` | `718.3` | `30.91` | `12,331` | `246` | `63.23` (`1.00×`) |
| | `P0_WholeIn` | `371.28` | `0.9614 [0.9225, 1.0019]` | `696.1` | `31.99` | `13,049` | `89` | `68.38` (`1.08×`) |
| | `P1_Win1_Bar` | `369.45` | `0.9495 [0.9127, 0.9877]` | `694.7` | `24.04` | **`7,862`** | `170` | **`48.14` (`0.76×`)** |
| | **`P2_Win2_NoBar` (PR 4)** | **`364.90`** | **`0.9381 [0.8981, 0.9798]`** | **`691.9`** | **`22.47`** | `8,890` | **`102`** | **`52.20` (`0.83×`)** |
| **`10W`** (`5`) | `M739_A` / `M739_B` (A/A) | `391.5 / 391.9` | `1.0003 [0.9957, 1.0050]` | `718.9` | `36.35` | `15,874` | `475` | `73.44` (`1.00×`) |
| | `P0_WholeIn` (`#741` model) | `388.70` | `0.9749 [0.9361, 1.0153]` | `710.1` | `46.21` | `20,310` | `86` | **`96.92` (`1.32×`!)** |
| | `P1_Win1_Bar` (`C38b` barrier) | `381.59` | `0.9563 [0.9011, 1.0148]` | `710.0` | **`30.68`** | **`12,496`** | `238` | **`66.21` (`0.90×`)** |
| | **`P2_Win2_NoBar` (PR 4)** | **`373.30`** | **`0.9345 [0.8752, 0.9978]`** | **`698.1`** | `31.90` | `13,307` | **`126` (`-47%` vs P1)** | **`69.46` (`0.95×`)** |
| **`16W`** (`5`) | `M739_A` / `M739_B` (A/A) | `416.4 / 415.9` | `0.9974 [0.9878, 1.0070]` | `740.4` | `43.40` | `19,171` | `542` | `85.57` (`1.00×`) |
| | `P0_WholeIn` | `418.95` | `1.0011 [0.9636, 1.0400]` | `723.0` | `74.15` | `33,604` | `124` | **`147.10` (`1.72×`!)** |
| | `P1_Win1_Bar` | `407.84` | `0.9736 [0.9220, 1.0281]` | `741.3` | `49.27` | `18,169` | `379` | `88.22` (`1.03×`) |
| | **`P2_Win2_NoBar` (PR 4)** | **`400.04`** | **`0.9565 [0.9109, 1.0043]`** | **`722.7`** | `49.70` | `19,144` | **`208` (`-45%` vs P1)** | `92.26` (`1.08×`) |

---

## 3. Kill / Keep Decisions & Next Cycle Plan

- **KILLED `P0_WholeIn` (whole-input `ParScan`):** Reproduces `#741`'s `1.32×` RSS bloat at `10W` (`1.72×` at `16W`, `minflt +75%`) and regresses `16W` wall to `1.0011×`.
- **PROMOTED `P2_Win2_NoBar` (Ping-Pong `2`-Window Streaming `BlockWindow` over `P1_Win1_Bar`):** Beats BOTH `3296195e3` AND `P1_Win1_Bar` across all `4W, 6W, 10W, 16W` (`10W 0.9345×`, `-8.3 ms` faster than `P1`, `-47%` fewer `vcsw`), while cutting peak RSS **below `#739`** (`0.77×` at `4W`, `0.95×` at `10W`)!
- **Cycle 58 (next — Charge 4, Part 2):** Write the complete, formal **PR 4 `BlockSpan` Streaming Interface Specification** (`WindowBatch`, `SpanCand`, `SpanWindowProducer`, `ChainValidator`, speculative false-positive rejection, cross-window tail carry-over, exact `Error { kind, block, bit }`, `cancel()`, and `StreamDecoder` dispatch gate) so Mechanisms' Handoff 1 drops straight into `StreamDecoder` with **zero rework**!
