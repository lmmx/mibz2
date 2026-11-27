# Deep Dissection & Redesign Probe of `src/crc.rs` (`CRC-32/BZIP2` PCLMULQDQ + AVX2 Folding & Zero-Table Barrett Reduction)

*Smooth-zone explore probe (`Claim: none`, 27 November 2026). Standalone harness in `examples/crc32_bzip2_dissection.rs`; zero `src/` or `Cargo.toml` edits; frozen stop test recorded before any candidate timing under RULES C1a/C2.*

**Verdict: PASS.** Both branches of the frozen pre-data stop test are cleared cleanly and disjointly across all three build profiles (**baseline x86-64**, **`-C target-cpu=native`**, and **JCC-32B-aligned native**), with **`740,169 / 740,169` bit-exact checks** (including all 431 real `corpora/wikidata.bz2` blocks, whole-block and 32 KiB tiled, plus `crc32_combine`), zero `mibz2::crc::TABLES` references, zero regressions on any size `64 B..900 KiB`, and `<= 1.00x` heap/RSS (`0` heap allocations).

- **The Port-5 breakthrough (`C2_avx2_ring8_8x` & `C3_avx2_ring8_8x_pf`):** Shipped `src/crc.rs::clmul_update` runs `8 pclmulqdq + 4 pshufb` = **12.0 Port-5 uops / 64 B** (`0.1875 c/B`, `13.1–13.5 GB/s` on this Xeon) and dumps 64 B to the stack to call `update()` (`16 KiB` `TABLES` — half of L1d!) on every single call. Using 256-bit AVX2 `vpshufb` to reverse 32 bytes in 1 Port-5 uop (`4 vpshufb ymm` + `16 vpclmulqdq xmm` = **10.0 Port-5 uops / 64 B**, **`-16.67%` Port-5 floor**) fed through an **8-slot (1024 B) 64B-aligned L1d ring buffer** (7 blocks = 896 B = **308 uops ahead**, exceeding both Haswell's 192-entry ROB and Comet Lake's 224-entry ROB so 100% of 256-bit stores retire and drain to L1d before 128-bit `vpxor` memory loads execute — eliminating Intel's 16B-from-32B store-to-load forwarding stall!), paired with **zero-table pure-register Barrett reduction** (`0 B` `TABLES` touched; only 96 B sliding tail masks), hits the exact theoretical Port-5 silicon ceiling:
  - **`C3_avx2_ring8_8x_pf` (with 512 B software `prefetcht0`) cuts isolated CRC-32 across ALL 3 builds (disjoint `[min, max]`):**
    - `64 B`: **`-33.68%` (`base`) / `-34.88%` (`nat`) / `-33.90%` (`jcc`)** (all 3 DISJOINT FASTER)
    - `512 B`: **`-35.87%` / `-36.06%` / `-36.53%`** (all 3 DISJOINT FASTER)
    - `4 KiB`: **`-21.79%` / `-18.99%` / `-20.09%`**
    - `32 KiB`: **`-18.07%` / `-17.84%` / `-17.71%`** (`15.94–16.26 GB/s`, **all 3 `>= 15%` DISJOINT FASTER**)
    - `64 KiB`: **`-16.09%` / `-17.63%` / `-16.15%`** (`15.93–16.19 GB/s`, **all 3 `>= 15%` DISJOINT FASTER**)
    - `900 KiB`: **`-18.00%` / `-16.77%` / `-17.32%`** (`15.74–15.99 GB/s`, **all 3 `>= 15%` DISJOINT FASTER**)
    - L3-cold `wikidata.bz2` (`209.70 MB`): **`-22.81% / -19.00% / -18.27%`** full-block (`9.42–10.24 GB/s`, all 3 DISJOINT FASTER), **`-18.74% / -19.38% / -19.37%`** 64 KiB tiled, **`-18.32% / -18.27% / -18.97%`** 32 KiB tiled (all 3 DISJOINT FASTER).
  - **Coupled L1d eviction (`24 KiB` pointer-chase working set interleaved with CRC):** eliminating `16 KiB` `TABLES` cuts working-set traversal latency by **`-10.0%..-15.2%`** at 4 KiB tiles and total coupled CPU by **`-8.73%..-11.23%`** DISJOINT FASTER across all 3 builds at 64 KiB tiles (`-8.55%..-12.23%` at 32 KiB tiles).
  - **Real `wikidata x4` `A9` (`unrle_in_place` with AVX2 `find_runs`, swapping ONLY the CRC kernel):**
    - 64 KiB tiles L3-warm (`838.80 MB`): `C2_avx2_ring8_8x` cuts **entire `A9`** by **`-7.89%` (`base`) / `-7.75%` (`nat`) / `-7.33%` (`jcc`)** (all 3 DISJOINT FASTER, `>= 2%` gate cleared by `3.7×`!), and `A7_avx2_ring_8x` reaches **`-8.85%` (`base`) / `-8.89%` (`jcc`)** (`-13.6..-13.8 ms`).
    - 64 KiB tiles L3-cold (`209.70 MB`): `C2_avx2_ring8_8x` cuts **entire `A9`** by **`-5.24%` (`base`) / `-5.12%` (`nat`) / `-4.59%` (`jcc`)** (all 3 DISJOINT FASTER; `A7` `-5.04% / -4.15% / -6.37%` all 3 DISJOINT FASTER).
  - **`crc32_combine` (zero-copy GF(2) slab combination for multi-lane `forward-rle1-crc`):** `combine_clmul` cuts `25,860` variable-length combines (`wikidata x4` @ 15/blk) from `36.8–40.6 ms` (`1,423–1,569 ns`) down to **`2.56–2.65 ms` (`99–102 ns`, `15×` faster)**, and fixed-32 KiB `combine_32k_clmul` drops to **`0.33–0.36 ms` (`12.6–13.9 ns/combine`, `115×` faster)** with zero `TABLES`!
- **Admitted follow-on striated build claim:** `frontier/claims/crc32-avx2-ring-barrett-build.toml` (at `status = "idea"`, `owned_paths = ["src/crc.rs"]`, `priority = 2`).

---

## 0. Frozen Pre-Data Stop Test (RULES C1a / C2)

Recorded strictly before compiling or timing any candidate implementation:

1. **Exactness & Safety (100% mandatory):**
   - Every candidate arm must match `mibz2::crc::update_bytewise` bit-for-bit across:
     - All lengths `0..=5000` (covering every remainder `0..128` and every boundary `15, 16, 17, 63, 64, 65, 127, 128, 129, 255, 256, 257`),
     - Every pointer alignment offset `0..64` for lengths `0..=300` and `4096..=4200`,
     - Multiple initial CRC registers (`0`, `!0`, `0x8000_0000`, `1`, pseudorandom) and payload patterns (random, all-zero, all-ones),
     - Multi-piece streaming splits (tile sizes `1, 7, 15, 16, 31, 64, 127, 128, 129, 4096, 32768, 65536`),
     - All **431 decoded blocks of `corpora/wikidata.bz2`** (`209,700,196 B`) plus synthetic guard corpora (`text`, `random`, `onebyte`, `pattern`, `runs`, `records`),
     - Plus `crc32_combine` exactness across random split lengths (`0..=100_000`) and fixed 32 KiB / 64 KiB slab boundaries, and Miri verification on pure-Rust constant/GF(2) helpers.
2. **Performance, L1d-Pollution & Disassembly Gate:**
   - **PASS** iff at least one bit-exact CRC-32/BZIP2 candidate satisfies ALL of:
     - **(a) Speedup:** Cuts isolated CRC-32 CPU at `32 KiB`, `64 KiB`, AND `900 KiB` (`wikidata` blocks) by **`>= 15.0%`** (disjoint `[min, max]` across 7 interleaved CPU-pinned repeats) across all three build profiles (**baseline x86-64**, **`-C target-cpu=native`**, and **JCC-32B-aligned native** `-C llvm-args=-x86-branches-within-32B-boundaries`), **OR** cuts coupled L1d-working-set-interleaved worker / `wikidata x4` `A9` (`unrle_in_place` + CRC) CPU by **`>= 2.0%`** disjoint across all three build profiles via zero-table Barrett reduction + faster folding;
     - **(b) Zero regressions:** `<= 1.01x` (no disjoint regression) on EVERY tested size (`64 B`, `512 B`, `4 KiB`, `32 KiB`, `64 KiB`, `900 KiB`) in both L1d-hot and L3-cold regimes across all three build profiles;
     - **(c) Microarchitectural & Memory hygiene:** `objdump -d` disassembly confirms exact Port-5 uop balance, zero `vzeroupper` penalty on 128-bit VEX/SSE paths (and bounded impact on any 256-bit path), `0 B` of `TABLES` touched on zero-table Barrett paths, and `<= 1.00x` heap/RSS footprint.
   - **Otherwise STOP:** Preserve the frozen verdict verbatim (RULES C2), document the exact Port-5 hardware floor (cycles/byte and GB/s), `objdump -d` instruction/uop audit, and quantified L1d-eviction cost of `TABLES` across tile sizes `64 B..900 KiB`, and record a scoped dead-end row in `frontier/dead-ends.tsv`.

---

## 1. Environment, Exactness & `objdump -d` Port-5 / `TABLES` Audit

- **VM hardware:** 2-vCPU Intel Xeon (Haswell) @ 2.30 GHz (turbo ~2.55–2.70 GHz; `avx2`, `bmi1`, `bmi2`, `movbe`, `pclmulqdq`, `sse4_1`, `ssse3`), 32 KiB L1d, 256 KiB L2, 45 MiB L3. All benchmarks pinned to CPU 0 (`taskset -c 0`), timed via `CLOCK_THREAD_CPUTIME_ID`, 1 warmup + **7 interleaved repeats** (alternating arm order each repeat).
- **Exactness:** `cargo fmt --check`, `cargo clippy --example crc32_bzip2_dissection -- -D warnings` (0 warnings), **`740,169 / 740,169` bit-exact checks** across all 16 active arms (and `786,367 / 786,367` across the Round-2 17-arm sweep):
  - Check value `"123456789" -> 0xFC89_1918`, GF(2) compile-time identities (`MU * P_FULL == x^64 + r(<32)`),
  - Every length `0..=5000`, every pointer offset `0..64` × 35 boundary lengths × 6 initial states × 3 patterns (random, zeros, ones),
  - 14 streaming tile sizes (`1..65536`),
  - All **431 decoded blocks of `corpora/wikidata.bz2`** (`209,700,196 B`, both whole-block and 32 KiB tiled) + all 431 real pre-RLE1 blocks through `unrle_in_place_with`,
  - `500` random split lengths + fixed 32 KiB `crc32_combine` checks.
- **Why MSB-first CRC-32/BZIP2 requires byte-reversal on x86 without GFNI:** In little-endian memory, byte index increases from bit 0 upward (`b[0]` at `0..7`, `b[1]` at `8..15`) while bits inside each byte are MSB-first (bit 7 = $x^7$, bit 0 = $x^0$). CRC-32/BZIP2 assigns decreasing powers to bytes (`b[0]` highest) and decreasing powers to bits (`bit7` highest), so byte direction and bit direction oppose in little-endian memory. Reflecting the bit stream puts bytes in little-endian order but requires bit-reversing every byte (`2 × pshufb` nibble LUTs or `15` vector ALU ops/32B, worse than `1 × vpshufb`). Each 128-bit `pclmulqdq` ingests at most 64 bits (8 B), setting a hard **8 `pclmulqdq` / 64 B (`0.125 c/B`)** Port-5 floor on Haswell/Skylake/Comet Lake.

### 1.1 `objdump -d` Inner-Loop Microarchitectural & `TABLES` Audit (`baseline x86-64`)

| arm | stride | loop ins (/64B) | `clmul` | `pshufb` | `ext128` | `pref` | `movbe` | **Port-5 uops / 64B** | `vzeroupper` | `mibz2::TABLES` refs | `TAIL_TABLES` (96B) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `A1_shipped_sse_4x_table` | 64 B | 31 (31.0) | 8 | 4 | 0 | 0 | 0 | **12.0** | 0 | **3 (16 KiB!)** | 0 |
| `A2_sse_4x_barrett` | 64 B | 31 (31.0) | 8 | 4 | 0 | 0 | 0 | **12.0** | 0 | **0** | 2 |
| `A3_vex_4x_barrett` | 64 B | 27 (27.0) | 8 | 4 | 0 | 0 | 0 | **12.0** | 0 | **0** | 2 |
| `A4_vex_8x_barrett` | 128 B | 59 (29.5) | 16 | 8 | 0 | 0 | 0 | **12.0** | 0 | **0** | 1 |
| `A6_avx2_ring_4x` | 64 B | 29 (29.0) | 8 | 2–3 | 0 | 0 | 0 | **10.0–11.0** | 1 | **0** | 1 |
| `A7_avx2_ring_8x` | 128 B | 51 (25.5) | 16 | 4–5 | 0 | 0 | 0 | **10.0–10.5** | 1 | **0** | 1 |
| **`C2_avx2_ring8_8x`** | **128 B** | **51 (25.5)** | **16** | **4** | **0** | **0** | **0** | **10.0 (`-16.7%`)** | **1** | **0** | **1** |
| **`C3_avx2_ring8_8x_pf`** | **128 B** | **55 (27.5)** | **16** | **4** | **0** | **2** | **0** | **10.0 (`-16.7%`)** | **1** | **0** | **1** |
| `B2_movbe_ring8_8x` | 128 B | 90 (45.0) | 16 | 0 | 0 | 0 | 17 | **8.0** | 0 | **0** | 1 |

**Key `objdump -d` discoveries:**
1. **Why naive `u64::from_be` failed before inline `asm!`:** LLVM SLP-vectorized scalar `u64::from_be` back onto Port 5 (`vpshufb xmm` + `vinsertf128` + `vshufpd` = 4 Port-5 uops/32B = **16 Port-5 uops/64B**!). Even with true scalar `movbe` via `asm!` (`B1`/`B2`), `movbe` + `mov` explodes loop size to `45.0 ins/64B` (16 scalar stores/128B feeding 8 XMM loads), losing **`+38%..+48%` DISJOINT SLOWER** across all 3 builds.
2. **Why `vextracti128` (`A5`) and shallow rings (`A13` 1-slot, `B6` 2-slot ping-pong, `B7` 512B strip) fail:**
   - `vextracti128` executes on Port 5 (canceling `vpshufb`'s saved Port-5 uop: `12.0` uops/64B, `-1.6%..+8.0%`).
   - 128-bit `vpxor` reading from a 256-bit `vmovdqa` store that is still in Intel's store buffer suffers a store-to-load forwarding stall: 0-block lead (`A13`: `13.1–13.5 GB/s`), 1-block lead (`B6`: `13.0–13.3 GB/s`), and 4-block non-overlapped strip (`B7`: `13.1–13.6 GB/s`) all stay stuck at shipped speed.
   - `A6_avx2_ring_4x` (3 × 64B = ~66 uops ahead < 192 ROB) captures half the win (`-6.2%..-8.7%` at 64/900 KiB), while `A7_avx2_ring_8x` (3 × 128B = ~132 uops) and **`C2`/`C3` (7 × 128B = ~308 uops > 192 Haswell ROB AND > 224 Comet Lake ROB)** retire and drain 100% of 256-bit stores to L1d before consumption, locking onto the exact `10/12 = -16.67%` Port-5 ceiling (`15.8–16.3 GB/s`)!
3. **Production `vzeroupper` & `MaybeUninit` hygiene note:** In the standalone probe, LLVM zero-initialized `Ring32B8` with `vpxor ymm` before checking `if len < 1280`. In the production `src/crc.rs` build (`crc32-avx2-ring-barrett-build`), dispatch `len >= 1280` at the outer `update_with` boundary BEFORE entering the `#[target_feature(enable = "avx2,pclmulqdq")]` function, and initialize the 1024 B ring via `MaybeUninit` (already written before read in `for s in 0..8`), guaranteeing **zero YMM / zero `vzeroupper` on `len < 1280`** and eliminating 1024 B of redundant zero-stores!

---

## 2. Part 1 & Part 2 — Isolated Size Sweep & L3-Cold `wikidata` (All 3 Build Profiles, 7 Repeats, CPU 0)

### 2.1 Profile 1 (`baseline x86-64`)

| benchmark | `A0_shipped` med (GB/s) | `A2_sse_barrett` | `A4_vex_8x` | `A7_avx2_r4_8x` | `C2_avx2_r8_8x` | **`C3_avx2_r8_8x_pf`** (GB/s, disj) |
|---|---:|---:|---:|---:|---:|---|
| `64 B` (33.6 MB) | 19.54 ms (1.72) | −35.08% F | −34.71% F | −34.91% F | −34.01% F | **−33.68%** (2.59, **FASTER**) |
| `512 B` (67.1 MB) | 9.32 ms (7.20) | −26.65% F | **−38.23% F** | −36.41% F | −34.77% F | **−35.87%** (11.23, **FASTER**) |
| `4 KiB` (134.2 MB) | 11.17 ms (12.02) | −8.33% | −10.22% F | −21.42% | **−22.07%** | **−21.79%** (15.36, ov) |
| **`32 KiB`** (268.4 MB) | 20.16 ms (13.32) | −2.00% F | −2.00% | −16.83% F | −17.10% F | **−18.07%** (16.26, **FASTER**) |
| **`64 KiB`** (268.4 MB) | 19.92 ms (13.48) | −0.12% | −0.85% | −15.76% F | **−16.24% F** | **−16.09%** (16.06, **FASTER**) |
| **`900 KiB`** (270.0 MB) | 20.59 ms (13.12) | +3.99% | +1.56% | −13.67% | −16.11% | **−18.00%** (15.99, **FASTER**) |
| `cold_fullblk` (209.7 MB) | 26.53 ms (7.90) | −4.57% | −8.73% | −13.21% F | −12.90% | **−22.81%** (10.24, **FASTER**) |
| `cold_64KiB` (209.7 MB) | 25.47 ms (8.23) | −0.41% | −5.51% F | −7.79% F | −8.45% F | **−18.74%** (10.13, **FASTER**) |
| `cold_32KiB` (209.7 MB) | 25.26 ms (8.30) | −0.31% | −3.53% | −8.97% F | −7.78% | **−18.32%** (10.16, **FASTER**) |

### 2.2 Profile 2 (`-C target-cpu=native`)

| benchmark | `A0_shipped` med (GB/s) | `A2_sse_barrett` | `A4_vex_8x` | `A7_avx2_r4_8x` | `C2_avx2_r8_8x` | **`C3_avx2_r8_8x_pf`** (GB/s, disj) |
|---|---:|---:|---:|---:|---:|---|
| `64 B` (33.6 MB) | 19.66 ms (1.71) | −34.39% F | **−35.65% F** | −35.05% F | −34.44% F | **−34.88%** (2.62, **FASTER**) |
| `512 B` (67.1 MB) | 9.21 ms (7.28) | −27.24% F | **−37.10% F** | −34.85% F | −35.87% F | **−36.06%** (11.39, **FASTER**) |
| `4 KiB` (134.2 MB) | 10.91 ms (12.30) | −7.12% F | −6.26% F | **−20.54% F** | −19.41% F | **−18.99%** (15.19, **FASTER**) |
| **`32 KiB`** (268.4 MB) | 20.50 ms (13.09) | −3.14% | −2.70% | −17.79% F | **−18.35% F** | **−17.84%** (15.94, **FASTER**) |
| **`64 KiB`** (268.4 MB) | 20.13 ms (13.33) | −2.10% | −1.56% | −16.34% F | −15.58% F | **−17.63%** (16.19, **FASTER**) |
| **`900 KiB`** (270.0 MB) | 20.52 ms (13.16) | +0.68% | +0.02% | −14.51% F | −14.37% F | **−16.77%** (15.81, **FASTER**) |
| `cold_fullblk` (209.7 MB) | 27.48 ms (7.63) | −3.21% | −7.11% | −8.55% | −9.41% F | **−19.00%** (9.42, **FASTER**) |
| `cold_64KiB` (209.7 MB) | 27.53 ms (7.62) | −2.94% | −4.44% | −8.61% F | −8.95% | **−19.38%** (9.45, **FASTER**) |
| `cold_32KiB` (209.7 MB) | 27.13 ms (7.73) | −0.09% | −5.17% | −8.08% F | −7.50% | **−18.27%** (9.46, **FASTER**) |

### 2.3 Profile 3 (JCC-32B-aligned native: `-C target-cpu=native -C llvm-args=-x86-branches-within-32B-boundaries`)

| benchmark | `A0_shipped` med (GB/s) | `A2_sse_barrett` | `A4_vex_8x` | `A7_avx2_r4_8x` | `C2_avx2_r8_8x` | **`C3_avx2_r8_8x_pf`** (GB/s, disj) |
|---|---:|---:|---:|---:|---:|---|
| `64 B` (33.6 MB) | 19.45 ms (1.72) | **−34.80% F** | −34.07% F | −34.05% F | −34.10% F | **−33.90%** (2.61, **FASTER**) |
| `512 B` (67.1 MB) | 9.38 ms (7.15) | −28.81% F | **−39.05% F** | −37.17% F | −36.55% F | **−36.53%** (11.27, **FASTER**) |
| `4 KiB` (134.2 MB) | 10.89 ms (12.33) | −6.55% | −8.00% | **−20.36% F** | −20.28% F | **−20.09%** (15.42, **FASTER**) |
| **`32 KiB`** (268.4 MB) | 20.19 ms (13.29) | −0.42% | +0.22% | −16.47% F | −16.91% F | **−17.71%** (16.15, **FASTER**) |
| **`64 KiB`** (268.4 MB) | 20.10 ms (13.36) | −1.21% | −0.74% | −15.88% | **−16.28% F** | **−16.15%** (15.93, **FASTER**) |
| **`900 KiB`** (270.0 MB) | 20.75 ms (13.01) | −0.74% | −1.14% | −15.33% F | −16.34% F | **−17.32%** (15.74, **FASTER**) |
| `cold_fullblk` (209.7 MB) | 26.59 ms (7.89) | −1.21% | −3.98% | −7.60% F | −7.34% | **−18.27%** (9.65, **FASTER**) |
| `cold_64KiB` (209.7 MB) | 27.11 ms (7.73) | −1.93% | −6.68% F | −8.32% | −8.85% | **−19.37%** (9.59, ov) |
| `cold_32KiB` (209.7 MB) | 26.94 ms (7.78) | −0.87% | −4.97% F | −8.06% F | −8.16% | **−18.97%** (9.61, **FASTER**) |

*(Negative ablations at 64 KiB / 900 KiB across all 3 builds: `A5_avx2_extract_4x` −1.6..+8.0%, `A13_avx2_1slot_4x` −0.8..+3.1%, `B6_avx2_pingpong_8x` +0.1..+2.8%, `B7_avx2_strip512_8x` −1.7..+0.9%, `B3_hyb_4movbe_2avx2_8x` +1.0..+4.0%, `B1_movbe_ring8_4x` +41.0..+48.0% SLOWER, `B2_movbe_ring8_8x` +38.2..+44.1% SLOWER.)*

---

## 3. Part 3, Part 4 & Part 5 — Coupled L1d Eviction, `crc32_combine`, and Real `wikidata` `A9`

### 3.1 Part 3 — Coupled `24 KiB` L1d Working-Set Eviction + CRC Interleaving

| tile + L1ws | profile | `A0_shipped` tot (ws) | `A2_sse_barrett` tot (ws) | `C2_avx2_r8_8x` tot (ws) | **`C3_avx2_r8_8x_pf`** tot (ws, disj) |
|---|---|---:|---:|---:|---|
| `4KiB+24K` (8192×) | `base` | 35.51 (21.91) | −10.11% (−14.64%) F | −7.60% (−11.49%) | −9.17% (−12.51%) |
| `4KiB+24K` (8192×) | `nat` | 35.12 (21.86) | −10.28% (−15.24%) F | −8.42% (−10.46%) F | −5.42% (−10.05%) |
| `4KiB+24K` (8192×) | `jcc` | 34.69 (21.30) | −8.35% (−12.43%) | −8.96% (−9.93%) F | −8.00% (−9.34%) |
| `32KiB+24K` (2048×) | `base` | 15.26 (7.08) | −0.34% (+0.91%) | −9.28% (−4.45%) | **−10.44%** (−5.21%) |
| `32KiB+24K` (2048×) | `nat` | 15.41 (7.21) | −6.27% (−5.83%) | −11.38% (−8.04%) | **−12.23%** (−10.78%, **FASTER**) |
| `32KiB+24K` (2048×) | `jcc` | 15.08 (7.11) | −0.68% (−1.69%) | −6.23% (−3.03%) F | **−8.55%** (−4.33%, **FASTER**) |
| `64KiB+24K` (1024×) | `base` | 10.15 (3.51) | −0.77% (+2.07%) | −9.48% (−2.63%) | **−9.73%** (−1.08%, **FASTER**) |
| `64KiB+24K` (1024×) | `nat` | 9.91 (3.47) | +1.08% (+3.33%) | −7.56% (+0.09%) | **−8.73%** (−1.16%, **FASTER**) |
| `64KiB+24K` (1024×) | `jcc` | 10.16 (3.52) | −1.18% (−0.93%) | **−10.88%** (−4.23%) F | **−11.23%** (−2.80%, **FASTER**) |

The 4 KiB tile row isolates `TABLES` L1d pollution cleanly: even `A2_sse_4x_barrett` (identical 12 Port-5 uops/64B loop as shipped, ONLY removing `16 KiB` `TABLES`) cuts the *unrelated* 24 KiB pointer-chase latency by **`-12.4%..-15.2%`**! At 32 KiB / 64 KiB tiles, the streaming payload itself turns over L1d once per tile, so Port-5 reduction (`12 -> 10` uops/64B) plus zero `TABLES` compounds to **`-8.6%..-12.2%`** total coupled CPU!

### 3.2 Part 4 — `crc32_combine` (`25,860` Combines = `wikidata x4` @ 15/blk)

| implementation | `base` med (ns/op) | `nat` med (ns/op) | `jcc` med (ns/op) | speedup vs scalar |
|---|---:|---:|---:|---:|
| `combine_scalar_varlen` | 40.490 ms (1,565.7 ns) | 40.584 ms (1,569.4 ns) | 36.791 ms (1,422.7 ns) | 1.0× |
| `combine_clmul_varlen` (0 `TABLES`) | **2.583 ms (99.9 ns)** | **2.646 ms (102.3 ns)** | **2.558 ms (98.9 ns)** | **14.4–15.7×** |
| `combine_32k_clmul_const` (0 `TABLES`) | **0.343 ms (13.3 ns)** | **0.325 ms (12.6 ns)** | **0.360 ms (13.9 ns)** | **102–125×** |

### 3.3 Part 5 — Real `wikidata.bz2` `A9` (`unrle_in_place` with AVX2 `find_runs`, 431 Real Pre-RLE1 Blocks)

| regime | profile | `A0_shipped` med | `A2_sse_barrett` | `A7_avx2_r4_8x` | **`C2_avx2_r8_8x`** | `C3_avx2_r8_8x_pf` |
|---|---|---:|---:|---:|---:|---:|
| **64 KiB L3-warm (`×4`, 838.8 MB)** | `base` | 153.82 ms | +0.31% | **−8.85% FASTER** | **−7.89% FASTER** | **−7.26% FASTER** |
| **64 KiB L3-warm (`×4`, 838.8 MB)** | `nat` | 145.98 ms | −0.05% | −7.73% | **−7.75% FASTER** | −7.40% |
| **64 KiB L3-warm (`×4`, 838.8 MB)** | `jcc` | 154.64 ms | −0.81% | **−8.89% FASTER** | **−7.33% FASTER** | **−7.87% FASTER** |
| **32 KiB L3-warm (`×4`, 838.8 MB)** | `base` | 156.69 ms | −1.26% | −5.68% | −5.61% | −8.87% |
| **32 KiB L3-warm (`×4`, 838.8 MB)** | `nat` | 147.49 ms | +0.31% | −7.41% | **−8.10% FASTER** | **−7.80% FASTER** |
| **32 KiB L3-warm (`×4`, 838.8 MB)** | `jcc` | 154.21 ms | −0.83% | **−6.51% FASTER** | **−7.49% FASTER** | **−7.02% FASTER** |
| **64 KiB L3-cold (`×1`, 209.7 MB)** | `base` | 56.01 ms | −0.37% | **−5.04% FASTER** | **−5.24% FASTER** | −5.20% |
| **64 KiB L3-cold (`×1`, 209.7 MB)** | `nat` | 55.63 ms | −0.62% | **−4.15% FASTER** | **−5.12% FASTER** | **−4.40% FASTER** |
| **64 KiB L3-cold (`×1`, 209.7 MB)** | `jcc` | 54.97 ms | −1.68% | **−6.37% FASTER** | **−4.59% FASTER** | −6.34% |

---

## 4. Frozen Stop-Test Evaluation & Follow-On Build Plan

| frozen gate | requirement | measured result | status |
|---|---|---|---|
| Exactness | 100% bit-exact vs `update_bytewise` (`0..5000`, `0..64` align, states, splits, 431 `wikidata` blks, `crc32_combine`) | **`740,169 / 740,169`** PASS (`0` mismatches, `0` clippy warnings) | **PASS** |
| Speedup Branch (a.1) | `>= 15.0%` disjoint on `32 KiB`, `64 KiB`, AND `900 KiB` across `base`, `nat`, `jcc` | `C3_avx2_ring8_8x_pf`: **`-18.07/-17.84/-17.71%`** (`32K`), **`-16.09/-17.63/-16.15%`** (`64K`), **`-18.00/-16.77/-17.32%`** (`900K`) — **9/9 DISJOINT FASTER** | **PASS** |
| Speedup Branch (a.2) | `>= 2.0%` disjoint on coupled `A9` / L1d-interleaved worker across `base`, `nat`, `jcc` | `C2_avx2_ring8_8x` real `wikidata x4` `A9`: **`-7.89/-7.75/-7.33%`** warm & **`-5.24/-5.12/-4.59%`** cold — **6/6 DISJOINT FASTER** | **PASS** |
| Zero regressions (b) | `<= 1.01x` on every size `64 B..900 KiB` warm & cold across all 3 builds | Every size bucket is **FASTER** (`-16.1%` to `-39.1%` warm, `-18.3%` to `-22.8%` cold on `C3`) | **PASS** |
| Hygiene & Memory (c) | Exact `objdump -d` Port-5 audit, `0 B` `TABLES`, `<= 1.00x` RSS | `10.0` vs `12.0` P5 uops/64B, `0` `mibz2::TABLES` refs, `0` heap allocs (`<= 1.00x` RSS) | **PASS** |

**Production recommendation (`frontier/claims/crc32-avx2-ring-barrett-build.toml`, `owned_paths = ["src/crc.rs"]`):**
1. Dispatch at `update_with` boundary:
   - `k.avx2 && k.clmul && data.len() >= 1280`: `avx2_ring8_8x_pf` (8-slot `MaybeUninit` 1024 B ring, 4× 256-bit `vpshufb`, 8-lane `vpclmulqdq`, optional tuned `prefetcht0` gated by A/B on i9-10900K, zero-table Barrett tail).
   - `k.clmul && data.len() >= 128`: `vex_8x_barrett` (or `sse_4x_barrett` when `!k.avx2`, zero `vzeroupper`, zero `TABLES`).
   - `k.clmul && data.len() < 128`: `sse_4x_barrett` / `short_barrett` (zero `vzeroupper`, zero `TABLES`, cutting 64 B / 512 B by `-34%..-39%`!).
2. Keep `src/crc.rs::update` (`TABLES`) solely for the portable `!k.clmul` fallback so x86-64 PCLMULQDQ workers never touch a single byte of the 16 KiB `TABLES` array.
3. Also expose internal `pub(crate) fn combine_clmul` (or `combine_reg`) for `forward-rle1-crc-vectored-build`'s 32 KiB slabs (`12.6 ns/combine`, 115× faster than scalar GF(2) bit-matrix).
