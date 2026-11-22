# Carbon Kitchen Sink (Neon) — Cycle 42: Fixed-Array `S3Arr` + Direct-Span `BandW` (`0.9448×` `2T`) & `Pair2` Register-Spill Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Structural Framing

Cycle 42 tested three structural levers across `S1`, `S3a`, and `S4` atop `Neon-C41`:
1. **`Pair2` (`S1` Distinct-Byte 2-Run Pipeline, Theorem 42.1):** In canonical `CAP = 127 > LIMIT = 16` records, consecutive thin runs (`w0 <= 16, w1 <= 16`) satisfy `c0 != c1`, guaranteeing disjoint `pt[a0]`/`pt[a1]` and disjoint `out` chunks. Does 2-run unrolling overlap write-allocate latency, or trip the 16-GPR spill cliff + Theorem 27.2 single-path AVX2 law?
2. **`S3Arr` (`S3a` Compile-Time Zero-Bounds-Check `Dictionary`):** Converting the arena-hosted `12 KiB` cache slice into fixed-size `&mut [u32; 1024]` (`pairs`) and `&mut [u32; 2048]` (`contents`) via `try_into().unwrap()` (with a cold tiny-block fallback) eliminates `is_empty()` branches and proves `pairs[slot]`, `pairs[slot+1]`, and `contents[h & 2047]` in-bounds at compile time.
3. **`BandW` (`S3a -> S4` Direct-Span `Band64`):** Storing band `width` directly in the low 20 bits of `Band(u64)` (rather than cumulative `k + width`) eliminates the loop-carried `k = band.end(); span = bend - k` subtraction chain in `S4` (`wire_xor` and serial `wire`).

---

## 2. Native Safe-Rust A/B Confrontation (`run_c42.rs`)

Built atop `Neon-C41` (`OneBr + FastJoin + S1Trim + NoStats + S3Fast + Inline2 + Cap127 + Pt64 + Wire16 + S2aFuse + S3Tag12`) and compared against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`, compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`).

- **Parity & Scratch:** `100%` byte-exact on `6/6` Wikidata blocks + `dense-300k` + `#703` `Cap::dense(300_000)` refusal (`None`); persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 `1T` Interleaved (`50` rounds/block, `35`-round `Pre` split) & `2T` Concurrent Barrier (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs `C41` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,688.9 (1.0000×)` | `6,081.5 (1.0000×)` | `+550.1` | `10,517.9 (1.0000×)` | `10,585.7 (1.0000×)` |
| `MasterA2` (A/A) | `10,719.4 (1.0029×)` | `6,094.1 (1.0021×)` | `+562.7` | `10,465.3 (0.9950×)` | `10,569.7 (0.9985×)` |
| `Neon-C41` (prev champ) | `10,176.7 (0.9521×)` | `5,531.4 (0.9095×)` | `0.0` | `9,974.9 (0.9484×)` | `10,057.9 (0.9501×)` |
| `Pair2` (`S1` 2-run unroll) | `10,407.4 (0.9737×)` | `5,765.8 (0.9481×)` | `+234.4` | `10,221.7 (0.9718×)` | `10,316.0 (0.9745×)` |
| **`S3Arr` (`[u32;1024/2048]`)** | **`10,115.2 (0.9463×)`** | **`5,507.7 (0.9057×)`** | **`−23.7`** | **`9,937.0 (0.9448×)`** | **`10,015.4 (0.9461×)`** |
| **`BandW` (direct-span `Band`)** | **`10,132.2 (0.9479×)`** | **`5,511.4 (0.9063×)`** | **`−20.0`** | — | — |
| `C42-Cmb` *(incl. Pair2)* | `10,320.3 (0.9655×)` | `5,725.7 (0.9415×)` | `+194.3` | `10,191.3 (0.9689×)` | `10,252.8 (0.9686×)` |
| `Master-Nobi` ($\mu=1$ base) | `11,142.9 (1.0000×)` | `5,331.9 (1.0000×)` | `+529.5` | `10,918.2 (1.0000×)` | `11,013.7 (1.0000×)` |
| **`Nobi-C41`** | **`10,577.8 (0.9493×)`** | **`4,802.4 (0.9007×)`** | `0.0` | — | — |
| `Nobi-C42` *(incl. Pair2)* | `10,806.1 (0.9698×)` | `5,014.5 (0.9405×)` | `+212.1` | `10,647.5 (0.9752×)` | `10,739.1 (0.9751×)` |

Per-block `1T` wins for **`S3Arr`** vs `MasterA1` (A/A `1.0029×`):
`block-0: 0.948`, `block-2: 0.954`, `block-85: 0.945`, `block-143: 0.948`, `block-210: 0.940`, `block-207: 0.947` (`6/6` clean wins!).

---

## 3. One-Line Kills & Promotions

- **KILL `Pair2` (`+234.4 µs` `Pre`, `+246.8 / +258.1 µs` `2T`):** Unrolling 2 thin runs pushes live GPRs past x86-64's 16-register ceiling and introduces a second `vmovdqu` store path in `S1`, confirming Theorem 37.1 & Wormhole Theorem 27.2.
- **PROMOTE `S3Arr` (`−23.7 µs` `Pre`, `−37.9 / −42.5 µs` `2T`) & `BandW` (`−20.0 µs` `Pre`) (`Neon-C42`, `+0 B` WS):** Eliminate `Dictionary` bounds checks and `S4`'s `bend - k` loop-carried dependency, reaching **`0.9057×` `Pre`** (`0.9463×` `1T`, **`0.9448×` p10 / `0.9461×` p50 `2T`** vs A/A `0.9950× / 0.9985×`) even before combining both cleanly!

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle42/` (`gen_c42.py`, `run_c42.rs`).
- **Cycle 43 (Neon):** Combine **`S3Arr + BandW`** cleanly and attack two untouched architectural frontiers in **`S2b` (`1,144 µs`)** and **`S3a` (`699 µs`)**:
  1. **Zero-`bands` / `Band` Pre-Reserve + `S3a` Byte-Slice Partition (`S3a`):** Can we eliminate `self.bands.push()` capacity checks via `bands.reserve_exact` or test `PAIR_BITS = 8` (`512` `u32` = `2 KiB` pairs + `8 KiB` contents = `10 KiB` HT)?
  2. **Algebraic Elimination of `Elim::assign_bottom` Pass (`S3a` `bottom`) OR `S2b` Single-Load `Narrow` Packed `prev|next` + `width|word` (`u64` Node Read)?** Test whether reading `width` & `word` as a single `u64` / `sl = &a[id*4..id*4+4]` inside `S2b`'s `eliminate` loop cuts bounds checks and loads in `S2b`!
  3. **Memory-Diet Bonus (`M_ws`):** Can we cut persistent scratch **BELOW `945.8 KiB`** (e.g., `inline16` exact capacity vs `1536` hint, `bands` exact capacity, `spelling` exact-fit)?
