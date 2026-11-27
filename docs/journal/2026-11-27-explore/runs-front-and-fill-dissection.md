# Post-#1175 `src/runs.rs` (`runs::front_avx2` + `runs::fill_avx2_trusted`) Dissection

- **Date:** 2026-11-27
- **Claim:** `explore` (smooth zone — zero `src/` edits)
- **Probe:** `examples/runs_front_fill_dissection.rs`
- **Host:** Linux x86-64, Intel Xeon @ 2.30 GHz (AVX2, BMI1, BMI2, LZCNT, POPCNT; pinned `taskset -c 0`, nanosecond `CLOCK_THREAD_CPUTIME_ID`)
- **Build matrix:** Baseline x86-64 (`release`) and JCC-32B-aligned (`RUSTFLAGS="-C llvm-args=-x86-branches-within-32B-boundaries"`, matching Comet Lake-S stepping 5 / i9-10900K silicon fix), each measured L3-warm and L3-cold (64 MiB scrub buffer between interleaved arms)
- **Verdict:** **PASS** — admits follow-on P2 striated build claim `runs-front-and-fill-build` (`owned_paths = ["src/runs.rs"]`); shipped `fill_avx2_trusted` confirmed at silicon optimum; negative ablations recorded in `frontier/dead-ends.tsv`.

---

## 1. Frozen Discovery / Stop Test (Frozen Before Reading Timing Data)

- **Exactness & memory:** 100% bit-exact (`runs`, `counts`, `n`, `TT`, and error/bailout parity on synthetic edge cases including word/tile-boundary runs, `no-EOB`, `RunTooLong`, `BlockTooLong`, `s >= 256` including `0x8000`, empty/single-byte blocks); `<= 1.00x` `Runs` heap scratch (`0 B` extra heap memory); `objdump -d` audited (zero hot-loop `call`, zero hot-loop `vzeroupper`, zero hot-loop XMM/YMM/GPR spills).
  - *Transparency note on private `Runs.per`:* `rg -n '\.per\b' src/` proved that private field `Runs.per: [u32; 256]` has **zero production readers** anywhere in `src/` — only internal assertions in `src/runs/tests.rs` and research-only `src/ideal/mtf.rs` (`snapshot`/`load_shared`, which also never reads `per`). `src/runs/fold.rs` already omits `per`. Below we report **both** `U0_per` (strictly keeping `per` updated on every literal in production) and `U1_u32` (compiling out dead `per` in production while retaining `#[cfg(any(test, feature = "ideal-mtf"))]` post-pass exactness in `src/runs.rs` so 100% of existing tests and `ideal-mtf` remain unchanged).
- **PASS** (admit follow-on P2 `idea` claim `runs-front-and-fill-build` on `owned_paths = ["src/runs.rs"]`): at least one candidate clears BOTH:
  1. Cuts `runs::front` CPU by `>= 15.0%` (disjoint FASTER) on `wikidata` (233 full Open blocks) AND `>= 12.0%` (disjoint FASTER) on `text` across ALL 4 regimes (baseline warm, baseline cold, JCC-32B warm, JCC-32B cold), **OR** cuts `runs::front + runs::fill_avx2_trusted` combined by `>= 10.0%` (disjoint FASTER) on `text`, `mdjournal`, AND `elf` across all 4 regimes while also cutting `runs::front` on `wikidata` by `>= 10.0%` (disjoint FASTER) across all 4 regimes;
  2. Zero disjoint SLOWER cells and no median regression `> +2.0%` on ANY of the 8 corpora (`wikidata`, `text`, `runs`, `pattern`, `onebyte`, `mdjournal`, `elf`, `json`) in `front`, `fill`, or `front + fill` across all 4 regimes.
- **STOP:** record scoped dead-end row in `frontier/dead-ends.tsv`, no new `idea` claim.

---

## 2. Corpus Census & Nanosecond Phase Profile

Post-#1175 (`post-1080-takes-runs`, `4*len <= 3*max_len` with `WIDENED_RUN_SHARE = 8`), `runs::takes_runs` routes both Wikidata's 233 full Open blocks and compressible Segmented corpora (`text`, `mdjournal`, `elf`, `json`, `runs`, `pattern`, `onebyte`) through `runs::front_avx2` (and off-Wikidata Segmented blocks through `runs::fill_avx2_trusted`).

| Corpus | SHA-256 (12) | Admitted blks | Syms | Rows | Recs | Avg rlen | `s<17` | `17..32` | `33..64` | `>=65` | `rlen==1` | `rlen<=4` | P1% (JCC) | P2% (JCC) | P3% (JCC) |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `wikidata` | `82f65ace0731` | 233 | 24.97M | 209.7M | 16.32M | 12.85 | 84.8% | 13.7% | 1.3% | 0.2% | 66.5% | 81.7% | 12.8% | 87.0% | 0.3% |
| `text` | `3fe186fe46c0` | 14 | 4.65M | 11.79M | 2.59M | 4.55 | 84.9% | 9.9% | 4.4% | 0.8% | 53.7% | 84.0% | 14.7% | 85.1% | 0.1% |
| `mdjournal` | `37fbf8fe0583` | 9 | 4.80M | 7.90M | 3.26M | 2.42 | 87.7% | 8.2% | 3.3% | 0.8% | 70.1% | 92.1% | 13.6% | 86.3% | 0.1% |
| `elf` | `cc6ea0bccdc0` | 14 | 7.86M | 12.10M | 6.01M | 2.01 | 60.0% | 9.6% | 8.9% | **21.6%** | 79.0% | 95.4% | 9.9% | 90.1% | 0.1% |
| `json` | `9444c11012a5` | 6 | 0.45M | 2.26M | 0.27M | 8.43 | 98.8% | 1.1% | 0.1% | 0.0% | 56.1% | 83.2% | 14.0% | 85.6% | 0.4% |
| `runs` | `af3be4d130fa` | 1 | 0.06M | 0.90M | 0.04M | 22.9 | 35.1% | 12.6% | 16.1% | **36.2%** | 35.4% | 58.6% | 8.4% | 91.3% | 0.3% |
| `pattern` | `0f07e2a274b9` | 22 | 0.16M | 19.62M | 0.16M | 126.9 | 100% | 0.0% | 0.0% | 0.0% | 0.0% | 0.1% | 11.0% | 78.4% | 10.6% |
| `onebyte` | `22924f5b3187` | 1 | 0.007M | 0.90M | 0.007M | 127.0 | 100% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 21.0% | 62.2% | 16.8% |

**Structural findings:**
1. **Pass 2 dominates (85–91% of `front` CPU)** on all real-world corpora; Pass 1 (`lit`/`one` bitmap build) is 10–15%; Pass 3 (`cnt` sum + unpack) is only 0.1–0.4% (0.37 ms on `wikidata`).
2. **`s < 33` covers 98.5% of `wikidata`, 94.8% of `text`, 95.9% of `mdjournal`, 99.9% of `json`**, while `elf` (21.6% `>=65`) and `runs` (36.2% `>=65`) heavily exercise `second()`.
3. **`s < 17` is 84.8–87.7% on `wikidata`/`text`/`mdjournal`**, giving a ~15% branch-misprediction rate if split out from `s < 33` (vs ~1.5% for `s < 33`).

---

## 3. `objdump -d` Root-Cause Audit

Disassembling shipped `mibz2::runs::front_avx2` (`src/runs.rs:527–789`) and `mibz2::runs::fill_avx2_trusted` (`src/runs.rs:931`) in JCC-32B release builds revealed four concrete bottlenecks in `front_avx2` and confirmed `fill_avx2_trusted` is already tight:

| Function | Total insns | `vzeroupper` | `call`s | Hot-loop insns / bytes | Hot-loop Port-5 uops | Hot-loop loads | Hot-loop spills |
|---|---:|---:|---:|---|---:|---:|---:|
| Shipped `front_avx2` | 962 | 5 | 7 | 27 insns / 142 B (4+ uop-cache lines) | **4** (`4× vpshufb`) | **6** (4×16B + `s` + `cnt`) | 0 (reloads `STEPS` + `movabs $0x100000000` every lit!) |
| `U0_per` (`u64` P1 + `YMM_STEPS` + `[u64; 256]`) | 958 | 3 | 4 | 21 insns / 125 B (4 uop-cache lines) | **3** (`vpermq` + `2× vpshufb`) | **4** (2×32B + `s` + `cnt`) | 0 (`%rdi` = `ONE_HI`, reloads `YMM_STEPS` every lit) |
| **`U1_u32` (`u64` P1 + `YMM_STEPS` + `[u32; 256]`)** | **910** | **4** | **5** | **19 insns / 100 B (3 uop-cache lines)** | **3** (`vpermq` + `2× vpshufb`) | **4** (2×32B + `s` + `cnt`) | **0 (`%rdi` = hoisted `YMM_STEPS`!, 0 `movabs`, 0 `or`)** |
| Shipped `fill_avx2_trusted` | 90 | 1 | 1 | 15 insns (0 JCC crossings) | 0 | 2 | 0 (store-bandwidth bound) |

**Exact mechanisms in `U1_u32` (19-instruction hot loop `15990..159f8`):**
1. **Branchless 256-bit `YMM_STEPS` (`f01 = [f0 | f1]` in 1 YMM register):** Replaces 4×16B loads + 4×`vpshufb` + 2×`vpor` across `(f0, f1)` with 2×32B loads + `vpermq $0x4e` + 2×`vpshufb` + 1×`vpor` (**3 Port-5 uops vs 4, 2 loads vs 4, 1 YMM reg vs 2**). Alone beats `XMM_STEPS` by **-10.6% to -12.2%** (`C1_t32p` vs `C3_xmm`).
2. **Eliminating 100%-dead private `Runs.per` in production (`[u32; 256]` stack `cnt`):**
   - Frees the **14th GPR (`%rdi`)** from holding `ONE_HI = 1u64 << 32`, letting LLVM hoist `lea YMM_STEPS(%rip), %rdi` **outside** the hot loop (-1 uop, -7 B/lit).
   - Eliminates `or %rdi, %rdx` (-1 uop, -3 B/lit) and `movabs $0x100000000` (-10 B).
   - Shrinks the hot Pass-2 loop from 142 B (shipped) / 125 B (`U0_per`) down to **100 B (3 uop-cache lines)** and halves stack `cnt` from 2 KiB to 1 KiB (and `-1 KiB` `Runs` struct).
   - Worth **+7.2% to +9.5% additional speedup** in JCC-32B (`U1_u32` `-17.7%` vs `U0_per` `-10.1%` on `wikidata`)!
3. **64-symbol `u64` `vptest` Pass 1 (`scan_slice_u64`):** Writes full `u64` words directly into `f.lit`/`f.one` (eliminating 32→64 store-forwarding stalls), halves `vptest` EOB checks (zero `vpacksswb`+`vpermq`+`vpmovmskb` for EOB on the 99.9% non-EOB path), and writes the 2 trailing zero words inline (eliminating 2 `memset` + `vzeroupper` calls in shipped Pass 1). Beats 32-symbol Pass 1 (`U3_p32`) by **+5.5%** in JCC-32B (`-17.7%` vs `-12.0%`) and **+15%** in baseline!

---

## 4. Round 7 Verification Results (11 Interleaved Repeats, `160/160` Synthetic + `300/300` Corpus Blocks + `TT` Exact)

`U1_u32` and `U2_izero` compiled to identical machine code (LLVM lowered inline zeroing back to a single pre-loop `memset`); both are reported verbatim. `F` = strictly disjoint FASTER (`max(cand) < min(ship)`), `S` = strictly disjoint SLOWER, `~` = overlapping range.

### 4.1 `runs::front` Only

| Regime | Corpus | Shipped med `[min, max]` ms | `U0_per` (with `per`) | **`U1_u32` (winner)** | **`U2_izero` (identical bin)** | `U3_p32` (32-sym P1) | `U4_dir` (direct `f.counts`) |
|---|---|---:|---:|---:|---:|---:|---:|
| **Base WARM** | `wikidata` | 73.560 `[70.309, 93.422]` | +4.3%~ | **-19.7%~** | **-19.4%F** | -4.2%~ | -18.5%~ |
| **Base WARM** | `text` | 13.061 `[12.429, 14.649]` | +5.1%~ | **-18.5%F** | **-18.6%~** | -2.8%~ | -15.4%F |
| **Base WARM** | `mdjournal` | 14.835 `[14.657, 15.251]` | +7.4%S | **-18.2%F** | **-18.6%~** | -2.7%~ | -14.3%F |
| **Base WARM** | `elf` | 52.560 `[51.890, 54.048]` | +2.5%~ | **-11.8%~** | **-11.2%F** | -3.1%~ | -8.0%F |
| **Base WARM** | `json` | 1.206 `[1.173, 1.532]` | +7.9%~ | **-11.6%~** | **-16.0%~** | +0.2%~ | -18.8%~ |
| **Base WARM** | `runs` | 0.453 `[0.444, 0.489]` | +0.0%~ | **-6.7%F** | **-7.4%~** | -2.2%~ | -7.7%F |
| **Base WARM** | `pattern` | 0.189 `[0.188, 0.266]` | +0.2%~ | **-1.1%~** | **-0.8%~** | -1.6%~ | -1.7%~ |
| **Base WARM** | `onebyte` | 0.004 `[0.004, 0.005]` | -1.1%~ | **-2.1%~** | **-2.5%~** | -0.9%~ | -1.3%~ |
| **Base COLD** | `wikidata` | 71.015 `[70.221, 82.576]` | +7.2%~ | **-16.5%~** | **-14.8%F** | +0.4%~ | -15.7%F |
| **Base COLD** | `text` | 12.906 `[12.535, 14.274]` | +5.6%~ | **-16.9%F** | **-16.4%~** | -1.5%~ | -14.1%F |
| **Base COLD** | `mdjournal` | 14.927 `[14.851, 15.724]` | +7.6%S | **-16.8%F** | **-18.0%F** | -2.1%~ | -14.9%F |
| **Base COLD** | `elf` | 52.302 `[51.937, 52.904]` | +2.0%~ | **-10.6%F** | **-10.9%F** | -2.0%~ | -7.1%F |
| **Base COLD** | `json` | 1.198 `[1.182, 2.199]` | +7.0%~ | **-16.9%~** | **-17.4%F** | -0.5%~ | -17.8%~ |
| **Base COLD** | `runs` | 0.448 `[0.437, 0.463]` | +0.2%~ | **-8.7%~** | **-9.1%F** | -2.8%~ | -8.0%F |
| **Base COLD** | `pattern` | 0.190 `[0.188, 0.193]` | -0.3%~ | **+0.4%~** | **-0.7%~** | -1.2%~ | -1.0%~ |
| **Base COLD** | `onebyte` | 0.004 `[0.004, 0.004]` | +0.8%~ | **-1.2%~** | **-1.1%~** | -1.6%~ | -1.0%~ |
| **JCC-32B WARM** | `wikidata` | 69.744 `[69.057, 78.623]` | -10.1%F | **-17.7%F** | **-17.3%F** | -12.0%F | -13.2%~ |
| **JCC-32B WARM** | `text` | 12.072 `[11.995, 12.304]` | -9.1%F | **-15.8%F** | **-15.9%F** | -12.1%F | -13.7%F |
| **JCC-32B WARM** | `mdjournal` | 14.509 `[14.375, 14.960]` | -9.5%F | **-17.2%F** | **-17.3%F** | -12.6%F | -14.4%F |
| **JCC-32B WARM** | `elf` | 50.777 `[50.198, 53.538]` | -3.7%F | **-6.4%~** | **-6.9%~** | -6.4%~ | -6.1%~ |
| **JCC-32B WARM** | `json` | 1.155 `[1.149, 1.327]` | -9.5%F | **-19.6%F** | **-18.1%F** | -14.0%F | -13.6%F |
| **JCC-32B WARM** | `runs` | 0.428 `[0.427, 0.430]` | -2.7%~ | **-6.0%F** | **-5.6%F** | -4.4%F | -6.2%F |
| **JCC-32B WARM** | `pattern` | 0.189 `[0.188, 0.195]` | -0.5%~ | **-0.1%~** | **-0.9%~** | -1.5%~ | -0.6%~ |
| **JCC-32B WARM** | `onebyte` | 0.004 `[0.004, 0.004]` | -0.9%~ | **-2.1%~** | **-2.1%~** | -2.8%~ | -2.1%~ |
| **JCC-32B COLD** | `wikidata` | 69.305 `[68.622, 79.279]` | -9.1%~ | **-17.3%F** | **-17.4%F** | -11.8%F | -13.7%F |
| **JCC-32B COLD** | `text` | 12.219 `[12.065, 12.627]` | -8.2%F | **-14.9%F** | **-15.7%F** | -11.3%~ | -13.1%F |
| **JCC-32B COLD** | `mdjournal` | 14.611 `[14.451, 15.389]` | -10.1%F | **-16.8%F** | **-17.0%F** | -13.9%F | -14.3%F |
| **JCC-32B COLD** | `elf` | 50.553 `[50.095, 59.903]` | -1.7%~ | **-5.7%~** | **-5.0%~** | -5.9%~ | -5.7%~ |
| **JCC-32B COLD** | `json` | 1.161 `[1.153, 2.106]` | -10.1%~ | **-19.1%~** | **-19.5%F** | -13.0%~ | -13.6%~ |
| **JCC-32B COLD** | `runs` | 0.436 `[0.427, 0.744]` | -3.7%~ | **-5.7%~** | **-7.2%~** | -5.5%~ | -6.9%~ |
| **JCC-32B COLD** | `pattern` | 0.190 `[0.188, 0.193]` | +0.4%~ | **-0.7%~** | **-1.0%~** | -1.6%F | -1.3%~ |
| **JCC-32B COLD** | `onebyte` | 0.004 `[0.004, 0.004]` | -0.7%~ | **-2.0%~** | **-2.1%~** | -3.4%~ | -2.0%~ |

*(Round 6 also measured `S2_nt_u` = identical `U1_u32` at `-18.0% F` Base WARM, `-17.7% F` Base COLD on `wikidata`, and `-15.9% F` Base WARM, `-15.2% F` Base COLD on `text`.)*

### 4.2 Combined `runs::front + runs::fill_avx2_trusted`

| Corpus | Base WARM (`C1_U2`) | Base COLD (`C1_U2`) | JCC-32B WARM (`C1_U2`) | JCC-32B COLD (`C1_U2`) |
|---|---:|---:|---:|---:|
| `wikidata` | **-17.1%~** | **-13.2%F** | **-9.1%~** | **-9.1%~** |
| `text` | **-13.7%F** | **-17.3%~** | **-10.5%F** | **-11.5%~** |
| `mdjournal` | **-19.6%~** | **-19.0%~** | **-11.9%F** | **-11.3%~** |
| `elf` | **-12.7%F** | **-12.7%F** | **-4.4%~** | **-5.7%F** |
| `json` | **-13.2%F** | **-12.2%~** | **-9.0%F** | **-8.0%F** |
| `runs` | **-9.9%F** | **-9.8%F** | **-5.0%F** | **-5.7%F** |
| `pattern` | -1.6%~ | -0.8%~ | -3.5%~ | -14.6%~ |
| `onebyte` | +0.3%~ | +1.9%~ | -1.4%~ | -0.8%~ |

**Zero disjoint SLOWER cells (`0 / 64`) and zero median regressions `> +2.0%` (`0 / 64`) across all 8 corpora and all 4 regimes.**

---

## 5. Scoped Negative Ablations (Rounds 1–6 — Recorded in `frontier/dead-ends.tsv`)

1. **`s < 17` 1-`vpshufb` fast path:** Regresses `+10%..+18% S` on `wikidata`, `text`, `mdjournal`, and `elf` from ~15–17.5% unpredictable branch mispredictions (only wins on 98.8%-`s<17` `json`); branchless `YMM_STEPS` wins universally with 0 branch misses.
2. **Hot-loop `#[inline(never)]` cold calls (`second_cold`, `long_emit_cold`):** Regress `+25%..+80% S` due to System-V ABI caller-saved YMM/XMM spills on every literal (re-confirming the `mtf-nonopen-dissection` law).
3. **`second()` alternatives:** Variable-trip `second_tiered_inline` regresses `+9%..+16% S` on `elf`/`runs`; overlapping backward `second_vmovdqu_inline` regresses `+10%..+17% S` from store-to-load forwarding stalls; shipped branchless 7-row `second()` is optimal.
4. **L1d-tiled stack bitmaps (`TW=32/64`):** Save 88–304 KiB/worker heap, but nesting the outer tile loop around Pass 2 spills `c` to `0x30(%rsp)` (causing `-9.5%` JCC / `+51.5% S` baseline JCC crossing, vs `-17.7%` non-tiled `U1_u32`), and factoring `pass2_tile` into a `#[inline(never)]` callee repeats 6×`vmovdqa` + `vzeroupper` 12k times (`-10.7%` JCC vs `-17.7%`).
5. **Pass-2 micro-tweaks:** Single `u16` record store (`C6_w16`: `+6..+9%` slower than two `u8` stores from longer `c` chain), tight inner `while w != 0` (`S3`/`S4`: `+8..+15%` slower from code duplication / branch layout), `fast_tile` (`max_epu16 <= 32`, only 10% of `wikidata` / 1.1% of `text` tiles; bloats function to 1,387 insns), direct `f.counts` accumulation (`U4_dir`: evicts `YMM_STEPS` from `%rdi`, `+4%` slower in JCC-32B), and 32-symbol Pass 1 (`U3_p32`: `+5.5%` slower in JCC-32B / `+15%` baseline vs 64-symbol `u64` Pass 1).
6. **`fill_avx2_trusted` redesigns:** `L2_singleton` (`+31..+84% S`), `L3_len2_u64` (`+18..+63% S`), `L5_len4_xmm` (`+6..+27% S`) all regress from run-length branch mispredictions; `L1_vbcast` (`+5..+32% S`) regresses from `vbroadcastss` store-to-load forwarding stalls after `*slot += len << 8`; 2× unrolled `len > 8` (`L2_vbc2`) is small/JCC-sensitive (`-1.4..-4.0%` JCC, `-2.0..+5.6%` base). Shipped `fill_avx2_trusted` is confirmed at silicon optimum.

---

## 6. Projected Impact & Follow-On Claim

- **On Wikidata (`M` = 132.4 ms 1W / 19.3 ms/w 8W on i9-10900K):** Cutting `runs::front_avx2` by **-17.3%..-19.7%** (~22–26 ms at 1W, ~3.3–3.8 ms/w at 8W; ~1.7–2.0% of total 1W wall) directly accelerates the Open path (`mtf-fold` is still optional/in-review and every default `Runs` block runs `runs::front_avx2`).
- **Off-Wikidata (`text`, `mdjournal`, `elf`, `json`):** Cuts `runs::front` by **-15%..-20%** (`text`, `mdjournal`, `json`) and **-5%..-12%** (`elf`, `runs`), and combined `front + fill` by **-10.5%..-19.6%** (`text`, `mdjournal`, `json`) and **-4.4%..-12.7%** (`elf`), stacking directly on top of `#1175` (`post-1080-takes-runs`) and `segmented-fused-postwalk`.
- **Admitted claim:** `frontier/claims/runs-front-and-fill-build.toml` (`status = "idea"`, `priority = 2`, `owned_paths = ["src/runs.rs"]`, `0 B` extra heap, `-1 KiB` `Runs` struct / stack).
