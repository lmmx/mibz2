# Carbon Cycle 26: `RegRelay6` — Scalar-Register Direct-1 + Scout-5 Relay (`0.8366×` `W = 4` Dense, `0.9866×` `W = 1` Corpus)

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree) & Cycle 23 `RegBurst6` (`dea87ec6`)  
**Artifact:** [c26_regrelay6.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle26/c26_regrelay6.rs)

---

## 1. Synthesis: Marrying Scalar-Register Bursts with Direct Lane-`0` Emission (`Q_rand`: `2.00 m -> 1.82 m`)

Cycle 23's `RegBurst6` eliminated stack-cursor stalls by keeping all 6 lanes in scalar GPRs, but walked `edges` `2.00×` (`100%` scouted in Stage 1 + `100%` emitted in Stage 3). Cycle 26's **`RegRelay6`** replaces Stage 1's 6th scout with **direct scalar-register Lane-`0` emission from `mark`** (`j0, at0` in GPRs alongside 5 scalar-register scouts `j1..j5, a1..a5`), with two structural upgrades over Wormhole's `DietRelay8Cells`:
1. **All 6 Stage-1 cursors live in GPRs** (`sc_cid` stays on stack, touched only at cut completions), with a single 6-way bitwise cut check `(((j0 ^ mark) & cm) == 0) | ... | (((j5 ^ mark) & cm) == 0)` (`mark` itself satisfies `((mark ^ mark) & cm) == 0`, so `j0 == mark` requires zero extra inner-loop branches).
2. **Zero-dummy-load tail exit:** The instant any scout finishes the last unclaimed cut (`active < 5`), Stage 1 drops out of the 6-lane burst so Lane `0` hops the already-scouted macro-cycle in `< 50` L1d steps (`~0.1 µs`) with zero wasted `edges[0]` loads.

We tested both `RELAY_MIN_M = 80_000` (confirming again that $m < 135{,}000$ loses `~4.6%` on `block-0`/`block-2`, with exact breakeven at `block-207`, $m = 135{,}921$: `0.9953×`) and **`RELAY_MIN_M = 135_000`**. Byte-exact parity (`warm` + `fresh` + `Plain`) passed on all `431` `wikidata.bz2` blocks (`W = 10` max heap: `1,275.2 KiB`, `+6.5 KiB`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control | Cycle 23 `RegBurst6` (`m >= 140k`) | `RegRelay6` (`m >= 80k`) | **Gated `RegRelay6` (`m >= 135k`)** |
|---|---:|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `1.0065×` | `0.9767×` | `0.9837×` | **`0.9725×`** (`11.24 / 10.93 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `1.0047×` | `0.9094×` | `0.8863×` | **`0.8923×`** (`12.45 / 11.11 ms`, `blk-270` **`0.8186×`**) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `1.0120×` | `0.9927×` | `1.0094×` | **`0.9867×`** (`351.0 / 346.3 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `1.0120×` | `0.9928×` | `1.0095×` | **`0.9866×`** (`351.3 / 346.6 ms`) |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `1.0043×` | **`0.9292×`** | `0.9555×` | `0.9747×` (`21.77 / 21.22 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0023×` | `0.8498×` | `0.8391×` | **`0.8366×`** (`25.64 / 21.45 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `1.0020×` | **`0.9776×`** | `0.9940×` | `0.9951×` (`720.2 / 716.7 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `1.0188×` | **`0.9853×`** | `0.9837×` | `0.9866×` (`296.7 / 292.7 ms`) |

---

## 3. `chaingrind` Diagnostic (`dense` & `six`)

| Arm | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `l2_miss` | `rep_stores` |
|---|---:|---:|---:|---:|---:|---:|
| `origin/master` (`six`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | `517,730` | `18,528` |
| Cycle 23 `RegBurst6` (`six`) | `104,671,163` | `36,301,541` | `407,793` | `774` | `823,351` | `30,816` |
| **Cycle 26 `RegRelay6` (`six`)** | `106,950,345` | **`34,562,362`** | **`281,374` (`-28.8%`)** | **`773`** | `866,239` | **`84,064`** |
| `origin/master` (`dense`) | `99,627,142` | `50,453,341` | `617,861` | `4,095` | `726,487` | `15,440` |
| Cycle 23 `RegBurst6` (`dense`) | `130,170,338` | `42,037,709` | `428,538` | `145` | `1,408,446` | `35,920` |
| **Cycle 26 `RegRelay6` (`dense`)** | **`127,786,285`** | **`40,512,740`** | **`386,784` (`-37.4%`)** | **`167`** | **`1,339,439`** | **`97,360`** |

**Diagnostic insight for Cycle 27:** `RegRelay6` beat `RegBurst6` at `W = 1` across the board (`0.8923×` vs `0.9094×` dense, `0.9866×` vs `0.9928×` corpus) and hit `0.8366×` on `T4 dense`, while `rep_stores` jumped `3×` (`30.8k -> 84.1k`) because `saved = [[0u8; 16]; 1024]` + `t_* = [0; 1024]` zeroed `32 KiB` of stack (`2/3` of `48 KiB` L1d!) per block when max tasks is $\le 262$ (`256` cuts + $\le 5$ scout splits).

---

## 4. Keep / Kill Decisions

- **`RegRelay6` below `m = 135_000` (`RELAY_MIN_M = 80_000`):** **KILLED** — loses `~4.6%` on `block-0`/`block-2` (`1.0095×` on `T1 wiki-431`).
- **Gated `RegRelay6` (`RELAY_MIN_M = 135_000`):** **RETAINED FOR `W = 1` / DENSE (`0.8366×` `T4 dense`, `0.9866×` `T1 wiki`)**, with its `32 KiB` oversized stack arrays (`[1024]` -> `[264]`) targeted immediately in Cycle 27 alongside producer `inline16` fusion.
