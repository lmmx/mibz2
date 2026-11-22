# Wormhole Cycle 53 (Neon) — Charge 3: Residual Locks, `notify_all`, & `L1_AtomicFast` Audit after `#739` (PR 2 Closed — Port Nothing More)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`, [`src/parallel.rs`](../../src/parallel.rs))  
**Charge:** Round 6 Charge #3 — *"What's left of PR 2: `L1_AtomicFast`, and the residual lock and `notify_all` counts after `#739`. Port only what still measures."*  
**Artifacts:** [`c53_pr2_locks_ab.rs`](2026-11-22-wormhole-cycle53/c53_pr2_locks_ab.rs), [`c53_results.txt`](2026-11-22-wormhole-cycle53/c53_results.txt), [`c53_pr2_locks.toml`](2026-11-22-wormhole-cycle53/c53_pr2_locks.toml)

---

## 1. Exact Source Audit of `notify_all()` & Locks in `3296195e3` (`#739`)

1. **Residual `notify_all()` census in [`src/parallel.rs`](../../src/parallel.rs):**
   - `line 247` (`Jobs::close()`): fires **once** at EOF (`nA = 1`).
   - `lines 386–387` (`Shared::wake_all()`): fires **only** on `cancel()` / error shutdown (`0` times on normal decode).
   - `lines 2171–4025`: exclusively inside opt-in/experimental `service` and `pipelined` paths.
   - **Result:** **`0` hot-path `notify_all()` broadcasts remain in `3296195e3`!** Every steady-state wake in `StreamDecoder::workers` / `with_inverse` is already a directed `notify_one()` (`cv.notify_one()`, `scan_cv.notify_one()`, `jobs.cv.notify_one()`).

2. **Residual lock acquisitions per block (`N = 431` blocks):**
   - `#739` acquires `shared.lock()` `6×/block` (`~2,590` total: scanner slot wait `1075`, scanner `issued += 1` `1101`, worker `syms`/`free_out` `1155`, worker `insert_locked` `1189`, reader `check_cancelled` `907`, reader `advance` `928`) + `jobs.lock()` `2×/block` (`862`), totaling `~3,450` locks.
   - Removing the redundant locks (`check_cancelled` + `ChannelSource` using `shared.stopped` atomic, scanner local `issued`) cuts `shared.lock()` to `1,529`; adding `L1_AtomicFast` (`taken_a` + `rbytes_a`) cuts `shared.lock()` further to **`1,098`** (`99.8%` atomic fast-path hits).

---

## 2. Native Safe-Rust A/B Confrontation (`#![forbid(unsafe_code)]`, `3296195e3` Same-Binary)

Tested across **four scenarios** including `wikidata-l1` (`379` blocks in `38 MB` = `3×` higher block rate per second!) and high concurrency (`10W`, `16W`):
- `M739_A` / `M739_B` (A/A): Unmodified `3296195e3` `StreamDecoder::with_inverse::<OpenCells>` (`read_block`).
- `Str_TrimLock`: Streaming pipeline with redundant `shared.lock()`s removed (`shL = 1,529`).
- `Str_L1Atom`: `Str_TrimLock` + `L1_AtomicFast` in scanner (`shL = 1,098–1,160`, `92%–100%` fast-path hits).
- `Par_MutexBP` vs `Par_L1Atom`: `ParScan` workers checking backpressure via mutex (`shL = 1,528`) vs `L1_AtomicFast` (`shL = 1,097–1,185`, `80%–100%` fast-path hits).

| Scenario | Arm | `med_ms` | `vs Ref [95% CI]` | `sys_ms` | `vcsw` | `rss_MB` | `shL` / `n1` / `nA` / `l1H` |
|---|---|---:|---:|---:|---:|---:|---|
| **`wikidata-l9` `4W`** (`12` rds) | `M739_A` / `M739_B` (A/A) | `365.9 / 363.6` | `1.0011 [0.9825, 1.0200]` | `24.5` | `216` | `56.6` | `~2590 / ~480 / 1 / 0` |
| | `Str_TrimLock` | `362.13` | `0.9985 [0.9815, 1.0158]` | `29.1` | `213` | `59.4` | `1529 / 480 / 1 / 0` |
| | `Str_L1Atom` | `363.65` | `0.9955 [0.9860, 1.0051]` | `28.6` | `208` | `58.8` | `1132 / 460 / 1 / 397` |
| | `Par_MutexBP` (vs `M739_A`) | **`343.78`** | **`0.9517 [0.9323, 0.9716]`** | **`13.0`** | **`138`** | **`35.6`** | `1528 / 140 / 0 / 0` |
| | `Par_L1Atom` (vs `Par_MutexBP`) | `342.84` | `0.9959 [0.9880, 1.0040]` | `13.3` | `127` | `35.5` | `1104 / 133 / 0 / 424` |
| **`wikidata-l1` `4W`** (`12` rds, `3×` blk rate) | `M739_A` / `M739_B` (A/A) | `95.4 / 95.9` | `1.0464 [0.9727, 1.1257]` | `8.9` | `265` | `17.9` | — |
| | `Str_TrimLock` | `94.58` | `0.9883 [0.9757, 1.0010]` | `8.3` | `268` | `18.7` | `1518 / 556 / 1 / 0` |
| | `Str_L1Atom` | `95.79` | `1.0055 [0.9811, 1.0304]` | `8.0` | `284` | `18.6` | `1160 / 643 / 1 / 358` |
| | `Par_MutexBP` (vs `M739_A`) | **`85.74`** | **`0.8985 [0.8852, 0.9120]`** | **`6.0`** | **`95`** | **`13.9`** | `1517 / 212 / 0 / 0` |
| | `Par_L1Atom` (vs `Par_MutexBP`) | `86.86` | `1.0059 [0.9916, 1.0205]` | `6.0` | `103` | `13.9` | `1156 / 229 / 0 / 361` |
| **`wikidata-l9` `10W`** (`5` rds) | `M739_A` / `M739_B` (A/A) | `394.7 / 393.8` | `0.9976 [0.9782, 1.0173]` | `39.8` | `461` | `77.6` | — |
| | `Str_TrimLock` | `402.08` | `0.9944 [0.9405, 1.0514]` | `42.7` | `418` | `83.8` | `1529 / 495 / 1 / 0` |
| | `Str_L1Atom` | `393.83` | `0.9904 [0.9464, 1.0364]` | `42.7` | `447` | `86.8` | `1099 / 498 / 1 / 430` |
| | `Par_MutexBP` (vs `M739_A`) | **`359.43`** | **`0.8994 [0.8492, 0.9526]`** | **`27.1`** | **`89`** | **`62.1`** | `1528 / 78 / 0 / 0` |
| | `Par_L1Atom` (vs `Par_MutexBP`) | `358.15` | `1.0029 [0.9861, 1.0201]` | `29.5` | `111` | `61.3` | `1097 / 54 / 0 / 431` |
| **`wikidata-l9` `16W`** (`5` rds) | `M739_A` / `M739_B` (A/A) | `402.1 / 406.7` | `1.0001 [0.9765, 1.0242]` | `45.6` | `555` | `85.6` | — |
| | `Str_TrimLock` | `415.56` | `1.0088 [0.9682, 1.0510]` | `56.8` | `545` | `104.5` | `1529 / 510 / 1 / 0` |
| | `Str_L1Atom` | `407.21` | `0.9940 [0.9328, 1.0593]` | `56.2` | `522` | `103.8` | `1098 / 529 / 1 / 431` |
| | `Par_MutexBP` (vs `M739_A`) | **`386.76`** | **`0.9419 [0.8889, 0.9980]`** | `41.6` | **`255`** | `84.5` | `1528 / 366 / 0 / 0` |
| | `Par_L1Atom` (vs `Par_MutexBP`) | `385.19` | `0.9956 [0.9793, 1.0121]` | `36.4` | `240` | `84.4` | `1185 / 345 / 0 / 343` |

---

## 3. Honest Charge-3 Verdict & Kill Decisions

**`#739` already captured `99.9%` of PR 2.** Saving `431` uncontended mutex lock/unlocks (~`20 ns` each = **`8.6 µs`** total across `210 MB`, or `0.02%` of a `40 ms` i9 decode) does not clear noise even at `3×` block rate (`wikidata-l1`) or `16W`:

- **KILLED `L1_AtomicFast` (both streaming `0.990–1.006×` and `ParScan` `0.996–1.006×`, all `95%` CIs span `1.000`):** Do **not** port to `src/parallel.rs` — adds two extra atomics and race complexity for `~8.6 µs`.
- **KILLED `Str_TrimLock` (`0.988–1.009×`):** Do **not** port as a standalone PR (total lock time is already `<0.04%` of wall time).
- **CLOSED PR 2 / Wall W5:** Port **nothing more** from PR 2; focus Coordinator effort 100% on **W0 (`trimmed_here` grid, C56)**, **W1 (`PR 4` bounded-window `BlockSpan` streaming, C57–C62)**, and **Theorem 52.1 zero-shift RLE1**!
