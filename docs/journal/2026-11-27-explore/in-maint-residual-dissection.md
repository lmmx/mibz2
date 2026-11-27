# Forensic dissection of `#1118`'s `7.490 ms` residual `in_maint` on physical i9-10900K hardware

An `explore` session (27 November 2026), started from the real physical i9-10900K hardware bundle
`origin/evidence` `c0b2341c` (`scanner-seam-window/run-20261127T070349Z.tar.gz`, recorded in `#1118`) and
the exact `ChannelSource` and `Scanner` sources (`src/parallel.rs`, `src/stream.rs`, `src/read.rs`, plus
`#1031` `3ae44390`). Zero `src/` edits, 100% safe Rust (`#![forbid(unsafe_code)]`), no frozen verdicts
altered (C2).

**Harness:** [`examples/in_maint_dissection.rs`](../../../examples/in_maint_dissection.rs).  
**Frozen discovery/stop test (frozen before timing):**
- **Exactness:** Every candidate produces bit-identical block counts, bit offsets, CRCs, and symbol
  hashes against `SliceSource` across `wikidata-x4` (`47,419,732 B`, `1,724` blocks) and 5 multi-chunk
  off-Wikidata guard corpora (`text_16m`, `random_18m`, `records_18m`, `runs_8m`, `onebyte`).
- **Discovery (admit `idea` claim):** (1) Prove `>= 50%` (`>= 3.75 ms` i9-scaled) of `#1031`'s
  `7.490 ms` residual `in_maint` is avoidable copy/drain/lock/alloc work, reproducing `base`'s `12`
  spans and `#1031`'s `879` spans on `wikidata-x4`; and (2) demonstrate a safe-Rust prototype cutting
  total `in_maint` by `>= 70%` vs shipped `base` and cutting end-to-end serial scan `S` on `wikidata-x4`
  with zero disjoint regression on any guard corpus and `<= 1.00×` peak window capacity vs `base`.
  Otherwise record a scoped dead-end in `frontier/dead-ends.tsv`.

---

## Executive summary: **DECISIVE DISCOVERY (`97.2%` of residual `in_maint` is an avoidable `LOOKAHEAD = 2 MiB` copy bug)**

On Louis's physical i9-10900K (`c0b2341c`), `#1031` (`scanner-seam-window`) was **`NOT ADOPTED`** (judged
10T `0.9811 [0.9670, 0.9954]` FASTER, 12T `0.9898 [0.9726, 1.0073]` INCONCLUSIVE) because median
`in_maint` only dropped **`-29.93%`** (`10.689 -> 7.490 ms`) instead of the modeled `>= 80%` (`~11 ms`).

Inspecting `trace-base.jsonl`, `trace-head.jsonl`, `main.jsonl`, and `#1031`'s `ChannelSource::view`
exposes the exact root cause — reproduced **to the exact integer** by `examples/in_maint_dissection.rs`:

1. **The smoking gun (`spans: 12 -> 879`, `42.73 MiB` still copied out of `45.22 MiB`):**
   - `src/read.rs` feeds **`CHUNK = 4 MiB`** chunks, while `src/stream.rs` calls `src.view(base, LOOKAHEAD)`
     with a sticky **`LOOKAHEAD = 2 MiB`** (`50%` of `CHUNK`) on **every single block**.
   - As soon as `from` crosses the `2.00 MiB` midpoint of any `4 MiB` chunk (`51.0%` of all `1,724`
     Wikidata blocks), `from + 2 MiB` crosses `next_base`. `#1031` enters `in_seam = true` two megabytes
     before the chunk boundary, copies the **entire `2.0 MiB` second half of chunk `i`** into `seam`
     (`seam_init`: **`21.70 MiB`**, **`48.5%`** of residual `in_maint`), and then incrementally copies the
     **first `2.0 MiB` of chunk `i+1`** into `seam` piece-by-piece across **`867` `WindowResume` spans**
     (`extend_from_slice`: **`21.04 MiB`**, **`48.7%`** of residual `in_maint`).
   - Together, those two avoidable `2 MiB` copies account for **`97.2%`** (`~7.28 ms` of `7.490 ms`
     i9-scaled!) of `#1031`'s residual `in_maint`. `#1031` still copied **`42.73 MiB`** (vs `base`'s
     `53.02 MiB`: `41.22 MiB` extend + `11.79 MiB` drain), explaining why measured `in_maint` fell only
     `-29.93%` on i9 (`-28.4%` on VM baseline). By contrast, `#1031`'s `879` `shared.lock().cancelled`
     calls took only `0.067–0.099 ms` (`0.6–0.9%`), `recycle/drop` `0.009 ms`, and `seam.reserve` `0.002 ms`.
2. **Why `12T` was INCONCLUSIVE (`0.9898 [0.9726, 1.0073]`) while `10T` was FASTER (`0.9811`):**
   - In `main.jsonl`, both `10T` and `12T` run 8 workers / 32 slots and have nearly identical median wall
     savings of only **`~2.0–2.3 ms` (`~1.0–1.2%`)** (`196.9 -> 194.8 ms` at 10T, median ratio `0.9901`;
     `197.3 -> 195.0 ms` at 12T, median ratio `0.9883`) because only `3.20 ms` of `in_maint` was removed.
     At `10T`, `base` hit two `+15..20 ms` outliers (`r06: 214.7 ms`, `r07: 209.8 ms`, 15/16 head<base),
     whereas at `12T`, `head` hit three `+5..11 ms` outliers (`r00: 207.9 ms`, `r01: 202.2 ms`,
     `r12: 204.5 ms`, 12/16 head<base), pushing the 95% CI upper bound to `1.0073`.
3. **Constructive rescue (`seam_adaptive`: per-block adaptive `want` + one-shot seam fill):**
   - Replacing sticky `LOOKAHEAD = 2 MiB` in `Scanner` with a non-sticky sliding-window peak over the last
     16 blocks (`want = (max16 + max16 / 4 + 4 KiB).clamp(64 KiB, 1 MiB)`, keeping the exact `br.overran()`
     retry up to `64 MiB`) and performing a **one-shot seam copy** (`tail + next[0..min(need + 128 KiB - next_base, next.len())]`)
     in `ChannelSource` cuts `wikidata-x4` bytes moved from `53.02 MiB` (`base`) / `42.73 MiB` (`#1031`)
     to **`2.29 MiB`** (**`-95.7%` vs `base`**, **`-94.6%` vs `#1031`**), drops spans from `879` to **`34`**,
     achieves **`0` overrun retries** across all 6 corpora, shrinks peak window capacity from `16.00 MiB`
     to **`4.38 MiB` (`-72.6%`)**, and cuts **`in_maint` by `-96.1%..-97.7%`** across all 3 compiler builds
     (derived i9 projection: **`10.689 -> ~0.42 ms`**, saving **`~10.3 ms`** of pure serial scanner wall,
     **3.2× `#1031`'s measured `3.20 ms` cut**).
   - End-to-end serial scan `S` on `wikidata-x4` drops **`-4.57%` baseline / `-3.46%` JCC-32B baseline /
     `-4.21%` JCC-32B native**, matching the zero-copy `slice_oracle` floor (`-4.46% / -3.34% / -5.11%`),
     and wins or ties on **all 5 off-Wikidata guard corpora** (including **`-6.50%..-7.42%` disjoint** on
     `random_18m` and **`-4.49%..-8.59%`** on `records_18m`, with `0` retries).
4. **Scoped negative finding (`frontier/dead-ends.tsv`):** Non-adaptive small/lazy lookahead
   (`seam_lazy_overran` `want = 1` and `seam_fixed_128k` `want = 128 KiB`) regress **`+23.3%..+39.6%`** on
   incompressible `random_18m` and `records_18m` due to `12–16` boundary overrun retries wasting
   `7.0–9.4 MiB` of re-scanned bytes.

Opened distinct `idea` claim [`scanner-adaptive-seam-window`](../../../frontier/claims/scanner-adaptive-seam-window.toml)
(`#1031`'s frozen `NOT ADOPTED` verdict is preserved untouched).

---

## 1. Real physical i9 hardware evidence (`origin/evidence` `c0b2341c`)

From `scanner-seam-window/run-20261127T070349Z.tar.gz` (`wikidata-x4`, `47,419,732 B` bz2 -> `838,800,784 B`
out, `1,724` blocks, 8 workers / 32 slots, 3 timeline rounds):

| Arm | Round | Wall (ms) | `in_maint` (ms) | `in_maint_spans` | `s.reading` (ms) | `s.slot_dec` (ms) | `w.decoding` | Window cap |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `base` (`95f4637b`) | 0 | 198.33 | 10.737 | **12** | 171.38 | 16.21 | 95.64% | `<= 16.8 MB` |
| `base` (`95f4637b`) | 1 | 200.00 | **10.689** | **12** | 172.05 | 16.60 | 95.79% | `<= 16.8 MB` |
| `base` (`95f4637b`) | 2 | 208.88 | 10.348 | **12** | 197.47 | 3.53 | 88.85% | `<= 16.8 MB` |
| `head` (`#1031`, `3ae44390`) | 0 | 199.41 | 7.328 | **879** | 167.83 | 20.04 | 96.81% | `<= 4.2 MB` |
| `head` (`#1031`, `3ae44390`) | 1 | 197.78 | **7.490** | **879** | 167.90 | 17.98 | 96.32% | `<= 4.2 MB` |
| `head` (`#1031`, `3ae44390`) | 2 | 196.46 | 7.701 | **879** | 168.31 | 17.40 | 96.54% | `<= 4.2 MB` |

Timed `main.jsonl` summary (16 paired rounds per count): `6T` `0.9969 [0.9923, 1.0015]` (8/16 head<base),
`8T` `1.0000 [0.9843, 1.0159]` (10/16), `9T` `0.9663 [0.9472, 0.9859]` (14/16), `10T` (judged, 8W)
`0.9811 [0.9670, 0.9954]` (15/16, median ratio `0.9901`), `12T` (judged, 8W) `0.9898 [0.9726, 1.0073]`
(12/16, median ratio `0.9883`).

---

## 2. Exact deterministic accounting on `wikidata-x4` (`45.22 MiB`, `1,724` blocks)

Every arm verified bit-identical (`blocks = 1724`, `digest = 0xb94448f9bc8f483d`, zero `unsafe`):

| Arm | `spans` | `recv` | `s_ent` | `s_res` | `drain` | `locks` | `ext_MB` | `sini_MB` | `drn_MB` | `tot_MB` | `drop` | `retry` | `buf_MB` | `seam_MB` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `base_shipped` | **12** | 12 | 0 | 0 | 6 | 12 | 41.22 | 0.00 | 11.79 | **53.02** | 4 | 0 | 16.00 | 0.00 |
| `seam_1031` | **879** | 12 | 11 | 867 | 0 | 879 | 21.04 | 21.70 | 0.00 | **42.73** | 4 | 0 | 4.00 | 4.00 |
| `seam_1031_atomic` | 879 | 12 | 11 | 867 | 0 | 0 | 21.04 | 21.70 | 0.00 | 42.73 | 4 | 0 | 4.00 | 4.00 |
| `seam_1031_pool4` | 879 | 12 | 11 | 867 | 0 | 0 | 21.04 | 21.70 | 0.00 | 42.73 | 1 | 0 | 4.00 | 4.00 |
| `seam_lazy_overran` | 34 | 12 | 11 | 22 | 0 | 0 | 1.38 | 0.21 | 0.00 | 1.59 | 0 | 22 | 4.00 | 0.55 |
| `seam_fixed_128k` | 34 | 12 | 11 | 22 | 0 | 0 | 1.57 | 1.18 | 0.00 | 2.75 | 0 | 0 | 4.00 | 0.25 |
| **`seam_adaptive`** | **34** | **12** | **11** | **22** | **0** | **0** | **1.58** | **0.72** | **0.00** | **2.29** | **0** | **0** | **4.00** | **0.38** |
| `slice_oracle` | 0 | 0 | 0 | 0 | 0 | 0 | 0.00 | 0.00 | 0.00 | 0.00 | 0 | 0 | 0.00 | 0.00 |

Exact reproduction: `base_shipped` hits `spans == 12` and `buf_MB == 16.00`; `seam_1031` hits `spans == 879`
and `buf_MB == 4.00`, matching Louis's physical i9 traces `12` and `879` identically.

---

## 3. Component-level `in_maint` microsecond breakdown (`wikidata-x4`, baseline x86-64, median of 5)

| Arm | `tot_ms` (uncont.) | `s_init` | `extend` | `drain` | `reserve` | `recy_drp` | `cancel` | `other` | `tot_ms` (contended) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `base_shipped` | 14.004 | 0.000 | 10.943 | 2.935 | 0.000 | 0.013 | 0.001 | 0.007 | 22.203 |
| `seam_1031` | 10.429 | **5.053** | **5.079** | 0.000 | 0.002 | 0.009 | 0.067 | 0.148 | 14.814 |
| `seam_1031_atomic` | 10.615 | 5.236 | 5.317 | 0.000 | 0.002 | 0.010 | 0.001 | 0.111 | 14.601 |
| `seam_1031_pool4` | 10.634 | 5.249 | 5.191 | 0.000 | 0.002 | 0.008 | 0.001 | 0.106 | 14.855 |
| `seam_lazy_overran` | 0.415 | 0.051 | 0.348 | 0.000 | 0.003 | 0.003 | 0.001 | 0.012 | 0.403 |
| `seam_fixed_128k` | 0.682 | 0.279 | 0.354 | 0.000 | 0.002 | 0.003 | 0.001 | 0.014 | 0.717 |
| **`seam_adaptive`** | **0.592** | **0.206** | **0.391** | **0.000** | **0.004** | **0.004** | **0.001** | **0.014** | **0.590** |

In `seam_1031`, `s_init` (`5.053 ms`, `48.5%`) + `extend` (`5.079 ms`, `48.7%`) = **`10.132 ms / 10.429 ms = 97.2%`**
of residual `in_maint` (`base_jcc32b`: `4.664 + 4.947 = 9.611 / 9.857 ms = 97.5%`).

---

## 4. End-to-end interleaved `S` (`scan + in_maint + retry`) across 3 compiler builds (`wikidata-x4`, `n = 7`)

2-vCPU Haswell KVM VM; exact thread on-CPU time (`/proc/thread-self/schedstat`), rotated arm order per repeat:

| Arm | `baseline` CPU ms (`vs_base`) | `baseline` `in_maint` (`cut`) | `base_jcc32b` CPU ms (`vs_base`) | `base_jcc32b` `in_maint` (`cut`) | `nat_jcc32b` CPU ms (`vs_base`) | `nat_jcc32b` `in_maint` (`cut`) |
|---|---:|---:|---:|---:|---:|---:|
| `base_shipped` | 305.17 (`+0.00%`) | 15.044 (`+0.0%`) | 306.47 (`+0.00%`) | 28.859 (`+0.0%`) | 298.54 (`+0.00%`) | 26.151 (`+0.0%`) |
| `seam_1031` | 300.91 (`-1.40%`) | 10.769 (`-28.4%`) | 303.06 (`-1.11%`) | 21.526 (`-25.4%`) | 294.59 (`-1.32%`) | 22.784 (`-12.9%`) |
| `seam_lazy_overran` | 295.51 (`-3.16%`) | 0.394 (`-97.4%`) | 297.68 (`-2.87%`) | 0.443 (`-98.5%`) | 289.98 (`-2.87%`) | 0.392 (`-98.5%`) |
| `seam_fixed_128k` | 293.00 (`-3.99%`) | 0.710 (`-95.3%`) | 293.86 (`-4.11%`) | 0.671 (`-97.7%`) | 285.05 (`-4.52%`) | 0.668 (`-97.4%`) |
| **`seam_adaptive`** | **291.23 (`-4.57%`)** | **0.581 (`-96.1%`)** | **295.85 (`-3.46%`)** | **0.709 (`-97.5%`)** | **285.98 (`-4.21%`)** | **0.603 (`-97.7%`)** |
| `slice_oracle` | 291.55 (`-4.46%`) | 0.000 (`-100.0%`) | 296.23 (`-3.34%`) | 0.000 (`-100.0%`) | 283.29 (`-5.11%`) | 0.000 (`-100.0%`) |

---

## 5. Multi-chunk off-Wikidata guard corpora (`n = 5`, 100% bit-identical across all 3 builds)

| Corpus | Arm | `spans` | `retry` | `tot_MB` | `wast_KB` | `baseline` CPU (`vs_base`) | `base_jcc32b` CPU (`vs_base`) | `nat_jcc32b` CPU (`vs_base`) | `in_maint` cut |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `text_16m` (112 blks) | `base_shipped` | 5 | 0 | 16.22 | 0.0 | 120.73 (`+0.00%`) | 121.99 (`+0.00%`) | 120.64 (`+0.00%`) | +0.0% |
| `text_16m` | `seam_1031` | 63 | 0 | 13.89 | 0.0 | 117.50 (`-2.67%`) | 120.49 (`-1.23%`) | 118.63 (`-1.67%`) | -31..-60% |
| `text_16m` | `seam_lazy_overran` | 14 | 9 | 0.98 | 696.7 | 121.31 (`+0.48%`) | 125.81 (`+3.13%`) | 121.66 (`+0.84%`) | -95..-97% |
| `text_16m` | **`seam_adaptive`** | **16** | **0** | **2.49** | **0.0** | **117.34 (`-2.80%` disj.)** | **117.69 (`-3.52%` disj.)** | **118.63 (`-1.66%`)** | **-91.5..-94.9%** |
| `random_18m` (30 blks) | `base_shipped` | 6 | 0 | 22.52 | 0.0 | 101.92 (`+0.00%`) | 104.76 (`+0.00%`) | 103.70 (`+0.00%`) | +0.0% |
| `random_18m` | `seam_lazy_overran` | 22 | 16 | 6.13 | 7499.7 | 130.47 (`+28.01%`) | 131.90 (`+25.91%`) | 129.68 (`+25.05%`) | -75..-88% |
| `random_18m` | `seam_fixed_128k` | 21 | 14 | 6.19 | 7530.3 | 130.97 (`+28.50%`) | 132.52 (`+26.50%`) | 130.19 (`+25.55%`) | -79..-90% |
| `random_18m` | **`seam_adaptive`** | **16** | **0** | **5.62** | **0.0** | **95.29 (`-6.50%` disj.)** | **97.22 (`-7.20%` disj.)** | **96.01 (`-7.42%` disj.)** | **-81.2..-91.3%** |
| `records_18m` (30 blks) | `base_shipped` | 6 | 0 | 22.57 | 0.0 | 105.95 (`+0.00%`) | 108.37 (`+0.00%`) | 106.95 (`+0.00%`) | +0.0% |
| `records_18m` | `seam_lazy_overran` | 22 | 16 | 7.04 | 9391.5 | 142.34 (`+34.35%`) | 151.31 (`+39.63%`) | 144.56 (`+35.16%`) | -72..-87% |
| `records_18m` | `seam_fixed_128k` | 20 | 12 | 6.03 | 7009.4 | 132.54 (`+25.10%`) | 134.48 (`+24.09%`) | 131.81 (`+23.25%`) | -80..-91% |
| `records_18m` | **`seam_adaptive`** | **16** | **0** | **5.62** | **0.0** | **101.19 (`-4.49%`)** | **99.06 (`-8.59%`)** | **99.57 (`-6.90%` disj.)** | **-80.2..-91.7%** |
| `runs_8m` (829 blks) | `base_shipped` | 3 | 0 | 6.00 | 0.0 | 70.16 (`+0.00%`) | 69.49 (`+0.00%`) | 69.40 (`+0.00%`) | +0.0% |
| `runs_8m` | `seam_1031` | 419 | 0 | 5.99 | 0.0 | 70.33 (`+0.25%`) | 69.64 (`+0.23%`) | 69.89 (`+0.71%`) | -37..-85% |
| `runs_8m` | **`seam_adaptive`** | **6** | **0** | **0.25** | **0.0** | **70.20 (`+0.06%`)** | **65.61 (`-5.58%` disj.)** | **65.59 (`-5.50%`)** | **-97.5..-99.4%** |

`onebyte` (`1` block, `51 B` bz2): `spans = 1`, `retry = 0`, `in_maint = 0.000 ms`, exact across all builds.

---

## 6. System implication (`frontier/DECODER.md` & `k = 8` knee)

In the thirteenth synthesis pass (`#1133`), `#1031`'s `NOT ADOPTED` verdict restored `~10–13 ms` to
post-`#1034` serial scan `S'` (`~141–148 ms` default / `~135–143 ms` aligned before `scanner-a2-fast-build`).
`scanner-adaptive-seam-window` recovers **`~10.3 ms`** of that `10.689 ms` i9 `in_maint` **(derived)** in
~40 LOC of safe Rust — **3.2× `#1031`'s measured `3.20 ms`**, with `-72.6%` smaller window capacity (`4.38 MiB`
vs `16.00 MiB`), zero retries on all 6 corpora, and orthogonal stacking with both `#1034` and
`scanner-a2-fast-build` (`#1117`). At `k = 8` on aligned Wikidata or off-Wikidata `text` (and at `k = 9–10`
on default Wikidata), that transfers directly into `~5–7%` wall gain without any worker-side change.
