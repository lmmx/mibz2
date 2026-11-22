# Carbon Kitchen Sink (Neon) — Cycle 39: Zero-`Counts` Hot-Path Specialization (`STATS=false`), Branchless `S3Fast`, & `u64`-Unpacked Serial `S4` (`0.928×–0.930×` `Pre`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack, `945.8 KiB` scratch)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Mathematical & Structural Framing — Attacking `S3a` (`783 µs`) & Serial `S4` (`763 µs`)

Auditing `origin/master` (`53f7ecf93`) revealed three orthogonal construction overheads surviving across `S1–S4`:
1. **Unconsumed Telemetry in Production (`NoStats`, `const STATS: bool = false`):** `OpenCells::invert_runs` discards `Counts`, yet `S1` increments `f.runs` `~513K` times, `S2b` increments `taken` `~195K` times, and `S3a` executes **six `Counts` field RMWs + a 64-bit `width * height` multiply + a `SATURATED` branch** on every non-zero band! Gating telemetry behind `const STATS: bool` (`true` for tests/`checks.rs`, `false` for `OpenCells` production) preserves 100% test compatibility with zero runtime cost.
2. **Branchless `Inline2` Keying & Single-Slice Cached Node Reads (`S3Fast`):** Offsetting rule IDs by `65_792` collapses `Dictionary::code`'s 2-branch `uid` dispatch to one branchless ALU expression `uid = id + (u32::from(g == 2) << 8)`. Replacing 4 separate bounds-checked `e.a` loads per band with one 16-byte slice fetch `Links::band_node(e.a, id) -> (next0, bottom, width, word)` cached across byte boundaries eliminates redundant node reloads and bounds checks.
3. **`8`-Wide `u64`-Unpacked Serial `OutQ16::wire` (`Nobi-W8`):** For the `nobi` ($\mu = 1$) regime (which matched `head` on the i9), unpacking `2 × u64::from_le_bytes` (`8` ranks per 2 loads) into `&mut dst[i..i + 8]` (1 bounds check) accelerates serial `S4` wiring.

---

## 2. Native Safe-Rust A/B Confrontation (`run_c39.rs`)

Built atop `Neon-C38` (`Inline2 + Cap127 + Pt64 + Wire16 + S2aFuse + S3Tag12`) and compared in the same binary against verbatim `origin/master` (`53f7ecf93`) and `Master-Nobi`, compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`).

- **Parity & Scratch:** `100%` byte-exact across all `10` kernels on `6/6` Wikidata blocks + `dense-300k`; persistent scratch unchanged at **`945.8 KiB` (`+0.00%`)**.

### 2.1 `1T` Interleaved (`50` rounds/block, `35`-round `Pre` split) & `2T` Concurrent Barrier (`35` passes)

| Candidate | `1T` Total (`µs`, ratio vs base) | `1T` `Pre` (`µs`, ratio vs base) | `ΔPre` vs `C38` | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,391.7 (1.0000×)` | `5,897.9 (1.0000×)` | `+268.8` | `10,546.1 (1.0000×)` | `10,595.1 (1.0000×)` |
| `MasterA2` (A/A) | `10,391.3 (1.0000×)` | `5,915.9 (1.0031×)` | `+286.8` | `10,561.3 (1.0014×)` | `10,615.4 (1.0019×)` |
| `Neon-C38` (prev champ) | `10,161.2 (0.9778×)` | `5,629.1 (0.9544×)` | `0.0` | `10,310.0 (0.9776×)` | `10,379.0 (0.9796×)` |
| `NoStats` (`STATS=false`) | `10,034.0 (0.9656×)` | `5,530.2 (0.9376×)` | `−98.9` | — | — |
| `S3Fast` (alone) | `10,136.9 (0.9755×)` | `5,629.5 (0.9545×)` | `+0.3` | — | — |
| **`C39-Cmb` (`NoStats+S3Fast`)** | **`10,029.8 (0.9652×)`** | **`5,484.3 (0.9299×)`** | **`−144.8`** | **`10,208.3 (0.9680×)`** | **`10,238.0 (0.9663×)`** |
| `Master-Nobi` ($\mu=1$ base) | `10,801.1 (1.0000×)` | `5,171.3 (1.0000×)` | `+259.6` | `10,966.9 (1.0000×)` | `11,054.2 (1.0000×)` |
| `Nobi-C38` | `10,584.6 (0.9800×)` | `4,911.7 (0.9498×)` | `0.0` | — | — |
| `Nobi-C39` (`+NoStats+S3Fast`) | `10,469.2 (0.9693×)` | `4,832.2 (0.9344×)` | `−79.5` | **`10,616.7 (0.9681×)`** | **`10,674.6 (0.9657×)`** |
| **`Nobi-W8` (`+u64 wire`)** | **`10,418.1 (0.9645×)`** | **`4,799.9 (0.9282×)`** | **`−111.8`** | — | — |

Per-block `1T` wins (`6/6` in BOTH modes):
- **`C39-Cmb / MasterA1`:** `block-0: 0.966`, `block-2: 0.971`, `block-85: 0.962`, `block-143: 0.971`, `block-210: 0.965`, `block-207: 0.954` (A/A `1.0000×`).
- **`Nobi-W8 / Master-Nobi`:** `block-0: 0.971`, `block-2: 0.971`, `block-85: 0.970`, `block-143: 0.952`, `block-210: 0.965`, `block-207: 0.967`.

---

## 3. One-Line Kills & Promotions

- **KILL standalone `S3Fast` (`+0.3 µs` `Pre` with `STATS=true`):** While `stats.*` live registers crowd `S3a`, caching `(next_id, node_bottom, node_width, node_word)` yields `0` net gain; once `NoStats` frees those registers, **`NoStats + S3Fast` unlocks `−45.9 µs` extra synergy** (`−144.8 µs` vs `−98.9 µs`).
- **PROMOTE `Neon-C39` (`NoStats + S3Fast + u64-serial-wire` atop `Neon-C38`, `+0 B` WS):** Cuts total construction `Pre (S1–S4)` by **`−7.01%` (`0.9299×`, `−413.6 µs`)** on `origin/master` (`53f7ecf93`) and **`−7.18%` (`0.9282×`, `−371.4 µs`)** on `Master-Nobi`, reaching **`0.9652×` `1T` / `0.9663×` `2T` p50** (vs `53f7ecf93`, A/A `1.0000×`) and **`0.9645×` `1T`** (vs `Master-Nobi`).

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle39/` (`gen_c39.py`, `run_c39.rs`).
- **Cycle 40 (Neon):** Deep structural push on **`S1` + `S2a` + `S4`** — (i) **Zero-Alloc `S2a` Bucket-Offset Pre-Placement (`S2a-Direct`):** can we eliminate `image_order` indirection or fuse `S2a` Pass 4 (`image_order` random strip lookup) by storing `(to, width)` directly inside `image_order`? (ii) **Exact `S1` Micro-Anatomy (`Cap127+Pt64`):** can we hoist `in_out_lim` / eliminate `w <= 16` redundant check / test `OutQ16` 4-rank/8-rank SIMD vs 16-rank iota? (iii) **Exact 6-stage re-profile of `Neon-C39` vs `53f7ecf93`** to update our Remaining-Gap Ranking!
