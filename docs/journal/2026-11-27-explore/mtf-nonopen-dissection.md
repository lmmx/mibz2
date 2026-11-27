# Deep Dissection & Redesign Probe of Non-Open `src/mtf.rs` (`mtf_avx2` + `mtf_checkpoints_avx2`)

*Smooth-zone `explore` probe (`Claim: explore`, zero production `src/` changes; probe harness in [`examples/mtf_nonopen_dissection.rs`](../../../examples/mtf_nonopen_dissection.rs)).*

---

## 1. Motivation & Context

`#1132` (`mtf-fold`) optimizes only admitted **Open** blocks (`src/runs/fold.rs`). Outside Open, `src/mtf.rs` (`mtf_avx2` and `mtf_checkpoints_avx2`) runs on every non-Open block (all 198 non-Open runt blocks of `wikidata.bz2`, and 100% of `Column` blocks on `text`, `random`, `records`, `runs`, `pattern`, and `onebyte`, plus cooperative/checkpointed decoding on every corpus).

Two prior dead-ends scoped adjacent questions without dissecting `mtf_checkpoints_avx2`:
- **`frontier/dead-ends.tsv` Row 29 (`#1060`, `mtf-column-front-dissection`):** Dissected standalone `mtf_avx2` on 5 Column corpora; best arm `R_t32_flat_unal7` (`-8.45%` base / `-9.36%` native; `-10.6..-15.4%` on `random`/`records`) fell below its frozen 15% Column gate and did not touch `mtf_checkpoints_avx2`.
- **`frontier/dead-ends.tsv` Row 32 (`cooperative-decoder-dissection`):** Directly ported `mtf_avx2`'s `(f0, f1)` + 7-row `second()` to `mtf_checkpoints_avx2` and regressed `+42%` on `records` and `+25..42%` on `random` because `mtf_avx2`'s `second()` runs `vpminub + vpcmpeqb + vperm2i128 + vpalignr` across all 7 rows on 75–91% of symbols.

This probe disassembles and instruments **both** shipped `mtf_avx2` and shipped `mtf_checkpoints_avx2`, then tests genuinely distinct redesigns grounded in the machine code and symbol census.

---

## 2. Phase 1: Disassembly (`objdump -d`) & Exact Symbol Census (Pre-Candidate)

### 2.1 `objdump -d` findings on `d2e5f463` (`target/release/examples/ab`)

1. **Shipped `mtf_checkpoints_avx2` (`28` `vzeroupper`/`call` instructions vs `5` in `mtf_avx2`):**
   - **Zero inline short-run splat:** Every single zero-run (even `run == 1`) invokes `vzeroupper` + `call memset@GLIBC` inside the `#[target_feature(enable = "avx2")]` inner loop (`fc686..fc689`), forcing **8 stack reloads** (`r11`, `r10`, `r15`, `ymm2`, `rsi`, `r9`, `out`, `counts`) on return.
   - **Inline `memmove` on every `nn >= 32`:** `std::ptr::copy(base, base.add(1), nn)` emits `vzeroupper` + `call memmove@GLIBC` (`fc74c..fc74f`) followed by 8 stack reloads.
   - **6 stack spills/reloads + 1-byte store-to-load forwarding hazard on EVERY literal:** Because `memset` and `memmove` live inside the loop, LLVM spills `r15` and `r11` on every literal (`fc632..fc63a`), reloads `b`, `counts`, `out`, and even unused `checkpoints` (`mov 0x10(%rbp), %rcx`) on every iteration (`fc79b..fc7c4`), performs an extra `s2u[v]` lookup on every literal and run (unlike `mtf_avx2` which pre-populates the list with `seq_to_unseq`), and writes a 1-byte scalar `mov %al, 0x220(%rsp)` right before the next symbol's 32-byte `vmovdqa 0x220(%rsp)` and unaligned `vpblendvb 0x21f(%rsp)` loads (12–15 cycle STLF stall).
2. **Shipped `mtf_avx2` (`5` `vzeroupper`/`call` instructions):**
   - Already pre-applies `seq_to_unseq`, keeps `f0`/`f1` in XMM registers (`xmm9`/`xmm10`), and inlines 2×32B YMM stores for `run <= 64`, but STILL keeps `memset` (`run > 64`) inline (`11fa07..11fa0a`), forcing `max` (`0x10(%rsp)`) to spill and reload on every run and emitting `vzeroupper` + YMM spills around `memset`, and uses the slower `vpminub + vperm2i128 + vpalignr` 7-row `second()` for `nn >= 32`.

### 2.2 Symbol census & shipped baseline timing (Phase 1, 5-repeat median thread CPU on `d2e5f463`)

Host (C5 diagnostic note): Linux x86-64 KVM VM with AVX2/BMI2, 32 KiB L1d, 256 KiB L2, `CLOCK_THREAD_CPUTIME_ID` (steal-free).

| Corpus | Blks | Bytes | Syms | `run%` | `nn==1%` | `<16%` | `<32%` | `>=32%` | avgRun | `<=64%` | Ckpts | Ship `mtf` (ms) | Ship `ck` (ms) | `ck / mtf` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wiki_nonopen` | 198 | 8,895 | 8,295 | 19.5% | 8.9% | 71.3% | 93.9% | 6.1% | 1.8 | 100.0% | 0 | 0.06 | 0.07 | **1.33×** |
| `wiki_all` | 431 | 209,706,557 | 24,979,299 | 39.4% | 14.5% | 84.8% | 98.5% | 1.5% | 45.3 | 93.9% | 12,582 | 182.45 | 225.44 | **1.24×** |
| `text` | 46 | 41,025,186 | 16,554,382 | 44.1% | 28.8% | 85.2% | 95.0% | 5.0% | 7.6 | 98.9% | 2,462 | 144.32 | 180.47 | **1.25×** |
| `random` | 36 | 32,000,007 | 31,998,087 | 0.8% | 0.8% | 11.8% | 24.3% | 75.7% | 1.0 | 100.0% | 1,920 | 400.95 | 427.70 | **1.07×** |
| `records` | 43 | 38,400,003 | 38,399,730 | 0.3% | 0.3% | 4.5% | 9.4% | 90.6% | 1.0 | 100.0% | 2,304 | 442.76 | 549.58 | **1.24×** |
| `runs` | 2 | 1,016,122 | 247,975 | 27.4% | 38.4% | 53.9% | 57.1% | 42.9% | 17.3 | 99.4% | 61 | 2.63 | 3.36 | **1.27×** |
| `pattern` | 88 | 78,400,000 | 10,626 | 92.6% | 19.5% | 100.0% | 100.0% | 0.0% | 127,271.6 | 0.0% | 4,704 | 3.35 | 3.53 | **1.06×** |
| `onebyte` | 2 | 1,568,630 | 79 | 91.1% | 40.0% | 100.0% | 100.0% | 0.0% | 392,156.2 | 0.0% | 94 | 0.06 | 0.07 | **1.05×** |

Key Phase 1 observation: `mtf_checkpoints_avx2` pays a **+24% to +33% tax** over `mtf_avx2` on `wiki_nonopen`, `wiki_all`, `text`, `records`, and `runs` (and `+7%` on `random`) purely from its `memset`/`memmove` `vzeroupper` spills, `s2u` lookups, and 1-byte `*base = v` STLF hazard — while `<= 64B` runs account for **93.9%–100.0%** of all runs on every non-degenerate corpus!

---

## 3. Frozen Pre-Data Stop Test & Candidate Mechanisms (RULES C1a/C2)

*Frozen BEFORE compiling or running any candidate kernel.*

### 3.1 Candidate mechanisms (all targeting 100% bit-, count-, checkpoint-, and error-exactness)

1. **`A_ck_paged_nos2u_splat`:** Keeps `PagedList` memory layout, eliminates `s2u` indirection (pre-populates `store[32..288]` from `seq_to_unseq`), replaces inline `memset` with inline 64B SIMD splat + `#[cold] #[inline(never)]` long-run fill, eliminates the 1-byte `*base = v` STLF hazard on `nn < 32` by inserting `v` into byte 0 of the 32B vector before storing, and moves `checkpoints.push(*counts)` into a `#[cold] #[inline(never)]` boundary helper.
2. **`B_ck_reg_flat_unal7`:** Register-resident `(f0, f1)` XMM 32-byte L1 front (pre-applied `s2u`, 2-cycle `f0` chain, zero STLF stall) + inline 64B XMM/YMM short-run splat + `#[cold] #[inline(never)]` long-run fill & checkpoint boundary helper + flat unaligned 7-row `vpcmpgtb` tail (`nn >= 32`, zero `memmove`, zero `vperm2i128`, zero `vpalignr`, zero `vzeroupper`, avoiding Row 32's `+42%` `second()` regression!).
3. **`C_ck_reg_cold_tail`:** Register-resident `(f0, f1)` XMM front with **zero YMM instructions** in the hot loop (4×16B XMM short-run splat, `#[cold] #[inline(never)]` `nn >= 32` flat unaligned 7-row tail, `#[cold] #[inline(never)]` long-run fill & checkpoint helper) — tests whether 100% XMM hot loop (zero `vzeroupper`, zero register spills) wins `wiki_nonopen`/`wiki_all`/`text` without hurting `random`/`records`!
4. **`D_ck_adapt_reg_p2`:** $O(1)$ 128-symbol per-block adaptive dispatch (~100 ns/block): blocks with `<= 40%` `nn >= 32` (`wiki_nonopen`, `wiki_all`, `text`, `runs`, `pattern`, `onebyte`) run the 100% XMM hot-loop `C_ck_reg_cold_tail` (zero `vzeroupper`, zero spills), while high-entropy Column blocks (`> 40%` `nn >= 32`: `random`, `records`) run flat unaligned 8-row `P2_flat_unal8_np` (zero branch on `nn`, zero `memmove`, zero 1-byte STLF stall, pre-applied `s2u`, cold checkpoint helper).
5. **`E_ck_reg_s16_split`:** Tests splitting `nn < 16` (single 1-cycle `vpshufb(f0, a)`, never touching `f1`, covering 71–85% of `wiki`/`text`) vs `16 <= nn < 32` vs flat unaligned 7-row tail (`nn >= 32`), to test whether the `nn < 16` branch helps or hurts vs branchless `nn < 32` (`STEPS`).
6. **`M_mtf_adapt_reg_p2` & `M_mtf_reg_flat_unal7` (secondary `mtf_avx2` check):** Applies the same cold-`memset` + XMM short-run splat + flat unaligned 7-row / adaptive `P2` engine to plain `mtf` (without checkpoints) vs shipped `mtf_avx2`.

### 3.2 Frozen PASS / STOP decision rule

- **Exactness gate (mandatory):** 100% identical `ll`, `counts`, `checkpoints` (for `mtf_checkpoints`), and `Result<usize, ErrorKind>` against `mtf::mtf_checkpoints_scalar` / `mtf::mtf_scalar` on ALL 648 blocks across all 7 corpora (`wiki_all` incl. 198 `wiki_nonopen`, `text`, `random`, `records`, `runs`, `pattern`, `onebyte`) PLUS 12 synthetic edge/error cases (`empty_eob`, `tiny_eob_5`, `exact_checkpoint_end`, `run_crossing_one_ckpt`, `run_crossing_multi_ckpts`, `run_exact_max_len`, `err_run_too_long`, `err_block_too_long_run`, `err_block_too_long_lit`, `trailing_run_no_eob`, `full_nn_sweep_255`, `clamp_over_256`).
- **Performance PASS gate (`mtf_checkpoints`):** Verdict is **PASS** iff at least one 100% exact candidate cuts `mtf_checkpoints` thread CPU (`CLOCK_THREAD_CPUTIME_ID`, 7 interleaved repeats, median) by **`>= 10.0%`** with **non-overlapping `[min..max]` spreads** vs shipped `mtf_checkpoints_avx2` on **BOTH** `wiki_nonopen` (198 non-Open Wikidata blocks) **AND** `text`, AND causes **no regression `> +2.0%`** on any of `random`, `records`, `runs`, `pattern`, `onebyte` (and `wiki_all`), across **ALL THREE build profiles**:
  1. `base`: `cargo build --release` (generic x86-64),
  2. `nat`: `RUSTFLAGS="-C target-cpu=native"`,
  3. `jcc`: `RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"` (Intel JCC-erratum 32B alignment control).
- **STOP rule:** Otherwise verdict is **STOP**, recorded as a scoped dead-end row in `frontier/dead-ends.tsv` with no build claim admitted.

## 4. Phase 2: Iterative Dissection, Disassembly, and 4-Profile Benchmarks

### 4.1 Exactness Verification

Every candidate arm across all rounds passed **100% bit-, count-, checkpoint-, and error-exactness** against `mtf::mtf_checkpoints_scalar` and `mtf::mtf_scalar` on all **648 corpus blocks** (`wikidata` 431 incl. 198 `wiki_nonopen`, `text` 46, `random` 36, `records` 43, `runs` 2, `pattern` 88, `onebyte` 2) plus all **12 synthetic edge/error cases** (including `exact_checkpoint_end`, `run_crossing_one_ckpt`, `run_crossing_multi_ckpts`, `err_run_too_long`, `err_block_too_long_run`, `err_block_too_long_lit`, `full_nn_sweep_255`, and `clamp_over_256`). Final committed harness `examples/mtf_nonopen_dissection.rs` passes **`EXACT PASS: 3959/3959`** (and intermediate 18-arm sweeps passed up to `16499/16499`).

### 4.2 Disassembly Discoveries (`objdump -d`) Across Redesign Rounds

Inspecting the generated x86-64 machine code after every round exposed **five concrete codegen and microarchitectural constraints** governing `src/mtf.rs`:

1. **System-V ABI XMM spills from *any* call inside the loop (`C_reg_cold7`, `H_uni_reg7`):**
   - Because System-V x86-64 has **zero callee-saved XMM/YMM registers**, placing even a `#[cold] #[inline(never)]` helper (`tail_flat_unal7_cold` in Arm C, or `push_checkpoint_cold`/`flush_crossing_run_cold` in Arm H) anywhere reachable from a loop back-edge forces LLVM on `-C target-cpu=native` (and even `base` in Arm H: `xmm_rsp=17`) to spill `f0` (`0x70(%rsp)`) and `f1` (`0x60(%rsp)`) on every symbol — erasing the register-resident `(f0, f1)` advantage (`C_reg_cold7` went from `-21.7%` on `base` `text` to `+6.7%` regression on `nat` `random` and `+0.2%` on `nat` `wiki_nonopen`).
2. **Zero-call-in-loop `ck_zerocall_reg7` (`insns=578`, `calls=0` in loop, `vzeroupper=1`, `xmm_rsp=0`):**
   - Pre-reserving `checkpoints.reserve(max / CHECKPOINT + 2)` in the prologue (before loading `f0`/`f1`), inlining run fills (`fill_run_inline_nocall`, replacing scalar byte tail loops with branch-staged 16/8/4/2/1B unaligned stores so LLVM cannot collapse them into `call memset`), and copying 1 KiB checkpoints with a compact inline AVX2 loop (`copy_ckpt_compact_nocall`) drops hot-loop calls from **`18 -> 0`**, `vzeroupper` from **`10 -> 1`** (epilogue only), and XMM stack spills to **`0`** across all 4 build profiles!
3. **12-arg leaf chunk spills, `compiler_fence`, and inline `asm!("rep stosb")` clobbers:**
   - Moving the inner loop into a separate `#[inline(never)]` leaf chunk function (`F_leaf_reg7` / `G_leaf_adapt_p2`) achieved `call=0`/`vzeroupper=1` inside the leaf, but passing 12 arguments spilled 7 pointers/limits onto the stack on every run.
   - Placing `compiler_fence(Ordering::AcqRel)` inside `fill_run_inline_nocall` blocked LLVM load/store hoisting across runs (`+5–13 ms` on `text`); replacing `memset` with inline `asm!("rep stosb")` acted as an opaque memory clobber (`+13–28%` slower on `wiki_nonopen`/`wiki_all`).
4. **4 KiB page-split alignment (`#[repr(align(512))]`) and Haswell/Broadwell cacheline-split cost:**
   - The unaligned `base.sub(1)` / `tail.sub(1)` loads in `A_paged_splat` and `ck_zerocall_p2_flat8` MUST back `Base320` and `Tail256` with `#[repr(align(512))]` (not `align(64)`), otherwise `base.sub(1)` straddles a 4 KiB page boundary (`+45–75 ms` penalty on `random`/`records`).
   - Even with `#[repr(align(512))]`, on older Haswell/Broadwell Xeons (`Intel(R) Xeon(R) CPU @ 2.30GHz`, 32 KiB L1d / 256 KiB L2), `A_paged_splat`'s unaligned 8-row `row.sub(1)` loads regressed **`+8.2%` (`jcc`) / `+12.2%` (`nat_jcc`)** on `random` vs shipped `mtf_checkpoints_avx2`, whereas `T_zerocall_adapt` (`ck_zerocall_p2_flat8` on high-entropy `> 50% nn >= 32` blocks) eliminated that regression (`-2.4%..-11.2%` on `random`, `-10.3%..-18.3%` on `records`).
5. **Fixed `2.3 KiB`/block Amdahl floor on `wiki_nonopen` + Intel 32B JCC alignment swings:**
   - `wiki_nonopen`'s 198 blocks average **45 bytes / 42 symbols** (`8,895 B` total = `0.004%` of `wikidata.bz2`, **0 checkpoints**). Mandatory per-block zeroing of `counts` (`1,024 B`) + initial `checkpoints[0]` (`1,024 B`) + `s2u` copy (`256 B`) = **`2,304 B`/block** consumes ~65–75% of the ~340 ns block time, capping total `wiki_nonopen` speedup on Haswell/Broadwell to **`-2.2%..-8.9%`** (and `-10%..-15%` only on newer Xeons).
   - Adding unrelated code shifted unmitigated `base` / `nat` timings of *unchanged* `ship_ck` and `A_paged_splat` by **`±15–23 ms` (`12–17%`)** across builds (confirming `#1109`/`#1119`'s Intel 32B JCC erratum finding), while `-x86-branches-within-32B-boundaries` stabilized `ship_ck` `text` to `±1.5 ms`.

---

### 4.3 Measured Results: Final Committed Harness (`examples/mtf_nonopen_dissection.rs`, `bench_final`)

Host (`lscpu` verified): `Intel(R) Xeon(R) CPU @ 2.30GHz` (Haswell/Broadwell KVM vCPU, AVX2/BMI2/ERMS, 32 KiB L1d, 256 KiB L2), pinned via `taskset -c 0`, 7 interleaved repeats, steal-free `CLOCK_THREAD_CPUTIME_ID` median ms (`*` = non-overlapping `[min..max]` faster, `!` = non-overlapping slower).

#### Primary Target: `mtf_checkpoints` (`A_paged_splat` & `T_zerocall_adapt` vs shipped `mtf_checkpoints_avx2`)

| Corpus | `base` `ship_ck` | `base` `A_paged` | `base` `T_zc_adapt` | `nat` `ship_ck` | `nat` `T_zc_adapt` | `jcc` `ship_ck` | `jcc` `T_zc_adapt` | `nat_jcc` `ship_ck` | `nat_jcc` `T_zc_adapt` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **`wiki_nonopen`** | 0.070 | -4.79% | **-0.57%** | 0.075 | **-8.88%\*** | 0.068 | **-2.18%** | 0.070 | **-6.51%\*** |
| `wiki_all` | 219.91 | -6.50% | -18.52% | 220.80 | -5.76% | 197.45 | **-11.83%\*** | 203.81 | **-9.61%\*** |
| **`text`** | 164.45 | +3.30% | **-12.02%\*** | 183.00 | **-9.83%\*** | 156.52 | **-11.87%\*** | 156.97 | **-13.42%\*** |
| `random` | 437.86 | +7.57% | -9.32% | 430.88 | **-11.17%\*** | 406.46 | -3.08% | 382.52 | **-2.36%\*** |
| `records` | 568.26 | -7.45%\* | **-15.74%\*** | 564.63 | **-18.27%\*** | 525.37 | **-10.37%\*** | 507.82 | **-10.27%\*** |
| `runs` | 3.250 | +1.70% | -7.56% | 3.327 | -1.02% | 3.044 | -0.08% | 3.012 | **-5.02%\*** |
| `pattern` | 3.564 | +0.16% | -1.37% | 3.572 | -0.47% | 3.672 | +0.71% | 3.560 | +1.26% |
| `onebyte` | 0.069 | -1.11% | -2.43% | 0.069 | +1.85% | 0.069 | +7.90% ovlp | 0.070 | +2.85% ovlp |

*(Earlier Round 3 / Round 9–10 measurements of the same zero-call `ck_zerocall_reg7` + `ck_zerocall_p2_flat8` core on the faster pre-restart Xeon host reached `-11.9%..-17.0%*` on `text`, `-13.5%..-19.8%*` on `wiki_all`, `-13.3%..-23.5%*` on `random`, and `-19.7%..-28.0%*` on `records`, but still missed the frozen `>= 10.0%` disjoint gate on `wiki_nonopen` under `jcc` at `-2.3%` once long-run dispatch was added to hold `pattern`/`onebyte` neutral.)*

#### Secondary Diagnostic: Plain `mtf` (`M_uni_adapt` vs shipped `mtf_avx2`)

| Corpus | `base` `ship_mtf` | `base` `M_uni_adapt` | `nat` `ship_mtf` | `nat` `M_uni_adapt` | `jcc` `ship_mtf` | `jcc` `M_uni_adapt` | `nat_jcc` `ship_mtf` | `nat_jcc` `M_uni_adapt` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `wiki_nonopen` | 0.063 | +32.03%! | 0.059 | +26.51%! | 0.050 | +33.37%! | 0.050 | +38.64%! |
| `wiki_all` | 194.65 | +12.35%! | 187.44 | +16.21%! | 158.24 | +20.74%! | 156.65 | +17.80%! |
| `text` | 152.92 | +9.17% | 144.55 | +14.69%! | 119.67 | +20.11%! | 125.93 | +14.30%! |
| `random` | 438.70 | -11.77% | 427.34 | **-13.22%\*** | 393.98 | -0.29% | 424.82 | **-8.83%\*** |
| `records` | 474.65 | -2.83% | 488.58 | -6.42% | 439.97 | +4.16% | 465.52 | -0.02% |
| `runs` | 2.870 | +9.86% | 2.757 | +16.73% | 2.416 | +19.61% | 2.557 | +14.63%! |
| `pattern` | 3.407 | **-40.95%\*** | 3.451 | **-39.26%\*** | 3.150 | **-35.34%\*** | 3.453 | **-34.61%\*** |
| `onebyte` | 0.066 | -45.85% | 0.070 | **-45.39%\*** | 0.066 | -45.16% | 0.066 | -40.02% |

Shipped `mtf_avx2` already pre-applies `seq_to_unseq`, keeps `(f0, f1)` in XMM registers, and inlines 2×32B YMM short-run splats (`r <= 64`). Replacing its compact cold `memset` + out-of-line `second()` with an inlined zero-call loop inflates hot-loop code size and regresses `+9%..+39%` on `wiki_nonopen`/`wiki_all`/`text`/`runs`, while only improving high-entropy `random`/`records` (`-9%..-21%`) and degenerate `pattern`/`onebyte` (`-35%..-46%`), independently confirming `frontier/dead-ends.tsv` Row 29 (`#1060`).

---

## 5. Verdict: **STOP** (Strict C2 — Zero Relaxation)

Under the frozen pre-data stop test (§3.2):
- **Exactness gate:** **PASS** (`3959/3959` identical across all 648 corpus blocks + 12 synthetic edge/error cases).
- **Performance PASS gate (`mtf_checkpoints`):** **FAIL -> `STOP`.**
  - While `T_zerocall_adapt` consistently accelerates large checkpointed blocks (`text -9.8%..-13.4%*` on Haswell/Broadwell and `-11.9%..-17.0%*` on the faster Xeon; `records -10.3%..-18.3%*` / `-19.7%..-28.0%*`; `wiki_all -5.8%..-18.5%` / `-13.5%..-19.8%*`) and eliminates Row 32's `+42%` regression on `random`/`records`, **no single candidate clears the frozen `>= 10.0%` disjoint gate on BOTH `wiki_nonopen` AND `text` across all three build profiles (`base`, `nat`, `jcc`)** without regressing `> +2.0%` on `pattern`/`onebyte` or `random`:
    - `wiki_nonopen` is structurally Amdahl-bounded by mandatory `2.3 KiB`/block setup (`counts` + `checkpoints[0]` + `s2u`) over 45-byte / 42-symbol runt blocks (`-0.57%` `base`, `-8.88%*` `nat`, `-2.18%` `jcc`, `-6.51%*` `nat_jcc` on Haswell/Broadwell),
    - `onebyte` shows `+2.85%..+7.90%` overlapping jitter at 0.07 ms scale under `jcc`/`nat_jcc`,
    - Unmitigated `base`/`nat` builds exhibit `±15–23 ms` Intel 32B JCC alignment swings on `src/mtf.rs`.
- **No build claim is admitted.** A scoped dead-end row is recorded in `frontier/dead-ends.tsv`.

### What Remains Open (Scoped Boundary)
This `STOP` closes standalone `mtf_checkpoints_avx2` / `mtf_avx2` redesigns gated on tiny 45-byte `wiki_nonopen` runt blocks or standalone plain `mtf_avx2` on low-entropy corpora. It does **not** close:
1. A future **i9-gated** `mtf_checkpoints_avx2` redesign specifically targeting **large** checkpointed `text`/`records`/`random`/`wiki_all` blocks under JCC-aligned builds (`#1119`/`#1139`), where `T_zerocall_adapt`'s zero-call blueprint (`pre-reserved checkpoints` + `ck_zerocall_reg7` + `ck_zerocall_p2_flat8` + `syms.len() <= 2048` PagedList dispatch, `0` hot-loop calls, `0` XMM spills, `vzeroupper=1`) demonstrated `-10%..-28%` wins without Row 32's regression;
2. AVX-512 `vpermb` MTF or Huffman+MTF fusion (already noted in Row 29).

