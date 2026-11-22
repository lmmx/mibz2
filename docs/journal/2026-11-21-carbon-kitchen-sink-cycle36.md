# Carbon Kitchen Sink (Neon) — Cycle 36: Post-Stack Re-Baseline on `origin/master` (`53f7ecf93`), Wormhole Gap Ranking, & First Construction Wins (`Wire16 + S2aFuse + S3Tag12`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Persona:** Gemini Neon  
**Baseline:** `origin/master` (`53f7ecf93`, `#719` construction stack adopted on i9 at `0.952 / 0.987 / 0.975` at `6 / 8 / 10` threads)  
**Scope:** Research-only; `src/`, `Cargo.toml`, `benches/`, and `examples/` untouched; attacking construction (`S1–S4`) exclusively.

---

## 1. Post-Stack Re-Baseline & Stage-Gap Ranking Against Wormhole `BMCP_CML` Bounds

Following `#719` (`53f7ecf93`), `origin/master` already ships our construction stack (`MuxStrips`, `Band64`, `Operad-MN16`, `GradedHandle`, `48 KiB` `spelling`, interned dictionary, `L4` bottom coordinates, `pop_tail`, `945.8 KiB` scratch) plus `S4` two-ended `wire_xor` (which proved neutral on the i9 at `1.001 / 0.991 / 0.989`, meaning the i9 speedup is 100% construction).

We extracted verbatim `origin/master` (`53f7ecf93`) into an instrumented 6-stage harness (`run_c36.rs`, `40` rounds/block across the `6` authenticated Wikidata blocks) in both **`Master (53f7)`** and **`Master-Nobi`** ($\mu = 1$ serial `wire` + `emit`) modes, and ranked the five construction stages against Wormhole's post-stack `BMCP_CML` bounds (`origin/research/carbon-wormhole` Cycles 22–26):

| Rank (`Nobi` / `Master`) | Construction Stage | `Master (53f7)` (`µs`, `% Pre`) | `Master-Nobi` (`µs`, `% Pre`) | Wormhole `BMCP_CML` Bottleneck (C22–26) | Remaining Structural Gap to Attack |
|---|---|---:|---:|---|---|
| **#1 / #1** | **`S1: Pass 1 + OutQ16`** | **`2,044.3 µs` (`33.7%`)** | **`2,014.1 µs` (`38.5%`)** | `489k` `Q_rand`, `25,815` L2m (`47%` hide), `1,036` DTLB | `~90` active `1 KiB` chunks exceed `64`-entry L1 DTLB; `maximal()` scans `records[i+1]` on `465k` runs `< 127` |
| **#2 / #3** | **`S2b: RV Elimination`** | **`1,202.0 µs` (`19.8%`)** | **`1,191.2 µs` (`22.8%`)** | `195k` serial `S_RV`, `~17.5k` L2m, `8.7k` DTLB | `len=2` bigrams / `len=3` ASCII trigrams allocate `16 B` rules instead of fitting in 22-bit handle; repeated sentinel `a[0]/a[1]` & survivor reloads |
| **#3 / #4** | **`S3a: Bot + Dedup + Band64`** | **`896.2 µs` (`14.8%`)** | **`876.4 µs` (`16.8%`)** | `36k` `Q_rand`, `1,398` L2m (`76%` hide) | `origin/master` lacks 8-bit hash tag in `contents`, `last_word` cache, 1-mul short hash, & pure-register `u128` shift-OR (`bytes[height] = byte` stack stall); `6` per-band `Counts` updates |
| **#4 / #2** | **`S4: OutQ16 Wire`** | **`1,565.0 µs` (`25.8%`)** | **`783.9 µs` (`15.0%`)** | `51k` `Q_rand`, `14.8k` L2m (`82%` hide), `2.3k` DTLB | `origin/master` runs `1×` scalar `wire_xor` with `if v == mark` on all `895k` ranks (`+781 µs` over serial `wire`!); serial `wire` still unpacks `u16` per rank |
| **#5 / #5** | **`S2a: Layout & Nodes`** | **`367.1 µs` (`6.0%`)** | **`365.2 µs` (`7.0%`)** | `20k` radix + 3 passes over `strips_raw` | `origin/master` scans `strips_raw[0..3*ns]` **3 separate times** (`mod.rs:424`, `endpoint.rs:675`, `endpoint.rs:684`) |
| — | **`Pre (S1–S4)` Total** | **`6,074.6 µs`** (`57.4%` tot) | **`5,230.8 µs`** (`47.6%` tot) | `945.8 KiB` persistent scratch | `S5 Emit = 4,500.8 µs` (`Master`) / `5,758.2 µs` (`Nobi`) |

---

## 2. Cycle 36 Construction Candidates & Native A/B Verification (`run_c36.rs`)

Built directly on verbatim `origin/master` (`53f7ecf93`), compiled with `rustc --edition 2021 -C opt-level=3 -C target-cpu=native -C codegen-units=1` (`#![forbid(unsafe_code)]`):
1. **`Wire16` (`S4`):** Unique-symbol `mark_byte` isolation (`255/256` bytes skip `mark`) + `Wire16/8/4` `u64`-unpacked slice-checked MSHR-burst `wire_xor_nomark`.
2. **`S2aFuse` (`S2a`):** Fuses `mod.rs:424` + `endpoint.rs:675` + `endpoint.rs:684` into **one single sequential pass** over `strips_raw[0..3*ns]` (`e.nodes = ns + 1`, writing strip node `i + 1` in place while creating domain gaps and `image_order`).
3. **`S3Tag12` / `S3Tag24` (`S3a`):** Pure-register `u128` shift-OR in `inline_word` (removing stack-array `bytes[height] = byte`), 8-bit high-hash tag in `contents`, per-byte `last_word` register cache, 1-mul short `spelling_hash`, and `12 KiB` (`9/11`) vs `24 KiB` (`10/12`) arena HT.

- **Parity & Scratch:** `100%` byte-exact on `6/6` Wikidata blocks + `dense-300k`; persistent scratch unchanged at **`945.8 KiB` (`968,484 B`, `+0.00%`)**.

### 2.1 6-Stage Profile Attribution (`40` rounds/block, `µs`)

| Arm | `S1` | `S2a` | `S2b` | `S3a` | `S4` | **`Pre (S1–S4)`** | `S5` | `Total` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `Master (53f7)` | `2,044.3` | `367.1` | `1,202.0` | `896.2` | `1,565.0` | **`6,074.6`** | `4,500.8` | `10,575.4` |
| **`C36-Combo`** | `1,996.9` | **`326.3` (`−40.8`)** | `1,167.6` | **`783.0` (`−113.2`)** | **`1,488.7` (`−76.3`)** | **`5,762.5` (`−312.1`)** | `4,495.8` | **`10,258.3`** |
| `Master-Nobi` | `2,014.1` | `365.2` | `1,191.2` | `876.4` | `783.9` | **`5,230.8`** | `5,758.2` | `10,989.0` |
| **`C36-NobiCmb`** | `2,028.0` | **`329.5` (`−35.7`)** | `1,168.7` | **`781.1` (`−95.3`)** | `763.2` | **`5,070.5` (`−160.3`)** | `5,755.1` | **`10,825.6`** |

### 2.2 Headline `1T` (`50` rounds/block) & `2T` (`35` passes) Interleaved A/A & A/B

| Arm | `1T` Total (`µs`, ratio) | `1T` `Pre` (`µs`, ratio, `Δ`) | `2T` p10 (`µs`, ratio) | `2T` p50 (`µs`, ratio) |
|---|---:|---:|---:|---:|
| `MasterA1` (`53f7ecf93`) | `10,655.2 (1.0000×)` | `6,072.4 (1.0000×, 0.0)` | `10,480.9 (1.0000×)` | `10,585.5 (1.0000×)` |
| `MasterA2` (A/A) | `10,626.4 (0.9973×)` | `6,045.0 (0.9955×, −27.4)` | `10,471.4 (0.9991×)` | `10,543.9 (0.9961×)` |
| `Wire16` (`S4`) | `10,545.2 (0.9897×)` | `5,943.6 (0.9788×, −128.9)` | — | — |
| `S2aFuse` (`S2a`) | `10,571.0 (0.9921×)` | `6,026.9 (0.9925×, −45.5)` | — | — |
| `S3Tag12` (`S3a`) | `10,633.6 (0.9980×)` | `6,062.9 (0.9984×, −9.6)` | — | — |
| `S3Tag24` (`S3a`) | `10,640.0 (0.9986×)` | `6,052.4 (0.9967×, −20.0)` | — | — |
| **`C36-Combo` (`W16+S2a+S3`)** | **`10,459.4 (0.9816×)`** | **`5,871.7 (0.9669×, −200.8)`** | **`10,196.0 (0.9728×)`** | **`10,378.5 (0.9804×)`** |
| `Master-Nobi` ($\mu=1$ ref) | `11,040.9 (1.0000×)` | `5,270.4 (1.0000×, 0.0)` | `10,808.3 (1.0000×)` | `10,975.1 (1.0000×)` |
| **`C36-NobiCmb` (`S2a+S3`)** | **`10,933.2 (0.9902×)`** | **`5,177.0 (0.9823×, −93.4)`** | `10,821.5 (1.0012×)` | **`10,913.5 (0.9944×)`** |

---

## 3. One-Line Kills & Promotions

- **KILL `S3Tag24`:** Strictly dominated by `S3Tag12` (`12 KiB` HT matches `24 KiB` dedup quality at half the L1D footprint and slightly faster `1T` `0.9980×` vs `0.9986×`).
- **PROMOTE `C36-Combo` (`Wire16 + S2aFuse + S3Tag12`, `+0 B` scratch):** Cuts construction `Pre` by **`−200.8 µs` (`0.9669×`, `−3.31%`)** on `origin/master` (`−93.4 µs` / `0.9823×` on `nobi`) and wins `6/6` blocks at **`0.9816×` `1T`** and **`0.9728×` p10 / `0.9804×` p50 `2T`** (vs A/A `0.9973 / 0.9991×`).

---

## 4. Artifacts & Next Step

- Reproducers archived in `docs/journal/2026-11-21-carbon-kitchen-sink-cycle36/` (`gen_c36.py`, `run_c36.rs`).
- **Cycle 37 (Neon):** Attack **Rank #1 Construction Stage (`S1: Pass 1 + OutQ16`, `2,044 µs` = `38.5%` of `Nobi Pre`)** — testing `CAP = 127` maximal-run continuation elision, L1D/4-DTLB-page write-combining staging (`16 KiB` / `32 KiB`), and `pt64` cursor fusion on top of `C36-Combo`.
