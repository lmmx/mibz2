# Carbon Cycle 20: Native `0.5.11` Phase Attribution & Fast Kill of Single-Touch `CodeTape`

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree, post-`#692` memory diet and `#703` dense-block admission)  
**Artifact:** [c20_codetape.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle20/c20_codetape.rs)

---

## 1. Native 5-Phase Attribution on `origin/master` (`90e67196`)

Before testing candidates against `origin/master`, we measured the exact native median phase split (`25` warm repetitions, single-threaded on Intel Xeon Platinum 8481C, `48 KiB` L1d / `2 MiB` L2 / `105 MiB` L3) across the `6` authenticated `admitted-bwt` blocks (`six`) and the `5` newly-admitted dense-band blocks (`dense`: `blk 41, 56, 165, 270, 278`):

| Block | $m$ | $n_s$ | Words | `Pass 1` (µs) | `Induced` (µs) | `Wire` (µs) | `Inline` (µs) | `Emit` (µs) | Total (µs) | `Emit` % |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0` | 89,874 | 3,448 | 1,326 | 235.4 | 451.1 | 83.1 | 13.0 | 479.9 | 1,262.5 | **38.0%** |
| `block-2` | 112,070 | 3,827 | 1,922 | 252.5 | 546.5 | 108.8 | 18.6 | 618.8 | 1,545.1 | **40.0%** |
| `block-85` | 156,404 | 1,565 | 1,220 | 338.5 | 244.1 | 80.8 | 11.3 | 846.3 | 1,521.0 | **55.6%** |
| `block-143` | 196,841 | 5,047 | 3,635 | 486.3 | 810.6 | 183.8 | 32.9 | 1,283.8 | 2,797.4 | **45.9%** |
| `block-207` | 135,921 | 3,474 | 2,259 | 376.1 | 522.1 | 122.6 | 20.1 | 801.1 | 1,842.1 | **43.5%** |
| `block-210` | 204,281 | 2,809 | 2,228 | 548.4 | 453.6 | 130.2 | 20.0 | 1,308.1 | 2,460.3 | **53.2%** |
| **`six` sum** | **895,391** | — | — | **2,237.2** | **3,028.0** | **709.3** | **115.9** | **5,338.0** | **11,428.4** | **46.7%** |
| `blk-41` | 231,600 | 2,619 | 2,605 | 566.0 | 450.5 | 146.0 | 24.0 | 1,522.8 | 2,709.2 | **56.2%** |
| `blk-56` | 230,034 | 2,275 | 2,330 | 570.2 | 394.8 | 127.4 | 21.4 | 1,528.6 | 2,642.4 | **57.8%** |
| `blk-165` | 221,139 | 2,729 | 2,608 | 569.3 | 474.6 | 140.6 | 24.2 | 1,475.2 | 2,683.9 | **55.0%** |
| `blk-270` | 199,189 | 568 | 706 | 699.8 | 88.3 | 47.9 | 5.0 | 1,406.8 | 2,247.8 | **62.6%** |
| `blk-278` | 201,418 | 2,181 | 1,990 | 538.1 | 360.2 | 112.8 | 17.2 | 1,264.2 | 2,292.5 | **55.1%** |
| **`dense` sum** | **1,083,380** | — | — | **2,943.4** | **1,768.4** | **574.7** | **91.8** | **7,197.6** | **12,575.8** | **57.2%** |

**Key structural observation:** `emit` accounts for **46.7%** of native time on `six` and **57.2%** on `dense`, averaging `5.34–7.06 ns/step` (`~15–20` CPU cycles per retained row) in an ~11-instruction loop. Because `15–20` cycles/step leaves almost zero slack for instruction bloat, any mechanism that adds ~35–50 instructions/step (~10+ cycles/step of front-end/branch/store work) cannot beat `origin/master` natively unless $m$ is very large.

---

## 2. Hypothesis Tested: Single-Touch `CodeTape8` (Eliminating `DietRelay8`'s `1.875×` Double-Walk)

Wormhole Cycle 10's `DietRelay8Cells` splits the walk at $S \le 256$ zero-bit cuts `((j ^ mark) & cm) == 0`, paying a **`1.875×` random-load tax** ($Q_{\text{rand}} = \frac{15}{8} m$) because Phase 4a scouts discard the walked `code`s and Phase 4b re-walks `7/8` of `edges[j]` from L2/L3.

**Cycle 20 Hypothesis (`CodeTape8`):** Walk `edges[..m]` **only once** ($Q_{\text{rand}} = 1.00 m$) across $\mu = 8$ lanes: Lane `0` emits directly into `out`, while `7` scouts pack walked `u16` code pairs (`2` codes per `u32`) into dead producer memory (`self.edges[m..]` + dead `self.bands`, keeping `W = 10` max heap within `1,350 KiB`). Phase 4b then drains the sequential `u32` code chunks left-to-right with **zero `edges[j]` random loads**. We also measured a reference port of 2-pass `Relay8` (`Relay8Ref`) in the same `#![forbid(unsafe_code)]` harness as a diagnostic anchor.

All arms passed byte-exact parity (`warm` + `fresh` + `Plain`) across all `431` blocks of `wikidata.bz2`.

---

## 3. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

In-process paired ABBA medians in the **same binary** against `origin/master` (`90e67196`), with codegen-identical `MasterA vs MasterB` A/A controls:

| Metric / Workload | Workers | `A/A` Control (`MasterB / MasterA`) | `Relay8Ref / MasterA` (2-pass ref) | `CodeTape8 / MasterA` (Cycle 20) |
|---|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `1.0110×` (`11.71 / 11.84 ms`) | `1.0516×` (`11.63 / 12.23 ms`) | **`1.2165×`** (`11.73 / 14.27 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `1.0176×` (`12.84 / 13.06 ms`) | **`0.9766×`** (`0.8978×` on `blk-270`) | **`1.2694×`** (`12.83 / 16.28 ms`) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `1.0040×` (`357.4 / 358.9 ms`) | `1.0734×` (`361.2 / 387.7 ms`) | **`1.1958×`** (`359.5 / 429.9 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `1.0040×` (`357.8 / 359.2 ms`) | `1.0733×` (`361.5 / 388.0 ms`) | **`1.1956×`** (`359.9 / 430.2 ms`) |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `0.9954×` (`24.05 / 23.94 ms`) | `1.0377×` (`23.39 / 24.27 ms`) | **`1.2017×`** (`23.76 / 28.55 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0169×` (`26.95 / 27.41 ms`) | **`0.9200×`** (`26.65 / 24.52 ms`) | **`1.2226×`** (`27.47 / 33.59 ms`) |
| `wiki-open233` (barrier-synced homogeneous) | `W = 4` | `1.0004×` (`755.8 / 756.1 ms`) | `1.0546×` (`761.0 / 802.5 ms`) | **`1.1786×`** (`751.1 / 885.2 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `1.0004×` (`756.2 / 756.5 ms`) | `1.0531×` (`764.3 / 804.9 ms`) | **`1.1784×`** (`751.5 / 885.6 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `0.9938×` (`442.6 / 439.9 ms`) | `1.0659×` (`439.8 / 468.8 ms`) | **`1.1613×`** (`443.2 / 514.6 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `1.0037×` (`294.6 / 295.7 ms`) | `1.0210×` (`292.5 / 298.7 ms`) | **`1.1079×`** (`299.1 / 331.4 ms`) |
| Worker Peak Heap (`max_KiB`, `W = 1 / 4 / 10`) | — | `1,351 / 1,342 / 1,269 KiB` | `1,359 / 1,352 / 1,348 KiB` | `1,361 / 1,354 / 1,350 KiB` |

---

## 4. `chaingrind` Diagnostic (Confirming the Optimism Gap)

| Arm (`six`, `5,399,692` B) | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `l2_miss` | Store-Buf Stall Cyc | DRAM B/outB |
|---|---:|---:|---:|---:|---:|---:|---:|
| `origin/master` (`90e67196`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | `517,730` | `704,702` | `0.798` |
| `Relay8Ref` (2-pass ref) | `135,965,622` | `38,404,320` | `33,611` (`-91.5%`) | `489` | `963,481` (`+86%`) | `2,091,263` | `0.890` |
| `CodeTape8` (1-touch tape) | `138,225,438` | `43,769,621` | `137,352` (`-65.3%`) | `1,451` | `726,740` (`+40%`) | `2,759,094` | `1.349` |

`CodeTape8` cut `dep_past_l2_loads` by `65.3%` (`395k -> 137k`) and halved `Relay8Ref`'s `l2_miss` overhead (`+40%` vs `+86%`), yet lost **`1.178×–1.196×` natively** (and `Relay8Ref`, despite `chaingrind` predicting `0.998×` parity on `six` and `0.910×` on `dense`, lost `1.053×–1.073×` natively on `wikidata`).

---

## 5. Kill Decisions & Structural Takeaway for Cycle 21

- **`CodeTape8` (`emit_codetape`):** **KILLED** — per-step `u16` chunked tape-packing branches and sequential re-read stores/loads add `+56.5%` instructions (`88.3M -> 138.2M`) and `3.9×` store-buffer stalls (`0.70M -> 2.76M`), losing `1.196×` (`W = 1`) and `1.178×` (`W = 4`) natively against `origin/master`.
- **Unscoped `Relay8Ref` on low/mid-`m` blocks:** **NOT PURSUED** — wins `0.920×` natively on `T4 dense-pass` (`0.898×` on `blk-270`) where $m \ge 200\text{k}$, but loses `1.053×` across `wikidata` because per-step scout state-machine instructions exceed the `15`-cycle L2 hit cost on $m < 180\text{k}$ (and Coordinator is already porting Wormhole's `DietRelay8Cells`).
- **Direction for Cycle 21:** Attack $D_{\text{serial}}$ **without adding a single instruction to `emit`'s inner loop** by breaking Wormhole's *non-jumping* hypothesis ($t_{\text{probe}} = 1$): perform an **out-of-order producer pass** over `edges` (`g -> g^2` super-edge contraction into `inline16`) so `emit` runs the **exact 11-instruction loop** over **fewer steps**.
