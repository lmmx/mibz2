# Wormhole Cycle 68 (Neon): Complete Six-Cost (`W, D, B, Q, M, S`) & `10W` DRAM `B/B` Trajectory on Live `origin/master` `a3f26aa01` (`#739 + #745`)

**Date:** 2026-11-22  
**Branch:** `research/carbon-wormhole`  
**Live Upstream Sync:** `origin/master` moved to **`a3f26aa01`** (`4a79f0631` merged Mechanisms' **`#745`** L1-resident Huffman block reader + `#750` re-queued `bench-open-r5` on the new master!); `origin/research/carbon-word-transport` remains at `300a29976`.  
**Live `a3f26aa01` Profile:** [`c68_profile_a3f26.txt`](2026-11-22-wormhole-cycle68/c68_profile_a3f26.txt)  
**Manifest:** [`c68_six_cost_trajectory.toml`](2026-11-22-wormhole-cycle68/c68_six_cost_trajectory.toml) (`check_manifest.py`: `PASS`)

---

## 1. Zero-Rework Verification on Live `origin/master` `a3f26aa01` (`#739 + #745`)

Mechanisms' `#745` (`4a79f0631`) modified `src/bits.rs`, `src/block.rs`, `src/runs.rs`, `src/stream.rs`, and added `src/huff.rs`. Recompiling our Cycle 63 **`PR4_Own_ZS52`** harness against `libmibz2` built from `a3f26aa01` succeeded with **zero source edits, zero errors, and zero warnings** (`ZERO_REWORK_COMPILE_ON_A3F26AA01_OK`), proving our Charge 4 `SpanFinder` / `read_item` interface boundary is rock-solid against `#745`.

### Live Measured `#745` Stage Delta (`3296195e3` Cycle 64 vs `a3f26aa01` Cycle 68)
| Corpus | `S1 read_item` (`3296195e3`) | **`S1 read_item` (`a3f26aa01` `#745`)** | **`#745` `S1` Δ** | Post-`#745` `S1` Share | Post-`#745` `S3 OpenCells` Share |
|---|---:|---:|---:|---:|---:|
| **`wikidata-l9`** (`509` blks) | `89.37 ms` | **`72.75 ms`** | **`-18.6%`** | `13.5%` | **`64.3%`** (`346.28 ms`) |
| **`wikidata-l1`** (`379` blks) | `34.42 ms` | **`25.00 ms`** | **`-27.4%`** | `16.9%` | **`65.5%`** (`96.94 ms`) |
| **`hetero-l9`** (`73` blks) | `86.46 ms` | **`73.44 ms`** | **`-15.1%`** | `18.5%` | `17.0%` + `57.6%` ColRoute |
| **`random-l9`** (`12` blks) | `41.42 ms` | **`36.19 ms`** | **`-12.6%`** | `20.6%` | `77.4%` ColRoute |

---

## 2. Complete Round 5 → Round 6 Six-Cost (`W, D, B, Q, M, S`) & `10W` DRAM `B/B` Trajectory

| Cost Dimension | `v0.5.12` (`6a36b0e10`, `7W/28`) | `#739` (`3296195e3`, `7W/28`) | **`a3f26aa01` (`+#745` `7W/28` / `9W/27`)** | **`+ PR4_Own_ZS52` (`W2M_L3`, `10W/30`)** | **`+ R4 OpenCells` (`10W/30`)** |
|---|---|---|---|---|---|
| **`W` (Total `1T` CPU, `210 MB`)** | `~625 ms` | `~552 ms` (`-11.7%`) | **`538.5 ms`** (`-13.8%`) | **`~516 ms`** *(w/ AVX2 `find_runs` + `ZeroShift52`)* | **`~471 ms`** (`-24.6%`) |
| **`D` (Serial Depth / Critical Path)** | `~58 ms` serial scan (`84%` par) | `~62.5 ms` serial scan + `7W` cap | `~46.5 ms` scan; `7W` cap -> **`9W` clears cap** | **`~2.2 ms` (`C38b`) = `99.2%` par (`10W` phys)** | **`~2.2 ms` (`99.5%` par)** |
| **`B` (`10W` DRAM `B/B` & DDR4 Floor)** | **`10.82 B/B`** (`54.4 ms` floor!) | **`~4.6 B/B`** (`23.1 ms` floor) | **`~4.6 B/B`** (`23.1 ms` floor) | **`~3.8 B/B`** (`19.1 ms` floor; `ZeroShift52 -1.56 B/B` L2/L3 R+W) | **`~3.5 B/B`** (`17.6 ms` floor) |
| **`Q` (Sync / Broadcasts / `vcsw`)** | `2,155` locks, `862` `notify_all`, `~980` `vcsw` | `0` hot `notify_all`, `~480–529` `vcsw` | `0` hot `notify_all`, `~480–529` `vcsw` | **`0` hot `notify_all`, `0` barriers, `131–185` `vcsw` (`-65%..-73%`)** | `131–185` `vcsw` |
| **`M` (Memory / `10W` Peak RSS / L3)** | `~110 MiB` RSS, `49.4 MiB` L3 (`0.90M ll` + `1.8M out`) | `71.9 MiB` RSS (`0.65×`), `~27 MiB` L3 (`0 B ll`) | `71.9 MiB` RSS, `~27 MiB` L3 | **`54.9 MiB` RSS (`0.76×` `#739`, `-50%` vs `v0.5.12`!), `~24 MiB` L3** | **`54.9 MiB` RSS, `~22.5 MiB` L3** |
| **`S` (OS / Cold-Start `minflt` @ `10W`)** | `26,700` `minflt`, `64.4 ms` `sys` | `15,800` `minflt` (`-40.7%`), `37.5 ms` `sys` | `15,800` `minflt`, `37.5 ms` `sys` | **`10,019` `minflt` (`-62.5%` vs `v0.5.12`!), `23.5 ms` `sys`** | `~9,800` `minflt` |
| **Anchored i9 `10T` Wall (`210 MB`)** | `~69.5 ms` *(real i9)* | **`66.9 ms`** *(real i9 `0.963×`)* | **`62.4 ms` (`7W` real) → `~53.2 ms` (`9W` grid)** | **`~48.5–50.0 ms`** *(pred `0.70–0.72×` `v0.5.12`)* | **`~43.5–45.0 ms`** *(pred `0.63–0.65×`)* |

---

## 3. Binding-Wall Closure Status (`W0`–`W5`)

| Wall | Status After `a3f26aa01` (`#739 + #745`) | Closure Mechanism |
|---|---|---|
| **W0 (`trimmed_here` `7W/28` cap)** | **STILL BINDS (`#1` on `a3f26aa01`!)** | Cycle 56 `28`-arm i9 `W×S` grid (ready to queue; predicts `9W/27` -> `53.2 ms` on `a3f26aa01`, `10W/30` on `+PR4`) |
| **W1 (Serial scan)** | **Reduced `-18.6%` by `#745`; still caps `10W`!** | Charge 4 `PR4_Own_ZS52` (`W2M_L3`, Invariants #1–#6, `0` scanner thread, `99.15%` parallelizable) |
| **W2 (Allocator / `minflt`)** | **Mostly closed by `#739` (`-40.7%` `minflt`)** | `PR4_Own_ZS52` takes `minflt` down another **`-36.5%` (`10W`) / `-49.9%` (`4W`)** (`-62.5%` cumulative vs `v0.5.12`) |
| **W3 (`10W` DRAM `B/B`)** | **BROKEN by `#739` (`10.82 → 4.6 B/B`)** | `ZeroShift52` (Thm 52.1) cuts RLE1 `copy_within` another `-46.6%` (`-1.56 B/B` R+W -> `~3.8 B/B` DRAM) |
| **W4 (Ready-window L3 spill)** | **CLOSED by `#739` (`12 MiB` ready cap)** | Preserved verbatim inside `PR4_Own_ZS52` |
| **W5 (Broadcast locks / PR 2)** | **CLOSED by `#739` (`0` hot `notify_all()`)** | Cycle 53 proved **port nothing more from PR 2** (`L1_AtomicFast` killed) |

---

## 4. Next Step

Cycle 69 runs the **Final `15`-Round Rotated Fresh-Process Multi-Regime Cumulative Ablation directly against live `origin/master` `a3f26aa01` (`#739 + #745`)** across `4W`, `6W`, `10W`, and `16W` + `wikidata-l1`, `hetero-l9`, and `random-l9`!
