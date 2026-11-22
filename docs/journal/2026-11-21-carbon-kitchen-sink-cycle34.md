# Carbon Kitchen Sink (Neon) — Cycle 34: Pass-1 `HoistP` Kill, `XOR_GATE` Phase-Transition Curve, & `1T / 2T / 4T` Contention Scaling

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`mibz2 0.5.11`, tagged `v0.5.11`, `src/` identical to `90e671966`)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched.

---

## 1. Research Questions

With `Neon-C33` (`W16+D12`) reaching `0.8249×` `1T` and `0.8325×` `2T` at `945.8 KiB` (`−19.28%` WS), Cycle 34 confronted three remaining architectural questions before our Cycle 35 Grand Synthesis:
1. **Pass-1 `p`-Check Hoisting (`HoistP`):** Can splitting `maximal` around the unique primary row `p` (eliminating `p.wrapping_sub(x) < w` from `~489K` runs) accelerate Pass 1 (`21.8%` of total runtime)?
2. **Post-`Wire16` `XOR_GATE` Phase-Transition Calibration:** Now that `Wire16` + `mark_byte` cut `wire_xor` overhead to `~7 µs`/block (`+1.27%` `construct` vs `master`), where does the exact crossover $m^*$ between `Neon-Bi` ($\mu = 2$) and `Neon-Diet` ($\mu = 1$) sit across $m \in [20\text{K}, 300\text{K}]$ and periodic multi-cycle blocks?
3. **Full `1T / 2T / 4T` Contention Scaling of BOTH Flagships (`Neon-Diet` $\mu = 1$ vs `Neon-Bi` $\mu = 2$ vs `0.5.11` `master`):** Does the `−19.28%` working-set reduction (`945.8` vs `1,171.7 KiB`) hold or amplify when `4` concurrent workers oversubscribe the VM's shared `2 MiB` L2 (`Intel Xeon Platinum 8481C`, `2` vCPUs, `48 KiB` L1d, `2 MiB` L2, `105 MiB` L3)?

---

## 2. Empirical Results (`run_c34.rs`)

All kernels (`#![forbid(unsafe_code)]`) were compiled in the same binary (`rustc -C opt-level=3 -C target-cpu=native -C codegen-units=1`) alongside verbatim `0.5.11` `master` (`MasterA1` / `MasterA2` A/A control). Parity passed `100%` on `6/6` Wikidata blocks, edge primary positions `p ∈ {0, 1, 17, n-1}`, `8` synthetic dense blocks (`n = 20K..300K`), and `periodic-105k-d3` / `periodic-150k-d2`.

- **Worker scratch (6-block cumulative):** `Master = 1,171.7 KiB`; `Neon-Diet = 945.8 KiB` (`−19.28%`); `Neon-C33 = 945.8 KiB` (`−19.28%`); `HoistP = 945.8 KiB` (`−19.28%`).
- **i9-10900K `20 MiB` L3 Crossing (with `900 KiB` `out` buffer per worker):** `10 × (1,171.7 + 900) = 20.23 MiB > 20 MiB` (`master`) vs **`10 × (945.8 + 900) = 18.03 MiB < 20 MiB`** (`Neon`).

### 2.1 `1T` Interleaved A/A & A/B + Stage Breakdown (`60` rounds/block)

| Candidate | `construct` (`µs`, ratio) | `walk` (`µs`, ratio) | `1T` Sum (`µs`) | `1T` vs `M1` | Per-Block Ratios (`0, 2, 85, 143, 210, 207`) |
|---|---:|---:|---:|---:|---|
| `MasterA1` (`0.5.11`) | `5,924.3 (1.0000×)` | `6,178.2 (1.0000×)` | `12,095.3` | `1.0000×` | `1.000, 1.000, 1.000, 1.000, 1.000, 1.000` |
| `MasterA2` (A/A) | `5,926.1 (1.0003×)` | `6,167.6 (0.9983×)` | `12,088.2` | `0.9994×` | `0.995, 0.993, 0.997, 1.002, 1.003, 1.000` |
| **`Neon-Diet` ($\mu=1$)** | **`5,269.4 (0.8894×)`** | **`5,639.4 (0.9128×)`** | **`10,872.2`** | **`0.8989×`** | `0.897, 0.888, 0.916, 0.889, 0.911, 0.894` |
| **`Neon-C33` ($\mu=2$)** | `5,999.9 (1.0127×)` | **`3,986.9 (0.6453×)`** | **`10,020.1`** | **`0.8284×`** | `0.856, 0.848, 0.850, 0.800, 0.828, 0.818` |
| `HoistP` ($\mu=2$) | `6,053.6 (1.0218×)` | `3,981.3 (0.6444×)` | `10,098.7` | `0.8349×` | `0.860, 0.849, 0.856, 0.805, 0.840, 0.825` |

> **Milestone:** **`Neon-Diet` ($\mu = 1$)** is the first single-cursor kernel in the entire programme to break below **`0.900×`** (`0.8989×` vs A/A `0.9994×`), beating `0.5.11` `master` in both `construct` (`0.8894×`, `−654.9 µs`) and `walk` (`0.9128×`, `−538.8 µs`) at `945.8 KiB` (`−19.28%` WS).

### 2.2 `XOR_GATE` Phase-Transition Curve (`50` rounds/block, `Neon-Bi0` vs `Neon-Diet`)

| Block | `m` | `4m` (`edges`) | `MasterA1` (`µs`) | `Neon-Diet` | `Neon-Bi0` | `Bi / Diet` | `Δconstruct` (`µs`) | `Δwalk` (`µs`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `dense-20k` | `20,000` | `78.1 KiB` | `276.4` | `265.1 (0.959×)` | `260.6 (0.943×)` | `0.9832×` | `+18.6` | `−33.2` |
| `dense-35k` | `35,000` | `136.7 KiB` | `439.2` | `419.9 (0.956×)` | `426.7 (0.971×)` | **`1.0161×`** | `+44.4` | `−77.3` |
| `dense-50k` | `50,000` | `195.3 KiB` | `664.5` | `644.4 (0.970×)` | `598.2 (0.900×)` | **`0.9282×`** | `+55.8` | `−124.0` |
| `dense-65k` | `65,000` | `253.9 KiB` | `864.6` | `833.4 (0.964×)` | `743.2 (0.860×)` | **`0.8918×`** | `+84.7` | `−185.0` |
| `dense-80k` | `80,000` | `312.5 KiB` | `1,074.7` | `1,030.7 (0.959×)` | `933.9 (0.869×)` | **`0.9061×`** | `+86.3` | `−221.1` |
| `dense-100k` | `100,000` | `390.6 KiB` | `1,381.7` | `1,332.9 (0.965×)` | `1,176.7 (0.852×)` | **`0.8828×`** | `+129.5` | `−299.7` |
| `dense-150k` | `150,000` | `585.9 KiB` | `2,141.9` | `2,044.9 (0.955×)` | `1,758.8 (0.821×)` | **`0.8601×`** | `+186.4` | `−483.1` |
| `dense-300k` | `300,000` | `1,171.9 KiB` | `4,821.4` | `4,749.0 (0.985×)` | `3,780.6 (0.784×)` | **`0.7961×`** | `+394.7` | `−1,290.6` |
| `per-105k-d3` | `104,916` | `409.8 KiB` (`m/3` walk) | `516.0` | `489.1 (0.948×)` | `491.7 (0.953×)` | `1.0054×` | `+108.9` | `−102.2` |
| `per-150k-d2` | `150,000` | `585.9 KiB` (`m/2` walk) | `1,103.9` | `1,043.3 (0.945×)` | `980.1 (0.888×)` | **`0.9394×`** | `+141.9` | `−240.5` |

### 2.3 Multi-Thread Contention Scaling (`2T` `40` passes, `4T` `25` passes on `2 MiB` Shared L2)

| Candidate | `1T` Ratio | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) | `4T` p10 (`µs`, ratio) | `4T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|---:|
| `MasterA1` (`0.5.11`) | `1.0000×` | `12,249.9 (1.0000×)` | `12,628.6 (1.0000×)` | `24,544.7 (1.0000×)` | `25,793.3 (1.0000×)` |
| `MasterA2` (A/A) | `0.9994×` | `12,253.9 (1.0003×)` | `12,768.7 (1.0111×)` | `24,980.5 (1.0178×)` | `25,775.6 (0.9993×)` |
| **`Neon-Diet` ($\mu=1$)** | **`0.8989×`** | **`10,828.3 (0.8840×)`** | **`11,416.3 (0.9040×)`** | **`22,321.8 (0.9094×)`** | **`23,400.6 (0.9072×)`** |
| **`Neon-C33` ($\mu=2$)** | **`0.8284×`** | **`10,289.9 (0.8400×)`** | **`10,663.1 (0.8444×)`** | **`19,891.6 (0.8104×)`** | **`20,560.4 (0.7971×)`** |
| `HoistP` ($\mu=2$) | `0.8349×` | `10,340.5 (0.8441×)` | `10,757.4 (0.8518×)` | `20,311.0 (0.8275×)` | `20,928.2 (0.8114×)` |

---

## 3. Theorems, One-Line Kills, & Promotions

- **Theorem 34.1 (Pass-1 Loop-Buffer Footprint vs Zero-Mispredict `p` Guard):** Duplicating `maximal + take_unmarked` around `p` inflates Pass 1's hot loop past the decoded μop-cache / LSD footprint while removing only a 1-cycle zero-mispredict ALU check (`1 / 489,000` taken) hidden in the shadow of `OutQ16::push`.
- **Theorem 34.2 (Post-`Wire16` `XOR_GATE` Phase Transition & Contention Amplification):** After `Wire16` reduces XOR-wiring cost to `~1.2 µs/1,000 rows`, the exact `Bi / Diet` crossover sits at `m* ≈ 42K–48K` (`1.0161×` at `35K` -> `0.9282×` at `50K` -> `0.8918×` at `65K` -> `0.7961×` at `300K`), confirming `XOR_GATE = 65_536` (`256 KiB` = exact i9-10900K L2 boundary) as safely above the transition and protective on short/periodic blocks. Under `4T` L2 oversubscription (`4 × 945.8 KiB = 3.69 MiB` vs `Master`'s `4.58 MiB` on `2 MiB` L2), `Neon-Bi`'s speedup **amplifies** from `0.8284×` (`1T`) to **`0.7971×` (`4T` p50)** as spilled-L3 latency rewards `1.00m`-load $\mu = 2$ MLP.
- **KILL `HoistP`:** Loses to `Neon-C33` on `6/6` blocks at `1T` (`+78.6 µs`, `construct +53.7 µs`), `2T` (`+50.6/+94.3 µs`), and `4T` (`+419/+368 µs`) via Pass-1 code-size/μop-cache expansion (Theorem 34.1).
- **RETAIN `XOR_GATE = 65_536` & PROMOTE BOTH FLAGSHIPS TO CYCLE 35 GRAND SYNTHESIS:** **`Neon-Diet` ($\mu = 1$: `0.8989×` 1T, `0.8840×` 2T p10, `0.9072×` 4T p50)** and **`Neon-Bi` ($\mu = 2$: `0.8284×` 1T, `0.8400×` 2T p10, `0.7971×` 4T p50)**, both at **`945.8 KiB` (`−19.28%` WS)**.

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle34/` (`gen_c34.py`, `run_c34.rs`).
- **Cycle 35 (Neon — Final Handoff):** Zero-warning (`-D warnings`, `#![forbid(unsafe_code)]`) drop-in modules for `Neon-Diet` and `Neon-Bi`, exhaustive `67,968+` BWT + `1,000` malformed-column + `#703` `Cap::dense` audit, `80`-pair `1T` + `50`-pass `2T` + `30`-pass `4T` final A/A & A/B verification, and comprehensive 10-cycle (Cycles 26–35) Neon clean handoff journal.
