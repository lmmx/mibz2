# Carbon Cycle 24: Register-Pressure (`K = 4, 6, 8`), Cut-Density (`128, 256, 512`), and Scout-Branch Ablations

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree) + Cycle 23 `RegBurst6` (`dea87ec6`)  
**Artifact:** [c24_regburst_ablations.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle24/c24_regburst_ablations.rs)

---

## 1. Hypotheses Tested

Building on Cycle 23's `RegBurst6` (`6` scouts, `6` emitters, `CUT_LOG = 8` $\le 256$ cuts, branchless `word_len: &[u16]`), Cycle 24 tested four single- and multi-factor variations to locate the exact native optimum and isolate why `RegBurst6` outperforms 8-lane relays:
1. **`RegBurst8_8` (`8` scouts, `8` emitters, `CUT_LOG = 9` $\le 512$ cuts, `MIN_M = 128k`, branched `lens + offsets`):** Tests higher nominal MLP ($\mu = 8$) and halved Rényi straggler tail (`512` cuts) at `+0.0 KiB` heap.
2. **`RegBurst8_4` (`8` scouts, `4` emitters, `CUT_LOG = 9`, `MIN_M = 128k`, branched `lens + offsets`):** Tests reducing emitter GPR pressure to `4` lanes while keeping `8` scouts.
3. **`ZeroHeap-Branched-6` (`6` scouts, `6` emitters, `CUT_LOG = 8`, `MIN_M = 140k`, `wlen_of` via `lens + offsets`):** Single-factor ablation removing `word_len: Vec<u16>` (`+0.0 KiB` heap) from Cycle 23's exact `6`-lane geometry.
4. **`Cut7-6` (`6` scouts, `6` emitters, `CUT_LOG = 7` $\le 128$ cuts, `MIN_M = 140k`, branchless `word_len`):** Single-factor ablation halving cut count (`2×` longer uninterrupted inner bursts vs `2×` longer Rényi straggler tail).

All four arms passed `431/431` byte-exact parity (`warm` + `fresh` + `Plain`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

All ratios are `Candidate / MasterA` (`origin/master` `90e67196`) in the same binary (`A/A` controls: `0.991×–1.009×` across all runs):

| Arm | Heap (`W=10` `max_KiB`) | `T1 six` (`W=1`) | `T1 dense` (`W=1`) | `T1 wiki-431` (`W=1`) | `T4 six` (`W=4`) | `T4 dense` (`W=4`) | `T4 wiki-431` (`W=4`) |
|---|---:|---:|---:|---:|---:|---:|---:|
| **Cycle 23 `RegBurst6` (`K=6, Cut8, u16`)** | `1,275.2` (`+6.5`) | **`0.9767×`** | **`0.9094×`** | **`0.9928×`** | **`0.9292×`** | **`0.8498×`** | **`0.9776×`** |
| `Cut7-6` (`K=6, Cut7=128, u16`) | `1,275.2` (`+6.5`) | `0.9849×` | `0.9196×` | `0.9938×` | `0.9372×` | **`0.8482×`** | `0.9782×` |
| `ZeroHeap-Branched-6` (`K=6, Cut8, lens+off`) | **`1,268.7` (`+0.0`)** | `1.0338×` | `1.0043×` | `1.0175×` | `1.0098×` | `0.9507×` | `1.0191×` |
| `RegBurst8_4` (`K=8/4, Cut9=512, lens+off`) | **`1,268.7` (`+0.0`)** | `1.0602×` | `1.0241×` | `1.0285×` | `1.0558×` | `0.9639×` | `1.0137×` |
| `RegBurst8_8` (`K=8/8, Cut9=512, lens+off`) | **`1,268.7` (`+0.0`)** | `1.0924×` | `1.0562×` | `1.0165×` | `1.0466×` | `1.0119×` | `1.0101×` |

---

## 3. Root-Cause Findings & Kill Decisions

- **`ZeroHeap-Branched-6` (`wlen_of` via `lens + offsets`):** **KILLED** — replacing branchless `word_len[(e >> bits) as usize]` (`1` `movzwl` instruction, `0` branches) with `if l != LONG { l } else { offsets[...] }` adds `6` conditional checks per Stage 1 burst and holds `offsets` (`ptr, len`) live in GPRs, losing **+9.5% natively on `T1 dense`** (`0.9094× -> 1.0043×`) and turning `W = 4` `wikidata` from `0.9776×` into `1.0191×`. The `6.5 KiB` `u16` length table buys its tiny L1d footprint back tenfold.
- **`RegBurst8_8` & `RegBurst8_4` (`8` scout lanes + `CUT_LOG = 9`):** **KILLED** — `8` scalar scout lanes (`24` live integers: `c0..c7`, `j0..j7`, `a0..a7`) exceed x86-64's `15` GPRs and spill to stack inside Stage 1, losing `1.010×–1.014×` at `W = 4` against `origin/master`.
- **`Cut7-6` (`128` cuts vs `256` cuts):** **TIE / SUBOPTIMAL AT `W = 1`** — virtually tied with `Cut8-6` at `W = 4` (`0.9782×` vs `0.9776×`, `0.8482×` vs `0.8498×` on `dense`), slightly slower at `W = 1` (`0.9196×` vs `0.9094×` on `dense`) due to longer end-of-stage Rényi straggler cuts; **`K = 6, CUT_LOG = 8` (`RegBurst6`) remains the retained baseline**.
