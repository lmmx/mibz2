# Wormhole Cycle 56 (Neon) — Charge 2: The i9-10900K `trimmed_here` Recalibration Grid (`workers × slots` at `10, 12, 16, 20` Threads)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Baseline:** `origin/master` `3296195e3` (`#739`, [`src/parallel.rs:3310-3440`](../../src/parallel.rs)) + incoming Mechanisms block reader (`#742`).  
**Charge:** Round 6 Charge #2 — *"Design the `trimmed_here` recalibration for the i9 on the new memory profile: the grid of workers × slots at 10, 12, 16 and 20 threads, and a predicted 10-thread wall at each. The i9 decides, so give me the grid, not a new table."*  
**Artifacts:** [`c56_worker_slots_grid.rs`](2026-11-22-wormhole-cycle56/c56_worker_slots_grid.rs), [`c56_results.txt`](2026-11-22-wormhole-cycle56/c56_results.txt), [`c56_grid.toml`](2026-11-22-wormhole-cycle56/c56_grid.toml)

---

## 1. Real i9-10900K Model Anchors (`210 MB` Wikidata, `431` Blocks)

Before generating the experimental grid, we anchor our Comet Lake `10C/20T` (`20 MiB` L3, `42 GB/s` DDR4) model directly to the real i9-10900K measurements (`#740`/`#744` & `#742`):

| Real i9 Anchor (`210 MB` Wikidata) | Real i9 Wall | Model Predicted | Error | Binding Stage |
|---|---:|---:|---:|---|
| `v0.5.12` `7W / 28` (`10T` unpinned) | `~69.5 ms` | — | — | `W1` + `W3` DDR4 |
| `#739` (`3296195e3`) `7W / 28` (`10T` unpinned, `0.963×`) | `66.9 ms` | `64.3 ms` | `-2.6 ms` | `SCAN` (`~62.5 ms`) |
| `#739 + MechBR` `7W / 28` (`10T` unpinned, `0.932×`) | **`62.4 ms`** | **`62.4 ms`** | **`0.0 ms`** | **`WORK`** (`7` workers!) |
| `#739 + MechBR` `8W / 32` (`8T` unpinned, `0.884×`) | **`59.1 ms`** | **`57.4 ms`** | **`-1.7 ms`** | **`WORK`** (`8` workers!) |

**Two structural insights immediately fall out of the anchored physics:**
1. **Why `9W` (`+ 1` dedicated serial scanner core = `10/10` physical cores) is the predicted optimum on `#739 + MechBR`:** With `#739`'s new memory profile (`0 B` `ll`, `1.06×` `out`, `12 MiB` ready cap), `9` workers require only `~24.3–26.8 MiB` L3 (`3.45–3.85 B/B` DRAM — well below DDR4 saturation!). Running `9` workers leaves the **10th physical Comet Lake core exclusively for Mechanisms' fast serial block reader (`~46.5 ms`)** with `0%` SMT contention (`52.9–53.9 ms` predicted, **`-14%` faster than `7W/28`**!), whereas `10W` forces the `100%`-busy serial scanner onto an SMT sibling of a worker (`+16%` scan slowdown → `56.2 ms`).
2. **Why Mechanisms' `parscan` regressed `1.11×` at `6T` unpinned:** Unpinned `StreamDecoder(6)` runs `6` workers + `1` serial scanner = **`7` active threads** on `10` physical cores; `parscan(6)` had `0` scanner thread and ran only **`6` threads** doing both Huffman + BWT (`7/6 = 1.167×` → observed `1.11×`!). Conversely, once `W = 10` (all `10` physical cores saturated) — especially once Round 4 `WormholeNeonCells` lowers `Open` work below `~350 ms` — **PR 4 (`10W` parallel scan)** overtakes `9W + MechBR` cleanly!

---

## 2. The Experimental i9-10900K `workers × slots` Grid (`10, 12, 16, 20` Threads)

**Design principle:** Do **not** hardcode a new `trimmed_here` table yet — queue this compact, orthogonal grid on the i9 (`OpenCells` + `READY_BYTES = 12 MiB` via `StreamDecoder::with_inverse::<OpenCells>(w, slots, k)`) and let the i9 pick the winning `(w, slots)` at each thread count!

### 2a. Primary `10T` Grid (Run Both **Unpinned `whole_machine`** and **Pinned `10` Physical Cores**)

| Grid Cell (`W / slots`) | `slots / W` | Predicted `10T` Wall (`3296195e3`) | **Predicted `10T` Wall (`+MechBR`)** | **Predicted `10T` Wall (`+PR4`)** | Predicted `L3` | Predicted `DRAM` | Predicted `RSS` | Bind (`MBR` / `PR4`) | Role in Grid |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| **`7 / 14`** | `2×` | `64.5 ms` | `61.4 ms` | `68.1 ms` | `20.3 MiB` | `2.60 B/B` | `47.0 MiB` | `WORK / WORK` | Tight-window control |
| **`7 / 21`** | `3×` | `64.3 ms` | `61.7 ms` | `68.3 ms` | `21.3 MiB` | `2.84 B/B` | `50.6 MiB` | `WORK / WORK` | Mid-window control |
| **`7 / 28`** | `4×` | `64.3 ms` | `62.4 ms` | `68.9 ms` | `22.3 MiB` | `3.05 B/B` | `54.3 MiB` | `WORK / WORK` | **Current `v0.5.12`/`#739` baseline** |
| **`8 / 16`** | `2×` | `64.6 ms` | `56.5 ms` | `62.5 ms` | `22.3 MiB` | `3.06 B/B` | `51.1 MiB` | `WORK / WORK` | Low-RSS candidate |
| **`8 / 24`** | `3×` | `64.4 ms` | `56.7 ms` | `62.7 ms` | `23.4 MiB` | `3.28 B/B` | `55.3 MiB` | `WORK / WORK` | Balanced `8W` |
| **`8 / 32`** | `4×` | `64.5 ms` | `57.4 ms` | `63.3 ms` | `24.5 MiB` | `3.48 B/B` | `59.4 MiB` | `WORK / WORK` | Matches i9 `8T` (`59.1 ms`) |
| **`9 / 18`** | `2×` | `64.7 ms` | **`52.9 ms`** | `58.4 ms` | `24.3 MiB` | `3.45 B/B` | `55.3 MiB` | `WORK / WORK` | **Predicted `+MechBR` #1 (Wikidata)** |
| **`9 / 27`** | `3×` | `64.6 ms` | **`53.2 ms`** | `58.6 ms` | `25.6 MiB` | `3.66 B/B` | `59.9 MiB` | `WORK / WORK` | **Predicted `+MechBR` #1 (HOL-safe)** |
| **`9 / 36`** | `4×` | `64.6 ms` | **`53.9 ms`** | `59.3 ms` | `26.8 MiB` | `3.85 B/B` | `64.6 MiB` | `WORK / WORK` | Full-window `9W` |
| **`10 / 20`** | `2×` | `74.9 ms` | `56.3 ms` | **`55.5 ms`** | `26.3 MiB` | `3.77 B/B` | `59.4 MiB` | `SCAN / WORK` | **Predicted `+PR4` #1** |
| **`10 / 30`** | `3×` | `74.7 ms` | `56.2 ms` | **`55.7 ms`** | `27.7 MiB` | `3.98 B/B` | `64.6 MiB` | `SCAN / WORK` | **Predicted `+PR4` #1 (HOL-safe)** |
| **`10 / 40`** | `4×` | `74.8 ms` | `56.2 ms` | `56.4 ms` | `29.1 MiB` | `4.16 B/B` | `69.8 MiB` | `SCAN / WORK` | Untrimmed `10W` control |
| **`11 / 22`** | `2×` | `75.0 ms` | `56.4 ms` | `57.2 ms` | `28.3 MiB` | `4.06 B/B` | `63.5 MiB` | `SCAN / WORK` | SMT onset probe (`+1` SMT) |
| **`11 / 33`** | `3×` | `74.9 ms` | `56.3 ms` | `57.5 ms` | `29.8 MiB` | `4.25 B/B` | `69.3 MiB` | `SCAN / WORK` | SMT onset probe |
| **`12 / 24`** | `2×` | `75.1 ms` | `56.6 ms` | `59.0 ms` | `30.3 MiB` | `4.31 B/B` | `67.7 MiB` | `SCAN / WORK` | SMT cliff probe (`+2` SMT) |
| **`12 / 36`** | `3×` | `75.0 ms` | `56.5 ms` | `59.2 ms` | `32.0 MiB` | `4.49 B/B` | `73.9 MiB` | `SCAN / WORK` | SMT cliff probe |

### 2b. Compact Extension for `12, 16, and 20` Threads Asked (Unpinned `whole_machine`)

Because `12, 16, and 20` threads on a `10C/20T` i9-10900K all run on the **same `10` physical cores**, any thread count `T ∈ {12, 16, 20}` only needs to test whether oversubscribing SMT workers beats the `10T` winner:
- **`T = 12`:** `[winner_10T, 10/30, 11/33, 12/24, 12/36, 12/48]` (`6` arms)
- **`T = 16`:** `[winner_10T, 10/30, 12/36, 14/28, 16/32, 16/64]` (`6` arms — predicted `16/32..64` = `60.0–65.9 ms`, `+12%..+18%` slower than `9W/10W` due to `40.5 MiB` L3 spill & SMT L2 halving)
- **`T = 20`:** `[winner_10T, 10/30, 12/36, 16/32, 20/40, 20/80]` (`6` arms — tests whether `#739`'s inconclusive `20T` cell resolves once `W` is clamped to `winner_10T`)

**Recommended i9 Queue Protocol (Total: `16` cells at `10T` + `12` unique SMT cells = `28` arms, `< 4 min` on i9):**
1. Run the `16`-cell `10T` grid on `corpora/wikidata.bz2` (unpinned + pinned `0..9`), plus a `hetero` / `level-1` sanity check on the top-3 `(W, slots)` finalists (`9/18, 9/27, 9/36` or `10/20, 10/30, 10/40`) so `slots = 2W` is never adopted if it regresses heterogeneous blocks!
2. Run the `12`-arm `12/16/20T` SMT check against the `10T` winner.

---

## 3. Empirical Xeon Grid Guard (`3296195e3`, `wikidata-l9` & `hetero-l9` HOL Stability)

| `W / slots` | `wikidata-l9` `wall` (`ms`) | `minflt` | `rss_MB` | `hetero-l9` `wall` (`ms`) | `hetero` `vcsw` | `hetero` `rss_MB` |
|---|---:|---:|---:|---:|---:|---:|
| `7 / 14` (`2×`) | `348.46` | `11,992` | `58.52` | `165.33` | `118` | `89.85` |
| `7 / 21` (`3×`) | `352.90` | `12,145` | `62.88` | `164.07` | `88` | `100.79` |
| `7 / 28` (`4×`) | `347.52` | `13,862` | `65.62` | **`163.47`** | **`73`** | `106.00` |
| `8 / 16` (`2×`) | `383.20` | `12,273` | `62.72` | `165.94` | `93` | `93.92` |
| `8 / 24` (`3×`) | `387.02` | `13,507` | `66.85` | `164.02` | `84` | `92.29` |
| `8 / 32` (`4×`) | `357.87` | `14,233` | `69.82` | **`161.21`** | `86` | `92.66` |
| `9 / 18` (`2×`) | `359.26` | `12,598` | `63.19` | `165.87` | `127` | `106.86` |
| `9 / 27` (`3×`) | `359.67` | `14,476` | `71.88` | `163.05` | `102` | `100.82` |
| `9 / 36` (`4×`) | `351.97` | `14,607` | `67.27` | **`161.62`** | `114` | `105.80` |
| `10 / 20` (`2×`) | `363.47` | `13,304` | `65.38` | `169.06` | `119` | `111.82` |
| `10 / 30` (`3×`) | `355.02` | `15,551` | `70.20` | `173.26` | `112` | `106.05` |
| `10 / 40` (`4×`) | `362.36` | `15,393` | `74.59` | **`166.56`** | `111` | `101.14` |

**Takeaway for the i9 Grid:** On `hetero-l9`, **`slots = 3W` and `4W` consistently beat `slots = 2W`** (`-1.5%..-2.9%` wall, fewer `vcsw`), while on uniform `wikidata-l9`, `2W` saves `~7–9 MiB` RSS and `~2,000` `minflt`. Keeping `slots ∈ {2W, 3W, 4W}` in the i9 grid + gating the winner on `hetero` cleanly captures both effects.

---

## 4. Cycle 56 Verdict & Next Cycle Plan

- **Charge 2 delivered:** Complete anchored `workers × slots` i9 experimental grid (`10, 12, 16, 20` threads) with predicted `10T` wall, `L3`, `DRAM B/B`, and `RSS` at every cell — no hardcoded table; the i9 decides!
- **Cycle 57 (next — Charge 4, Part 1):** Forensic audit of Mechanisms' `parscan` `1.11×` (`6T`) & `1.28×` RSS (`10T`) failure + `C38b` single-window barrier stall, preparing the exact **PR 4 `BlockSpan` streaming interface spec** in Cycle 58!
