# Carbon Kitchen Sink (Neon) — Cycle 45: Round-5 Construction Grand Synthesis (`0.8895×` / `0.8767×` `Pre`, `0.9304×` `1T`, `+0 B` WS) & 10-Cycle Clean Handoff (`Cycles 36–45`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack + `#714` `GradedHandle/Prefix15` + Wormhole `L4/L5` + Mechanisms splice, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched (`git diff 90e671966 -- src/ Cargo.toml benches/ examples/` empty).

---

## 1. Executive Summary (10-Cycle Round-5 Handoff)

Round 5 (`Cycles 36–45`) re-baselined on `origin/master` (`53f7ecf93`, where `#719` had already shipped our Round-4 construction stack on the i9 at `0.952 / 0.987 / 0.975` at `6/8/10` threads and two-ended emission proved neutral) and attacked **what remained in construction (`S1–S4`)** with strict **`+0 B` persistent scratch growth** (`945.8 KiB` to the exact byte).

Across 10 autonomous cycles we stacked **10 verified, zero-warning (`-D warnings`, `#![forbid(unsafe_code)]`) construction wins** hitting **all 5 construction stages (`S1`, `S2a`, `S2b`, `S3a`, `S4`) simultaneously**, cutting post-`#719` construction time by **`−11.05%` (`0.8895×`, `−663.0 µs`)** on `head` (`53f7ecf93`) and **`−12.33%` (`0.8767×`, `−639.7 µs`)** on `nobi` ($\mu = 1$ serial `wire` + `emit`, matching the i9 `nobi` diagnostic) — translating to **`0.9304×` `1T` / `0.9332×` `2T` p50 / `0.9366×` `4T` p50** (`head`) and **`0.9411×` `1T` / `0.9384×` `2T` p10 / `0.9424×` `2T` p50 / `0.9329×` `4T` p50** (`nobi`), with **`6/6` Wikidata blocks under `0.935×`** on `head` (`0.9257×–0.9343×`).

---

## 2. Theorem 45.1: Exhaustive Audit Catch & Zero-Branch Injective `Inline2` Key Fix

During Cycle 45's `1,100`-case arbitrary-byte (`0..alpha`) malformed audit at `n = 70,000` (`case = 1056`, `alpha = 2`, bytes `0x00` and `0x01`), the harness caught a subtle `0x00`-byte edge case in Cycle 38/39's `Inline2` + `S3Fast` key encoding (`uid = id + (u32::from(g == 2) << 8)`):
- When `literal(0)` (`g = 1, id = 0`) was followed by retained byte `0x00`, `key = (uid << 8) | 0 = 0`, colliding with `Dictionary::pairs`'s zero-initialized empty bucket sentinel (`0 == 0`) and returning `code = 0` (`[0]` instead of `[0, 0]`). Neither ASCII text nor `origin/master`'s own unit test (`for first in [1, 241]`) contained `0x00`.
- **Theorem 45.1 (Zero-Branch Injective `Inline2` Key Space):** Setting `uid = id + 1 + (u32::from(g == 2) << 8)` and rule base offset `65_793` maps:
  - `g == 1` (`literal(b)`, `b ∈ 0..=255`) $\mapsto \text{uid} \in [1, 256]$;
  - `g == 2` (`Inline2(b0, b1)`, `id ∈ 0..=65_535`) $\mapsto \text{uid} \in [257, 65\_792]$;
  - `g >= 3` (arena rules, `id >= 65_793`) $\mapsto \text{uid} \ge 65\_794$.
  Hence `key = (uid << 8) | byte >= 256 > 0` is strictly non-zero and injective across all `0x00..=0xFF` bytes and all handles (folding `+ 1` into a single x86 `lea` with zero branches and zero cycles).

---

## 3. Final Native Safe-Rust Verification & Benchmarks (`run_c45.rs`)

Compiled with **`rustc -D warnings --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1`** (`#![forbid(unsafe_code)]`, **0 warnings**) against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`.

### 3.1 Exhaustive Parity & Persistent Scratch Audit
- **Exhaustive Correctness (`100%` byte-exact on `R5-Head`, `R5-NobiW16`, and `R5-NobiSIMD`):**
  - **`67,968` valid BWTs:** all `32,766` binary strings (`n = 1..=14`), all `29,523` ternary strings (`n = 1..=9`), and `5,679` pseudo-random BWTs (`n = 1..=512`, `|Σ| ∈ {2, 3, 4, 8, 16, 26, 256}`).
  - **`1,100` malformed / multi-cycle BWT columns:** `1,000` small (`n = 1..=256`) + `100` large (`n = 70,000`, `m > 65,536` `EPOCH` wrap & byte `0x00`), 100% bit-for-bit identical to `origin/master`.
  - **Stress & admission controls:** `spiral-200k` (`199,998`-wide thick strip forcing `eliminate::<true>`), `periodic-150k-d2`, `periodic-105k-d3`, `dense-300k`, `6/6` Wikidata blocks, and `#703` `Cap::dense(300_000)` refusal (`None`).
- **Persistent Working Set (Wikidata steady-state, byte-identical to `origin/master`):**
  - `Master53f7`: **`945.8 KiB`** (`queues=6144, bands=65536, spelling=49152, edges=817124, inline16=28672, lens=1792, endpoint=64`)
  - **`R5-Head`:** **`945.8 KiB`** (**`+0 B`**, `dict_l1 = 20,519 B`)
  - **`R5-NobiW16`:** **`945.8 KiB`** (**`+0 B`**, `dict_l1 = 20,519 B`)

### 3.2 Headline `1T` (`80` rds/blk), `Pre/Emit` Split (`60` rds/blk), `2T` (`50` passes), & `4T` (`30` passes)

| Kernel | `1T` Total (`µs`, ratio) | `1T` `Pre (S1–S4)` (`µs`, ratio, `Δµs`) | `1T` `Emit` (`µs`) | `2T` p10 / p50 (ratio) | `4T` p50 (ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,581.0 (1.0000×)` | `5,999.5 (1.0000×, +0.0)` | `4,518.6` | `9,777.9 / 10,387.3 (1.0000 / 1.0000×)` | `22,251.3 (1.0000×)` |
| `MasterA2` (A/A) | `10,530.5 (0.9952×)` | `5,990.9 (0.9986×, −8.6)` | `4,510.6` | `9,855.9 / 10,289.0 (1.0080 / 0.9905×)` | `22,055.0 (0.9912×)` |
| **`R5-Head`** | **`9,844.6 (0.9304×)`** | **`5,336.5 (0.8895×, −663.0)`** | `4,503.6` | **`9,295.6 / 9,693.7 (0.9507 / 0.9332×)`** | **`20,840.9 (0.9366×)`** |
| `Master-Nobi` ($\mu=1$) | `10,872.5 (1.0000×)` | `5,188.9 (1.0000×, +0.0)` | `5,653.8` | `10,291.8 / 10,746.2 (1.0000 / 1.0000×)` | `23,650.7 (1.0000×)` |
| **`R5-NobiW16` (champ)** | **`10,231.5 (0.9411×)`** | **`4,549.2 (0.8767×, −639.7)`** | `5,663.3` | **`9,658.1 / 10,127.3 (0.9384 / 0.9424×)`** | **`22,062.9 (0.9329×)`** |
| `R5-NobiSIMD` (`chunks(2)`) | `10,267.1 (0.9443×)` | `4,586.4 (0.8839×, −602.5)` | `5,646.7` | `9,718.0 / 10,146.8 (0.9442 / 0.9442×)` | `21,668.4 (0.9162×)` |

Per-block `1T` ratios for **`R5-Head`** vs `MasterA1` (A/A `0.9854–0.9987×`):
`block-0: 0.9343`, `block-2: 0.9257`, `block-85: 0.9339`, `block-143: 0.9277`, `block-210: 0.9300`, `block-207: 0.9329` (**`6/6` blocks `< 0.935×`!**).
Per-block `1T` ratios for **`R5-NobiW16`** vs `Master-Nobi`:
`block-0: 0.9383`, `block-2: 0.9394`, `block-85: 0.9514`, `block-143: 0.9393`, `block-210: 0.9410`, `block-207: 0.9379` (**`6/6` clean wins**; beats `R5-NobiSIMD` on `1T` `−35.6 µs`, `Pre` `−37.2 µs`, `2T p10` `−59.9 µs`, and `2T p50` `−19.5 µs`, so **`R5-NobiW16` is archived as `cand_r5_nobi/`**!).

### 3.3 Final 6-Stage Construction Gap Closure (`50` rds/blk) vs `origin/master` (`53f7ecf93`) & Wormhole C22–26 `BMCP_CML`

| Stage | `Mst-53f7` (`µs`) | **`R5-Head`** (`µs`, `Δ`) | `Nobi-53f7` (`µs`) | **`R5-NobiW16`** (`µs`, `Δ`) | Wormhole C26 Floor (`µs`) | Verdict |
|---|---:|---:|---:|---:|---:|---|
| **`S1` (Run Pass + `OutQ16`)** | `1,959.5` | **`1,645.8 (−313.7, −16.0%)`** | `1,976.3` | **`1,619.9 (−356.4, −18.0%)`** | `~1,889` | **Broke below Wormhole C26 `S1` floor by `~240–270 µs`!** |
| **`S2a` (Strip → Node Setup)** | `358.7` | **`318.6 (−40.1, −11.2%)`** | `356.8` | **`322.5 (−34.3, −9.6%)`** | — | 3→1 sequential pass (`S2aFuse`) |
| **`S2b` (`Elim::eliminate`)** | `1,155.5` | **`1,090.7 (−64.8, −5.6%)`** | `1,176.6` | **`1,087.5 (−89.1, −7.6%)`** | — | `Inline2 + FastJoin + S2b-NI + NoStats` |
| **`S3a` (`Dictionary` + Bands)** | `846.4` | **`707.6 (−138.8, −16.4%)`** | `856.9` | **`718.7 (−138.2, −16.1%)`** | `~761` | **Broke below Wormhole C26 `S3` floor by `~42–53 µs`!** |
| **`S4` (Queue Wiring)** | `1,532.7` | **`1,467.0 (−65.7, −4.3%)`** | `759.1` | **`738.8 (−20.3, −2.7%)`** | `~750` (serial) | `BandW + Wire16` (`head`) / `BandW + W16` (`nobi`) |
| **`Pre (S1–S4)` Total** | `5,852.8` | **`5,229.7 (0.8935×, −623.2)`** | `5,125.8` | **`4,487.3 (0.8754×, −638.5)`** | — | **5/5 stages won simultaneously!** |

---

## 4. Complete Round-5 Ledger (`Cycles 36–45`): Survivors & One-Line Kills

### 4.1 Promoted Round-5 Construction Stack (All 10 Survivors, `+0 B` Persistent Scratch)
1. **`Cap127 + Pt64` (Cycle 37, `S1`, Theorem 37.1):** Canonical `CAP = 127 > LIMIT = 16` skips the per-run merge branch on `95.3%` of runs and fuses `thin[a]` + `(tail << 10) | fill` into `pt: [u64; 256]`, unlocking a super-additive `−155 µs` register-pressure reduction.
2. **`S1Trim` (Cycle 40, `S1`, Theorem 40.1):** Exact-trimmed `out[..in_out * BYTES]` fuses the `in_out_lim` check and slice bounds check into a single `out.get_mut(at..at+32)` with `(m ^ m_next) < EPOCH`.
3. **`OneBr` (Cycle 41, `S1`, Theorem 41.1):** Single bitwise predicate `(w <= 16) & (f <= CHUNK - 16) & ((m as u16) < 65_520)` collapses 3 serial branches in `push_thin` into 1.
4. **`S2aFuse` (Cycle 36, `S2a`):** Fuses `S2a`'s 3 passes over `strips_raw` into 1 sequential pass (`−40 µs`).
5. **`Inline2` + Theorem 45.1 Injective Key Fix (Cycles 38 & 45, `S2b/S3a`):** Packs all 2-byte bigrams into `ID_BITS = 22` handles (`(2 << 22) | b0 | (b1 << 8)`), cutting `S2b` rules `−21.1%` (`−159 KiB` arena writes) with 100% injective non-zero key `uid = id + 1 + ((g == 2) << 8)` (rule base `65_793`).
6. **`FastJoin` (Cycle 41, `S2b`):** Single `sum < 16` / `sum <= SATURATED` length check replaces two `SATURATED` checks in `Elim::join`.
7. **`S2b-NI` (Cycles 43 & 44, `S2b`):** `#[inline(never)]` `Elim::eliminate_narrow_plain` specializes `Narrow` (`STRIDE = 4`) via single-slice `&self.a[xi..xi+4]` / `&self.a[yi..yi+4]` reads/writes (`16 → 7` bounds checks, `−3` loads/step, `−52 µs` `Pre`).
8. **`NoStats` (Cycle 39, `S2b/S3a`):** Drops inner-loop per-band telemetry (`words/copies/traced/thick_area/unit_work/longest`) from production `S3a` (keep gated under `#[cfg(test)]` / `const STATS: bool` for test suites!).
9. **`S3Tag12 + S3Fast + S3Arr + S3a-C8` (Cycles 36, 39, 42, 44, `S3a`):** Arena-hosted **`10 KiB` `Dictionary` HT** (`&mut [u32; 512]` pairs + `&mut [u32; 2048]` 8-bit-tagged contents, zero bounds checks), `last_word` single-entry cache, 1-mul short hash, register `u128` shift-OR, cached `Links::band_node`, and `bands.reserve_exact(8192)` on `n >= 65_536` (`−139 µs` `S3a`).
10. **`BandW + Wire16 / Nobi-W16` (Cycles 36, 42, 43, 45, `S4`):** Stores band `width` directly in `Band(u64)` low 20 bits (dropping `bend - k`), isolates `mark_byte` outside the hot loop, and widens wiring to `16`-wide `u64` MSHR bursts (`wire_xor`) / `16`-wide `u64` serial unpack (`Nobi-W16`, beating both `W8` and `chunks_exact(2)`).

### 4.2 Complete One-Line Kills (`Cycles 36–45`, 14 Kills)
- **KILL `S3Tag24` (C36):** Doubling the `S3a` cache to `24 KiB` increased zeroing/L1d pressure and lost to `12 KiB` (`+12 µs`).
- **KILL `WC64` (C37):** Software 64-byte write-combining staging buffers broke `OutQ16` single-write optimality (`+914.8 µs` `Pre`, Theorem 37.2).
- **KILL `WC128` (C37):** 128-byte staging buffers still doubled queue traffic (`+159.3 µs` `Pre`).
- **KILL `ColdMrg` (C37):** Outlining run-merge without `CAP = 127` added ABI overhead on `4.7%` of runs (`+7.3 µs`).
- **KILL standalone `Pt64` (C37):** Without `Cap127` freeing GPRs, `u64` packing regressed (`+70.0 µs`; unlocks `−155 µs` synergy only with `Cap127`).
- **KILL `Inline23` (C38):** Inlining 3-byte trigrams cut rules `−36.3%` but added 3 branches + 7-bit unpacking (`+55.6 µs` `Pre`, Theorem 38.1).
- **KILL `RegElim` (C38):** Tracking hot `x`/`y` node fields in registers across elimination steps spilled GPRs (`+149.3 µs` `Pre`, `+197.3 µs` `2T`).
- **KILL standalone `S3Fast` (C39):** Without `NoStats` freeing `S3a` registers, branchless `uid` was neutral (`+0.3 µs`; unlocked `−45.9 µs` once paired with `NoStats`).
- **KILL `ColdThk` (C40):** Outlining `take_thick_cold` paid `24K` `#[inline(never)]` ABI calls/block (`+90.2 µs` `Pre`).
- **KILL `S2aSlim` (C40):** `set_links` in `S2a` degraded `S2a/S2b` codegen (`+18.4 µs` `Pre`).
- **KILL `Iota8` (C41):** `16 B` XMM stores left `2.4%` of thin runs (`9..16`) taking mispredicted cold slow paths (`+267.5 µs` `Pre`, Theorem 41.2).
- **KILL `Iota4` (C41):** `8 B` SWAR stores left `7.1%` of thin runs (`5..16`) on cold slow paths (`+768.0 µs` `Pre`, Theorem 41.2).
- **KILL `Pair2` (C42):** Unrolling 2 distinct-byte thin runs pushed live GPRs past x86-64's 16-GPR cliff and broke single-path AVX2 inlining (`+234.4 µs` `Pre`, `+247/+258 µs` `2T`).
- **KILL `S2aTunnel` (C44):** Tunneling `to` through `L::BOTTOM` caused a read-modify-write store-forwarding stall with `append_to(..., 1)` (`+13.6 µs` `S2a`, `+27/+28 µs` `2T`, `+9.4 µs` `Nobi Pre`).

---

## 5. Coordinator i9 Port Specification & Archived Drop-In Modules

Both spotless, `-D warnings`, `#![forbid(unsafe_code)]` drop-in modules are archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle45/`:
- **`cand_r5_head/`** (`mod.rs`, `endpoint.rs`, `outq.rs`): Direct drop-in over `origin/master` (`53f7ecf93`) keeping two-ended `wire_xor` (`0.8895×` `Pre`, `0.9304×` `1T`, `0.9332×` `2T` p50, `+0 B` WS).
- **`cand_r5_nobi/`** (`mod.rs`, `endpoint.rs`, `outq.rs`): Identical Round-5 construction stack in `nobi` ($\mu = 1$ serial `W16` `wire` + serial `emit`, `0.8767×` `Pre`, `0.9411×` `1T`, `0.9384×` `2T` p10, `+0 B` WS) — recommended for i9 `6/8/10`-thread A/B alongside `cand_r5_head/`, since `#719` showed two-ended emission was neutral on the i9 while `nobi` cuts `S4` in half (`1,467 → 739 µs`).
- **One test-suite note for porting into `src/open/`:** Gate the 6 `NoStats` inner-loop `stats.*` counters behind `#[cfg(test)]` (or `const STATS: bool`) so `cargo test` retains exact `Counts` assertions while release builds get the full `NoStats` register freedom; and preserve `Theorem 45.1`'s `uid = id + 1 + ((g == 2) << 8)` + `65_793` rule base (plus add a `0x00`-byte regression test in `endpoint::tests`!).

---

## 6. Remaining Open Frontier (Post-Round-5)

Construction (`Pre`) on `nobi` is now down to **`4,487 µs`** (`44.6%` of total decode time; `S5` dependent emit walk is `55.4%`!). The remaining construction mass breaks down as:
1. **`S1` (`1,620 µs`, `36.1%` of `Nobi Pre`):** Already `~270 µs` below Wormhole's C26 `S1` floor (`1,889 µs`); further gains require either pre-RLE2/MTF fusion (changing the record stream format upstream of `Open`) or hardware-assisted `u16` gather/scatter.
2. **`S2b` (`1,088 µs`, `24.2%`):** Dominated by the inherently data-dependent `x == y` / `aw <=> bw` interval-exchange control flow (`~47K` steps/block); `Inline2 + FastJoin + S2b-NI` already stripped rule writes `−21%` and bounds checks `16 → 7`.
3. **`S4` (`739 µs` `nobi` / `1,467 µs` `head`):** On `nobi`, `739 µs` is within `2%` of streaming L2/L3 bandwidth for writing `edges[0..m)` sequentially (`~817 KiB`); on `head`, the extra `~728 µs` is the random-scatter XOR writeback cost of two-ended emission.
4. **`S3a` (`708–719 µs`, `15.8%`):** Already `~42–53 µs` below Wormhole's C26 `S3` floor (`761 µs`).
5. **`S2a` (`319–323 µs`, `7.1%`):** Single-pass `S2aFuse`; only `3%` of total runtime.
