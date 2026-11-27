# `A6c` (`Reader::wire` / `Reader::wire_xor`) Dissection & Redesign Probe

*Date:* 27 November 2026
*Zone:* Smooth (`examples/a6c_wire_dissection.rs`, zero `src/` edits)
*Primary corpus:* `corpora/wikidata.bz2` (11,854,933 B compressed; 209,700,196 B decoded; 431 blocks → 233 Open-admitted blocks; `26,567,581` retained rows across `827,858` bands) + Open-admitted synthetic guard corpora (`text`, `records`, `runs`) + overflow/epoch/fixed-point stress tests.
*Machine (VM diagnostic only, C5):* 2-vCPU Haswell Xeon @ 2.30 GHz (32 KiB L1d / 256 KiB L2 / 45 MiB L3, AVX2), pinned to CPU 0 (`taskset -c 0`), steal-free Linux `/proc/thread-self/schedstat` CPU time (`se.sum_exec_runtime`) alongside `Instant` wall time.

---

## 1. Frozen Stop Test (C1a / C2 — frozen before any Phase 2 candidate timing)

- **Exactness & safety (mandatory):** every candidate must produce bit-for-bit identical `edges[0..m]` (and `predecessor` for bidirectional XOR wiring) against shipped `src/open/outq.rs` (`Reader::wire` / `Reader::wire_xor`) across all 233 Open blocks of `wikidata.bz2`, all Open blocks of the guard corpora (`text`, `records`, `runs`), and synthetic stress cases (overflow chunks `ovf > 0`, exact `EPOCH = 65536` boundaries, `m < 65536`, `m == 65536`, `mark` at `0 / 65535 / 65536 / m-1`, and self-loop `u == v`). Safe arms use zero `unsafe`; any runtime-dispatched AVX2 arm must document exact slice bounds invariants.
- **PASS (admit follow-on `idea` claim):** at least one exact candidate cuts isolated `A6c` steal-free thread CPU (7 interleaved repeats, alternating arm order) by **`>= 15.0%`** (disjoint 7-repeat `[min..max]` spreads and paired 95% CI upper bound `<= 0.850`) across **both** default baseline x86-64 **and** JCC-32B-aligned `-C target-cpu=native` (`-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`), in **both** warm and L3-cold regimes, on:
  - **Track F (post-`#1110` `forward-anchor-lanes`, `Reader::wire`)**, and/or
  - **Track B (shipped `bidi` default, `Reader::wire_xor`)**,
  with **`<= 1.02×`** peak worker scratch memory and **zero `> 2.0%`** disjoint regression on any Open guard corpus (`text`, `records`, `runs`). Any candidate modifying `OutQ16` chunk geometry must additionally cut joint `A6a + A6c` by `>= 8.0%`.
- **Otherwise (STOP):** record a scoped dead-end row in `frontier/dead-ends.tsv` and update `frontier/surface.tsv` row `C-A6c`.

---

## 2. Phase 1 Census, Disassembly & Sub-Cost Ablation (Completed Before Candidate Implementation)

### 2.1 Wikidata Open-Block `A6c` Census (233 Open Blocks)

- **Totals:** `m = 26,567,581` retained rows (`avg 114,024.0 / blk`), `827,858` bands (`avg 3,553.0 / blk`, **mean `span = 32.09`**), `57,410` slow-path calls (`6.93%` of bands), `65,033` `OutQ16` chunks (`avg 279.1 / blk`, `ovf = 0`, `cuts = 288`), `active_bytes = 85.9 / blk`, `wide = 0`, `bidi_eligible = 232 / 233`.
- **Cache footprint:** `edges[0..m]` is `p50 = 398.4 KiB`, `p90 = 659.8 KiB`, `max = 904.7 KiB` — **`232 / 233` blocks exceed the 256 KiB L2 cache** (L3-resident when warm).
- **`span` distribution:**
  - `span < 8`: **`40.60%` of bands (`336,087` bands)**, `4.13%` of rows -> skips 8-wide SIMD entirely.
  - `span in 9..32`: **`38.35%` of bands (`317,453` bands)**, `21.36%` of rows -> fails `>= 32` 4× AVX2 branch and hits variable 8-wide + scalar tail.
  - `span >= 33`: **`21.06%` of bands (`174,318` bands)**, **`74.50%` of rows** (`58.85%` in `span >= 65`).
- **Address locality:**
  - `dst` jump `|dst_i - (dst_{i-1} + span)|`: `<16` (same 64B cache line) `34.2%`, `16..63` `5.7%`, `64..255` `1.9%`, `256..4K` `8.4%`, `4K..64K` `34.3%`, `>=64K` `15.6%`.
  - `OutQ16` `v` step (`v_i - v_{i-1}`, monotonic within each byte `a`): **`dv == 1` is `46.2%`**, **`2..15` (same 64B cache line) is `27.4%`** (`73.6%` same cache line!), `16..63` `18.0%`, `>=64` `8.4%`.

### 2.2 Phase 1 Sub-Cost Ablation (5 repeats, CPU 0, `schedstat` CPU ms)

| Ablation Arm | Baseline WARM | Baseline L3-COLD | JCC-Native WARM | JCC-Native L3-COLD |
|---|---:|---:|---:|---:|
| `1_shipped_bidi_wire_xor` | `99.65 ms` (100%) | `112.24 ms` (100%) | `86.71 ms` (100%) | `100.01 ms` (100%) |
| `2_bidi_no_v_scatter` (drop `edges[v] ^= u`) | `51.95 ms` (52.1%) | `67.88 ms` (60.5%) | `48.06 ms` (55.4%) | `62.04 ms` (62.0%) |
| `3_fill_zero_only` (`edges.fill(0)` only) | `3.99 ms` (4.0%) | `15.98 ms` (14.2%) | `4.00 ms` (4.6%) | `15.93 ms` (15.9%) |
| `4_shipped_fwd_wire` (`#1110` `Reader::wire`) | `40.38 ms` (40.5%) | `43.88 ms` (39.1%) | `35.89 ms` (41.4%) | `36.13 ms` (36.1%) |
| `5_fwd_seq_write_no_rfo` (sequential `edges[k..]`) | `35.55 ms` (88.0% of fwd) | `39.96 ms` (91.1% of fwd) | `33.51 ms` (93.4% of fwd) | `38.91 ms` (107.7% of fwd) |
| `6_outq_read_only` (zero `edges` writes) | `40.69 ms` | `39.91 ms` | `41.59 ms` | `38.42 ms` |

### 2.3 Disassembly Findings (`objdump -d`)

1. **`shipped_fwd_wire` (`Reader::wire`, `35.9–43.9 ms`):** scattered `dst` RFOs account for only **`~0–4.8 ms` (`0–12%`)**! The remaining **`~34–40 ms` (`88–100%`)** is pure CPU/branch/spill overhead:
   - Because `#[inline(never)] wire_slow(&mut self, ...)` takes a stack pointer to `Reader`, LLVM spills the entire `Reader` struct to `0x10(%rsp)..0x48(%rsp)` and executes **8 stack loads + 2 stack stores + 5 checks** (`at+span <= 512`, `read+span <= cut`, `id < in_out`, `out` slice overflow, `out` bounds) on every one of the `827,858` bands.
   - `78.95%` of bands (`span < 32`) fail the 4× AVX2 dispatch (`cmp $32, %edx`) and `40.60%` (`span < 8`) fail the 1× AVX2 / 2× SSE2 dispatch, causing unpredictable branch cascades (`>= 8`, `>= 32`, `% 32`, `& 24`, `% 8`, scalar tail exit) on every band.
   - On baseline x86-64 (shipped wheels/crates), `Reader::wire` has no runtime AVX2 dispatch (`movq` + `punpcklwd` SSE2 only).
2. **`shipped_bidi_wire` (`Reader::wire_xor`, `86.7–112.2 ms`):**
   - `edges.fill(0)` costs `4.0 ms` warm / `15.9–16.0 ms` L3-cold (`14–16%`).
   - The 18-instruction scalar inner loop (`0x19dc0..0x19dfe`) executes **26.57M times** with **two per-element bounds checks** (`edges[u]` and `edges[v]`), an unvectorizable aliased `edges[u] ^= pack | v`, and `if v == mark` (3 instructions × 26.57M rows for 1 hit/block).

---

## 3. Phase 2 Candidate Design & Results

### 3.1 Candidate Mechanisms

All candidates were verified **bit-identical (`4,336 / 4,336` checks per build; `13,008 / 13,008` total across all 3 builds)** against shipped `src/open/outq.rs` on all 233 Wikidata Open blocks, all 18 guard Open blocks (`guard_text`, `guard_records`, `guard_runs`), and synthetic stress cases (`m = 131, 65535, 65536, 70000`, `out_bytes = 1024/4096` forcing heavy `ovf > 0`, `mark` at `0/1/65535/65536/m-1`, and self-loop `u == v`).

- **Track F (post-`#1110` forward `Reader::wire`, zero `edges.fill(0)`):**
  - `F0_shipped_q512`: exact shipped `Reader::wire` on `OutQ16<512, 512>`.
  - `F1_reg_cursor_safe` (100% safe): register-resident chunk cursor (`cur: &[u8]`) avoiding `&mut self` stack spills, keeping LLVM's variable-length loop.
  - `F2_reg_overlap_safe` (100% safe): register-resident chunk cursor + branch-minimized fixed-width `8/4/2/1` overlapping tail (eliminates LLVM's `>= 32` / `% 32` / `% 8` / scalar-tail branch cascade across the `78.95%` of bands with `span < 32`).
  - `F3a_avx2_8w` / `F3b_avx2_16w`: runtime-dispatched `#[target_feature(enable = "avx2")]` `vpmovzxwd + vpor + vmovdqu` 8-wide (`8/4/2/1`) and 16-wide (`16/8/4/2/1`) overlapping tail with register-resident cursor.
  - `F4_avx2_16w_prefetch`: `F3b_avx2_16w` + `_mm_prefetch` (`_MM_HINT_T0`) 4 bands ahead on `dst`.
  - `F5_reg_q1008_safe` (100% safe) / `F6_avx2_16w_q1008`: `F2` / `F3b` on `OutQ16<1008, 1024>` (halving chunk transitions from `65,033` to `~33k`, `ovf = 0`, matching `#1132` `mtf-fold-l1-batch-redesign`'s hoisted `OutQ16` chunking).
- **Track B (shipped bidirectional `Reader::wire_xor`):**
  - `B0_shipped_q512`: exact shipped `edges.fill(0)` + `Reader::wire_xor` on `OutQ16<512, 512>`.
  - `B1_reg_zerobc_nomark` (100% safe): register-resident chunk cursor + `padded_len(m)` epoch slice (`&mut [u32; 65536]`) eliminating bounds checks on `edges[v]` + hoisted `mark_byte` (recorded in Pass 1) and `mark_epoch` eliminating `if v == mark` on `99.6%` of rows.
  - `B1b_cell_zerobc_4x` (100% safe): wraps `&mut edges[..padded_len(m)]` with `std::cell::Cell::from_mut(...).as_slice_of_cells()`, allowing `dst_cells: &[Cell<u32>]` and `ep: &[Cell<u32>; 65536]` to coexist simultaneously in 100% safe Rust — eliminating **both** per-element bounds checks (`edges[u]` AND `edges[v]`), hoisting `ep` per epoch and `mark_byte`/`mark_epoch`, and unrolling 4× in a single `OutQ16` pass.
  - `B2_split_band_safe` / `B4_split_band_avx2`: split each band into vectorized `edges[u] ^= pack | v` pass + scalar `edges[v] ^= u` pass (testing whether vectorizing the `u` half pays).
  - `B3_2pass_nofill_safe` / `B5_2pass_nofill_avx2`: two full passes over `OutQ16` — Pass 1 overwrites `edges[u] = pack | v` (eliminating `edges.fill(0)` entirely!), Pass 2 XORs `edges[v] ^= u`.
  - `B6_cell_q1008_safe` (100% safe) / `B7_2pass_q1008_avx2`: `B1b` / `B5` on `OutQ16<1008, 1024>`.

### 3.2 Track B — Bidirectional `Reader::wire_xor` (`wikidata x4`, 7 interleaved repeats, CPU 0)

| Arm (`wikidata x4`, 7 reps) | Base WARM (ms / ratio / 95% CI) | Base L3-COLD (ms / ratio / 95% CI) | JCC-32B WARM (ms / ratio / 95% CI) | JCC-32B L3-COLD (ms / ratio / 95% CI) |
|---|---|---|---|---|
| `B0_shipped_q512` | `399.05` (`1.0000×`) | `472.54` (`1.0000×`) | `351.25` (`1.0000×`) | `413.65` (`1.0000×`) |
| `B1_reg_zerobc_nomark` (safe) | `290.96` (`-27.40%`, `[0.7136, 0.7387]`, F) | `372.92` (`-22.17%`, `[0.7627, 0.7942]`, F) | `291.37` (`-18.17%`, `[0.7991, 0.8379]`, F) | `356.62` (`-14.26%`, `[0.8463, 0.8687]`, F) |
| **`B1b_cell_zerobc_4x` (safe, Q512)** | **`292.32` (`-26.91%`, `[0.7211, 0.7408]`, F)** | **`361.17` (`-23.55%`, `[0.7475, 0.7818]`, F)** | **`279.00` (`-20.19%`, `[0.7732, 0.8238]`, F)** | **`343.59` (`-17.53%`, `[0.8138, 0.8357]`, F)** |
| `B2_split_band_safe` | `323.96` (`-18.91%`, `[0.7985, 0.8234]`, F) | `397.25` (`-15.60%`, `[0.8212, 0.8674]`, F) | `320.42` (`-9.39%`, `[0.8708, 0.9427]`, ovl) | `383.50` (`-7.69%`, `[0.9081, 0.9384]`, F) |
| `B3_2pass_nofill_safe` | `325.43` (`-18.60%`, `[0.8068, 0.8213]`, F) | `366.79` (`-22.92%`, `[0.7533, 0.7888]`, F) | `324.59` (`-9.17%`, `[0.8862, 0.9308]`, F) | `354.77` (`-14.04%`, `[0.8504, 0.8689]`, F) |
| `B4_split_band_avx2` | `321.90` (`-19.38%`, `[0.7963, 0.8163]`, F) | `394.78` (`-16.43%`, `[0.8176, 0.8541]`, F) | `323.11` (`-9.05%`, `[0.8896, 0.9297]`, F) | `388.71` (`-5.99%`, `[0.9185, 0.9622]`, F) |
| `B5_2pass_nofill_avx2` | `315.89` (`-19.94%`, `[0.7752, 0.8269]`, F) | `348.97` (`-25.22%`, `[0.7262, 0.7700]`, F) | `321.99` (`-7.47%`, `[0.8888, 0.9633]`, F) | `354.21` (`-14.27%`, `[0.8454, 0.8693]`, F) |
| **`B6_cell_q1008_safe` (safe, Q1008)** | **`290.96` (`-27.38%`, `[0.7117, 0.7410]`, F)** | **`358.40` (`-23.83%`, `[0.7447, 0.7792]`, F)** | **`265.88` (`-24.38%`, `[0.7240, 0.7898]`, F)** | **`330.89` (`-20.15%`, `[0.7899, 0.8071]`, F)** |
| `B7_2pass_q1008_avx2` | `297.76` (`-24.00%`, `[0.7311, 0.7901]`, F) | `332.57` (`-28.62%`, `[0.6878, 0.7407]`, F) | `309.97` (`-12.40%`, `[0.8588, 0.8936]`, F) | `342.03` (`-17.04%`, `[0.8107, 0.8489]`, F) |

*Native (`-C target-cpu=native`, unaligned, `1×`, 7 reps) confirms even stronger wins:* `B1b_cell_zerobc_4x` `-28.11%` (`[0.6869, 0.7524]`, F) warm / `-24.83%` (`[0.7341, 0.7698]`, F) cold; `B6_cell_q1008_safe` **`-33.99%`** (`[0.6322, 0.6893]`, F) warm / **`-28.33%`** (`[0.7030, 0.7306]`, F) cold.

**Key disassembly finding (`objdump -d` on `B1b_cell_zerobc_4x`):** `Cell::from_mut(&mut edges[..padded_len(m)]).as_slice_of_cells()` compiles the 4× unrolled hot loop (`0x1f519..0x1f564`) with **zero per-element bounds checks on BOTH `edges[u]` and `edges[v]`** and **zero `if v == mark` branches** on `99.6%` of rows, in 100% safe Rust. Single-pass `Cell` (`B1b`/`B6`) beats both split-band (`B2`/`B4`) and 2-pass no-fill (`B3`/`B5`/`B7`) on warm and JCC-32B cold because reading `OutQ16` once (and touching each `edges` cache line once) outweighs eliminating `edges.fill(0)`.

### 3.3 Track F — Forward `Reader::wire` (`wikidata x4`, 7 interleaved repeats, CPU 0)

| Arm (`wikidata x4`, 7 reps) | Base WARM (ms / ratio / 95% CI) | Base L3-COLD (ms / ratio / 95% CI) | JCC-32B WARM (ms / ratio / 95% CI) | JCC-32B L3-COLD (ms / ratio / 95% CI) |
|---|---|---|---|---|
| `F0_shipped_q512` | `157.39` (`1.0000×`) | `174.33` (`1.0000×`) | `135.99` (`1.0000×`) | `148.01` (`1.0000×`) |
| `F1_reg_cursor_safe` (safe) | `159.25` (`+1.29%`, `[0.9823, 1.0444]`, ovl) | `174.67` (`+1.38%`, `[0.9759, 1.0531]`, ovl) | `136.00` (`-0.66%`, `[0.9744, 1.0128]`, ovl) | `149.68` (`+2.06%`, `[0.9996, 1.0421]`, ovl) |
| `F2_reg_overlap_safe` (safe, Q512) | `121.85` (`-21.53%`, `[0.7546, 0.8161]`, F) | `142.81` (`-17.59%`, `[0.7931, 0.8563]`, F) | `109.32` (`-19.99%`, `[0.7756, 0.8254]`, F) | `129.84` (`-12.72%`, `[0.8561, 0.8899]`, F) |
| `F3a_avx2_8w` | `117.85` (`-25.09%`, `[0.7377, 0.7607]`, F) | `135.71` (`-23.03%`, `[0.7437, 0.7966]`, F) | `115.96` (`-15.34%`, `[0.8274, 0.8662]`, F) | `132.33` (`-9.38%`, `[0.8810, 0.9321]`, F) |
| `F3b_avx2_16w` (Q512) | `111.83` (`-29.28%`, `[0.6955, 0.7191]`, F) | `131.62` (`-24.99%`, `[0.7361, 0.7644]`, F) | `108.08` (`-20.30%`, `[0.7853, 0.8090]`, F) | `128.14` (`-12.15%`, `[0.8383, 0.9207]`, F) |
| `F4_avx2_16w_prefetch` | `119.91` (`-23.20%`, `[0.7559, 0.7803]`, F) | `142.92` (`-18.25%`, `[0.7858, 0.8504]`, F) | `116.08` (`-15.87%`, `[0.8152, 0.8683]`, F) | `132.41` (`-10.28%`, `[0.8772, 0.9177]`, F) |
| **`F5_reg_q1008_safe` (safe, Q1008)** | **`112.02` (`-28.10%`, `[0.7003, 0.7383]`, F)** | **`135.78` (`-22.22%`, `[0.7555, 0.8009]`, F)** | **`104.09` (`-23.74%`, `[0.7373, 0.7887]`, F)** | **`123.79` (`-15.59%`, `[0.8267, 0.8619]`, F)** |
| **`F6_avx2_16w_q1008` (AVX2, Q1008)** | **`103.99` (`-33.24%`, `[0.6588, 0.6766]`, F)** | **`127.38` (`-27.37%`, `[0.7130, 0.7399]`, F)** | **`104.12` (`-23.43%`, `[0.7260, 0.8075]`, F)** | **`124.20` (`-15.53%`, `[0.8315, 0.8581]`, F)** |

*Native (`1×`, 7 reps):* `F2_reg_overlap_safe` `-27.48%` warm / `-19.00%` cold (`DISJ_FASTER`), `F5_reg_q1008_safe` `-29.31%` warm / `-18.56%` cold, `F6_avx2_16w_q1008` `-28.01%` warm / `-15.92%` cold (`DISJ_FASTER`).

**Key findings on Track F:**
1. **`F1_reg_cursor_safe` alone is null (`-0.66%..+2.06%`)** — register-hoisting the chunk cursor only unlocks gains when paired with **branch-minimized overlapping fixed-width `8/4/2/1` (`F2`) or `16/8/4/2/1` (`F3b`) SIMD** (eliminating LLVM's `>= 32` branch cascade on the `78.95%` short bands; `objdump -d` confirms pure 3-insn AVX2 `vpmovzxwd + vpor + vmovdqu` loop + 3-insn overlapping tail in 100% safe Rust on native).
2. **Runtime-dispatched AVX2 (`F3b`/`F6`) adds `-5.1..-7.7 pp` on baseline x86-64** (shipped wheels/crates) because LLVM otherwise only emits SSE2.
3. **Software `_mm_prefetch` (`F4`) regresses `+3..+6%` vs `F3b`** across every build and cache regime (prefetch instruction overhead outweighs the small `~4–15 ms` RFO floor).
4. **In JCC-32B L3-cold**, `F5_reg_q1008_safe` (`-15.59%`, 95% CI `[0.8267, 0.8619]`) and `F6_avx2_16w_q1008` (`-15.53%`, `[0.8315, 0.8581]`) clear the `>= 15%` point estimate and disjoint-spread clause, while their 95% CI upper bounds (`0.8619` / `0.8581`) sit `0.8–1.2 pp` above `0.850` strictly in that single cell because L3-cold streaming bandwidth + scattered `dst` RFOs form an irreducible `~15–20 ms` floor on forward `A6c`.

### 3.4 Joint `A6a + A6c` Gate (`>= 8.0%` required for `Q1008` arms, 7 interleaved repeats, WARM)

| Joint Arm (233 Open blocks) | Baseline x86-64 (`x4` run) | Native (unaligned) | JCC-32B Native |
|---|---|---|---|
| `J0_fwd_shipped_q512` (control) | `281.61 ms` (`1.0000×`) | `260.42 ms` (`1.0000×`) | `246.48 ms` (`1.0000×`) |
| `J1_fwd_reg_ovl_q512` | `271.88 ms` (`-3.51%`, `[0.9569, 0.9728]`) | `247.80 ms` (`-5.37%`, `[0.9005, 0.9945]`) | `240.22 ms` (`-3.04%`, `[0.9474, 0.9922]`) |
| `J2_fwd_avx2_q512` | `272.85 ms` (`-3.17%`, `[0.9447, 0.9924]`) | `250.24 ms` (`-5.13%`, `[0.9186, 0.9797]`) | `240.98 ms` (`-2.48%`, `[0.9555, 0.9953]`) |
| **`J3_fwd_avx2_q1008_h`** | **`253.02 ms` (`-10.02%`, `[0.8869, 0.9128]`)** | `247.85 ms` (`-6.57%`, `[0.9085, 0.9609]`) | **`226.08 ms` (`-8.52%`, `[0.8771, 0.9541]`)** |
| `J4_bidi_shipped_q512` (control) | `343.26 ms` (`1.0000×`) | `329.49 ms` (`1.0000×`) | `303.12 ms` (`1.0000×`) |
| `J5_bidi_cell4x_q512` | `317.97 ms` (`-7.40%`, `[0.9039, 0.9486]`) | `293.17 ms` (`-11.60%`, `[0.8506, 0.9187]`) | `282.74 ms` (`-8.04%`, `[0.8829, 0.9577]`) |
| **`J6_bidi_cell4x_q1008_h` (safe)** | **`301.34 ms` (`-12.79%`, `[0.8623, 0.8820]`)** | **`292.24 ms` (`-12.18%`, `[0.8466, 0.9110]`)** | **`263.54 ms` (`-13.51%`, `[0.8395, 0.8911]`)** |
| `J7_bidi_2pass_avx2_q1008_h` | `304.34 ms` (`-11.02%`, `[0.8624, 0.9181]`) | `296.25 ms` (`-11.04%`, `[0.8574, 0.9229]`) | `272.41 ms` (`-10.95%`, `[0.8657, 0.9160]`) |

Both `J6_bidi_cell4x_q1008_h` (`-12.18%..-13.51%`, up to `-14.82%` on `1×` baseline) and `J3_fwd_avx2_q1008_h` (`-8.52%..-10.02%` on baseline & JCC-32B) clear the frozen `>= 8.0%` joint `A6a + A6c` gate cleanly.

### 3.5 Guard Corpora (5 interleaved repeats, WARM, zero `> 2.0%` disjoint regressions)

| Corpus / Track | Key Safe & AVX2 Arms | Baseline x86-64 | Native | JCC-32B Native |
|---|---|---|---|---|
| `guard_text` (6 blks, `x55`) — Bidi | `B1b_cell_zerobc_4x` / `B6_cell_q1008_safe` | **`-38.07%` F / `-40.33%` F** | **`-49.62%` F / `-50.68%` F** | **`-29.96%` F / `-31.62%` F** |
| `guard_records` (6 blks, `x15`) — Bidi | `B1b_cell_zerobc_4x` / `B6_cell_q1008_safe` | **`-34.16%` F / `-35.85%` F** | **`-42.03%` F / `-45.35%` F** | `-21.36%` / **`-30.11%` F** |
| `guard_runs` (6 blks, `x39`) — Bidi | `B1b_cell_zerobc_4x` / `B6_cell_q1008_safe` | **`-27.58%` F / `-30.64%` F** | **`-31.33%` F / `-33.08%` F** | `-16.52%` / **`-21.02%` F** |
| `guard_runs` (6 blks, `x39`) — Fwd | `F2` / `F3b` / `F5` / `F6` | **`-33.5%..-41.9%` F** | **`-35.2%..-44.0%` F** | **`-24.2%..-32.0%` F** |
| `guard_records` (6 blks, `x15`) — Fwd | `F2` / `F3b` / `F5` / `F6` | `-21.6%..-48.1%` (4–8 ms tick) | `-15.0%..-35.7%` (4–8 ms tick) | `-17.8%..-41.2%` (4–8 ms tick) |
| `guard_text` (6 blks, `x55`) — Fwd | `F2` / `F3b` / `F5` / `F6` | `-32.8%..-36.3%` (`4 ms` tick floor) | `-11.3%..-53.2%` (`4 ms` tick floor) | `-2.6%..-3.5%` (`4 ms` tick floor) |

Every single winning candidate (`B1b`, `B6`, `F2`, `F3b`, `F5`, `F6`) is faster across all three guard corpora and all three builds, with **zero disjoint regressions**. Peak worker scratch memory is **`1.00×`** (`padded_len(m)` rounds `m` up by `<= 65,535` u32s only within `edges`'s existing `> 1 MiB` pre-allocated arena; `B1b`/`F2`/`F3b` add **0 bytes** of workspace and keep `OutQ16<512, 512>` unchanged — avoiding the `+524 KiB/worker` L3-cliff failure observed in `#1151` for fixed 64K `A6b` slabs).

---

## 4. Honest Verdict & Architectural Implications

### 4.1 Verdict Against Frozen Stop Test: **PASS**

- **Track B (shipped bidirectional `Reader::wire_xor`) — UNANIMOUS PASS (100% safe Rust, `#![forbid(unsafe_code)]`):**
  - **`B1b_cell_zerobc_4x`** (unchanged `OutQ16<512, 512>`, 0 extra bytes) clears **every single clause** of the frozen stop test: `-26.91%` (`[0.7211, 0.7408]`) base warm, `-23.55%` (`[0.7475, 0.7818]`) base cold, `-20.19%` (`[0.7732, 0.8238]`) JCC-32B warm, `-17.53%` (`[0.8138, 0.8357]`) JCC-32B cold — all `DISJ_FASTER` with 95% CI upper `<= 0.8357 < 0.850`, `4,336/4,336` exact per build, `1.00×` memory, and `-16.5%..-49.6%` faster on all 3 guard corpora.
  - **`B6_cell_q1008_safe`** (`OutQ16<1008, 1024>`, 100% safe Rust) strengthens this to `-27.38%` base warm, `-23.83%` base cold, `-24.38%` JCC-32B warm, `-20.15%` (`[0.7899, 0.8071]`) JCC-32B cold, and **`-12.18%..-13.51%`** joint `A6a + A6c` (clearing the `>= 8.0%` joint gate by `1.5–1.7×`).
- **Track F (post-`#1110` forward `Reader::wire`) — PASS on baseline warm/cold & JCC-32B warm; borderline (`0.8–1.2 pp` on CI upper) on JCC-32B L3-cold:**
  - **`F5_reg_q1008_safe`** (100% safe) and **`F6_avx2_16w_q1008`** (runtime-dispatched AVX2) cut baseline `Reader::wire` by `-28.10%..-33.24%` warm / `-22.22%..-27.37%` cold (95% CI upper `<= 0.801`), JCC-32B warm by `-23.43%..-23.74%` (`<= 0.808`), and JCC-32B L3-cold by **`-15.59%`** (`[0.8267, 0.8619]`) / **`-15.53%`** (`[0.8315, 0.8581]`), plus `-8.52%..-10.02%` joint `A6a + A6c` on baseline & JCC-32B.
  - Even on unchanged `OutQ16<512, 512>`, `F2_reg_overlap_safe` (`-17.6%..-21.5%` base, `-20.0%` JCC-32B warm, `-12.7%` JCC-32B cold) and `F3b_avx2_16w` (`-25.0%..-29.3%` base, `-20.3%` JCC-32B warm, `-12.2%` JCC-32B cold) recover substantial CPU in ~40 LOC with 0 extra memory.

### 4.2 Scoped Dead-Ends Recorded (Not Worth Pursuing)

1. **Register cursor alone (`F1_reg_cursor_safe`)** without overlapping branch-minimized fixed-width SIMD is null (`-0.66%..+2.06%`).
2. **Split-per-band XOR (`B2`/`B4`)** and **2-pass no-fill XOR (`B3`/`B5`/`B7`)** lose to single-pass `Cell` zero-bounds-check (`B1b`/`B6`) in warm and JCC-32B cold (`-6.0%..-17.0%` vs `-17.5%..-24.4%`): double-reading `OutQ16` costs more than `edges.fill(0)`.
3. **Software `_mm_prefetch` 4 bands ahead (`F4`)** adds `+3..+6%` overhead over `F3b_avx2_16w` across all builds and cache regimes.

### 4.3 Modeled i9-10900K Impact & Follow-On Stacking Plan (VM-derived, C5)

- **Shipped default (`bidi`, `A6c ≈ 161.4 ms` at 1W):** `B1b` / `B6` project **`~33–44 ms` 1W (`~2.5–3.3%` of 1,306 ms wall) / `~4.5–5.5 ms/w` 8W** in ~80 LOC of **100% safe Rust** with **zero extra worker RSS**.
- **Post-`#1110` `forward-anchor-lanes` (`A6c ≈ 55–65 ms` at 1W):** `F2`/`F5` (safe) or `F3b`/`F6` (runtime-dispatched AVX2 for baseline wheels/crates) project **`~9–20 ms` 1W / `~1.3–2.8 ms/w` 8W** in ~40–70 LOC with **zero extra worker RSS**.
- **Clean follow-on claim (`frontier/claims/a6c-wire-redesign.toml`, `status = "idea"`, P3):** bundles `B1b`/`B6` (`Cell::from_mut` dual-slice zero-bc 4× `wire_xor` + Pass-1 `mark_byte` hoisting) and `F2`/`F5` (register-cursor overlapping fixed-width `wire`, with optional `Kernels::avx2` runtime dispatch behind B5/Miri) in `src/open/outq.rs` + `src/open.rs`. Stays at `status = "idea"` until `mtf-fold-l1-batch-redesign` (`#1132`, currently holding `src/open/outq.rs`) merges, so it avoids hold contention and stacks directly onto `#1132`'s `Q496`/`Q1008` `OutQ16` layout.

