# Spelling census: where emission's 298 ms (`E_consume`) goes

Claim [`spelling-census`](../../../../frontier/claims/spelling-census.toml) — smooth-zone characterisation before theory. Zero changes to `src/`, zero `unsafe`, zero changes to production defaults.

**Harness:** [`examples/spelling_census.rs`](../../../../examples/spelling_census.rs), mirroring the production Open constructor (`src/open.rs`, `src/open/endpoint.rs`, `src/open/outq.rs`, `LIMIT = 16`, `INLINE = 16`, `CERTIFIED_WIDTH = 32_768`, `Narrow`/`Wide` arena, `OutQ16`, `Dictionary`), full production emission (`emit_bidirectional` / `emit`), and the zero-copy floor's exact `E_consume` spelling loop (`emit_bidirectional_codes` / `emit_codes` from PR #921).

**Machine, method & caveats:**
- **Hardware:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2, 32 KiB L1d, 256 KiB L2, 45 MiB L3), single-threaded, reusable `out` buffer across blocks.
- **No hardware vPMU:** `perf_event_open(PERF_TYPE_HARDWARE)` returns `ENOENT` (`errno = 2`) for all six hardware counters (`cycles`, `instructions`, `cache-references`, `cache-misses`, `branches`, `branch-misses`) inside this KVM VM. Branch-misprediction and cache/streaming stall costs are therefore measured directly via **controlled code-level ablations** (same-loop 256-code chunked order ablation with built-in zero-long controls on `runs`/`records`, plus L1d-pinned dictionary and output-buffer ablations).
- **Method:** Interleaved repeats (`7` repeats on `wikidata`, `runs`, `text`, `records`, `pattern`; `5` repeats on `wikidata ×4`) after a full warm-up pass where every full-output arm (`exact`) is verified **100% byte-identical** to `bwt::walk` on every admitted block. Every timing table reports the median and `min..max` spread for `ns/code`, `ns/byte`, and paired per-repeat ratio against the production spelling mirror (`mirror_e_consume = 1.000×`).
- **Cross-machine anchor:** On the i9-10900K (5.02 GHz, [`ideal-parts/results/floor/README.md`](../ideal-parts/results/floor/README.md)), `E_consume` on `wikidata ×4` (`106,270,324` codes, `838,790,648` Open bytes) takes **298.2 ms** at 1 worker (**2.806 ns/code**, **0.356 ns/byte**, 22% of the 1,343 ms wall) and **37.2 ms** at 8 workers, accounting for **43.2%** of total emission (`SECMRD / SCMRD`). On this 2.30 GHz Xeon VM, `mirror_e_consume` takes **845.9 ms** on `wikidata ×4` (**7.960 ns/code**, **1.008 ns/byte**), accounting for **56.2%** of full emission (`prod_walk_and_spell`) — a 2.84× clock/IPC scale factor with consistent proportions; all structural splits below are reported as ratios against `mirror_e_consume` on the same machine.

---

## Executive summary

**`E_consume` is 7.6× slower than `memcpy` (1.008–1.034 ns/byte vs 0.136–0.139 ns/byte) — NOT because of copying bytes, but because of bidirectional shift overhead and a storm of 1-byte literals.**

1. **81.58% of all emitted codes on Wikidata are 1-byte literals (`code < 256`), producing only 10.34% of the bytes.** Combined inline codes (`≤ 16 B`) are **93.57%** of codes (`24.86M` codes, mean `1.58 B`) for only **18.71%** of bytes. Meanwhile, **long words (`> 16 B`) are only 6.43% of codes (`1.71M`), yet produce 81.29% of all bytes (`170.5 MB`, mean `99.8 B`)** — and copy at near-`memcpy` speed (**0.098–0.114 ns/byte**, just **7.8–9.2%** of `E_consume`).
2. **Where `mirror_e_consume` spends its time on Wikidata (`8.158 ns/code` on `wikidata`, `7.960 ns/code` on `wikidata ×4`, ratios against `mirror_e_consume`):**
   - **Bidirectional spelling overhead (`mirror − forward_only`):** **41.6–42.6%** (`+3.31..3.47 ns/code`, ≈125 ms of the i9's 298 ms). Splits cleanly into:
     - **24.2–24.3%** (`+1.92..1.98 ns/code`, ≈72 ms) for the runtime `u128 << 8*(16 - lb)` right-alignment shift on every back-side inline word (paid on 232/233 Wikidata blocks). Pre-computing a right-aligned `inline_back` table in A6b (`bidi_pre_shifted`, byte-exact) recovers **26.3–26.8%** of `E_consume` with zero cache penalty even while staying bidirectional.
     - **17.4–18.3%** (`+1.39..1.49 ns/code`, ≈53 ms) for dual-cursor control (`hi - lo >= INLINE`, two pointers/indices, `single` check).
   - **Per-code lookup, branch & loop (`no_copy_length_only`):** **34.8–36.3%** (`+2.77..2.96 ns/code`, ≈106 ms), including `codes[]` scan (**5.8–6.1%**), `lens[code]` load + `at += len` chain, and **`if len != LONG` branch mispredictions** (**7.2–7.9%**, `+0.585..0.629 ns/code`, ≈23 ms; 8.06% 2-bit miss rate, verified against 0% zero-long controls on `runs`/`records`).
   - **Inline 16 B copy (`forward_only − long_only`):** **18.5–18.9%** (`+1.50..1.51 ns/code`, ≈56 ms) to emit 18.7% of bytes (**6.5×** more expensive per byte than long-word copies).
   - **Long-word copy (`forward_only − inline_only`):** **7.8–9.2%** (`+0.63..0.73 ns/code`, ≈25 ms) to emit **81.3%** of all bytes.
   - *(Out-of-order overlap between lookup and copy is −4.4% to −5.1%.)*
3. **Cache footprint is NOT the bottleneck:** `inline16 + lens` fits in 32 KiB L1d on **233/233** Wikidata blocks (median **18.0 KiB**, max **28.4 KiB**), and the entire dictionary including `spelling` fits in 256 KiB L2 on **233/233** blocks (median **61.8 KiB**, max **87.3 KiB**). Dictionary payload cache stalls (`forward_only − dict_l1_pinned`) account for only **7.4–7.5%** of `E_consume`, and sequential `out` writes (`forward_only − out_l1_pinned`) are **~0%**.
4. **Literals arrive in long consecutive streaks:** **88.65%** of literals are immediately followed by another literal (mean streak **8.81** literals; **79.03%** of all codes sit in literal streaks of length ≥ 2; **88.35%** of adjacent code pairs fit in 16 B). Greedily fusing consecutive `≤ 16 B` codes (`fused_16b_upper_bound`, byte-exact) cuts code count by **81.3%** (`26.57M → 4.97M`) and drops spelling to **0.215–0.217×** (`4.6×` faster, `0.220 ns/byte` — only 1.6× above `memcpy`).

---

## 1. Corpora & structural census

Baseline counts on `corpora/wikidata.bz2` (SHA-256 checked against `bench/wikidata.json`) match `rauzy_census` and `examples/census.rs` to the exact integer: `233` Open blocks out of `431` (`209,697,662` Open rows = `99.996%` of `209,706,557` total rows), `26,567,581` retained rows (`m`) and emitted codes. On `ab gen corpora`, production `OpenCells` admits `runs.bz2` (`1` block); `text.bz2`, `records.bz2`, and `pattern.bz2` are refused by production density/sparsity gates and measured under `--forced` to bracket the regimes (dense literals, pure literals, pure long words).

### 1.1 Block overview & dictionary cache footprint

| corpus (admission) | Open blocks (`bidi`) | exact cycle | Open bytes (`n`) | emitted codes (`L`) | mean B/code | dict entries/blk `p50/p90/max` | `inline+lens` KiB `p50/p90/max` | `spelling` KiB `p50/p90/max` | total dict KiB `p50/p90/max` | blocks >32 KiB L1d / >256 KiB L2 |
|---|---:|---:|---:|---:|---:|---|---:|---:|---:|---|
| **wikidata** (prod) | 233 (232, 99.6%) | 233 (100%) | 209,697,662 | 26,567,581 | 7.893 | 1,083 / 1,197 / 1,712 | 18.0 / 19.9 / 28.4 | 43.5 / 49.8 / 68.5 | 61.8 / 68.8 / 87.3 | 229 (0 `inline`) / **0** |
| **wikidata ×4** (prod) | 932 (928, 99.6%) | 932 (100%) | 838,790,648 | 106,270,324 | 7.893 | 1,083 / 1,197 / 1,712 | 18.0 / 19.9 / 28.4 | 43.5 / 49.8 / 68.5 | 61.8 / 68.8 / 87.3 | 916 (0 `inline`) / **0** |
| **runs** (`ab gen`, prod) | 1 (1, 100%) | 1 (100%) | 508,269 | 145,985 | 3.482 | 276 / 276 / 276 | 4.6 / 4.6 / 4.6 | 0.0 / 0.0 / 0.0 | 4.6 / 4.6 / 4.6 | 0 / 0 |
| **text** (`ab gen`, forced) | 26 (26, 100%) | 26 (100%) | 22,983,779 | 13,317,069 | 1.726 | 2,607 / 2,735 / 2,751 | 43.3 / 45.4 / 45.7 | 0.8 / 1.0 / 1.1 | 43.8 / 46.2 / 46.6 | 25 (25 `inline`) / 0 |
| **records** (`ab gen`, forced) | 22 (22, 100%) | 22 (100%) | 19,200,003 | 19,200,003 | 1.000 | 256 / 256 / 256 | 4.2 / 4.2 / 4.2 | 0.0 / 0.0 / 0.0 | 4.2 / 4.2 / 4.2 | 0 / 0 |
| **pattern** (`ab gen`, forced) | 44 (0, 0%) | 44 (100%) | 39,200,000 | 44 | 890,909 | 257 / 257 / 257 | 4.3 / 4.3 / 4.3 | 878.9 / 878.9 / 878.9 | 883.2 / 883.2 / 883.2 | 44 (0 `inline`) / 44 |

Key structural facts:
- **100.0% of blocks** on every corpus have `exact_cycle` (`sum(word_len) == n`, `cycle_len == m`). Every non-literal dictionary entry built in A6b is used at least once (`100.0%` used; the `16.44%` unused entries on Wikidata are solely unused ASCII/byte slots among the fixed 256 literal slots).
- **99.6% of Wikidata Open blocks (232/233)** take `emit_bidirectional` (`m >= 65_536 && !wide`).
- On Wikidata, `inline16 + lens` never exceeds 32 KiB L1d (`0/233` blocks, max `28.4 KiB`), and `inline16 + lens + spelling` never exceeds 256 KiB L2 (`0/233` blocks, max `87.3 KiB`).

### 1.2 Inline (≤ 16 B) vs long (> 16 B) split and word-length distribution

#### By class on `wikidata` (233 blocks, `26,567,581` codes, `209,697,662` bytes)

| class | dict entries (used) | emitted codes | code share | emitted bytes | byte share | mean length |
|---|---:|---:|---:|---:|---:|---:|
| **literal (`1 B`, `code < 256`)** | 59,648 (19,097) | 21,672,913 | **81.58%** | 21,672,913 | **10.34%** | 1.00 B |
| **inline word (`2..=16 B`)** | 122,718 (122,718) | 3,186,535 | **11.99%** | 17,563,701 | **8.38%** | 5.51 B |
| **ALL inline (`≤ 16 B`)** | 182,366 (141,815) | 24,859,448 | **93.57%** | 39,236,614 | **18.71%** | 1.58 B |
| **long word (`> 16 B`)** | 64,314 (64,314) | 1,708,133 | **6.43%** | 170,461,048 | **81.29%** | 99.79 B |

*(Boundary note: `len == 16` specifically is only `39,767` codes = `0.15%` of codes and `0.30%` of bytes.)*

#### Fine length histogram on `wikidata`

| length bucket | dict entries | emitted codes | code % | emitted bytes | byte % |
|---|---:|---:|---:|---:|---:|
| `1` | 59,648 | 21,672,913 | 81.58% | 21,672,913 | 10.34% |
| `2` | 42,639 | 860,822 | 3.24% | 1,721,644 | 0.82% |
| `3..=4` | 25,339 | 812,321 | 3.06% | 2,724,227 | 1.30% |
| `5..=8` | 32,798 | 934,987 | 3.52% | 5,838,357 | 2.78% |
| `9..=16` | 21,942 | 578,405 | 2.18% | 7,279,473 | 3.47% |
| `17..=32` | 17,549 | 466,392 | 1.76% | 10,360,280 | 4.94% |
| `33..=64` | 18,072 | 580,356 | 2.18% | 27,867,509 | 13.29% |
| `65..=128` | 12,527 | 262,655 | 0.99% | 23,932,787 | 11.41% |
| `129..=256` | 9,688 | 243,515 | 0.92% | 45,324,300 | 21.61% |
| `257..=1024` | 6,369 | 153,742 | 0.58% | 60,440,475 | 28.82% |
| `> 1024` | 109 | 1,473 | 0.01% | 2,535,697 | 1.21% |

**Length percentiles on `wikidata` (`min / p10 / p25 / p50 / p75 / p90 / p95 / p99 / max`):**
- **By emitted codes:** `1 / 1 / 1 / 1 / 1 / 6 / 26 / 193 / 8,693`
- **By emitted bytes:** `1 / 1 / 36 / 135 / 307 / 435 / 588 / 1,391 / 8,693`
- **By dictionary entries:** `1 / 1 / 2 / 4 / 18 / 78 / 165 / 398 / 8,693`

#### Class mix across other corpora

| corpus | literal (1 B) codes / bytes | inline (2..16 B) codes / bytes | long (>16 B) codes / bytes | `p_long` / 2-bit miss | mean lit streak / `≥ 2` lit-streak code share |
|---|---:|---:|---:|---:|---:|
| **wikidata** | 81.58% / 10.34% | 11.99% / 8.38% | 6.43% / 81.29% | 6.43% / **8.06%** | **8.81** / **79.03%** |
| **runs** (prod) | 31.12% / 8.94% | 68.88% / 91.06% | 0.00% / 0.00% | 0.00% / 0.00% | 1.02 / 1.08% |
| **text** (forced) | 73.02% / 42.31% | 26.90% / 56.62% | 0.08% / 1.07% | 0.08% / 0.08% | 3.84 / 68.70% |
| **records** (forced) | 100.00% / 100.00% | 0.00% / 0.00% | 0.00% / 0.00% | 0.00% / 0.00% | 872,727 / 100.00% |
| **pattern** (forced) | 0.00% / 0.00% | 0.00% / 0.00% | 100.00% / 100.00% | 100.00% / 0.00% | 0.00 / 0.00% |

### 1.3 Word reuse, top-k hotness, and `spelling` buffer waste (`wikidata`)

#### Per-block reuse distribution (`wikidata`, 233 blocks)

| reuse count | ALL used: distinct % / codes % / bytes % | INLINE (2..16 B): distinct % / codes % / bytes % | LONG (>16 B): distinct % / codes % / bytes % |
|---|---:|---:|---:|
| `1` (one-shot) | 13.22% / 0.10% / 0.41% | 14.77% / 0.57% / 0.50% | 13.46% / **0.51%** / **0.45%** |
| `2..=4` | 15.66% / 0.34% / 1.27% | 18.10% / 1.98% / 1.71% | 14.28% / 1.50% / 1.38% |
| `5..=16` | 26.28% / 2.08% / 9.48% | 28.13% / 11.08% / 11.07% | 27.03% / 10.37% / 10.51% |
| `17..=64` | 32.45% / 7.14% / 42.47% | 32.07% / 35.00% / 35.43% | 37.35% / **38.47%** / **48.53%** |
| `65..=256` | 6.91% / 6.50% / 26.08% | 5.58% / 24.49% / 27.71% | 6.81% / **30.86%** / **28.98%** |
| `> 256` | 5.47% / 83.83% / 20.29% | 1.36% / 26.88% / 23.58% | 1.07% / **18.29%** / **10.15%** |

- **Long words are heavily reused:** mean `26.56` emissions per distinct long word (`1,708,133 / 64,314`); **87.6%** of long-word emissions (**87.7%** of long bytes) come from words reused ≥ 17 times. Only `0.45%` of emitted long bytes are one-shot.
- **Top-k concentration (`wikidata`, per-block median / corpus-weighted mean):**
  - **Top 16 codes:** `59.35% / 59.56%` of codes (`6.66% / 7.55%` of bytes)
  - **Top 64 codes:** `85.63% / 86.19%` of codes (`30.00% / 31.42%` of bytes)
  - **Top 256 codes:** `94.44% / 94.92%` of codes (`73.03% / 73.18%` of bytes) — fits in **4 KiB** of `inline16`
  - **Top 512 codes:** `98.60% / 98.56%` of codes (`94.59% / 94.06%` of bytes) — fits in **8 KiB** of `inline16`
  - **Top 1024 codes:** `100.00% / 99.98%` of codes (`100.00% / 99.93%` of bytes)
- **`spelling` buffer slack built in A6b (`10,125,582 B` total across 233 blocks, ~43.5 KiB/block):**
  - Reused long words: `6,083,269 B` (**60.08%**)
  - One-shot long words: `769,879 B` (**7.60%**)
  - Unused long words: `0 B` (**0.00%**)
  - **`memo_dedup_slack`: `3,272,434 B` (32.32%, ~14 KiB/block)** — bytes permanently retained in `spelling` when `Dictionary::code` deduplicates a newly-expanded long word whose child rules already memoized offsets into that expansion (`src/open/endpoint.rs` lines 548–556).

---

## 2. Timing census against the production spelling mirror (`E_consume`)

### 2.1 `wikidata` (233 Open blocks, `26,567,581` codes, `209,697,662` bytes, 7 interleaved repeats)

| arm | check | ms med (`min..max`) | ns/code med (`min..max`) | ns/byte med (`min..max`) | ratio vs `mirror` (`min..max`) |
|---|---|---:|---:|---:|---:|
| `prod_walk_and_spell` | exact | 375.30 (`357.48..446.67`) | 14.126 (`13.456..16.813`) | 1.7897 (`1.7048..2.1301`) | 1.747× (`1.639..1.871×`) |
| **`mirror_e_consume`** | **exact** | **216.75** (`208.21..238.72`) | **8.158** (`7.837..8.985`) | **1.0336** (`0.9929..1.1384`) | **1.000×** |
| `bidi_pre_shifted` | exact | 159.69 (`153.06..169.43`) | 6.011 (`5.761..6.377`) | 0.7615 (`0.7299..0.8080`) | **0.737×** (`0.674..0.790×`) |
| `bidi_no_shift_ablat` | ablat | 164.09 (`157.49..170.86`) | 6.176 (`5.928..6.431`) | 0.7825 (`0.7510..0.8148`) | 0.758× (`0.660..0.796×`) |
| **`forward_only`** | **exact** | **124.51** (`122.15..134.04`) | **4.686** (`4.598..5.045`) | **0.5937** (`0.5825..0.6392`) | **0.569×** (`0.540..0.618×`) |
| `inline_only_skip_long` | ablat | 107.69 (`103.17..135.67`) | 4.054 (`3.883..5.106`) | 0.5136 (`0.4920..0.6470`) | 0.495× (`0.481..0.574×`) |
| `long_only_skip_inline` | ablat | 84.43 (`81.17..89.05`) | 3.178 (`3.055..3.352`) | 0.4026 (`0.3871..0.4247`) | 0.395× (`0.348..0.410×`) |
| `no_copy_length_only` | ablat | 78.66 (`73.21..97.93`) | 2.961 (`2.756..3.686`) | 0.3751 (`0.3491..0.4670`) | 0.354× (`0.341..0.410×`) |
| `codes_scan_only` | ablat | 13.27 (`12.71..14.20`) | 0.499 (`0.478..0.534`) | 0.0633 (`0.0606..0.0677`) | 0.061× (`0.054..0.066×`) |
| `memcpy_out_floor` | exact | 29.21 (`27.07..32.63`) | 1.099 (`1.019..1.228`) | 0.1393 (`0.1291..0.1556`) | **0.132×** (`0.119..0.151×`) |
| `chunk_mixed_pairs` | exact | 123.70 (`121.58..128.92`) | 4.656 (`4.576..4.853`) | 0.5899 (`0.5798..0.6148`) | 0.572× (`0.509..0.596×`) |
| `chunk_sorted_pairs` | ablat | 108.16 (`107.23..115.07`) | 4.071 (`4.036..4.331`) | 0.5158 (`0.5113..0.5487`) | 0.505× (`0.449..0.527×`) |
| `dict_l1_pinned` | ablat | 108.18 (`106.99..136.48`) | 4.072 (`4.027..5.137`) | 0.5159 (`0.5102..0.6508`) | 0.505× (`0.453..0.630×`) |
| `out_l1_pinned` | ablat | 124.25 (`123.41..132.76`) | 4.677 (`4.645..4.997`) | 0.5925 (`0.5885..0.6331`) | 0.579× (`0.518..0.620×`) |
| `packed_15b_single_arr` | exact | 124.93 (`122.81..158.19`) | 4.702 (`4.622..5.954`) | 0.5957 (`0.5856..0.7544`) | 0.577× (`0.542..0.730×`) |
| `inline_32b` | exact | 124.45 (`120.76..130.44`) | 4.684 (`4.546..4.910`) | 0.5935 (`0.5759..0.6220`) | 0.584× (`0.524..0.602×`) |
| `direct_meta_no_clamp` | exact | 139.78 (`137.30..210.98`) | 5.261 (`5.168..7.941`) | 0.6666 (`0.6548..1.0061`) | 0.659× (`0.615..0.884×`) |
| `freq_sorted_dict` | exact | 122.79 (`121.75..129.91`) | 4.622 (`4.583..4.890`) | 0.5856 (`0.5806..0.6195`) | 0.562× (`0.514..0.605×`) |
| `unrolled_4x_forward` | exact | 117.57 (`115.10..118.14`) | 4.425 (`4.332..4.447`) | 0.5607 (`0.5489..0.5634`) | **0.542×** (`0.495..0.560×`) |
| **`fused_16b_upper_bound`** | **exact** | **46.02** (`44.58..52.09`) | **1.732** (`1.678..1.961`) | **0.2195** (`0.2126..0.2484`) | **0.217×** (`0.188..0.240×`) |

### 2.2 `wikidata ×4` (932 Open blocks, `106,270,324` codes, `838,790,648` bytes, 5 interleaved repeats)

| arm | check | ms med (`min..max`) | ns/code med (`min..max`) | ns/byte med (`min..max`) | ratio vs `mirror` (`min..max`) |
|---|---|---:|---:|---:|---:|
| `prod_walk_and_spell` | exact | 1504.27 (`1453.79..1513.23`) | 14.155 (`13.680..14.239`) | 1.7934 (`1.7332..1.8041`) | 1.722× (`1.689..1.810×`) |
| **`mirror_e_consume`** | **exact** | **845.86** (`834.66..895.78`) | **7.960** (`7.854..8.429`) | **1.0084** (`0.9951..1.0679`) | **1.000×** |
| `bidi_pre_shifted` | exact | 619.51 (`612.04..672.75`) | 5.830 (`5.759..6.331`) | 0.7386 (`0.7297..0.8020`) | **0.732×** (`0.718..0.751×`) |
| `bidi_no_shift_ablat` | ablat | 641.58 (`634.22..671.48`) | 6.037 (`5.968..6.319`) | 0.7649 (`0.7561..0.8005`) | 0.751× (`0.742..0.777×`) |
| **`forward_only`** | **exact** | **494.18** (`485.29..514.15`) | **4.650** (`4.567..4.838`) | **0.5892** (`0.5786..0.6130`) | **0.574×** (`0.568..0.592×`) |
| `inline_only_skip_long` | ablat | 416.43 (`412.29..434.61`) | 3.919 (`3.880..4.090`) | 0.4965 (`0.4915..0.5181`) | 0.492× (`0.460..0.514×`) |
| `long_only_skip_inline` | ablat | 334.62 (`327.54..350.64`) | 3.149 (`3.082..3.299`) | 0.3989 (`0.3905..0.4180`) | 0.396× (`0.368..0.420×`) |
| `no_copy_length_only` | ablat | 294.36 (`292.34..307.30`) | 2.770 (`2.751..2.892`) | 0.3509 (`0.3485..0.3664`) | 0.346× (`0.329..0.368×`) |
| `codes_scan_only` | ablat | 49.39 (`48.66..51.17`) | 0.465 (`0.458..0.482`) | 0.0589 (`0.0580..0.0610`) | 0.058× (`0.054..0.061×`) |
| `memcpy_out_floor` | exact | 114.36 (`108.55..123.43`) | 1.076 (`1.021..1.161`) | 0.1363 (`0.1294..0.1472`) | **0.132×** (`0.121..0.148×`) |
| `chunk_mixed_pairs` | exact | 506.00 (`485.38..521.22`) | 4.761 (`4.567..4.905`) | 0.6033 (`0.5787..0.6214`) | 0.598× (`0.542..0.624×`) |
| `chunk_sorted_pairs` | ablat | 439.21 (`431.77..449.78`) | 4.133 (`4.063..4.232`) | 0.5236 (`0.5147..0.5362`) | 0.517× (`0.482..0.532×`) |
| `dict_l1_pinned` | ablat | 431.87 (`427.15..444.43`) | 4.064 (`4.019..4.182`) | 0.5149 (`0.5092..0.5298`) | 0.511× (`0.477..0.532×`) |
| `out_l1_pinned` | ablat | 522.21 (`491.50..554.39`) | 4.914 (`4.625..5.217`) | 0.6226 (`0.5860..0.6609`) | 0.618× (`0.553..0.656×`) |
| `packed_15b_single_arr` | exact | 494.89 (`487.82..562.88`) | 4.657 (`4.590..5.297`) | 0.5900 (`0.5816..0.6711`) | 0.577× (`0.545..0.666×`) |
| `inline_32b` | exact | 495.36 (`488.00..576.83`) | 4.661 (`4.592..5.428`) | 0.5906 (`0.5818..0.6877`) | 0.577× (`0.554..0.683×`) |
| `direct_meta_no_clamp` | exact | 557.17 (`544.23..578.41`) | 5.243 (`5.121..5.443`) | 0.6643 (`0.6488..0.6896`) | 0.653× (`0.608..0.685×`) |
| `freq_sorted_dict` | exact | 489.62 (`485.48..501.92`) | 4.607 (`4.568..4.723`) | 0.5837 (`0.5788..0.5984`) | 0.575× (`0.543..0.593×`) |
| `unrolled_4x_forward` | exact | 468.22 (`460.99..473.72`) | 4.406 (`4.338..4.458`) | 0.5582 (`0.5496..0.5648`) | **0.554×** (`0.515..0.568×`) |
| **`fused_16b_upper_bound`** | **exact** | **185.30** (`181.96..243.36`) | **1.744** (`1.712..2.290`) | **0.2209** (`0.2169..0.2901`) | **0.215×** (`0.207..0.292×`) |

### 2.3 Component decomposition summary (`wikidata` & `wikidata ×4`, plus `ab gen` corpora)

| component | `wikidata` ns/code (% mir) | `wikidata ×4` ns/code (% mir) | `runs` prod (% mir) | `text` forced (% mir) | `records` forced (% mir) | `pattern` forced (% mir) |
|---|---:|---:|---:|---:|---:|---:|
| **`mirror_e_consume` total** | **8.158** (100%) | **7.960** (100%) | 6.085 (100%) | 6.426 (100%) | 6.396 (100%) | 100,364 / `0.113 ns/B` (100%) |
| **1. Bidirectional overhead** (`mir − fwd`) | **+3.472 (42.6%)** | **+3.309 (41.6%)** | +2.909 (47.8%) | +3.127 (48.7%) | +3.134 (49.0%) | +1.7% (0 bidi blocks) |
| — of which `u128` shift (`mir − nosh`) | **+1.982 (24.3%)** | **+1.922 (24.2%)** | +1.727 (28.4%) | +1.843 (28.7%) | +1.875 (29.3%) | +0.1% |
| — of which dual-cursor (`nosh − fwd`) | **+1.490 (18.3%)** | **+1.387 (17.4%)** | +1.182 (19.4%) | +1.283 (20.0%) | +1.259 (19.7%) | +1.6% |
| — `bidi_pre_shifted` saving (`mir − pre`) | **+2.148 (26.3%)** | **+2.130 (26.8%)** | +2.104 (34.6%) | +2.246 (35.0%) | +2.290 (35.8%) | −0.7% |
| **2. Per-code lookup+branch** (`no_copy`) | **+2.961 (36.3%)** | **+2.770 (34.8%)** | +1.853 (30.5%) | +2.054 (32.0%) | +2.042 (31.9%) | 0.1% |
| — of which `codes[]` scan | +0.499 (6.1%) | +0.465 (5.8%) | +0.278 (4.6%) | +0.505 (7.9%) | +0.488 (7.6%) | 0.0% |
| **3. Inline 16 B copy** (`fwd − lng_only`) | **+1.508 (18.5%)** | **+1.501 (18.9%)** | +1.645 (27.0%) | +1.655 (25.8%) | +1.625 (25.4%) | −2.7% |
| **4. Long-word copy** (`fwd − inl_only`) | **+0.633 (7.8%)** | **+0.732 (9.2%)** | −0.039 (−0.6%) | +0.007 (0.1%) | +0.011 (0.2%) | **97.8%** (`0.110 ns/B`) |
| *(OoO overlap: `2+3+4 − fwd`)* | *(−0.416, −5.1%)* | *(−0.353, −4.4%)* | *(−0.283, −4.7%)* | *(−0.416, −6.5%)* | *(−0.417, −6.5%)* | *(−3.1%)* |
| **Orthogonal: `len != LONG` mispred** (`mix − sort`) | **+0.585 (7.2%)** | **+0.629 (7.9%)** | +0.049 (0.8%) | −0.048 (−0.7%) | −0.072 (−1.1%) | noise (44 codes) |
| **Orthogonal: dict cache** (`fwd − dict_l1`) | **+0.614 (7.5%)** | **+0.586 (7.4%)** | +0.487 (8.0%) | +0.432 (6.7%) | +0.314 (4.9%) | −5.4% |
| **Orthogonal: `out` writes** (`fwd − out_l1`) | **+0.010 (0.1%)** | **−0.264 (−3.3%)** | −0.039 (−0.6%) | +0.067 (1.0%) | −0.042 (−0.6%) | **97.5%** |
| **`memcpy_out_floor` ratio** | **0.132×** | **0.132×** | 0.046× | 0.037× | 0.022× | **1.114×** |
| **`unrolled_4x_forward` ratio** | **0.542×** | **0.554×** | 0.389× | 0.408× | 0.412× | 1.069× |
| **`fused_16b_upper_bound` ratio** (code cut) | **0.217×** (−81.3%) | **0.215×** (−81.3%) | 0.116× (−76.7%) | 0.072× (−88.4%) | 0.035× (−93.7%) | 0.965× (0%) |

### 2.4 Validation of the branch-misprediction isolator

The same-loop `emit_chunk_pairs` isolator (`chunk_mixed_pairs` vs `chunk_sorted_pairs`, sorting inline before long within each 256-code L1d window while executing the exact same `#[inline(never)]` function) passes both zero-controls and hardware physics checks:
- **Zero controls:** on `runs` (`0%` long), `text` (`0.08%` long), and `records` (`0%` long), `chunk_mixed − chunk_sorted` is `+0.049`, `−0.048`, and `−0.072 ns/code` (zero within noise).
- **Wikidata signal:** on `wikidata` (`6.43%` long, `8.06%` 2-bit miss rate), `chunk_mixed − chunk_sorted` is **`+0.585 ns/code`** (`+0.629 ns/code` on `wikidata ×4`). At ~17 cycles (`7.4 ns`) per branch misprediction on 2.30 GHz Haswell/Broadwell, `0.0806 × 7.4 ns = 0.596 ns/code` — matching the measured delta within 2%.

---

## 3. What the dictionary already knows that spelling recomputes

Auditing `src/open/endpoint.rs` (`Elim`, `Dictionary`) against `src/open.rs` (`emit`, `emit_bidirectional`):

1. **Right-aligned back-side inline words (`24–27%` of `E_consume`):** `emit_bidirectional` runs on 232/233 Wikidata blocks and executes `u128::from_le_bytes(inline[cb]) << (8 * (16 - lb))` on ~50% of inline codes (`12.4M` shifts on `wikidata`, `49.7M` on `wikidata ×4`) — even though `Dictionary::insert` already knows `lb` once per distinct dictionary entry (~`1,083` entries/block, a 11,400× amplification!). Pre-storing `inline_back` recovers **26.3–26.8%** of `E_consume` (`0.732–0.737×` of mirror) with zero cache penalty. *(Caveat: in production `emit_bidirectional` with the pointer-chase walk still present, random `edges[]` load latency may overlap part of the shift ALU cost; measuring on full `emit_bidirectional` belongs in a build probe.)*
2. **Exact cycle length / no-overrun guarantee (`100%` of Open blocks):** `emit_bidirectional` checks `hi - lo >= INLINE` on every code and clamps `word.len().min(hi - lo)` on every long word. A6b's bands already sum to `n` across all `m` retained rows (`exact_cycle = 233/233` on Wikidata), so the walk only ever needs the tail clamp for the last ≤ 16 bytes (or short cycles, 0/233 on Wikidata).
3. **Code frequencies before emission:** every band in A6b carries `(width, code)`, so A6b already knows exact per-code frequency before wiring or emission. However, frequency-sorting `inline16`/`lens` (`freq_sorted_dict`) gains only **1.0–1.4%** over `forward_only` because `inline16 + lens` (`18 KiB` median) already fits inside 32 KiB L1d.
4. **Long-word `(start, end)` unpacking:** `long_word` unpacks two `u32`s from `[u8; 16]` on every long code. Storing a separate `direct_meta` array actually regressed (+12% vs `forward_only`) due to touching a third array; the existing packing inside the unused `inline16` slot is already better.
5. **Short-rule `15 B + 1 B len` arena format vs `inline16` + `lens` split:** `Elim::join` stores short rules as 15 payload bytes + 1 length byte in 16 bytes, which `Dictionary` splits into `inline16` and `lens`. Merging them back (`packed_15b_single_arr`) is **neutral** (`0.577×` vs `0.569–0.574×`), because `lens[]` (`~1.1 KiB`) is already 100% L1d-resident and keeping it separate avoids a 3-way branch for `len == 16`.
6. **Dead `memo_dedup_slack` in `spelling` (`32.3%` of `spelling` bytes in A6b):** `Dictionary::code` leaves `3.27 MB` (`~14 KiB/block`) of unreferenced bytes in `spelling` after deduplicating newly-expanded long words with memoized children. This inflates A6b's write traffic (`A6b-dict` is 29.6% of A6b), though `spelling` still fits in L2 during emission.

---

## 4. Implications for `rauzy-output` and `forward-emission`

This census directly resolves which representations can and cannot pay on `E_consume` (`298 ms` at 1w / `37 ms` at 8w on i9):

| candidate direction | verdict from census | quantitative basis |
|---|---|---|
| **1. Lowering `LIMIT` (16 → 8 or 12, or per-byte) in `rauzy-output`** | **HIGHEST-LEVER TARGET** | Directly attacks the **81.58% 1-byte literal storm** (mean streak 8.81 literals) by absorbing thin runs `5..16` into thick strips, cutting `m` (= both walk steps AND spelling codes!). `fused_16b_upper_bound` shows cutting code count by 81% cuts spelling by **78%** (`0.215×`, ≈230 ms at 1w on i9). Conversely, **raising `LIMIT` (24, 32, 64)** will create *more* 1-byte literals and worsen both walk and spelling unless A6b savings overwhelm it. |
| **2. Forward emission (`forward-emission`) + 4× unrolled inline fast path** | **HIGH-LEVER, IMMEDIATELY ACTIONABLE** | Switching from bidirectional to forward emission removes **41.6–42.6% of `E_consume`** (`0.569–0.574×`, ≈125 ms at 1w on i9) automatically — and adding 4× unrolled inline batching (`((l0\|l1\|l2\|l3) & 0x80) == 0`) reaches **0.542–0.554×** (`−45%` of `E_consume`, and `−59%` on literal-heavy `text`). Even without leaving bidirectional emission, pre-shifting `inline_back` recovers **26.3–26.8%** of `E_consume` (needs check with pointer-chase present). |
| **3. Fused consecutive short words / literal-streak collapsing** | **STRONG UPPER BOUND** | **79.03%** of all Wikidata codes sit in ≥ 2-literal streaks and **88.35%** of adjacent code pairs fit in ≤ 16 B. Any construction-time mechanism that fuses consecutive retained literals (or shrinks `m`) captures up to **78% of `E_consume`** (`0.215×`). |
| **4. SLP expansion during emission (instead of flat `spelling`)** | **DO NOT DO FOR EMISSION** | Long words are only **6.43% of codes** (`1.71M`), produce **81.29% of bytes**, are reused **26.6×** on average (only 0.45% one-shot bytes), sit entirely in L2 (`43.5 KiB` median), and already copy at near-`memcpy` speed (**7.8–9.2%** of `E_consume`, ~25 ms at 1w on i9). Replacing flat `memcpy` with SLP traversal during emission would add branches and lose; flat `spelling` should stay (only A6b's `32.3%` `memo_dedup_slack` is worth trimming). |
| **5. Raising `INLINE` to 32 B / single-array 15 B / frequency-sorted dict / direct meta** | **NO PAYOFF (all ≤ 1.4% or negative)** | `inline_32b` (`0.577–0.584×` vs `forward_only` `0.569–0.574×`, worse on `text`/`runs`), `packed_15b_single_arr` (`0.577×`), `freq_sorted_dict` (`0.562–0.575×`, −1%), `direct_meta_no_clamp` (`0.653–0.659×`, +12% slower). `inline16 + lens` is already L1d-resident (`18 KiB` median), so table layout micro-tweaks cannot move the needle. |
| **6. Eliminating `len != LONG` branch mispredictions alone** | **MODEST CEILING (~7–8% of `E_consume`, ≈23 ms at 1w)** | Real and cleanly measured (`+0.585..0.629 ns/code` = 7.2–7.9% of `E_consume`, 12.5–13.5% of `forward_only`), but small compared to code-count reduction and removing bidirectional shift overhead. |
