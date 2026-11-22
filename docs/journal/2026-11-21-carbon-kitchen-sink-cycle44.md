# Carbon Kitchen Sink (Neon) — Cycle 44: `S2b-NI` (`−52 µs`) & `S3a-C8` (`−33 µs`) Promotions, `S2aTunnel` Kill, 6-Stage Gap Closure (`0.8767×` Nobi `Pre`), & `4T` Audit

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Structural Framing

Cycle 44 tested three final construction levers atop `C43-Base` (`S3Arr + BandW`), ran a full 6-stage (`S1, S2a, S2b, S3a, S4, S5`) gap-closure profile against `origin/master` (`53f7ecf93`) and Wormhole's C22–26 `BMCP_CML` bounds, and executed a `2T` + `4T` concurrent barrier contention audit:

1. **`S2b-NI` (`#[inline(never)]` `Elim::eliminate_narrow_plain`):** Resolves Cycle 43's code-layout sensitivity on `head` by giving the specialized `Narrow` single-slice elimination loop its own 32-byte-aligned function boundary instead of letting LLVM intermittently inline it into `Endpoint::run`.
2. **`S3a-C8` (`10 KiB` `Dictionary` HT + `bands.reserve_exact(8192)`):** Halves the `pairs` hash table from `9` bits (`4 KiB`, `1024` words) to `8` bits (`2 KiB`, `512` words, total HT `10 KiB` inside L1d), halving per-block `pairs.fill(0)` clearing overhead, and pre-reserves `bands` to its warmed-up `8,192`-slot capacity (`64 KiB`, `+0 B` steady-state scratch) on `n >= 65_536`.
3. **`S2aTunnel` (Dead-Link Tunneling, Theorem 44.1):** Tests storing `to = sw[1] + first[a]` into node `sid`'s dead side-1 slot `L::BOTTOM` during `S2a` Pass 1 so `strips_raw` stays read-only and Pass 2 never reads `strips_raw` randomly.

---

## 2. Native Safe-Rust A/B Confrontation (`run_c44.rs`)

Compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`) against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`.

- **Parity & Scratch:** `100%` byte-exact on `2,000` fuzz BWTs + `6/6` Wikidata + `dense-300k` + `#703` `Cap::dense(300_000)` refusal (`None`); persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 `1T` Interleaved (`50` rds/blk) & `Pre` Split (`35` rds/blk)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs `C43` | `Emit` (`µs`) |
|---|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,676.7 (1.0000×)` | `6,103.7 (1.0000×)` | `+574.7` | `4,570.7` |
| `MasterA2` (A/A) | `10,723.0 (1.0043×)` | `6,131.6 (1.0046×)` | `+602.6` | `4,564.0` |
| `C43-Base` (prev champ) | `10,128.4 (0.9486×)` | `5,529.0 (0.9058×)` | `0.0` | `4,582.5` |
| `S2aTun` (dead-link tunnel) | `10,108.7 (0.9468×)` | `5,518.8 (0.9042×)` | `−10.2` | `4,580.1` |
| **`S2b-NI` (`#[inline(never)]`)** | **`10,091.7 (0.9452×)`** | **`5,477.0 (0.8973×)`** | **`−52.0`** | `4,617.3` |
| **`S3a-C8` (`10 KiB` HT + reserve)** | **`10,074.5 (0.9436×)`** | **`5,495.6 (0.9004×)`** | **`−33.3`** | `4,573.2` |
| **`C44-Head` (`S2aTun + S2b-NI`)** | **`10,070.3 (0.9432×)`** | **`5,447.2 (0.8925×)`** | **`−81.7`** | `4,602.7` |
| `Master-Nobi` ($\mu=1$ base) | `11,069.6 (1.0000×)` | `5,311.1 (1.0000×)` | `+618.1` | `5,741.0` |
| **`Nobi-W16` (`C43` nobi)** | **`10,471.5 (0.9460×)`** | **`4,693.0 (0.8836×)`** | `0.0` | `5,781.5` |
| `C44-Nobi` (`+S2aTun + S2b-NI`) | `10,472.5 (0.9461×)` | `4,702.4 (0.8854×)` | `+9.4` | `5,764.7` |

*(Note: `S3a-C8` beat `C43-Base` on `6/6` blocks and is not yet stacked with `S2b-NI` — Cycle 45 combines both without `S2aTun`!)*

### 2.2 Full 6-Stage Profile (`40` rds/blk) — Exact Post-Round-5 Gap Closure vs `53f7ecf93`

| Profiled Kernel | `S1` (`µs`) | `S2a` (`µs`) | `S2b` (`µs`) | `S3a` (`µs`) | `S4` (`µs`) | **`Pre (S1–S4)`** | `S5` (`µs`) |
|---|---:|---:|---:|---:|---:|---:|---:|
| `Prof-Mst53f7` (`53f7ecf93`) | `2,044.9` | `370.8` | `1,213.0` | `872.8` | `1,598.1` | `6,099.6 (1.0000×)` | `4,533.3` |
| `Prof-C43Base` (no `S2aTun`/`S2b-NI`) | `1,726.7` | **`330.0`** | `1,174.7` | **`742.8`** | `1,514.7` | `5,488.9 (0.8999×)` | `4,155.2` |
| `Prof-C44Head` (`+S2aTun + S2b-NI`) | **`1,696.5`** | `343.6` | **`1,143.6`** | `748.9` | **`1,513.9`** | **`5,446.5 (0.8929×)`** | `4,160.2` |
| `Prof-Nobi53f7` ($\mu=1$ base) | `2,100.0` | `367.6` | `1,218.4` | `875.0` | `768.8` | `5,329.8 (1.0000×)` | `5,828.8` |
| `Prof-C44Nobi` (`+S2aTun + S2b-NI`) | **`1,701.9`** | `337.9` | **`1,123.2`** | **`745.7`** | **`763.8`** | **`4,672.5 (0.8767×)`** | `5,790.4` |

**Stage-by-stage closure achieved before stacking `S3a-C8` (and before removing `S2aTun`'s `+13.6 µs` `S2a` regression):**
- **`S1`:** `−348.4 µs` (`−17.0%`, `−398.1 µs / −19.0%` on `Nobi`) — **`~190 µs` below Wormhole's `1,889 µs` C26 `S1` floor!**
- **`S2a`:** `−40.8 µs` (`−11.0%`, `C43Base` without `S2aTun`).
- **`S2b`:** `−69.4 µs` (`−5.7%`, `−95.2 µs / −7.8%` on `Nobi`).
- **`S3a`:** `−130.0 µs` (`−14.9%`) — already below Wormhole's `761 µs` `S3` floor even before `S3a-C8`'s extra `−33.3 µs`!
- **`S4`:** `−84.2 µs` (`−5.3%`) on `head` `wire_xor`; on `nobi`, `W16` (`763.8 µs`) vs `53f7` auto-vectorized `chunks_exact(2)` (`768.8 µs`) reveals that Cycle 39's scalar `W8` had actually de-vectorized `chunks_exact(2)` by `+50 µs` before `W16` recovered it — Cycle 45 will A/B `chunks_exact(2)` directly against `W16` on `nobi`!

### 2.3 `2T` (`35` passes) & `4T` (`25` passes) Concurrent Barrier Contention Audit

| Candidate | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) | `4T` p10 (`µs`, ratio) | `4T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,473.0 (1.0000×)` | `10,599.5 (1.0000×)` | `20,826.9 (1.0000×)` | `22,368.3 (1.0000×)` |
| `MasterA2` (A/A) | `10,461.9 (0.9989×)` | `10,615.5 (1.0015×)` | `21,593.4 (1.0368×)` | `22,343.4 (0.9989×)` |
| `C43-Base` | `9,907.6 (0.9460×)` | `10,015.2 (0.9449×)` | **`19,626.0 (0.9423×)`** | `21,089.6 (0.9428×)` |
| `S2aTun` | `9,934.5 (0.9486×)` | `10,043.3 (0.9475×)` | `20,415.5 (0.9802×)` | `20,827.1 (0.9311×)` |
| **`C44-Head`** | **`9,868.7 (0.9423×)`** | **`10,002.3 (0.9437×)`** | `20,034.8 (0.9620×)` | **`20,990.5 (0.9384×)`** |
| `Master-Nobi` | `10,813.8 (1.0000×)` | `11,011.0 (1.0000×)` | `22,975.2 (1.0000×)` | `23,819.4 (1.0000×)` |
| **`C44-Nobi`** | **`10,328.0 (0.9551×)`** | **`10,462.5 (0.9502×)`** | **`21,648.9 (0.9423×)`** | **`22,559.4 (0.9471×)`** |

---

## 3. One-Line Kills & Promotions

- **KILL `S2aTunnel` (`+13.6 µs` in `S2a`, `+26.9 / +28.1 µs` `2T`, `+9.4 µs` `Nobi Pre`):** Tunneling `to` through `L::BOTTOM` adds a Pass-1 store into `edges` and triggers a read-modify-write store-forwarding stall with `append_to(..., 1)` in Pass 2, losing to L2-resident `strips_raw` reads.
- **PROMOTE `S2b-NI` (`#[inline(never)]` `eliminate_narrow_plain`, `−52.0 µs` `Pre`, `−31.1 µs` `S2b`, `−39 µs` `2T` p10, `+0 B` WS):** Eliminates Cycle 43's alignment jitter on `head` and pushes `S2b` down to `1,143.6 µs` (`head`) / `1,123.2 µs` (`nobi`), cutting `C44-Head` `Pre` below `0.90×` (`0.8925×`) and `2T` to `0.9423× / 0.9437×`.
- **PROMOTE `S3a-C8` (`8`-bit `pairs` `10 KiB` HT + `bands.reserve_exact(8192)`, `−33.3 µs` `Pre`, `−53.9 µs` `1T`, `6/6` wins, `+0 B` steady-state WS):** Halves `pairs.fill(0)` clearing work and tightens `Dictionary` L1d footprint to `10 KiB`.

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle44/` (`gen_c44.py`, `run_c44.rs`).
- **Cycle 45 (Neon — Final Round-5 Grand Synthesis & 10-Cycle Clean Handoff):**
  1. Assemble spotless, zero-warning (`-D warnings`, `#![forbid(unsafe_code)]`) drop-in modules **`cand_r5_head/`** and **`cand_r5_nobi/`** combining ALL **9 promoted Round-5 construction winners** (`Cap127 + Pt64 + S1Trim + OneBr + S2aFuse + Inline2 + FastJoin + S2b-NI + NoStats + S3Tag12 + S3Fast + S3Arr + S3a-C8 + BandW + Wire16`, without `S2aTunnel`!), preserving `STATS = true` for `#[cfg(test)]` exact `Counts` parity, and directly A/B-testing `chunks_exact(2)` vs `W16` on `nobi`.
  2. Run the full exhaustive audit (`67,968` small BWTs + `1,100` malformed incl. `100` at `n = 70,000` + `#703` `Cap::dense` + `spiral-200k` + `periodic-150k-d2` + `periodic-105k-d3` + `dense-300k` + `6/6` Wikidata + `STATS = true` exact `Counts` verification), plus `80`-round `1T` + `60`-round `Pre` + `50`-pass `2T` + `30`-pass `4T` A/A & A/B against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`.
  3. Publish the complete 10-cycle handoff journal (`Cycles 36–45`), archive `cand_r5_head/` + `cand_r5_nobi/`, clean `/tmp/cks_c36`, verify zero production diffs, push, and stop cleanly.
