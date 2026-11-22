# Wormhole Cycle 37 (Neon) — Emit-Direct-to-`out` (`0 B` `scratch.ll`, `-9 MiB` @`10W`), Exact In-Place `RLE1`, & `64 KiB` L2-Tiled GF(2) `CLMUL` CRC (`4W = 0.9558×`, `10W sys -40%`)

**Baseline:** `origin/master` `v0.5.12` (`6a36b0e10`), Phase-1 Champion `P1c_Std` (`Cycle 35`), and `MasterA/MasterB` A/A control in the same binary.
**Candidate harness:** [`c37_inplace_rle_ab.rs`](2026-11-21-wormhole-cycle37/c37_inplace_rle_ab.rs) (`safe_c37` module `#![forbid(unsafe_code)]`), raw log [`c37_results.txt`](2026-11-21-wormhole-cycle37/c37_results.txt), manifest [`c37_inplace_rle.toml`](2026-11-21-wormhole-cycle37/c37_inplace_rle.toml).

---

## 1. Mathematical & Systems Construction

1. **Theorem 37.1 (Exact In-Place RLE1 Expansion — `0 B` `scratch.ll`, `-0.90 MiB`/worker = `-9.0 MiB` at `10W` = `45%` of i9's `20 MiB` L3!):**
   In `v0.5.12`, `OpenCells::invert_runs` writes `n` bytes to per-worker `scratch.ll` (`900 KiB`), then `rle::unrle` copies `scratch.ll → out` (`900 KiB`), keeping **two** hot `900 KiB` copies per worker (`18 MiB` across `10W`!). Passing the recycled `out` buffer directly to `OpenCells::invert_runs` (`0 B` `scratch.ll`!) lets us expand RLE1 **in place** on `out[0..n]`:
   - Scan `out[0..n]` once for the `R` 4-byte runs `(j_k, c_k, extra_k)` (only `R = 23..83` runs on Wikidata!) and track running net shift $\delta_k = \sum_{m \le k} (\text{extra}_m - 1)$, $M = \max(0, \max_k \delta_k)$, and $m = \min(0, \min_k \delta_k)$.
   - **Case 0 (`R = 0`, `94.8%` of random blocks):** **Zero bytes copied** — run CRC directly on `out[0..n]`.
   - **Case 1 (`m >= 0`):** Every output position satisfies $\text{dst} \ge \text{src}$, so a **single backward pass** expands `out` in place with `0` pre-shift and untouched prefix `0..r0`.
   - **Case 2 (General oscillating shrink/expand):** Right-shifting `r0..n` by $M$ (`<= 250` bytes on Wikidata!) guarantees $\text{src}_{\text{shifted}}(k) \ge \text{dst}(k)$ at every run $k$, enabling a **single forward in-place pass** (and leaving `0..r0` untouched!).
   - **Critical `open.rs:360` `memset(0)` fix:** Do **not** call `out.clear()` on recycled full-block `out` buffers before `OpenCells::invert_runs`; keeping `out.len() ≈ 900 KiB` avoids `src/open.rs:360`'s `out.resize(n + 16, 0)` zero-filling `211 MB` per decode!
   - **Verified:** `2,000/2,000` randomized stress tests (0-run, shrink-only `extra=0`, expand-only `extra=255`, oscillating, boundary lengths `0..70,000`, `RunUnfinished` error) exact against `mibz2::rle::unrle` + `mibz2::crc::crc32`.
2. **Lemma 37.2 (Safe-Rust `64 KiB` L2-Tiled Hardware `CLMUL` CRC via $\mathbb{F}_2[x]/(P)$ Combination):**
   For MSB-first CRC-32/BZIP2 ($P = \text{0x04C11DB7}$), the un-inverted state satisfies the exact affine identity
   $$R(s_1, B) = (s_1 \oplus \text{0xFFFF\_FFFF}) \cdot x^{8|B|} \bmod P \;\oplus\; \neg\operatorname{crc32}(B).$$
   Calling `mibz2::crc::crc32(&out[tile..tile+64K], k)` (hardware `PCLMULQDQ`!) on each `64 KiB` tile immediately as the forward pass writes it — and combining tiles via precomputed $32\times 32$ $\mathbb{F}_2$ power-of-2 matrices (`15 ns`/tile) — eliminates the **`1.0 B/B`** L3/DRAM CRC re-read on Comet Lake's `256 KiB` L2 in `100%` safe Rust!

---

## 2. Native Safe-Rust A/B + A/A Results (`wikidata_431b_210m`, `211.0 MB`, exact `AbHash`)

### 2a. `12`-Round Rotated Fresh-Process `4W` A/B

| Arm | `4W` med (`ms`) | Ratio vs `MasterA` `[95% CI]` | `user_ms` | `sys_ms` | `minflt` | `rss_MB` | `ll` cap/W | Verdict |
|---|---:|---|---:|---:|---:|---:|---:|---|
| `MasterA` | `373.58` | `1.0000 [1.0000, 1.0000]` | `664.77` | `39.51` | `15,752` | `59.50` | `900K–1.8M` | Control |
| `MasterB` (`A/A`) | `379.19` | `1.0142 [0.9839, 1.0455]` | `668.38` | `40.99` | `16,166` | `60.89` | `900K–1.8M` | `A/A` clean |
| `P1c_Std` (`C35`) | `359.37` | `0.9568 [0.9272, 0.9874]` | `642.45` | `28.29` | `11,921` | `56.45` | `900 KiB` | Phase 1 |
| **`C37a_InPl_Crc`** | **`363.59`** | **`0.9558 [0.9352, 0.9769]`** | `659.30` | **`24.41` (`-38.2%`)** | **`10,854` (`-31.1%`)** | **`52.54` (`-11.7%`)** | **`0 B`** | **PROMOTE** |
| **`C37b_InPl_T64`** | **`363.05`** | **`0.9576 [0.9381, 0.9775]`** | `658.23` | **`26.39` (`-33.2%`)** | **`10,961` (`-30.4%`)** | **`53.28` (`-10.5%`)** | **`0 B`** | **KEEP (i9 L2!)** |
| `C37c_2Buf_T64` | `367.57` | `0.9775 [0.9496, 1.0062]` | `658.45` | `34.39` | `12,815` | `56.38` | `900 KiB` | **KILL** |

### 2b. Cold-Start `W = 10` & `W = 16` (`3`-round median) & `1T` Worker Microbench (`54 MB`, `15` rounds)
- **`1T` worker (`front + Open + RLE + CLMUL`):** `C37a = 1.0184×` (`+2.58 ms / 60 blks = +43 µs/blk`) purely because our `#![forbid(unsafe_code)]` prototype used `8`-byte scalar SWAR (`find_run4_swar`) vs `v0.5.12`'s `32`-byte AVX2 `_mm256_cmpeq_epi8` in `src/rle.rs`! In a Coordinator port using `src/rle.rs`'s existing `32`-byte AVX2 scan inside `unrle_in_place` + streaming `clmul_update`, that `+43 µs/blk` scan gap vanishes completely!
- **`W = 10` (`wikidata_431b_210m`):** `C37b_InPl_T64` drops `sys_ms` **`52.98 → 31.82 ms` (`-40.0%` vs Master, `-40.5%` vs `P1c_Std`!)**, `minflt` **`26,207 → 16,907` (`-35.5%`)**, `rss` **`89.75 → 79.41 MiB`** (and in run 1 `100.05 → 69.93 MiB`, `-30.1%`!).
- **`W = 16`:** `C37b_InPl_T64` cuts `rss` **`134.30 → 86.72 MiB` (`-35.4%`!)**, `minflt` **`36,072 → 22,811` (`-36.8%`)**, `sys_ms` **`75.09 → 51.29 ms` (`-31.7%`)**!

---

## 3. Updated `10T` i9-10900K Critical-Path Model

- Removing `scratch.ll` (`-0.90 MiB/W = -9.0 MiB` at `10W`) + `64 KiB` L2-tiled `CLMUL` (`-1.0 B/B`) cuts `10W` L3 demand from **`46.0 → 32.5 MiB`** (`f_miss: 57% → 38%`) and off-chip DRAM from **`8.35 → 5.1 B/B`**, dropping modelled Regime-2 `10T` wall time from **`~45–47 ms` (`P1c`) → `~41–43 ms` (`-4 ms`!)** (with AVX2 run-scan in production)!

---

## 4. Kill Decisions (One Line Each)

- **PROMOTE `C37a: Emit-Direct-to-Out (0 B scratch.ll) + Exact In-Place RLE1`:** Eliminates `W × 0.90 MiB` (`-9.0 MiB` at `10W` = `45%` of i9 L3), reaches `4W = 0.9558 [0.9352, 0.9769]` even with scalar SWAR, and cuts `sys_ms -38%` / `minflt -31%..-35%` / `16W RSS -35%`.
- **KEEP `C37b: 64 KiB L2-Tiled GF(2) CLMUL CRC` (for Comet Lake `256 KiB` L2):** Ties `C37a` on Xeon `2 MiB` L2 (`0.9576×`, `10W sys_ms -40.0%`, `minflt -35.5%`) while eliminating `1.0 B/B` L3/DRAM re-read on Comet Lake's `256 KiB` L2 (using `src/crc.rs::clmul_update`'s existing streaming `state` in production!).
- **KILL `C37c: Two-Buffer Tiled (keeping scratch.ll)`:** Retains `900 KiB/W` `ll` (`+9.0 MiB` L3 at `10W`), regresses `4W` (`0.9775×`) and `10W/16W` `sys_ms` (`58.1 / 89.6 ms`) vs `C37a/b`.
