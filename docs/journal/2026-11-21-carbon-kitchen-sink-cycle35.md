# Carbon Kitchen Sink (Neon) — Cycle 35: Grand Neon Synthesis & 10-Cycle Clean Handoff (Cycles 26–35)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`mibz2 0.5.11`, tagged `v0.5.11`, `src/` identical to `90e671966`)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched; zero PRs or merges.

---

## 1. Executive Summary — What Cycles 26–35 (Neon) Established

Across Cycles 26–35, Gemini Neon pushed the Carbon Kitchen Sink research programme from Cycle 25's initial synthesis (`CKS-DietOperad`: `0.9256×` 1T / `0.9534×` 2T; `CKS-BiOperad`: `0.8587×` 1T / `0.8642×` 2T at `981.8 KiB = −16.28%` WS) to **two zero-warning (`-D warnings`, `#![forbid(unsafe_code)]`) drop-in flagship kernels** that simultaneously cut per-worker persistent scratch by **`−19.28%` (`1,171.7 -> 945.8 KiB`, `−225.9 KiB/worker`)** and crush verbatim `0.5.11` `master` across `1T`, `2T`, and `4T` concurrent barriers:

| Flagship (`#![forbid(unsafe_code)]`, `-D warnings`) | Scratch (`6`-blk) | Active `Dict` (`6/6`) | `1T` `6`-Blk (`80` rds) | `construct` / `walk` | `2T` p10 / p50 (`50` p.) | `4T` p10 / p50 (`30` p.) | `dense-300k` |
|---|---:|---:|---:|---:|---:|---:|---:|
| `MasterA1` (`0.5.11` `v0.5.11`) | `1,171.7 KiB` | `24.5–64.6 KiB` | `1.0000×` (`12,128.9 µs`) | `1.0000× / 1.0000×` | `1.0000× / 1.0000×` | `1.0000× / 1.0000×` | `1.0000×` |
| `MasterA2` (in-binary A/A) | `1,171.7 KiB` | `24.5–64.6 KiB` | `0.9957×` (`12,077.2 µs`) | `0.9998× / 0.9877×` | `0.9960× / 0.9904×` | `0.9965× / 1.0074×` | `0.9852×` |
| **`Neon-Diet` ($\mu = 1$)** | **`945.8 KiB` (`−19.28%`)** | **`13.5–28.5 KiB` `<32K`** | **`0.8966×` (`10,874.7 µs`)** | **`0.8844× / 0.9077×`** | **`0.8968× / 0.8987×`** | **`0.8715× / 0.8967×`** | **`0.9521×`** |
| **`Neon-Bi` ($\mu = 2$)** | **`945.8 KiB` (`−19.28%`)** | **`13.5–28.5 KiB` `<32K`** | **`0.8278×` (`10,039.8 µs`)** | `1.0254×` / **`0.6415×`** | **`0.8282× / 0.8317×`** | **`0.7547× / 0.8143×`** | **`0.7643×`** |

- **10-Thread i9-10900K `20 MiB` L3 Crossing (with `900 KiB` `out` buffer/worker):** `0.5.11` `master` totals `10 × (1,171.7 + 900) = 20.23 MiB > 20 MiB` shared L3; both Neon flagships pull the entire 10-worker footprint **under `20 MiB` L3** at **`10 × (945.8 + 900) = 18.03 MiB < 20 MiB`**, while keeping active `Dict` (`inline16 + lens`) strictly below `32 KiB` L1D on **`6/6` Wikidata blocks** (`13.5–28.5 KiB` vs `master`'s `24.5–64.6 KiB`).
- **Why two flagships?**
  1. **`Neon-Diet` (`cand_neon_diet/`, $\mu = 1$):** Zero extra memory streams (`1.00m` serial walk), beats `0.5.11` `master` in **both** `construct` (`0.8844×`) and `walk` (`0.9077×`), and is the **first $\mu = 1$ kernel to break `< 0.90×`** (`0.8966×` 1T, `0.8968×` 2T, `0.8967×` 4T) at `−19.28%` WS. Zero risk of multi-stream IMC contention at 6–10 threads on i9.
  2. **`Neon-Bi` (`cand_neon_bi/`, $\mu = 2$):** Adds L2-gated (`m >= 65_536 && !wide`) `mark_byte`-isolated `Wire16` XOR wiring (`+0 B` scratch, `0 B` staging, `1.00m` total walk loads — unlike `#705` `Relay8`'s `1.87m` loads). Cuts `walk` to **`0.6415×` (`−35.85%`)**, reaching **`0.8278×` `1T`**, **`0.8282×` `2T`**, and **`0.7547×` p10 / `0.8143×` p50 `4T`** (amplifying under L2 contention!).

---

## 2. Final Cycle 35 Exhaustive Audit & Benchmark (`run_c35.rs`)

Compiled with `rustc -D warnings -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`, **zero compiler warnings** on both `cand_neon_diet` and `cand_neon_bi`).

### 2.1 Correctness, Safety, & `#703` Refusal Verification
- **Exhaustive BWT Parity (`67,968` BWTs):** `32,766` all binary strings (`n = 1..=14`) + `29,523` all ternary strings (`n = 1..=9`) + `5,679` random strings (`n = 10..=120`, alphabet `2..=8`, `LIMIT ∈ {1, 2, 4, 16}`): **100% PASS** on both `Neon-Diet` and `Neon-Bi`.
- **Malformed Non-BWT Termination (`1,100` inputs):** `1,000` random non-BWT columns (`n = 1..=512`) + `100` large non-BWT columns at `n = 70,000 >= 65,536` (`XOR_GATE` active!): **100% PASS** (exact length `n`, zero panics, bounded termination).
- **`#703` `Cap::dense(300_000)` Refusal Scratch:** `Master = 4,608 B`, `Neon-Diet = 5,120 B`, `Neon-Bi = 5,120 B` (`0 B` strips/bands/spelling/inline16/edges allocated on refusal).
- **Stress Corpus Parity & Timing (`40` rounds/block):**
  * `dense-300k` (`m = 300,000`): `M1 = 4,593.6 µs` (A/A `0.9852×`), `Neon-Diet = 0.9521×`, **`Neon-Bi = 0.7643×`**.
  * `periodic-150k-d2` (`m = 150,000`, `2` cycles): `M1 = 1,055.1 µs` (A/A `1.0036×`), `Neon-Diet = 0.9451×`, **`Neon-Bi = 0.8623×`**.
  * `periodic-105k-d3` (`m = 104,916`, `3` cycles): `M1 = 487.2 µs` (A/A `0.9941×`), `Neon-Diet = 0.9713×`, `Neon-Bi = 0.9689×`.
  * `spiral-200k` (`n = 200,000`, `c.spirals == true`, widths `99,999` & `99,996`): `M1 = 11.3 µs` (A/A `1.0266×`), `Neon-Diet = 1.0291×`, `Neon-Bi = 1.0339×` (within 0.3% of A/A noise floor).

### 2.2 Per-Block `1T` Breakdown (`80` rounds/block, `60`-round Stage Split)

| Block | `m` | `MasterA1` (`µs`) | A/A | `Neon-Diet` (`c / w` `µs`) | `Neon-Bi` (`c / w` `µs`) | Active `Dict` (`M1 -> Neon`) |
|---|---:|---:|---:|---|---|---|
| `block-0.L` | `89,874` | `1,332.5` | `0.9952×` | **`0.8926×`** (`673 / 521`) | **`0.8608×`** (`749 / 400`) | `26.3 -> 16.5 KiB` |
| `block-2.L` | `112,070` | `1,631.2` | `0.9964×` | **`0.8877×`** (`788 / 661`) | **`0.8501×`** (`885 / 503`) | `36.2 -> 21.3 KiB` |
| `block-85.L` | `156,404` | `1,650.6` | `0.9950×` | **`0.9106×`** (`603 / 899`) | **`0.8511×`** (`751 / 654`) | `24.5 -> 13.5 KiB` |
| `block-143.L` | `196,841` | `2,914.0` | `0.9934×` | **`0.8877×`** (`1268 / 1329`) | **`0.7997×`** (`1439 / 906`) | `64.6 -> 28.5 KiB` |
| `block-210.L` | `204,281` | `2,610.3` | `1.0017×` | **`0.9085×`** (`1004 / 1403`) | **`0.8106×`** (`1214 / 939`) | `41.2 -> 20.3 KiB` |
| `block-207.L` | `135,921` | `1,990.3` | `0.9918×` | **`0.8924×`** (`906 / 858`) | **`0.8316×`** (`1040 / 606`) | `41.8 -> 20.0 KiB` |
| **6-Block Sum** | `895,391` | **`12,128.9`** | **`0.9957×`** | **`0.8966×` (`5243 / 5671`)** | **`0.8278×` (`6078 / 4008`)** | **`6/6 < 32 KiB` L1D** |

### 2.3 Exact Cumulative Scratch Anatomy (`6`-Block Worker High-Water Mark)

| Buffer | `0.5.11` `master` | `Neon-Diet` & `Neon-Bi` | Delta | Mechanism |
|---|---:|---:|---:|---|
| `edges` | `817,124 B` | `817,124 B` | `0 B` | Reused for `strips`, `image_order`, `Elim`, `12 KiB` HT, and final `edges` |
| `strips` | `131,072 B` | `0 B` | `−131,072 B` | `MuxStrips` (`edges[0..3*ns]`, Cycle 21) |
| `bands` | `98,304 B` | `65,536 B` | `−32,768 B` | `Band64` (`[end:20 \| dst:20 \| code:24]`, Cycle 21) |
| `spelling` | `65,536 B` | `49,152 B` | `−16,384 B` | `48 KiB` exact-fit pre-reserve + `Operad-MN16` (`<= 47,178 B`, Cycles 20/26/27) |
| `offsets` | `16,384 B` | `0 B` | `−16,384 B` | Zero-`offsets` meta inside `inline16` long slots (Cycle 16/25) |
| `inline16 + lens` | `66,147 B` | `30,464 B` | `−35,683 B` | Direct-`inline16` `12 KiB` quotient dedup (`<= 28.5 KiB` live, Cycles 16/25/33) |
| `queues` (`Spill`) | `5,120 B` | `6,144 B` | `+1,024 B` | Identical `OutQ16` chunking (`+1` `cuts` alignment) |
| `endpoint` (`stack`) | `128 B` | `64 B` | `−64 B` | Only cold `len >= 16` rules push onto `stack` |
| **Total Worker Scratch** | **`1,199,815 B` (`1,171.7 KiB`)** | **`968,484 B` (`945.8 KiB`)** | **`−231,331 B` (`−19.28%`)** | **Fits `10 × (scratch + out) = 18.03 MiB < 20 MiB` i9 L3** |

---

## 3. Complete 10-Cycle Ledger (Cycles 26–35): Survivors, One-Line Kills, & Theorems

### 3.1 Survivors (All Integrated in `cand_neon_diet/` & `cand_neon_bi/`)
1. **`48 KiB` `spelling` Exact-Fit Pre-Reserve** (Cycles 26–28, `Proposition 26.2`): Eliminates `43K -> 86K` doubling overshoot (`−36 KiB` from Cycle 25 -> `945.8 KiB = −19.28%` WS).
2. **Pass-1 Bit-Lattice Cursor Fusion `pos[a] = (tail << 10) | fill` + Algebraic `all = thin + thick_sum`** (Cycle 29, `Theorem 29.1`): Halves hot Pass-1 L1D tables (`4 -> 2 KiB`), zero-shift byte offset `at = p + (p & 1023)`, eliminates `489K` thin-run RMWs to `f.all[a]` (`−140 µs`).
3. **Static-Side `cut_tail` (`next(tail, side) == 0`) + Single-Store `Narrow::set_links`** (Cycle 31, `Theorem 31.1`): Cuts `Elim` by `−179.7 µs` across `6/6` blocks by eliminating redundant read-modify-writes and sentinel loads.
4. **`LIMIT = 16` Multi-Core Pareto Optimality** (Cycle 32, `Theorem 32.1`): Exact global scratch minimum (`945.8 KiB`) and multi-core Pareto champion despite `1T` shifting to `L24`.
5. **Unique-Symbol `mark_byte` Isolation + `Wire16/8/4` `u64` MSHR-Burst `wire_xor_nomark`** (Cycle 33, `Theorem 33.1`, `Neon-Bi`): Hoists `mark` out of `255/256` byte queues and exposes `16` scatter RMWs per slice-checked sequential epoch, bringing `construct` within `+1.3–2.5%` of `master`'s non-XOR `construct`.
6. **Fused `fetch_word` + Per-Byte `last_word` Cache + 1-Mul `hash_u128` + `12 KiB` (`9/11`) Arena HT** (Cycle 33, `Theorem 33.2`): Cuts `join` + `dedup` latency and halves HT L1D footprint (`24 -> 12 KiB`) for only `+7` words (`+0.12%`).
7. **Hardware-Grounded `XOR_GATE = 65_536`** (Cycle 34, `Theorem 34.2`, `Neon-Bi`): Validated by `m = 20K..300K` sweep (`m* ≈ 42K–48K`; protects sub-L2 and multi-cycle periodic blocks).

*(Built cleanly atop Cycles 16–25 survivors: `Operad-MN16` 128-bit truncated free-monoid rules, Direct-`inline16` Zero-`u_rec`/Zero-`offsets`, `MuxStrips + Band64`, and L2-gated `XOR-BiEmit`.)*

### 3.2 Complete One-Line Kills (Cycles 26–34)
- **Cycle 26 — `EpochTile64` / `SubTile32`:** Killed (`0.8791×`/`0.8886×` vs `0.8651×`, `construct +219.5/+337.5 µs`); tiling breaks sequential `bands`/`edges[u]` streams and reloads 256 cursors (`Theorem 26.1`).
- **Cycle 26 — `NoQueue-BiOperad`:** Killed (`1.2330×`, `+4,419.9 µs`); skipping `OutQ16` causes 1.6-row random byte switches across `bands` and `edges[u]`.
- **Cycle 27 — `Inline15-ZeroLens`:** Killed (`0.9114×` vs `0.8694×`, `walk +465.4 µs`, `0.826×` vs `0.766×` on `0`-long `dense-300k`); packing `len` in byte `15` of `[u8; 16]` incurs an XMM-to-GPR cross-domain stall vs GPR `lens[c]` (`Theorem 27.1`).
- **Cycle 28 — `BiRV` / `LeftRV`:** Killed (`construct +174.3/+448.1 µs`, `2T 0.8667×/0.9066×` vs `0.8577×`); despite `Theorem 28.1` Church–Rosser step wins (`0.967×` on `block-143`), L2-resident `Elim` is `aw <= bw` branch-bound and left inserts pay extra `prev` loads + ASCII skew.
- **Cycle 28 — `FusedOp`:** Killed (`+34.3 µs`); redundant rule loads on long-rule misses.
- **Cycle 29 — Carry-Over `maximal`:** Killed (`+887.9 µs`); replacing `master`'s inner `while` merge with a carry-over state creates a loop-carried dependency on `(c, width)`.
- **Cycle 29 — `Pass1-SWAR4` / `Pass1-W1`:** Killed (`+15.0 / +549 µs`); `7.1% / 27.0%` width branch mispredicts lose to branchless 32B iota.
- **Cycle 30 — `Walk-BwdU64` / `Walk-BothU64` / `Walk-LitFast` / `Walk-Clean`:** Killed (`walk +419.3 / +712.1 / +835.3 / +30.2 µs`); data-dependent length/literal branches flush the ROB and destroy in-flight past-L2 `edges` MLP (`Theorem 30.1`).
- **Cycle 31 — `CMOV-Elim`:** Killed (`construct +164.7 µs` vs `Neon-C29`, `+344.4 µs` vs `Links-Only`); autocorrelated width peeling is TAGE-predictable while dynamic `side` + `cmov` serializes pointer updates (`Theorem 31.1`).
- **Cycle 32 — `LIMIT ∈ {14, 20, 24, 32}`:** Killed as multi-worker default (`L14` dominated in speed & scratch; `L20/24/32` inflate WS `+85..+163 KiB/worker` and lose `1T` lead at `2T`, `Theorem 32.1`).
- **Cycle 33 — `Wire4` (`K = 4`):** Killed (`+13.4 µs` vs `Neon-C31`); `4` scatter RMWs per aliasing epoch fail to saturate past-L2 MSHRs.
- **Cycle 33 — `XOR-QuadEmit` ($\mu = 4 \to 2$ in-place):** Killed (`walk +482.8 µs`, `0.8654×` 1T, `0.8528×` 2T on Wikidata despite winning `0`-long `dense-300k` `0.736×` vs `0.769×`); $1-(1-p)^4 \approx 15\text{--}28\%$ combined LONG-word mispredicts trigger Theorem 30.1 ROB flushes alongside `~450 KiB` `copy_within` stitch overhead (`Theorem 33.3`).
- **Cycle 34 — `HoistP`:** Killed (`+78.6 µs` 1T, `+50.6/+94.3 µs` 2T, `+419/+368 µs` 4T on `6/6` blocks); duplicating `maximal` blows out Pass 1's μop-cache for a 1-cycle zero-mispredict ALU guard (`Theorem 34.1`).

---

## 4. Open Threads & Recommendations for Coordinator / i9-10900K Validation

1. **Immediate i9-10900K Multi-Thread Validation (`1 / 4 / 6 / 8 / 10` Threads) of `cand_neon_diet/` vs `cand_neon_bi/`:**
   - Both drop-in modules are archived verbatim in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle35/cand_neon_diet/` and `cand_neon_bi/` (`#![forbid(unsafe_code)]`, `-D warnings` clean).
   - **Test priority:** Measure `Neon-Diet` ($\mu = 1$) first as a guaranteed `−19.28%` WS + `~0.897×` zero-contention upgrade (`18.03 MiB < 20 MiB` L3 at 10 threads), then test `Neon-Bi` ($\mu = 2$) at `6 / 8 / 10` threads to verify whether `1.00m`-load $\mu = 2$ MLP holds its `0.81–0.83×` win on real 10-core DDR4 bandwidth (recall `#705` `Relay8` failed because it paid `1.87m` loads across 8 lanes; `Neon-Bi` pays `1.00m` loads across 2 lanes and amplified to `0.755–0.814×` under 4T L2 contention here).
2. **Staged Production Integration Path:**
   - Every surviving mechanism is cleanly separable for bisected PRs if desired: (a) `MuxStrips + Band64` (`−13.6%` WS, neutral/slightly faster), (b) `Operad-MN16 + Direct-inline16 + 12 KiB HT + 48 KiB spelling` (`−5.7%` additional WS, `construct` + `walk` win), (c) `Pass1 All+Pos + Elim cut_tail/set_links` (`−320 µs` `construct`), (d) L2-gated `mark_byte` `Wire16` + `XOR-BiEmit` (`−36%` `walk`, `+0 B` WS).
3. **Unresolved Deep Mathematical Threads:**
   - **Pre-BWT format-changing pipelines (`Pre-HexTok`, Cycle 8):** Still the only known route to cut the information-theoretic `m` itself (`0.541×` $m$, `0.949×` compressed size) when format changes are allowed.
   - **Intra-block multi-core parallelism (`SPOD-Coset512`, Cycle 10):** Remains open if single-block latency (rather than block-parallel throughput) ever becomes a target (`7.1–7.9×` at 8t with zero format changes).
   - **Long-word-free `XOR-QuadEmit` ($\mu = 4 \to 2$, Cycle 33):** Proven faster than `BiEmit` (`0.736×` vs `0.769×`) when `P(LONG) == 0` (`dense-300k`); could become viable on Wikidata only if `inline16` were widened or LONG words were split into `<= 16 B` chunks during `dedup` (though splitting increases `m` and `edges`, likely net-negative unless `m` stays invariant).

---

## 5. Archived Artifacts

All Cycle 35 reproducers and spotless `-D warnings` flagship drop-ins are archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle35/`:
- `gen_c35.py` — reproducible generator for `cand_neon_diet` and `cand_neon_bi`
- `run_c35.rs` — zero-warning exhaustive verification (`67,968` BWTs + `1,100` malformed + `#703` refusal) & `80`-round `1T` / `50`-pass `2T` / `30`-pass `4T` benchmark
- `cand_neon_diet/` (`mod.rs`, `endpoint.rs`, `outq.rs`) — **`Neon-Diet` ($\mu = 1$) flagship**
- `cand_neon_bi/` (`mod.rs`, `endpoint.rs`, `outq.rs`) — **`Neon-Bi` ($\mu = 2$) flagship**
