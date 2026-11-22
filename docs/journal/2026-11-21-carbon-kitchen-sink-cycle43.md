# Carbon Kitchen Sink (Neon) — Cycle 43: Clean `S3Arr+BandW` (`0.9400×` `2T`), `16`-Wide Serial `Nobi-W16` (`0.8834×` `Pre`), & `S2b-NarrowSlice` Isolation

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Structural Framing

Cycle 43 cleanly assembled `S3Arr + BandW` without Cycle 42's killed `Pair2` and confronted two remaining construction frontiers in **`S2b` (`Elim::eliminate`)** and **`S4` (serial `OutQ16::wire` for `nobi`)**:

1. **`S2b-NarrowSlice` (`Elim::eliminate_narrow_plain`):** On 100% of Wikidata blocks (and every block with `< 65,536` open nodes), `Links` is `Narrow` (`STRIDE = 4`: `[side0_prev|next, side1_prev|next, width, word]`). In `eliminate::<false>`, each plain step performs `16` separate bounds-checked `self.a[id*4 + off]` accesses and re-reads `prev(x,0)`, `next(y,0)`, and `y[0]` during `pop_tail + insert + cut`. Specializing `!SPIRALS && L::IS_NARROW` via single-slice `&self.a[xi..xi+4]` / `&self.a[yi..yi+4]` reads and direct fused `aw < bw` / `aw == bw` / `aw > bw` link rewrites cuts bounds checks `16 → 7` and eliminates 3 redundant loads/step (exact algebraic equivalence across all aliasing cases including `y == px`, `x == py`, `y_prev == 0`, verified on `5,000` exhaustive small/adversarial BWTs!).
2. **`Nobi-W16` (`16`-Wide `u64`-Unpacked Serial `OutQ16::wire`):** In `nobi` ($\mu = 1$ serial `wire` + `emit`, matching the i9 diagnostic where two-ended emission proved neutral), Cycle 39's `8`-wide `u64` unpack left `2×` loop overhead on long thin-queue chunks (`CHUNK = 1,024` bytes = `512` slots). Extending the hot inner loop to `16` slots (`32` bytes = 4× `u64` loads → 16 `u32` stores per iteration, with `8`-wide / `1`-wide tails) halves loop control and widens store-buffer coalescing with `+0 B` scratch.

---

## 2. Native Safe-Rust A/B Confrontation (`run_c43.rs` & `run_c43_check.rs`)

Compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`) against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`.

- **Parity & Scratch:** `100%` byte-exact across **`5,000` pseudo-random/small BWTs (`n = 1..384`, `|Σ| ∈ {2, 3, 4, 8, 26}`)** + `6/6` Wikidata blocks + `dense-300k` + `#703` `Cap::dense(300_000)` refusal (`None`); persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 Primary Interleaved A/A & A/B (`run_c43.rs`: `50` rds/blk `1T`, `35` rds/blk `Pre` split, `35` passes `2T`)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs Base | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,626.3 (1.0000×)` | `6,018.7 (1.0000×)` | `+593.5` | `10,493.5 (1.0000×)` | `10,562.2 (1.0000×)` |
| `MasterA2` (A/A) | `10,666.0 (1.0037×)` | `6,050.5 (1.0053×)` | `+625.2` | `10,453.8 (0.9962×)` | `10,568.2 (1.0006×)` |
| `Neon-C41` (prev champ) | `10,092.3 (0.9497×)` | `5,456.5 (0.9066×)` | `+31.2` | — | — |
| **`C43-Base` (`S3Arr+BandW`)** | **`10,021.1 (0.9430×)`** | **`5,425.3 (0.9014×)`** | **`0.0`** | **`9,863.5 (0.9400×)`** | **`9,979.9 (0.9449×)`** |
| `C43-S2b` (`+NarrowSlice`) | `10,066.6 (0.9473×)` | `5,452.5 (0.9059×)` | `+27.2` | `9,897.1 (0.9432×)` | `9,988.0 (0.9456×)` |
| `Master-Nobi` ($\mu=1$ base) | `11,027.7 (1.0000×)` | `5,251.7 (1.0000×)` | `+512.1` | `10,873.6 (1.0000×)` | `10,961.1 (1.0000×)` |
| `Nobi-C42` (`S3Arr+BandW+W8`) | `10,521.7 (0.9541×)` | `4,739.6 (0.9025×)` | `0.0` | `10,383.2 (0.9549×)` | `10,492.4 (0.9572×)` |
| `Nobi-C43` (`+NarrowSlice`) | `10,471.8 (0.9496×)` | `4,696.9 (0.8944×)` | `−42.7` | `10,368.7 (0.9536×)` | `10,436.2 (0.9521×)` |
| **`Nobi-W16` (`+NarrowSlice+W16`)** | **`10,403.2 (0.9434×)`** | **`4,639.6 (0.8834×)`** | **`−100.0`** | **`10,301.7 (0.9474×)`** | **`10,377.7 (0.9468×)`** |

Per-block `1T` wins for **`C43-Base`** vs `MasterA1` (A/A `1.0037×`):
`block-0: 0.952`, `block-2: 0.952`, `block-85: 0.941`, `block-143: 0.942`, `block-210: 0.936`, `block-207: 0.941` (`6/6` beat `Neon-C41`!).
Per-block `1T` wins for **`Nobi-W16`** vs `Master-Nobi`:
`block-0: 0.938`, `block-2: 0.941`, `block-85: 0.953`, `block-143: 0.945`, `block-210: 0.941`, `block-207: 0.941` (`6/6` beat `Nobi-C42`!).

### 2.2 `2×2` Factorial Isolation of `S2b-NarrowSlice` vs `W16` (`run_c43_check.rs`, `40` rds/blk)

Because `cand_c43_s2b/endpoint.rs` and `cand_c43_nb43/endpoint.rs` are **byte-for-byte identical** yet moved oppositely (`+27.2 µs` vs `−42.7 µs`) in `c43_bin`, we immediately ran an isolated `2×2` factorial binary (`c43_chk`):

| Variant | `Pre` (`µs`) | `ΔPre` vs Row Base | `Emit` (`µs`) |
|---|---:|---:|---:|
| `Head-Base` (`C43-Base`) | `5,466.4` | `0.0` | `4,558.3` |
| `Head-S2b` (`+NarrowSlice`) | `5,453.3` | **`−13.1`** | `4,537.6` |
| `Nobi-W8` (`Nobi-C42`) | `4,760.1` | `0.0` | `5,807.1` |
| `Nobi-W8+S2b` (`+NarrowSlice`) | `4,687.3` | **`−72.8`** | `5,765.9` |
| `Nobi-W16` (`+W16` alone) | `4,704.8` | **`−55.3`** | `5,780.3` |
| **`Nobi-W16+S2b` (both)** | **`4,666.7`** | **`−93.4`** | `5,785.3` |

`W16` is an unambiguous, layout-invariant **`−55.3` to `−57.3 µs`** construction win on `Nobi`, whereas `S2b-NarrowSlice` wins consistently on `Nobi` (`−42.7` to `−72.8 µs`) and `−13.1 µs` on `Head` in `c43_chk`, but exhibits code-layout/inlining sensitivity (`+27.2 µs`) on `Head` in `c43_bin` — so we defer final `S2b-NarrowSlice` promotion on `Head` to Cycle 44's 6-stage `S2b`-isolated profiler!

---

## 3. One-Line Promotions & Deferrals

- **PROMOTE `C43-Base` (clean `S3Arr + BandW`, `+0 B` WS):** Sets new `head` records at **`0.9014×` `Pre` (`−593.5 µs`)**, **`0.9430×` `1T`**, and **`0.9400×` p10 / `0.9449×` p50 `2T`** (A/A `0.9962× / 1.0006×`), beating `Neon-C41` on `6/6` blocks (`−71.2 µs` `1T`).
- **PROMOTE `Nobi-W16` (`16`-wide `u64` serial `OutQ16::wire`, `+0 B` WS):** Cuts `Nobi Pre` by **`−55.3` to `−57.3 µs`** over `8`-wide serial `wire`, pushing `Nobi` to **`0.8834×` `Pre` (`−612.1 µs` vs `Master-Nobi`!!)**, **`0.9434×` `1T`**, and **`0.9474× / 0.9468×` `2T`**.
- **DEFER `S2b-NarrowSlice` (verify on `Head` in Cycle 44 6-stage profiler):** Verified 100% byte-exact on `5,000` fuzz BWTs + Wikidata; wins `−38..−73 µs` on `Nobi` in both binaries and `−13 µs` on `Head` in `c43_chk`, but showed `+27 µs` code-layout jitter on `Head` in `c43_bin` — needs direct `S2b` stage measurement (`#[inline(never)]` / `#[inline(always)]` check) before promoting into `head`.

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle43/` (`gen_c43.py`, `run_c43.rs`, `run_c43_check.rs`).
- **Cycle 44 (Neon):**
  1. **Resolve `S2b-NarrowSlice` (`#[inline(always)]` vs `#[inline(never)]`) + test two final low-level construction levers:**
     - **`S2a` Direct-`to` Second Pass (`S2aNoUnpack`):** In `S2a`'s second pass over `image_order`, avoid unpacking unused `(from, byte)` from `strips_raw[3*idx..3*idx+3]` and read only `(to, width)` cleanly (plus check `bands.reserve_exact(8192)` in `S3a` to cut Vec growth checks).
     - **`S3a` `PAIR_BITS = 8` (`10 KiB` HT: `2 KiB` pairs + `8 KiB` contents) vs `PAIR_BITS = 9` (`12 KiB` HT):** Check L1d pressure vs collision rate.
     - **`Head` `Wire16` `mark_byte` 32-byte (`16`-slot) inner loop check** (confirm `head` `wire_xor` already matches `W16`'s `16`-wide sweet spot!).
  2. **Full 6-stage (`S1, S2a, S2b, S3a, S4, S5`) post-stack re-profile** of both `head` and `nobi` champions against verbatim `53f7ecf93` and Wormhole's C22–26 `BMCP_CML` bounds, plus **`1T / 2T / 4T`** concurrent barrier contention audit!
