# Wormhole Cycle 18 — Stage `S3+S4` Memory & Pointer-Walk Closure: Chaitin `StripBandMux` (`-39..102 KiB` Scratch) + Pohl Bidirectional Side-1 Walk vs Fused Tail-Arena (`Theorem 18.1`)

**Session:** `research/carbon-wormhole` (Round 3, Cycle 18 of 20; 8th of 10 in Round 3)  
**Baseline `origin/master`:** `90e6719663af0615fb417b038906ad2636193aa3` (`mibz2::bwt::OpenCells`)  
**Verification harness:** [`verify_cycle18.rs`](2026-11-21-wormhole-cycle18-s3-stripband-mux-and-bidi-bottom/verify_cycle18.rs) (`#![forbid(unsafe_code)]`, `0` warnings, `5,229` Gate G1 + `BidiBottom` `0/1/2/3/N` + `StripBandMux` + `SPIRALS` cases byte-exact + `21`-round rotating native A/B + A/A + `M0` + `C17` confrontation), [`s3_stripband_bidi.toml`](2026-11-21-wormhole-cycle18-s3-stripband-mux-and-bidi-bottom/s3_stripband_bidi.toml) (`PASS`), [`cycle18_output.txt`](2026-11-21-wormhole-cycle18-s3-stripband-mux-and-bidi-bottom/cycle18_output.txt), [`cycle18_results.json`](2026-11-21-wormhole-cycle18-s3-stripband-mux-and-bidi-bottom/cycle18_results.json)

---

## 1. One-Line Keep & Kill Decisions (Cycle 18)

- **PROMOTE `B1: Chaitin StripBandMux + Pohl Bidirectional Side-1 Bottom Walk` (on top of `C17: Lazy15` + `C15: GradedRV`):** Cuts per-worker working set by **`-39` to `-102 KiB`** (`1,404–1,929 KiB = 1.37–1.884 MiB`, bringing **all 6 blocks strictly under `1.89 MiB`/worker** so `10` i9 workers fit in `18.84 MiB <= 20 MiB` L3), halves serial pointer-chase depth on `S3`'s `side-1` bottom walk (`D = H/2` via 2-way meet-in-the-middle), wins **`7/7` blocks on `S3+S4`** against both `C17:Lazy15` (`-0.4` to `-11.5 µs`) and `M0:Clone` (`-22.5` to `-60.0 µs`, up to `-11.6%`), and wins **`7/7` blocks end-to-end** against `M0:Clone` (**`0.938×–0.990×`**, mean `0.9577×` vs `C17`'s `0.9615×`) and `A1:Master` (**`0.846×–0.892×`**).
- **KILL `B2: FusedS3S4-TailArena` (Fusing `S3` Band extraction directly into `S4` `Reader::wire` by shifting `Elim` arena to `edges[m..]`):** Killed by **Theorem 18.1 (Arena–Edges Chaitin Live-Range Exclusion Law)** — holding `edges[0..m)` and `Elim` live simultaneously inflates per-worker working set by **`+156` to `+493 KiB`** (up to `2,422 KiB = 2.37 MiB`/worker on `block-143.L`, **`23.7 MiB > 20 MiB` i9 L3 at 10 workers**!) and regresses `S3+S4` on `5/6` Wikidata blocks (`+1.3` to `+11.7 µs` vs `C17`) due to L1d/LFB interference between random `Elim` SLP rule loads and `OutQ16` page streams.

---

## 2. The Two Chains Tested in Cycle 18

Cycle 17's handoff posed a sharp structural question on `Stage S3+S4`: why does `origin/master` materialize `self.bands: Vec<Band>` (`49–98 KiB`, written in `S3a` and immediately read & discarded in `S4`) instead of either **(a)** multiplexing its lifetime with another dead buffer, or **(b)** deleting `Band` altogether by streaming `S3` directly into `Reader::wire` while `Elim`'s arena sits above `edges[0..m)`?

We tested both routes head-to-head on top of `C17: Lazy15` (`GradedRV` + `pop_tail` + `Prefix-15 DirectInline16`):

### Chain A (`B1`, PROMOTED): Chaitin (1982) `Strip`–`Band` Lifetime Overlay + Pohl (1971) Circular Doubly-Linked Meet-in-the-Middle

1. **Link 1 — Chaitin (*POPL* 1982) Interference-Graph Lifetime Colouring on `(strips, bands)` (`proved` + `verified`):**
   - `strips` (`4,100–8,150` thick strips, `65–131 KiB`) is populated in `S1`, read in `S2a` to seed `Elim`'s `side-0` and `side-1` doubly linked lists, and is **100% dead before `Elim::eliminate` (`S2b`) begins** (`open.rs:647`).
   - `bands` (`3,300–8,350` band triples, `40–100 KiB`) is clear until `S3a` (`after S2b` finishes!) and dies at the end of `S4`.
   - Therefore `Live(strips) ∩ Live(bands) = ∅`: multiplexing both into a single `sb_buf` buffer eliminates the second heap `Vec`, keeps the `65–131 KiB` L2 cache lines warm across `S1 -> S2a -> S3a -> S4`, and cuts persistent per-worker scratch by **`-39` to `-102 KiB`** across all 6 Wikidata blocks (`1,929 KiB = 1.884 MiB` worst-case on `block-143.L` vs `2,022 KiB = 1.975 MiB` on `M0`!). *(Note for Cycle 19 synthesis: packing `Strip` as `[from, to, (width << 8) | u32::from(byte)]` makes `Strip` and `Band` share the exact `12 B` `[u32; 3]` type, saving another `-16..33 KiB` during `S1–S2a` with single-check slice indexing!)*
2. **Link 2 — Pohl (1971) Bi-Directional Search on `Elim`'s Circular Doubly-Linked `side-1` List (`proved` + `verified`):**
   - In `origin/master` (`open.rs:664-673`), `S3` opens with a strictly serial `MLP = 1` forward pointer chase (`id = next(0, 1)`) across all surviving `side-1` critical intervals (`~4,500–13,500` nodes scattered across the `450–1,350 KiB` `Elim` arena) to assign each interval's retained rank `bottom(id) = x`.
   - Because `Elim::close(1, ...)` (`endpoint.rs`) *already* constructs `side-1` as an exact circular doubly-linked list (`next(0, 1) = head`, `prev(0, 1) = tail`) and total retained count `m` is known upfront from `S1`, a **Pohl (1971) 2-way meet-in-the-middle walk** (`compute_bottom_bidi(m)`) advances `fwd = next(fwd, 1)` from `x_fwd = 0` upward and `bwd = prev(bwd, 1)` from `x_bwd = m` downward simultaneously in lockstep (`terminating cleanly on 0, 1, odd, and even node counts via fwd == bwd` and asserting `x_fwd == x_bwd`), cutting `side-1` serial pointer-chase depth in half (`D = H / 2`, `MLP = 2`) at **`0 B` extra memory and `0` extra instructions per node**!

### Theorem 18.1 — Arena–Edges Chaitin Live-Range Exclusion Law (`proved` + `verified`, Killing `B2: FusedS3S4-TailArena`)

- **Statement (`proved`):** In `origin/master` and `B1`, `Elim`'s arena (`|A| = 4 * (ns + top)` bytes, `1,100–1,480 KiB`) occupies `edges[0..]` during `S2–S3`, and `edges[0..m)` (`4m` bytes, `346–807 KiB`) overwrites `edges[0..m)` in `S4` *only after* `Elim` dies at the end of `S3` — decoupled solely by `bands` (`|B| = 12 * n_bands` bytes, `40–100 KiB`). Any fused `S3+S4` pass (`B2`) that writes `edges[0..m)` while reading `Elim` must keep both `edges[0..m)` and `Elim` live simultaneously, raising peak `edges` footprint from $\max(|A|, 4m) + |B|$ to $4m + |A|$, an unavoidable working-set penalty of:
  $$\Delta M_{\mathrm{fused}} = \min(|A|, 4m) - |B| = 4m - |B| \in [+156\text{ KiB}, +493\text{ KiB}] \quad \text{on Wikidata!}$$
- **Empirical Witness (`measured` in [`verify_cycle18.rs`](2026-11-21-wormhole-cycle18-s3-stripband-mux-and-bidi-bottom/verify_cycle18.rs)):**
  - On `block-143.L`, `B2` inflates per-worker working set from `1,929 KiB` (`B1`) to **`2,422 KiB` (`2.37 MiB`)** — at `10` i9 workers, `23.7 MiB > 20 MiB` L3, **breaking `#692`'s L3 residency boundary**!
  - Furthermore, even at 1 worker, interleaving random SLP rule loads from `e_f.a` with `OutQ16` `4 KiB` page reads and `edges_out[dst..]` RFO stores inside one loop degrades L1d/LFB locality, making `B2` **`+1.3` to `+12.7 µs` slower on `S3+S4`** than `B1` on `6/6` Wikidata blocks!

---

## 3. Stage `S3+S4` & End-to-End Native Safe-Rust Confrontation (`21` Rotating Rounds)

- **Per-Worker Working Set (`B1: MuxBidi`):** **`1,404–1,929 KiB` (`1.371–1.884 MiB`)** on Wikidata (`-39` to `-102 KiB` below `M0:Clone`'s `1,445–2,022 KiB`, so `10` workers = `18.84 MiB <= 20 MiB` L3 even on worst-case `block-143.L`!).
- **DRAM Traffic:** `0.034–0.059 B/B` (`0` extra `2048K` L3 misses).
- **Acquisition Cost:** ~35 lines in `src/open/endpoint.rs` (`compute_bottom_bidi`) + multiplexing `strips`/`bands` in `src/open.rs`, `#![forbid(unsafe_code)]`.

| Block | WS (`M0 / B1 / B2 KiB`) | `M0` (`S34 µs`) | `C17` (`S34 µs`) | **`B1:MuxBidi`** (`S34 µs`) | `B2:Fused` (`S34 µs`) | `A2/A1` | `C17 / M0` | **`B1 / M0`** | **`B1 / A1`** | `B2 / M0` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1445 / 1404 / 1741` | `303.8` | `274.0` | **`272.0`** (`-10.5%`) | `275.2` | `0.988x` | `0.944x` | **`0.945x`** | **`0.887x`** | `0.942x` |
| `block-2.L` | `1590 / 1526 / 1900` | `377.2` | `342.5` | **`335.9`** (`-11.0%`) | `348.5` | `0.992x` | `0.931x` | **`0.938x`** | **`0.885x`** | `0.933x` |
| `block-85.L` | `1640 / 1601 / 1754` | `222.9` | `204.5` | **`200.4`** (`-10.1%`) | `208.2` | `0.987x` | `0.971x` | **`0.973x`** | **`0.882x`** | `0.960x` |
| `block-143.L` | `2022 / 1929 / 2422` | `597.6` | `537.9` | **`537.6`** (`-10.0%`) | `549.3` | `1.008x` | `0.952x` | **`0.953x`** | **`0.888x`** | `0.962x` |
| `block-210.L` | `1964 / 1862 / 2137` | `354.2` | `331.7` | **`329.2`** (`-7.1%`) | `329.2` | `0.998x` | `0.980x` | **`0.962x`** | **`0.892x`** | `0.958x` |
| `block-207.L` | `1697 / 1627 / 1967` | `391.1` | `357.2` | **`345.8`** (`-11.6%`) | `358.3` | `0.994x` | `0.956x` | **`0.943x`** | **`0.875x`** | `0.956x` |
| `dense-703.syn` | `1150 / 1150 / 1160` | `35.8` | `35.6` | **`35.0`** (`-2.2%`) | `32.7` | `0.991x` | `0.998x` | **`0.990x`** | **`0.846x`** | `0.998x` |

---

## 4. Next Step (Cycle 19)

Stages `S1`, `S2`, `S3`, and `S4` inside `open.rs` are now bounded from below and squeezed from above! Two obligations remain before Cycle 20's full 18-block multi-worker confrontation and final Round 3 handoff:
1. **Upstream `runs::front` Boundary Audit (Obligation #5):** Quantify the exact `CAP = 127` record-splitting overhead in `src/runs.rs` (`records.len()` vs `maximal_runs`), test whether widening `CAP = 127 -> 255` (`u16` `(sym:8, width:8)`) or upstream maximal-run coalescing saves anything in `S1`, and either promote or cleanly bound & close Obligation #5.
2. **Unified `WormholePreEmitCells` Synthesis (`[u32; 3]` `StripBandMux` + `pop_tail` + `GradedHandle` + `Lazy15-DirectInline16` + `BidiBottom`):** Assemble all promoted zero-regression Round 3 mechanisms into the clean drop-in `WormholePreEmitCells: InverseBwt` (`with S5 emit held 100% verbatim identical to origin/master in headline Arm B1, plus orthogonal composition check with Round 2's Relay8 in diagnostic Arm B2`).
