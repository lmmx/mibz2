# Dissecting `Segmented` on off-Wikidata corpora: sequential LF column build, invariant-checked run fill, and vectorized reverse assembly (PASS)

*Smooth-zone probe [`segmented-fallback-dissection`](../../../frontier/claims/segmented-fallback-dissection.toml), 27 November 2026 (zero `src/` edits; probe code in [`examples/segmented_dissection_probe.rs`](../../../examples/segmented_dissection_probe.rs)).*

---

## 1. Motivation and scope

[`route-census.md`](route-census.md) (#1023) established that off Wikidata, `OpenCells` takes only 17 of 107 full blocks (16%); the remaining **84%** across prose, source code, JSON, ELF, DNA, and random execute the `Segmented` fallback (`src/bwt.rs` + `src/runs.rs::fill`). Under [`PERFORMANCE-POLICY.md`](../../team/PERFORMANCE-POLICY.md), `text` (ordinary prose/text) is a **primary corpus** alongside `wikidata`.

This probe dissects `Segmented`'s exact worker-side phase breakdown across six deterministic non-Wikidata corpora (including `ab gen text`) and tests concrete, 100% byte-exact candidate levers against shipped `Segmented`, measured over 7 interleaved repeats on **both** baseline x86-64 and `-C target-cpu=native`.

**Host (C5 diagnostic note):** 2-vCPU Intel Xeon @ 2.30 GHz VM (L1d 32 KiB, L2 256 KiB, L3 45 MiB, AVX2 + PCLMULQDQ, no cgroup CPU quota; shared hypervisor). Both wall clock (`Instant::now()`) and steal-free thread CPU (`CLOCK_THREAD_CPUTIME_ID`) are reported side by side.

---

## 2. Deterministic corpora and route census

Six `bzip2 -9` corpora built locally (not checked in):

| corpus | raw bytes | bz2 bytes | bz2 SHA-256 (12) | full blocks | Open | `RunsFallback` | `Column` | `Segmented` share |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| `text` (`ab gen` scale=8) | 25,489,807 | 4,439,341 | `c88849155f92` | 25 | 0 | 25 | 0 | **100.0%** |
| `prose` (`docs/` + `/usr/share/doc`) | 10,013,194 | 2,589,055 | `61c8062d8a61` | 11 | 0 | 3 | 8 | **100.0%** |
| `pysrc` (Python 3.12 stdlib) | 10,000,065 | 1,792,240 | `01e8dbb30b40` | 10 | 1 | 8 | 1 | **90.0%** |
| `rustsrc` (unique repo `.rs`) | 4,988,773 | 862,622 | `e3a487dd093e` | 5 | 0 | 5 | 0 | **100.0%** |
| `json` (repo `.json`/`.jsonl`) | 13,197,838 | 1,648,072 | `cb89ad42af28` | 15 | 7 | 6 | 2 | **53.3%** |
| `elf` (`/usr/bin` ELF binaries) | 18,485,896 | 6,868,127 | `5b6ae8c87af4` | 20 | 0 | 1 | 19 | **100.0%** |
| **Total** | **82,175,573** | **18,199,477** | — | **86** | **8** | **48** | **30** | **90.7% (78/86)** |

On the primary `ab gen text` corpus, **25 of 25 full blocks (100%)** run `Segmented` (`RunsFallback`), never `Open`. Across `prose/text + source` (`text + prose + pysrc + rustsrc`), **50 of 51 full blocks (98.0%)** run `Segmented` (41 `RunsFallback`, 9 `Column`).

---

## 3. Full worker phase breakdown on `Segmented` blocks (best of 3 thread-CPU ms)

| corpus | target | `front` (MTF/runs) | `build` (T-vec) | `walk + asm` | `RLE1 + CRC` | inverse share | `build / inv` |
|---|---|---:|---:|---:|---:|---:|---:|
| `text` | base / nat | 25.31 / 25.23 | 32.26 / 32.81 | 57.51 / 59.06 | 24.41 / 24.48 | 64.4% / 64.9% | **35.9% / 35.7%** |
| `prose` | base / nat | 39.62 / 38.88 | 20.70 / 20.19 | 26.92 / 26.72 | 3.52 / 3.49 | 52.5% / 52.5% | **43.5% / 43.0%** |
| `pysrc` | base / nat | 13.01 / 13.00 | 13.30 / 13.21 | 21.11 / 21.30 | 8.45 / 8.37 | 61.6% / 61.8% | **38.6% / 38.3%** |
| `rustsrc` | base / nat | 4.93 / 4.89 | 6.31 / 6.34 | 11.60 / 11.69 | 4.99 / 4.87 | 64.4% / 64.9% | **35.3% / 35.2%** |
| `json` | base / nat | 13.79 / 13.51 | 10.86 / 10.80 | 19.93 / 20.11 | 2.46 / 2.56 | 65.5% / 65.8% | **35.3% / 34.9%** |
| `elf` | base / nat | 110.12 / 109.05 | 40.89 / 40.56 | 47.17 / 47.66 | 9.99 / 9.87 | 42.3% / 42.6% | **46.4% / 46.0%** |

**Key structural finding:** Inverse BWT accounts for **52.5–65.8%** of worker time across text/prose/source/JSON (42.3% on ELF), and **T-vector construction (`build`) alone is 35–46% of inverse-BWT time** — far larger than on Wikidata, for two distinct reasons:
1. **Column route (`src/bwt.rs:440`, `build_one_pass`):** constructs the forward `Ψ` table (`tt[ftab[c]] = ll[ftab[c]] | (i << 8)`), performing **two 256-bucket random memory streams per byte** (random gather `ll[s]` + random scatter `tt[s]`) plus volatile reads on every byte.
2. **Run-route fallback (`src/runs.rs:776`, `fill_avx2`):** constructs the backward `LF` table sequentially, but on every single run record (~225k runs/block on `text`) executes three dependent tiling checks (`len == 0 || len > n - q || len > lim[c] - next[c]`) plus a second 1 KiB `lim[256]` array load — even though `Runs` is an opaque struct whose private fields are only populated by `runs::front`, which already enforces exact tiling.
3. **Reverse assembly (`src/bwt.rs:792`, `assemble_reversed`):** copies segments backward byte-by-byte (`dst[..].iter_mut().rev().zip(...)`), which fails to vectorize on baseline x86-64 (`13.7 ms` vs `7.5 ms` with AVX2 `pshufb`).

---

## 4. Candidate levers tested (100% byte-exact on 78/78 blocks across all arms)

### Winning zero-extra-memory stack (`C_trustfill+rev`)
1. **`A_col_lf` (sequential LF build with 8-byte equal-run fast path on `Column` route):** replaces random-scatter `Ψ` `bwt::build` + `walk_segments_room` with a 100% sequential `LF` build (`tt[i] = ll[i] | (ftab[ll[i]] << 8)`, with an 8-byte `w == w.rotate_left(8)` run fast path avoiding store-forwarding stalls on ELF zero-runs and prose runs) + `walk_back_room`. Zero random reads or writes during build; zero extra memory.
2. **`B_col_lf+avx2rev` (+ AVX2/u64 vectorized `assemble_reversed`):** reverses each segment 32 bytes at a time using `_mm256_shuffle_epi8` + `_mm256_permute2x128_si256` (with 8-byte `u64::swap_bytes` scalar fallback). Zero extra memory.
3. **`C_trustfill+rev` (+ invariant-checked `runs::fill_avx2`):** skips the redundant per-run `len == 0 || len > n - q || len > lim[c] - next[c]` checks and `lim[256]` array inside `fill_avx2` when `Runs` carries its `runs::front` tiling invariant (`r.trusted && n == r.len` checked once in O(1); external/synthetic test `Runs` retain the checked path). Zero extra memory.

### Negative-result levers also tested and ruled out
- **`D_lf2_u64` (Kärkkäinen–Kempa–Puglisi 2-step fused `LF²` in `Vec<u64>`, 7.2 MiB):** cuts walk time by **-15.0% to -15.7%** (`120.85 -> 101.89 ms` baseline, `115.14 -> 97.87 ms` native), but `build_lf2` explodes **3.9–4.2×** (`75.1 -> 289.4 ms` baseline, `73.4 -> 308.4 ms` native) due to 256-stream `ll[s]` reads, 256 KiB `ftab2` updates, and 2× table writes, netting **+104.5% to +121.4% SLOWER** overall (+3.6 MiB/worker). Recorded in `frontier/dead-ends.tsv`.
- **Walk micro-tuning sweep (exploratory 13-arm pass, all 78/78 byte-exact):**
  - Branchless `sstep!` without 8× unrolling regressed walk `+30%`.
  - Unstaged direct-write walk (removing the 8 KiB L1d `tile` staging buffer) regressed `+16..+40%` on baseline due to 32 live segment-buffer write streams thrashing L1d/TLB.
  - `PF_NTA` regressed walk `+56..+168%`; `PF_OFF` regressed baseline `+18.8%`; `L ∈ {16, 24}` / `PF_L2` were noisy or near-neutral vs shipped `L=32, PF_L1`.
  - Scalar `len == 1` branch inside `runs::fill_avx2` (`fill_hyb`) mispredicted on run lengths and regressed `build` `+18%` vs uniform AVX2 stores.

---

## 5. Final paired measurements (7 interleaved repeats, rotated order, shared scratch)

### 5.1 Primary gate: `prose/text + source` (`text + prose + pysrc + rustsrc`, 50 Segmented blocks)

**Baseline x86-64 (`target_avx2=false`, runtime AVX2 via `Kernels::detect()`):**

| arm | CPU med (ms) | CPU `[min..max]` | CPU Δ% | CPU disjoint | Wall med (ms) | Wall `[min..max]` | Wall Δ% | `bld` / `wlk` / `asm` (ms) |
|---|---:|---|---:|---|---:|---|---:|---|
| `shipped` | 195.57 | `[193.56..200.61]` | +0.00% | — | 196.66 | `[193.99..252.31]` | +0.00% | 75.11 / 120.85* / 1.23* |
| `A_col_lf` | 183.76 | `[182.41..190.80]` | -6.04% | **FASTER** | 183.91 | `[182.31..260.68]` | -6.48% | 65.28 / 104.99 / 13.68 |
| `B_col_lf+avx2rev` | 176.31 | `[174.00..184.10]` | -9.85% | **FASTER** | 176.24 | `[174.48..194.77]` | -10.38% | 65.41 / 102.60 / 7.54 |
| **`C_trustfill+rev`** | **165.37** | **`[161.15..170.73]`** | **-15.44%** | **FASTER** | **165.64** | `[161.00..204.10]`† | **-15.77%** | **51.27 / 106.39 / 7.66** |
| `D_lf2_u64` | 400.75 | `[391.14..434.13]` | +104.91% | SLOWER | 402.08 | `[391.95..694.01]` | +104.46% | 289.44 / 101.89 / 11.09 |

**Native (`-C target-cpu=native`, `target_avx2=true`):**

| arm | CPU med (ms) | CPU `[min..max]` | CPU Δ% | CPU disjoint | Wall med (ms) | Wall `[min..max]` | Wall Δ% | `bld` / `wlk` / `asm` (ms) |
|---|---:|---|---:|---|---:|---|---:|---|
| `shipped` | 189.88 | `[187.10..221.09]` | +0.00% | — | 189.91 | `[186.96..220.93]` | +0.00% | 73.44 / 115.14* / 1.28* |
| `A_col_lf` | 179.18 | `[174.60..183.00]` | -5.63% | **FASTER** | 179.31 | `[174.46..193.92]` | -5.58% | 62.11 / 107.97 / 9.50 |
| `B_col_lf+avx2rev` | 177.88 | `[176.18..184.04]` | -6.32% | **FASTER** | 179.21 | `[176.75..189.06]` | -5.63% | 62.37 / 107.98 / 9.47 |
| **`C_trustfill+rev`** | **159.40** | **`[156.27..163.13]`** | **-16.05%** | **FASTER** | **159.69** | `[156.14..189.64]`† | **-15.91%** | **45.01 / 106.16 / 8.64** |
| `D_lf2_u64` | 417.60 | `[412.72..421.64]` | +119.94% | SLOWER | 420.35 | `[412.83..432.27]` | +121.35% | 308.41 / 97.87 / 12.10 |

*\*In `shipped`, `walk_back_room` bundles reverse assembly inside `wlk` for `RunsFallback` blocks; only `Column` forward assembly (`~1.2 ms`) appears in `asm`.*
*†In both baseline and native, 1 of 7 wall repeats of `C_trustfill+rev` experienced a 26–33 ms VM hypervisor preemption (`wall - cpu = +33.4 ms` baseline, `+26.5 ms` native); steal-free `CLOCK_THREAD_CPUTIME_ID` spreads are cleanly disjoint by `+22.8 ms` (baseline) and `+24.0 ms` (native).*

### 5.2 Route split and per-corpus breakdown (thread CPU median ms, Δ% vs shipped)

| arm (target) | `RunsFallback` (48 blks) | `Column` (30 blks) | All 6 (78 blks) | `elf` | `json` | `prose` | `pysrc` | `rustsrc` | `text` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `shipped` (base) | 181.23 | 132.28 | 313.02 | 87.69 | 30.35 | 48.19 | 35.30 | 18.21 | 93.27 |
| `A_col_lf` (base) | -1.38% | **-23.36%** | **-10.49%** | -23.7% | -4.2% | -17.7% | -4.3% | -0.6% | -1.3% |
| `B_col_lf+avx2rev` (base) | **-5.40%** | **-26.07%** | **-13.99%** | -26.2% | -7.9% | -20.5% | -7.2% | -5.2% | -5.1% |
| **`C_trustfill+rev` (base)** | **-13.48%** | **-25.96%** | **-18.43%** | **-26.4%** | **-15.5%** | **-21.8%** | **-16.2%** | **-15.2%** | **-11.7%** |
| `shipped` (nat) | 174.96 | 130.35 | 306.03 | 86.96 | 29.04 | 46.71 | 33.84 | 17.35 | 91.66 |
| `A_col_lf` (nat) | -1.29% | **-24.80%** | **-11.77%** | -25.6% | -4.5% | -16.2% | -4.6% | -0.7% | -2.4% |
| `B_col_lf+avx2rev` (nat) | -1.45% | **-24.74%** | **-11.45%** | -26.4% | -3.1% | -16.6% | -2.2% | +1.1% | -3.8% |
| **`C_trustfill+rev` (nat)** | **-13.96%** | **-23.27%** | **-18.54%** | **-24.4%** | **-12.0%** | **-20.5%** | **-13.6%** | **-10.5%** | **-15.9%** |

---

## 6. Verdict and next step

**VERDICT: PASS** under `segmented-fallback-dissection`'s frozen `stop_test` (with VM hypervisor-steal caveat explicitly recorded):
- `C_trustfill+rev` is 100% byte-exact on all 78/78 `Segmented` blocks across all 6 corpora, adds **0 bytes** of peak memory, and touches **0 Wikidata blocks** (all 233 Wikidata full blocks run `Open`).
- On the primary `prose/text + source` corpora, it cuts `Segmented` inverse-BWT time by **-15.44% baseline / -16.05% native** (steal-free thread CPU, cleanly disjoint `[161.15..170.73]` vs `[193.56..200.61]` and `[156.27..163.13]` vs `[187.10..221.09]`; wall median **-15.77% / -15.91%**), exceeding the `>= 10.0%` gate on every single corpus (`-10.5%` to `-26.4%`, `-18.4%` across all 6 corpora).
- On primary `ab gen text`, inverse BWT is ~64.5% of worker time, so a 12–16% inverse-BWT reduction translates to an expected **~8–10% worker-side wall gain on `text`** (directly evaluated by CI's paired bench under `PERFORMANCE-POLICY.md`!) with zero Wikidata regression risk.

**Admitted follow-on (`status = "idea"`):** [`segmented-fallback-build`](../../../frontier/claims/segmented-fallback-build.toml) (`owned_paths = ["src/bwt.rs", "src/runs.rs"]`, `mechanisms = ["segmented-fallback"]`).
**Dead-end recorded:** KKP 2-step fused `LF²` (`Vec<u64>`) in `frontier/dead-ends.tsv`.
