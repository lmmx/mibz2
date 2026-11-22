# Cycle 55 P3 — `C55-UltraLean-v1`: Orthogonal Super-Fusion (`CS-48 + CS-50`) & Surgical Slimming

**Champion Drop-In `#729` Candidate:** [`cand_c55_ultralean_v1_moonshot.rs`](./cand_c55_ultralean_v1_moonshot.rs) (`pub type Candidate = C55UltraLeanV1Moonshot`, `#![forbid(unsafe_code)]`)  
**20-Arm Standalone Ablation Harness:** [`c55_ultralean_ablation.rs`](./c55_ultralean_ablation.rs) (`#![forbid(unsafe_code)]`)

---

## 1. Executive Summary

Cycle 55 P3 cleanly fuses Cycle 54's two `#729` champions—**`C54-PipeLean-v2` (`CS-48`)** (`instr = 0.757`, `W=1 = 0.889`, `W=2 = 0.901`) and **`C54-DramLean-v1` (`CS-50`)** (`L2 = 24.6 B/B`, `L3 = 13.7 B/B`, `W=8 DRAM = 0.98 B/B`, `W=10 DRAM = 4.60 B/B`, `PeakCap = 960.00 KiB`)—isolates every single sub-mechanism via 20-arm single-feature ablation, kills 5 non-improving mechanisms under Rule 5, and adds two strictly verified Cycle 55 modern-CS wins:

1. **`S1_Ultra` (`CS-54` = `S1_SplitP` + `S1_AtPacked` + `S1_HoistOut`):**
   - Splits `stage1_scan` at primary index `p` (Phase A checks `next_pos <= p`; Phase B strictly after `p` eliminates 100% of `p` checks on both thin and thick records).
   - Packs `(thin:32 | f:11 | at:21)` in `pt[a]: u64` (`f <= 496` compiles to single `and $0xffe00000; cmp $0x3e1fffff`; `at = v & 0x1F_FFFF` eliminates duplicate `p + f` adds; single `v + wu * ((1 << 32) | (1 << 21) | 2)` updates all 3 fields at once).
   - Hoists `out_u: &mut [u8]` and `m: u32` out of `Pass`/`Queues` behind an `#[inline(never)]` `stage1_scan::<LIM>` boundary so `out.as_ptr()`, `out.len()`, and `m` stay in GPRs with zero per-record stack reloads.
   - **Standalone impact (`F02` -> `S08`):** **`-1,639,661 Ir` (`-9.87%` of `S1`)**, **`-6.3%..-6.7%` faster `S1` Xeon wall time** (`proxy6` `213.5 -> 200.0 us`, `wiki6` `294.7 -> 274.9 us`).
2. **`S2aColdGrow` + Stage-Boundary Split (`CS-55`):**
   - Outlines cold `grow_edges_exact` inside `stage2a_build` (`use_zero_hdr == true`) right before `edges.split_at_mut(nodes_words)`, and separates `stage2a_build` / `stage3a_bands` into `#[inline(never)]` register scopes, eliminating cross-stage stack spills (**`-93,398 Ir`** in ablation, **`-132,900 Ir`** + **`-80k Dw`** on the drop-in file).
3. **`L = 12` vs `StackU16` Root-Cause Diagnosis (`U32Edges` Retained):**
   - Phase 1 proved **`L = 12` drives `99.8%` of `C54-DramLean-v1`'s L2/L3/DRAM/PeakCap win** (`PeakCap = 960.02 KiB` identically for `U32Edges`, `StackU16`, and `StackU32` — `U16Dict` saved `0.00 KiB` `PeakCap`!).
   - Retaining `edges`-reused `U32Edges` (`Dictionary` with 7-bit hash tags = `0.78%` false-positive rate vs `12.5%` for 3-bit `u16`, zero `contents_u32_fb` fallback allocation, zero `6 KiB` extra stack frame) strictly beats `StackU16` at `L = 12` on **`W8_2.5M` (`-100` misses)**, **`W10_2.0M` (`-81` misses)**, and **`Ir` (`-59k` with `S2aColdGrow`)**!

---

## 2. Exact Drop-In File Head-to-Head (`proxy10` Warmed, `callgrind` `x86-64-v3`)

Compiled directly from the drop-in files (`cand_c54_pipelean_v2_moonshot.rs`, `cand_c54_dramlean_v1_moonshot.rs`, `cand_c55_ultralean_v1_moonshot.rs`) and verified **100% bit-identical across all `447/447` real blocks** (`431` wikidata + `6` admitted + `10` non-wikidata/code-docs):

| Drop-In File | `Ir` (`proxy10` warm) | `dIr vs PLv2` | `dIr vs DLv1` | `Dr` | `Dw` | `256K L2` (`DLmr+DLmw`) | `2.5M W=8` | `2.0M W=10` | `PeakCap (431 ALL)` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `C54-PipeLean-v2 (CS-48)` | `59,802,121` | `+0` | `-5,580,578` | `10,686,070` | `10,045,651` | `533,377` | `1,913` | `9,108` | `998.95 KiB` |
| `C54-DramLean-v1 (CS-50)` | `65,382,699` | `+5,580,578` | `+0` | `12,104,814` | `10,188,244` | `490,062` | `1,913` | `7,050` | `960.02 KiB` |
| **`C55-UltraLean-v1 (P3)`** | **`59,544,481`** | **`-257,640 (-0.43%)`** | **`-5,838,218 (-8.93%)`** | **`10,938,945`** | **`9,700,234`** | **`490,448 (-8.05%)`** | **`1,913`** | **`6,345 (-30.3% / -10.0%)`** | **`960.02 KiB (-38.93 KiB)`** |

---

## 3. Complete 20-Arm Single-Feature & Fusion Ablation (`c55_ultralean_ablation.rs`)

### 3.1 Deterministic `callgrind` (`proxy10` steady-state warmed `--warmup-all --reuse-out`, `x86-64-v3`)

| Arm | `Ir` | `dIr vs F00` | `dIr vs F02` | `Dr` | `Dw` | `L2_256K` (`dL2`) | `W8_2.5M` (`dW8`) | `W10_2M` (`dW10`) | Status |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `F00_PipeLean_v2` (`CS-48`, `L=16`) | `60,102,455` | `+0` | `---` | `9,763,483` | `9,990,834` | `534,930` (`+0`) | `6,106` (`+0`) | `12,101` (`+0`) | C54 `instr/W1/W2` base |
| `F00_DramLean_v1` (`CS-50`, `L=12`) | `64,676,143` | `+4,573,688` | `---` | `11,821,792` | `10,065,692` | `492,222` (`-42,708`) | `5,202` (`-904`) | `11,812` (`-289`) | C54 `L2/DRAM` base |
| `F01_PLv2_L12_U32Edg` | `60,714,786` | `+612,331` | `+34,337` | `10,164,434` | `10,024,643` | `492,603` (`-42,327`) | `5,044` (`-1,062`) | `11,785` (`-316`) | `L=12` isolates 99.8% of L2/DRAM |
| `F02_PLv2_L12_StkU16` | `60,680,449` | `+577,994` | `+0` | `10,153,451` | `9,956,583` | `492,406` (`-42,524`) | `5,055` (`-1,051`) | `11,834` (`-267`) | Exact `CS-48 + CS-50` base |
| `F02b_PLv2_L12_StkU32` | `60,562,843` | `+460,388` | `-117,606` | `10,106,822` | `9,979,424` | `493,392` (`-41,444`) | `5,289` (`-817`) | `11,905` (`-196`) | Higher `W8/L2` (+10 KB stack) |
| `F03b_PLv2_L11_StkU16` | `60,986,722` | `+884,267` | `+306,273` | `10,311,794` | `9,985,794` | `480,463` (`-54,373`) | `1,921` (`-4,185`) | `8,078` (`-4,023`) | Regresses `adm6` (`+816k Ir`, `+216 W8`) & Xeon |
| `F04b_PLv2_L13_StkU16` | `60,367,527` | `+265,072` | `-312,922` | `10,017,485` | `9,923,536` | `500,867` (`-33,969`) | `5,393` (`-713`) | `12,090` (`-11`) | Loses `L2/W10` DRAM win vs `L=12` |
| `S01_F02_S2aColdGrow` | `60,587,051` | `+484,596` | **`-93,398`** | `10,049,162` | `9,916,649` | `492,500` (`-42,430`) | `5,192` (`-914`) | `11,830` (`-271`) | **PROMOTED (`CS-55`)** |
| `S02_F02_S5Packed15` (v1 / v2) | `60,982,553` / `62,449,887` | `+880k` / `+2.35M` | `+302k` / `+1.77M` | `9.84M` / `11.12M` | `9.42M` / `9.97M` | `491,250` (`-43,586`) | `4,821` | `11,841` | **KILLED (Rule 5: `+302k..+1.77M Ir`, `+1.2%` Xeon)** |
| `S03_F02_S1SplitP` | `60,232,985` | `+130,530` | **`-447,464`** | `10,324,001` | `9,946,666` | `492,414` (`-42,422`) | `5,234` | `11,878` | **PROMOTED (in `S1_Ultra`)** |
| `S04_F02_S1AtPacked` | `60,412,792` | `+310,337` | **`-267,657`** | `10,140,227` | `9,976,956` | `492,406` (`-42,430`) | `5,077` | `11,871` | **PROMOTED (in `S1_Ultra`)** |
| `S05_F02_S1HoistOut` (solo) | `63,263,033` | `+3,160,578` | `+2,582,584` | `10,786,068` | `10,549,955` | `491,901` | `4,959` | `11,888` | **Solo KILLED (`+2.58M Ir`), synergizes with `S03+S04` for `-1.64M Ir`!** |
| `S06_F02_S2bChunk4` | `61,827,278` | `+1,724,823` | `+1,146,829` | `10,403,831` | `10,279,158` | `492,433` | `5,042` | `11,848` | **KILLED (Rule 5: `+1.15M Ir` in `S2b`)** |
| `S07_F02_S4HoistBnd` | `60,780,050` | `+677,595` | `+99,601` | `10,178,809` | `9,965,794` | `492,295` | `4,958` | `11,846` | **KILLED (Rule 5: `+100k Ir` in `S4`)** |
| `S08_Ultra_L12_StkU16` | `59,040,788` | `-1,061,667` | `-1,639,661` | `10,047,547` | `9,649,123` | `492,113` (`-42,817`) | `5,050` (`-1,056`) | `11,862` (`-239`) | Runner-up (`StkU16`) |
| `S09_Ultra_L12_U32Edg` | `59,075,125` | `-1,027,330` | `-1,605,324` | `10,058,530` | `9,717,183` | `492,242` (`-42,688`) | **`4,939` (`-1,167`)** | **`11,736` (`-365`)** | Best LLC (`U32Edg`) |
| **`S11_UltraCG_L12_U32`** | **`58,981,715`** | **`-1,120,740`** | **`-1,698,734`** | **`9,954,247`** | **`9,677,243`** | **`492,326` (`-42,604`)** | **`4,950` (`-1,156`)** | **`11,781` (`-320`)** | **CHAMPION (`C55-UltraLean-v1`)** |
| `S12_UltraPN_L12_U16` | `59,060,991` | `-1,041,464` | `-1,619,458` | `10,009,120` | `9,375,137` | `492,061` | `5,083` | `11,920` | **KILLED (Rule 5: `+20k Ir` & `+1.0%` Xeon S5 vs `S08`)** |

### 3.2 Uncontended Xeon `1W / 2W / 4W` Paired Wall Timings (native `-C target-cpu=native`, `F00_AA_Control = 0.996–1.006×`)

| Corpus | `F00_PipeLean_v2` | `F00_AA` | `F00_DramLean_v1` | `F02` (Naive Fusion) | **`S11_UltraCG_L12_U32` (`C55-UltraLean-v1`)** |
|---|---:|---:|---:|---:|---:|
| **`431 ALL` Wiki `1W / 2W / 4W`** | `276.9 / 281.8 / 571.7 ms` | `0.998 / 0.978 / 0.998×` | `1.086 / 1.060 / 1.078×` | `1.033 / 1.015 / 1.010×` | **`1.019 / 0.997 / 1.005×`** (`-6.2%..-6.8%` vs `DLv1`) |
| **`233 Full` Wiki `1W / 2W / 4W`** | `273.8 / 275.2 / 561.5 ms` | `0.999 / 1.000 / 0.996×` | `1.084 / 1.089 / 1.078×` | `1.023 / 1.033 / 1.024×` | **`1.012 / 1.020 / 1.019×`** (`-6.4%..-6.6%` vs `DLv1`) |
| **`6+4` Proxy `1W / 2W / 4W`** | `6.59 / 6.65 / 13.36 ms` | `1.006 / 0.995 / 0.981×` | `1.080 / 1.063 / 1.100×` | `1.037 / 1.033 / 1.022×` | **`1.026 / 1.030 / 1.014×`** (`-5.0%..-7.8%` vs `DLv1`) |
| **`6` Admitted `1W / 2W / 4W`** | `8.78 / 8.86 / 18.42 ms` | `0.996 / 0.995 / 0.993×` | `1.092 / 1.094 / 1.083×` | `1.046 / 1.032 / 1.009×` | **`1.031 / 1.029 / 1.036×`** (`-5.6%..-5.9%` vs `DLv1`) |
| **`6` Non-Wiki `1W / 2W / 4W`** | `5.33 / 5.25 / 10.87 ms` | `0.989 / 1.009 / 1.007×` | `0.987 / 1.020 / 1.003×` | `1.001 / 1.020 / 0.992×` | **`0.995 / 0.999 / 0.993×`** |
| **`4` Code & Docs `1W / 2W / 4W`** | `33.86 / 33.87 / 70.63 ms` | `1.003 / 1.001 / 1.018×` | `1.020 / 1.016 / 1.005×` | `0.965 / 0.969 / 0.960×` | **`0.963 / 0.965 / 1.000×`** (`-3.6%` vs `PLv2`) |

---

## 4. Exactness, Safety & Repo Hygiene Verification

- `#![forbid(unsafe_code)]`, `RUSTUP_TOOLCHAIN=stable rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`: **0 warnings, 0 errors**.
- `cand_c55_ultralean_v1_moonshot.rs` unit tests: **`11/11` passed in Release (`4.51s`) AND `11/11` passed in Overflow-Checked Debug (`-C debug-assertions=on -C overflow-checks=on`, `3.81s`)**, including `c55_s1_ultra_split_p_at_packed_and_l12_exact` testing every `p` boundary (`0, 1, 999, 1000, 1001, 1150, 1299, 1300, 65535, 65536, 131072, n-1`), multi-127 coalesced thick runs, and `S1_AtPacked` multi-chunk/overflow paths.
- `c55_ultralean_ablation.rs --exact-only` (Release & Overflow-Checked Debug): **`5,760` small valid + `330` malformed + `65` detached (`d = 1..=65`) + forced sentinel/u16fb + `3` stress + `447/447` real blocks = 100% EXACT across ALL 20 arms**.
- Direct drop-in head-to-head parity: **`447/447` real blocks 100% bit-identical** across `C54-PipeLean-v2`, `C54-DramLean-v1`, and `C55-UltraLean-v1`.
- Atlas classification: `CS-54` (`S1_Ultra`), `CS-55` (`S2aColdGrow + Stage-Split`), plus 5 honestly documented Rule-5 Kills (`S02`, `S05-solo`, `S06`, `S07`, `S12`) — strictly `2020s` modern CS, `0` classical relabels.
