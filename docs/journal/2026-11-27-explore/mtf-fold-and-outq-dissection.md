# Dissecting `M` (`runs::front_avx2`), `OutQ16`, and the `#1010` `mtf-fold` Prototype

*Cold primary-evidence `explore` pass on `src/runs.rs`, `src/runs/fold.rs`, `src/open.rs`, `src/open/outq.rs`, and `corpora/wikidata.bz2` (233 Open-admitted blocks / 431 total blocks, 209,700,196 decoded bytes, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`). Smooth zone only: zero `src/` edits, `#![forbid(unsafe_code)]`, 100% exactness verified.*

**Reproducible probe:** [`examples/mtf_a6a_dissection.rs`](../../../examples/mtf_a6a_dissection.rs)
```bash
cargo run --release --example mtf_a6a_dissection -- corpora/wikidata.bz2 7
cargo run --release --features mtf-fold,trace --example mtf_a6a_dissection -- corpora/wikidata.bz2 7
RUSTFLAGS="-C target-cpu=native" cargo run --release --features mtf-fold,trace --example mtf_a6a_dissection -- corpora/wikidata.bz2 7
```

**Environment & method:** 2-vCPU Haswell Xeon @ 2.30 GHz KVM VM (AVX2, BMI1/BMI2, LZCNT, POPCNT, PCLMULQDQ; 32 KiB L1d / 256 KiB L2 / 45 MiB L3; no vPMU). All measurements use 7 interleaved repeats with alternating arm order, reporting both Linux `/proc/thread-self/schedstat` on-CPU time (`run_ns` via `mibz2::trace` level 2 or `thread_cpu_ns()`) and wall clock (`min / median / max`), plus paired per-repeat ratios. Disassemblies taken directly from the compiled release binaries (`objdump -d --demangle`).

**System context (`frontier/DECODER.md`):** On the i9-10900K (`wikidata ×4`, `character-stage1`), `M` (`A5 = 132.4 ms` at 1W, `19.3 ms/w` at 8W) and `C-A6a` (`200.3 ms` at 1W, `28.2 ms/w` at 8W) combine to **`332.7 ms`** (`25.5%` of the 1,306 ms 1W wall; `47.5 ms/w` = `25.7%` of 8W worker busy). At `k = 8`, worker gains are masked by serial scan (`S = 185.0 ms` vs `W/8 = 184.7 ms`) until scanner work (`scanner-coalesced-u8-decode`, `huff-bmi2-build`, `scanner-seam-window`) drops in tandem.

---

## Executive summary

1. **Premise check / falsification of `#1010`'s current `mtf-fold` implementation (before `mtf-fold-run` spends an i9 workstation run):**
   On `corpora/wikidata.bz2` (`233/233` Open blocks folded, `0` bailed, byte- and CRC-exact), `#1010` (`src/runs/fold.rs::front_avx2::<open::FoldSink>`) is **actively slower than shipped** on every worker-CPU and fused-stage metric at `k = 1` (unmasked by scanner) and `k = 2`:
   - **Baseline x86-64 (`k = 1`, 7 interleaved repeats, `trace` level 2):**
     - Worker on-CPU (`wkr_cpu_ms`): `690.70` → `701.74 ms`, paired **`1.0161× [1.0083..1.0275]` (+1.61% slower, 7/7 repeats slower)**.
     - Exact fused scope (`A5 + A6 + A7`): `659.53` → `666.93 ms`, paired **`1.0104× [1.0013..1.0519]` (+1.04% slower, 7/7 repeats slower)**: removing `A5` (`76.15 → 2.14 ms`, −74.0 ms) inflates `A6+A7` by **+81.0 ms** (`583.85 → 664.88 ms`), net **+7.0 ms slower**.
     - `k = 2`: `wkr_cpu_ms` `691.45` → `710.45 ms` (**+1.71% slower**), `wall_ms` `425.31` → `435.85 ms` (**+2.98% slower**).
   - **Native Haswell (`-C target-cpu=native`, matching how i9 benches compile):**
     - `k = 1`: `wkr_cpu_ms` `690.15` → `709.91 ms`, paired **`1.0433× [0.9603..1.0624]` (+4.33% slower)**.
     - `k = 2`: `wkr_cpu_ms` `724.09` → `746.21 ms`, paired **`1.0381× [0.9869..1.0528]` (+3.81% slower)**.
   - **Disassembly (`objdump -d`) exposes four exact root causes in `#1010`'s direct per-run `FoldSink` design** (not a refutation of MTF-into-A6a folding itself, which still eliminates `124.5 MiB` of L2-spilling `runs: Vec<u16>`, `CAP=127` splitting, `counts`/`per`, and `maximal()`):
     1. **Catastrophic register-spill explosion:** `runs::front_avx2` already occupies all 15 x86-64 GPRs (22-insn register-clean inner loop). Inlining `FoldSink + Pass::take + OutQ16::push` blows past x86-64's register file: on baseline x86-64, LLVM spills `sink` (`0x28(%rsp)`, reloaded **10 times** per literal!) plus 7 GPRs (`after`, `c`, `self.c`, `recs`, `t2`, `n`, `w`); on `-C target-cpu=native`, LLVM inlines `fold::front_avx2` into `worker` and **spills both MTF registers `f0` (`0x120(%rsp)`) and `f1` (`0x180(%rsp)`) to the stack on every single literal**, placing ~5-cycle store-to-load forwarding directly onto the serial MTF chain!
     2. **Memory-aliased `FoldSink` + dead `c == self.c` merge check:** Passing `&mut FoldSink` through `S: Sink` across `push_slow(&mut self)` forces per-literal reloads/stores of `sink.c`, `sink.width` (tested twice per literal), `sink.p`, `sink.pos`, `f.runs` (`incq`), `f.m`, `out.ptr/len`, and `in_out`, and checks `c == self.c && width != 0` on every literal even though adjacent same-byte runs occur **`0 / 15,141,602` times** on Wikidata (`seq_to_unseq` is injective).
     3. **`417,041` `vzeroupper` + 5-YMM constant reloads on `push_slow` (baseline x86-64):** `runs::fold::front_avx2` is `#[target_feature(enable = "avx2,...")]` while `outq::Queues::push_slow` is not, triggering `vzeroupper` + `ymm9`/`xmm10` spill + 5 RIP-relative YMM reloads (`ymm11..ymm15`) on every `push_slow` call (`1.67M` times on `wikidata ×4`).
     4. **Port-5 shuffle contention:** `front_avx2` issues **4 `vpshufb`** per literal on Port 5; inlining `OutQ16::push` under `avx2` adds `vpbroadcastw` (a **5th Port-5 shuffle uop**) + `vpaddw` + `vmovdqu %ymm0` into the same spilled loop.

2. **`OutQ16` tail-guard discovery (`84.3%` of all `push_slow` calls are pure end-of-chunk overrun guards):**
   Across all `14,386,594` `OutQ16::push` calls on `wikidata` (233 Open blocks), shipped `CHUNK = 512, BYTES = 1024` hits `#[inline(never)] push_slow` **`417,041` times (`2.90%`)** — of which **`351,722` (`84.3%`)** are NOT chunk allocations or epoch crossings, just ranks in `f = 497..511` where `f + w <= 512` fits in the chunk but `f + 16 > 512` would overrun into chunk `tail + 1` (~7.8 wasted slow calls at the tail of every chunk!).
   Separating usable capacity `C` from power-of-2 chunk stride `S` (`C = 496, S = 512` [`1024 B`, `tail << 10`] or `C = 1008, S = 1024` [`2048 B`, `tail << 11`]) guarantees `f + w <= C => f + 16 <= S` with **zero overrun**, **zero overflow chunks (`ovf_chunks = 0` on all 233 blocks)**, and cuts `push_slow` calls **6.2× (`417,041 → 67,001`)** or **10.3× (`417,041 → 40,597`)**. On pre-merged packed `u32` maximal runs (`pm_guard_496_512 / pm_shipped_512`), this cuts A6a **`−17.65%` (`0.8235× [0.7735..0.8868]`, non-overlapping)** on native Haswell (and `pm_guard_1008_1024` cuts **`−6.37%` (`0.9363× [0.9216..0.9977]`, non-overlapping)** on baseline x86-64).

3. **Three clean negative/null findings in `M` and `A6a` (closed off so future sessions do not retry them):**
   - **Branching on `s < 17` (`nn <= 15`) in `runs::front_avx2`:** `84.83%` of Wikidata literals stay inside `f0` (1 `vpshufb` vs 4), but the 2-bit branch predictor miss rate is **`17.47%`** (`2.64M` misses ≈ `+2.8` cycles/literal), offsetting the ~3-cycle Port-5 saving. By contrast, shipped `s < 33` covers `98.48%` at only `1.69%` 2-bit misses.
   - **Narrower `OutQ16::push` fast paths (`w <= 4` 8B `u64` SWAR or `w <= 8` 16B `iota8`):** Despite covering `92.6%` (`4×` less write amp) and `96.3%` (`2×` less write amp) of thin runs, adding the width branch is null-to-regressing (`1.000×..1.095×` baseline, `1.000×..1.016×` on `packed_max`) because `7.4%`/`3.7%` width-branch mispredictions outweigh write-bandwidth savings inside L1d/L2-warm `out`.
   - **Worker `out.resize(..., 0)` zero-fill tax:** Unlike the scanner's `40.2 MiB` runt-truncation `syms.resize(max_out, 0)` tax (`scanner-full-dissection`), `out` buffers only zero **`0.87 MiB`** sequentially and **`6.88 MiB`** in an 8-worker pool across all 233 Open blocks (`0.4%–3.4%` of 200 MiB output), identical for shipped and `mtf-fold`.

4. **Concrete rescue / follow-on opened at `status = "idea"`:** [`frontier/claims/mtf-fold-l1-batch-redesign.toml`](../../../frontier/claims/mtf-fold-l1-batch-redesign.toml) — decouple `front_avx2` from `Pass::take + OutQ16` via a **`1 KiB` L1d-resident `[u32; 256]` packed maximal-run batch (`(c << 24) | total`)** + **`OutQ16` power-of-2 guard zone (`496/512` or `1008/1024`)**, preserving `front_avx2`'s register-clean 22-insn loop (zero `f0`/`f1`/GPR spills, zero `vzeroupper` in `OutQ16`, zero `FoldSink` aliasing/dead merge check) while still eliminating `124.5 MiB` of L2-spilling `runs: Vec<u16>`, `CAP=127` splitting, `counts`/`per`, and `maximal()`.

---

## 1. Structural census of `M` (`runs::front_avx2`) on `wikidata` (233 Open blocks)

Total symbols across the 233 Open blocks: **`24,971,004`** (`9,829,169` run symbols [`39.36%`], `15,141,602` literals [`60.64%`], `233` EOBs). Per-block `syms` size: `p50 = 188.3 KiB`, `p90 = 297.5 KiB`, `max = 390.2 KiB` (`47.63 MiB` total; `lit + one` bitmaps `5.95 MiB`).

| MTF literal index `nn = s - 1` | count | share of literals | production handling |
|---|---:|---:|---|
| `nn == 1` (`s == 2`, swap top 2) | 2,200,777 | 14.53% | 4× `vpshufb` + 2× `vpor` |
| `nn <= 3` (`s <= 4`) | 4,502,564 | 29.74% | 4× `vpshufb` + 2× `vpor` |
| `nn <= 7` (`s <= 8`) | 7,941,414 | 52.45% | 4× `vpshufb` + 2× `vpor` |
| **`nn <= 15` (`s < 17`, `f0` only)** | **12,844,873** | **84.83%** | 4× `vpshufb` + 2× `vpor` (could be 1× `vpshufb`, **but 17.47% 2-bit miss!**) |
| `16 <= nn <= 31` (`f0 <-> f1`) | 2,066,928 | 13.65% | 4× `vpshufb` + 2× `vpor` |
| `32 <= nn <= 63` (`second()` row 0) | 195,001 | 1.29% | 1 AVX2 row blend |
| `nn >= 64` (`second()` rows 0–6) | 34,800 | 0.23% | 7 AVX2 row blends |

- **Branch predictability:**
  - `s < 17` vs `s >= 17`: transitions `0→0 = 11,147,122`, `0→1 = 1,697,754`, `1→0 = 1,697,751`, `1→1 = 598,975`; 2-bit saturating predictor misses **`2,644,498` (`17.47%`)**.
  - `s < 33` vs `s >= 33`: transitions `0→0 = 14,717,709`, `0→1 = 194,093`, `1→0 = 194,092`, `1→1 = 35,708`; 2-bit misses **`255,860` (`1.69%`)**.
- **Literal-to-literal symbol spans:** `span = 1` (0 run syms) `71.67%`, `span = 2` `13.82%`, `span = 3` `5.27%`, `span = 4..=7` `8.10%`, `span = 8..=22` `1.14%`.
- **64-symbol `lit` words:** `390,267 / 390,289` (`99.99%`) non-empty, mean `38.80` literals/word (`2.58%` word-boundary crossing rate).
- **Adjacent same-byte maximal runs:** **`0 / 15,141,602`** (`FoldSink::run`'s per-literal `c == self.c && self.width != 0` merge check is 100% dead on every valid bzip2 block).
- **Output buffer `resize(..., 0)` zero-fill:** RLE1 shrinks `187/233` (`80.3%`) blocks, keeps `1` (`0.4%`) equal, and expands `45` (`19.3%`), totaling `209,697,662 B` BWT → `209,691,304 B` decoded. Reusing buffers zeroes only **`0.87 MiB`** (1-buffer sequential) and **`6.88 MiB`** (8-buffer pool), identical between shipped `out.resize(n + 16, 0)` and `mtf-fold` `buf.resize(bound + 16, 0)`.

---

## 2. Real `#1010` (`mtf-fold`) end-to-end & level-2 trace dissection on `wikidata`

All runs verified `209,700,196` bytes, CRC `0xa78fe5d8`, `fold=OFF` (`folded=0, bailed=0`), `fold=ON` (`folded=233, bailed=0, not_open=0`). 7 interleaved repeats (alternating `OFF/ON` and `ON/OFF`), `StreamDecoder::workers_with::<OpenCells>(k, 4*k, Kernels::detect())` with zero-copy `read_block`.

### 2.1 Baseline x86-64 (`--features mtf-fold,trace`, `k = 1` and `k = 2`)

| workers | metric | `fold = OFF` med `[min..max]` | `fold = ON` med `[min..max]` | paired `ON / OFF` `[min..max]` | delta |
|---:|---|---:|---:|---:|---:|
| **1** | **`wkr_cpu_ms`** (`schedstat`) | **690.70** `[681.19..694.97]` | **701.74** `[691.18..712.14]` | **`1.0161×`** `[1.0083..1.0275]` | **+1.61%** (7/7 slower) |
| **1** | `wkr_wall_ms` | 697.03 `[684.56..707.83]` | 704.44 `[692.93..737.80]` | **`1.0099×`** `[1.0001..1.0585]` | **+0.99%** (7/7 slower) |
| **1** | **`A5 + A6 + A7`** | **659.53** `[647.59..669.63]` | **666.93** `[655.77..693.80]` | **`1.0104×`** `[1.0013..1.0519]` | **+1.04%** (7/7 slower) |
| 1 | `A5_ms` | 76.15 `[74.75..77.45]` | 2.14 `[2.02..2.62]` | `0.0279×` | folded into `Invert` |
| 1 | `A6 + A7` | 583.85 `[571.45..592.85]` | 664.88 `[653.70..691.46]` | `1.1404×` | +81.0 ms (vs −74.0 ms A5) |
| 1 | `A9_ms` | 37.49 `[36.96..38.20]` | 37.71 `[37.16..44.00]` | `1.0102×` | neutral |
| 1 | `wall_ms` | 715.51 `[703.68..726.08]` | 724.78 `[711.60..779.94]` | `1.0133×` `[0.9947..1.0902]` | +1.33% |
| **2** | **`wkr_cpu_ms`** (`schedstat`) | **691.45** `[676.81..728.64]` | **710.45** `[685.44..727.55]` | **`1.0171×`** `[0.9913..1.0317]` | **+1.71%** |
| 2 | `wall_ms` | 425.31 `[411.87..469.97]` | 435.85 `[424.15..510.63]` | `1.0298×` `[0.9967..1.0865]` | +2.98% |

*(Without `trace`, an earlier 7-repeat baseline run gave the same sign: `k = 1` `wall_ms` `786.15` vs `806.14 ms`, `ON/OFF = 1.0036×`.)*

### 2.2 Native Haswell (`-C target-cpu=native --features mtf-fold,trace`)

| workers | metric | `fold = OFF` med `[min..max]` | `fold = ON` med `[min..max]` | paired `ON / OFF` `[min..max]` | delta |
|---:|---|---:|---:|---:|---:|
| **1** | **`wkr_cpu_ms`** (`schedstat`) | **690.15** `[660.44..729.25]` | **709.91** `[695.30..725.59]` | **`1.0433×`** `[0.9603..1.0624]` | **+4.33%** |
| **2** | **`wkr_cpu_ms`** (`schedstat`) | **724.09** `[703.10..737.06]` | **746.21** `[725.15..765.20]` | **`1.0381×`** `[0.9869..1.0528]` | **+3.81%** |

*(Host VM CPU contention inflated `wall_ms` during the native run, while `schedstat` `wkr_cpu_ms` stayed tight near `~690–746 ms`.)*

### 2.3 Disassembly root-cause comparison (`objdump -d`)

- **Shipped `mibz2::runs::front_avx2`** (`0x66250..0x662ce` baseline; `0x5d130..0x5d1a7` native):
  - **22 instructions** per literal; `f0` and `f1` live permanently in XMM registers (`%xmm10`, `%xmm2`), zero stack spills on the hot path.
- **`#1010` `mibz2::runs::fold::front_avx2::<open::FoldSink>` (baseline x86-64, `0x75640..0x76744`):**
  - `sink` pointer spilled at `0x28(%rsp)` and reloaded **10 times** in a single hot thin-run iteration (`76363, 7639f, 7658a, 765a9, 765fd, 76634, 76651, 76679, 766d5, 766eb`).
  - 7 live GPRs spilled/reloaded per literal: `after` (`0x60(%rsp)`), `c` (`0x90(%rsp)`), `self.c` (`0x48(%rsp)`), `recs` (`0x30(%rsp)`), `t2` (`0x58(%rsp)`), `n` (`0x40(%rsp)`), `w` (`0x38(%rsp)`).
  - `FoldSink` fields (`sink.c`, `sink.width` tested twice, `sink.p`, `sink.pos`, `f.runs`, `f.m`, `out.ptr/len`, `in_out`) reloaded/stored via `sink` pointer every literal.
  - `0x7673c`: `vzeroupper` + `vmovdqa %ymm9, 0xc0(%rsp)` + `vmovdqa %xmm10, 0x70(%rsp)` + 5 RIP-relative YMM constant reloads (`ymm11..ymm15`) on every `push_slow` call (`417,041` calls on `wikidata`).
  - `0x76661`: `vpbroadcastw %xmm0, %ymm0` (Port 5) runs in the same loop as `front_avx2`'s 4× `vpshufb` (Port 5).
- **`#1010` inlined into `worker` (native `-C target-cpu=native`, `0x746c0..0x746fd`):**
  - LLVM spills **both MTF registers** (`vmovdqa 0x120(%rsp), %xmm2` for `f0`; `vmovdqa 0x180(%rsp), %ymm3` + `vmovdqa %ymm1, 0x180(%rsp)` for `f1`) on **every single literal**, adding store-to-load forwarding directly onto the serial MTF chain.

---

## 3. `OutQ16` `push_slow` & guard-zone census and ablations

### 3.1 Exact `push_slow` breakdown (233 Open blocks, `14,386,594` thin pushes)

| layout (`C` usable / `S` stride) | total `push_slow` (% of pushes) | `first_chunk` | **`tail_guard`** (`f+w<=C`, `f+16>C`) | `chunk_cross` | `epoch_cross` | `chunks` | `ovf_chunks` |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`shipped_512_512`** (`1024 B`) | **417,041 (2.90%)** | 20,024 | **351,722 (84.3%)** | 45,007 | 288 | 65,033 | **0** |
| **`guard_496_512`** (`1024 B`, `<<10`) | **67,001 (0.47%)** | 20,024 | **0** | 46,689 | 288 | 66,713 | **0** |
| `plain_1024_1024` (`2048 B`, `<<11`) | 197,729 (1.37%) | 20,024 | 157,539 (79.7%) | 19,878 | 288 | 39,903 | **0** |
| **`guard_1008_1024`** (`2048 B`, `<<11`) | **40,597 (0.28%)** | 20,024 | **0** | 20,285 | 288 | 40,310 | **0** |
| `guard_752_768` (`1536 B`) | 49,341 (0.34%) | 20,024 | 0 | 29,029 | 288 | 49,053 | **0** |
| `guard_240_256` (`512 B`) | 123,408 (0.86%) | 20,024 | 0 | 103,096 | 288 | 123,121 | **0** |

Because `1 <= w <= 16` on the thin fast path, `f + w <= C` with `S = C + 16` guarantees `f + 16 <= C - w + 16 <= C + 15 < S`. All 233 Open blocks fit completely inside `out` (`ovf_chunks = 0` for every layout up to `2048 B` chunks).

### 3.2 Interleaved A6a timings (7 repeats, 100% bit-for-bit A6a state-exact verified)

#### Width ablations (baseline x86-64, `schedstat` CPU ms, vs `prod_a6a = 127.68 ms`)

| arm | cpu median `[min..max]` | paired ratio vs `prod_a6a` | verdict |
|---|---:|---:|---|
| `prod_a6a` (shipped `512/512`, 32B iota16) | 127.68 ms `[123.69..131.84]` | `1.0000×` | baseline |
| `a6a_no_outq` | 49.44 ms `[43.94..52.00]` | `0.3870×` (`−61.3%`) | OutQ16 = 61.3% of A6a |
| `a6a_u64x2_iota8` (`w<=8` 16B → 32B) | 127.92 ms `[123.92..130.86]` | `0.9998× [0.9707..1.0360]` | null |
| `a6a_arr_iota8` (`w<=8` `[u16;8]` → 32B) | 135.99 ms `[130.37..139.96]` | `1.0535× [1.0321..1.1315]` | regresses |
| `a6a_u64_iota4` (`w<=4` 8B SWAR → 32B) | 138.71 ms `[133.22..142.37]` | `1.0946× [1.0417..1.1284]` | regresses (7.4% branch miss) |
| `a6a_chunk1024` (same binary as above) | 111.24 ms `[107.39..111.44]` | `0.8666× [0.8448..0.8990]` | −13.34% (alignment-sensitive; see below) |
| `a6a_chunk256` | 139.69 ms `[131.96..141.36]` | `1.0994× [1.0335..1.1281]` | regresses (+9.94%) |

#### Guard-zone & packed-maximal ablations (both baseline x86-64 and `-C target-cpu=native`, `schedstat` CPU ms)

| arm | baseline CPU med (`paired vs base`) | native CPU med (`paired vs base`) |
|---|---:|---:|
| `prod_a5_avx2` (`runs::front`) | 71.95 ms `[71.10..72.00]` | 65.43 ms `[62.21..67.95]` |
| `shipped_512_512` (A6a) | 103.28 ms `[99.83..105.62]` (`1.000×`) | 109.46 ms `[105.17..132.53]` (`1.000×`) |
| `guard_496_512` (A6a) | 107.20 ms (`1.0176× [0.9955..1.0810]`) | **102.40 ms** (**`0.9393× [0.7591..0.9575]`, −6.07%**) |
| `plain_1024_1024` (A6a) | 119.86 ms (`1.1686× [1.1441..1.1995]`)* | **100.88 ms** (**`0.9025× [0.7612..0.9916]`, −9.75%**) |
| `guard_1008_1024` (A6a) | 113.67 ms (`1.0965× [1.0602..1.1613]`)* | **101.98 ms** (**`0.9175× [0.7777..0.9805]`, −8.25%**) |
| `pm_shipped_512` (packed `u32` max) | 111.90 ms (`1.000× pm`) | 99.74 ms (`1.000× pm`) |
| **`pm_guard_496_512`** | **110.29 ms** (`0.9672× [0.8994..1.0088]`) | **82.14 ms** (**`0.8235× [0.7735..0.8868]`, −17.65%**) |
| **`pm_guard_1008_1024`** | **107.94 ms** (**`0.9363× [0.9216..0.9977]`, −6.37%**) | 102.34 ms (`1.0297× [0.9702..1.2591]`) |

*\*Note on baseline code-alignment sensitivity:* Across two builds with different surrounding functions, `plain_1024_1024 / shipped_512_512` swung from `0.8666×` (Run 2) to `1.1686×` (Run 3) on baseline Haswell (classic 32B JCC/LSD loop-alignment effect already documented in `examples/mtf_two_level.rs:14`). Standalone `guard_496_512` alone is therefore **not** promoted as a standalone claim, whereas its **6.2×–10.3× reduction in `push_slow` calls** (`417k → 67k / 40.6k`) directly removes the `vzeroupper` / slow-call tax and stacks naturally inside the L1d-batched fold redesign (`pm_guard_496_512`: `−17.65%` native).

---

## 4. Implications & recommended rescue (`mtf-fold-l1-batch-redesign`)

1. **For `mtf-fold-run` (currently held):** Do not be surprised if `#1010` fails K1 (`F / none <= 0.975`) or even regresses on the i9-10900K (Comet Lake shares Haswell's 15-GPR file, 1-Port-5 shuffle pipe, and store-to-load forwarding latency when `f0`/`f1` spill in `-C target-cpu=native`!). Per `CLAUDE.md` ("Optimise and rescue before abandoning a promising direction"), `#1010`'s regression is a **register-allocation and `Sink`-abstraction pathology**, not a refutation of MTF-into-A6a folding.
2. **Proposed redesign (`frontier/claims/mtf-fold-l1-batch-redesign.toml`, `status = "idea"`):**
   - Keep `runs::fold::front_avx2` as a tight, register-clean, `#[inline(never)]` AVX2 loop (zero `f0`/`f1`/GPR spills), dropping `CAP=127` splitting and unused `cnt[256]` RMW, checking `seq_to_unseq` injectivity once before the loop (eliminating `FoldSink`'s dead `c == self.c` merge check), and writing packed `u32` maximal runs `(c << 24) | total` sequentially into a **`1 KiB` L1d-resident `[u32; 256]` stack batch**.
   - Every 256 runs (~59k flushes vs 15.14M per-run calls), call a separate `#[inline(never)]` `flush_batch(&[u32], &mut Pass)` with all 15 GPRs dedicated to `Pass::take + OutQ16` and `OutQ16`'s **`496/512` (or `1008/1024`) power-of-2 guard zone** (eliminating `351,722` tail-guard `push_slow` calls).
   - Gate locally first on `corpora/wikidata.bz2` (`k = 1` `wkr_cpu_ms` and `A5+A6+A7` on both baseline x86-64 and `-C target-cpu=native` with `objdump -d` verification of zero `f0`/`f1`/GPR spills in Pass 2) before spending an i9 workstation run.
