# Carbon Cycle 27: `LeanRegBurst6_256` — Zero Extra `rep_stores`, `inline16` Cache-Warming Law, and `0.8153×` `W = 4` Dense

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree), Cycle 23 `RegBurst6` (`dea87ec6`), & Cycle 26 `RegRelay6` (`17971266`)  
**Artifact:** [c27_lean_regburst6.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle27/c27_lean_regburst6.rs)

---

## 1. Hypotheses & Three Microarchitectural Discoveries

Cycle 26 flagged `RegRelay6`'s `32 KiB` stack zero-init (`rep_stores = 84,064` vs `origin/master`'s `18,528`) and proposed fusing `inline16`/`lens` population directly into `endpoint.rs:induced_with` while `spelling[w0..]` is hot in L1d. Systematic native A/B + A/A testing across seven controlled variants (`C27A–C27G`) uncovered three sharp hardware laws on Xeon Platinum 8481C (`48 KiB` L1d = `24 KiB` per SMT sibling at `W = 4`):

1. **The Phase-3 Cache-Eviction Law (Why `Fused DirectInline` Loses `+35–65 µs`/block):**  
   Populating `inline16` (`50–100 KiB`) inside `induced_with` runs *before* Phase 3 (`queue.wire` streaming `400–900 KiB` of `edges` + `out` queues). Phase 3 flushes `inline16` and `lens` cold out of L1d and half of L2 right before `emit` starts, regressing even `m < 140k` serial `block-0` (`1.0527×`) and `block-2` (`1.0457×`). Running `inline_words` at the start of `walk()` (*after* `queue.wire`) acts as a sequential hardware-prefetched pre-warming pass so `inline16`, `lens`, and `word_len` start `emit` hot in L1d/L2.
2. **Temporal Working-Set Separation on SMT (`RegBurst6` vs `RegRelay6`):**  
   At `W = 4` (`24 KiB` L1d per SMT thread), `RegBurst6` touches *only* `word_len` (`~6.5 KiB`) in Stage 1 and *only* `lens + inline16` in Stage 3, whereas `RegRelay6` touches `word_len + lens + inline16 + out + saved` simultaneously in Stage 1. Passing `&mut RelayScratch` (`C27A`) also consumed an extra GPR and spilled lane cursors (`1.0185×` `T4 wiki`).
3. **Exact `[256]` Cut Bound + Single-Caller `#[inline(always)] emit_serial` (`LeanRegBurst6_256`):**  
   Since `m <= (1 << bits)` and `shift = bits.saturating_sub(8)`, every `j < m` satisfies `(j >> shift) < 256` strictly. Shrinking all stack arrays to `[...; 256]` with `t_start: [u32; 256]` (and avoiding `u16::MAX` fill via vectorised zero-init) dropped extra `rep_stores` to **zero** (`18,528` vs `18,528` on `six`; `15,440` vs `15,440` on `dense`), while separating `#[cold] emit_serial_cold` preserved single-caller inlining of `emit_serial` into `walk()`.

Byte-exact parity (`warm` + `fresh` + `Plain`) passed on all `431` `wikidata.bz2` blocks (`W = 10` max heap: `1,275.2 KiB` vs `1,268.7 KiB`, `+6.5 KiB`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control | `C27A` (`&mut RelayScratch`) | `C27B` (`FusedInline` + `Burst`) | `C27G-1` (`Hybrid256`) | **`C27G-2` (`LeanRegBurst6_256`)** |
|---|---:|---:|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks) | `W = 1` | `0.9915×` | `0.9694×` | `0.9962×` | `0.9767×` | **`0.9670×`** (`11.43 / 11.05 ms`) |
| `dense-sum` (`5` dense blocks) | `W = 1` | `0.9902×` | `0.8944×` | `0.9127×` | `0.9114×` | **`0.8957×`** (`12.54 / 11.23 ms`, `blk-270` **`0.8275×`**) |
| `wiki-open233` (`233` Open blocks) | `W = 1` | `0.9949×` | `1.0132×` | `1.0220×` | `0.9898×` | **`0.9837×`** (`357.6 / 351.7 ms`) |
| `wiki-all431` (`431` blocks) | `W = 1` | `0.9948×` | `1.0132×` | `1.0221×` | `0.9898×` | **`0.9837×`** (`358.0 / 352.1 ms`) |
| `six-pass` (barrier-synced) | `W = 4` | `1.0490×` | `0.9598×` | `0.9888×` | **`0.9418×`** | **`0.9743×`** (`21.82 / 21.26 ms`) |
| `dense-pass` (barrier-synced) | `W = 4` | `1.0070×` | `0.8653×` | `0.8704×` | `0.8735×` | **`0.8153×`** (`25.95 / 21.15 ms`) |
| `wiki-all431` (barrier-synced) | `W = 4` | `0.9930×` | `1.0185×` | `1.0127×` | `0.9864×` | **`0.9816×`** (`724.3 / 711.0 ms`) |
| `E2E BzReader` (`wikidata.bz2`) | `W = 1` | `0.9929×` | `1.0077×` | `1.0109×` | `0.9867×` | **`0.9796×`** (`427.3 / 418.6 ms`) |
| `E2E BzReader` (`wikidata.bz2`) | `W = 4` | `0.9932×` | `0.9866×` | `1.0011×` | `0.9798×` | **`0.9798×`** (`286.7 / 280.9 ms`) |

---

## 3. `chaingrind` Diagnostic (`LeanRegBurst6_256` vs `origin/master` & Prior Cycles)

| Arm | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `rep_stores` |
|---|---:|---:|---:|---:|---:|
| `origin/master` (`six`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | **`18,528`** |
| Cycle 23 `RegBurst6` (`six`) | `104,671,163` | `36,301,541` | `407,793` | `774` | `30,816` |
| Cycle 26 `RegRelay6` (`six`) | `106,950,345` | `34,562,362` | `281,374` | `773` | `84,064` |
| **Cycle 27 `LeanRegBurst6_256` (`six`)** | **`104,031,156`** | **`35,541,062`** | **`321,210` (`-18.8%`)** | **`805` (`-72.1%`)** | **`18,528` (`+0`)** |
| `origin/master` (`dense`) | `99,627,142` | `50,453,341` | `617,861` | `4,095` | **`15,440`** |
| Cycle 23 `RegBurst6` (`dense`) | `130,170,338` | `42,037,709` | `428,538` | `145` | `35,920` |
| **Cycle 27 `LeanRegBurst6_256` (`dense`)** | **`128,973,597`** | **`41,529,194`** | **`425,156` (`-31.2%`)** | **`144` (`-96.5%`)** | **`15,440` (`+0`)** |

---

## 4. Keep / Kill Decisions (One-Line Reasons)

- **`C27A` (`&mut RelayScratch` heap struct pointer):** **KILLED** — extra live pointer GPR inside the 6-lane burst spills scalar lane cursors (`1.0132×` `T1 wiki`, `1.0185×` `T4 wiki`).
- **`C27B/C` (`Fused DirectInline` in `induced_with`):** **KILLED** — Phase 3 (`queue.wire`) evicts `inline16` and `lens` from L1d/L2 before `emit`, regressing even serial blocks by `+3.5–5.3%` (`1.0221×` `T1 wiki`).
- **`C27D` (`.zip()`-based `inline_words_fast` + `&[..ncodes]`):** **KILLED** — multi-iterator zip + `resize` regressed serial `block-0` by `+3.1%` (`1.0104×` `T1 wiki`).
- **`C27G-1` (`Hybrid256` switching to `RegRelay6_256` at `m >= 195k`):** **KILLED** — loses to `LeanRegBurst6_256` on both `T4 dense` (`0.8735×` vs `0.8153×`) and `T1 wiki` (`0.9898×` vs `0.9837×`) due to SMT L1d contention.
- **`C27G-2` (`LeanRegBurst6_256`):** **PROMOTED AS NEW WORKING BASELINE** — zero extra `rep_stores` over `origin/master` and new bests on `T1 wiki` (`0.9837×`), `T4 dense` (`0.8153×`), `E2E W = 1` (`0.9796×`), and `E2E W = 4` (`0.9798×`).
