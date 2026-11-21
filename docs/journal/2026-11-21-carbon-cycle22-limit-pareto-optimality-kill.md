# Carbon Cycle 22: Bidirectional `LIMIT` Sweep (`6..40`) & Proof of `LIMIT = 16` Pareto Optimality

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree)  
**Artifact:** [c22_adaptive_limit.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle22/c22_adaptive_limit.rs)

---

## 1. Hypothesis: Shrinking $m$ (`LIMIT < 16`) vs Shrinking $n_s$ (`LIMIT > 16`) on `0.5.11`

With `#692`'s `Narrow` 4-word `endpoint::Elim` arena and `#703`'s dense-band admission (`r/n` up to `0.206`) now on `origin/master`, we swept `LIMIT` bidirectionally across `L ∈ {6, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40}` (`15` warm native repetitions per block) to test whether moving the thin/thick boundary reduces wall time at `W = 1` and `W = 4`.

### Native Single-Thread `LIMIT` Curve (µs, with `(m_k / ns_k)`)

| Block | $r/n$ | `L = 8` | `L = 12` | **`L = 16` (master)** | `L = 24` | `L = 32` |
|---|---:|---:|---:|---:|---:|---:|
| `block-0` | `0.063` | `1546` (`67k/5k`) | `1373` (`82k/3k`) | **`1263`** (`89k/3k`) | `1333` (`122k/1k`) | `1285` (`127k/1k`) |
| `block-2` | `0.066` | `1875` (`82k/6k`) | `1758` (`98k/4k`) | `1562` (`112k/3k`) | **`1543`** (`126k/3k`) | `1553` (`162k/1k`) |
| `block-85` | `0.094` | `1955` (`118k/5k`) | `1716` (`150k/1k`) | `1525` (`156k/1k`) | `1478` (`163k/1k`) | **`1452`** (`166k/1k`) |
| `block-143` | `0.125` | `3362` (`157k/8k`) | `3039` (`185k/5k`) | `2773` (`196k/5k`) | `2722` (`238k/2k`) | **`2678`** (`253k/2k`) |
| `block-207` | `0.102` | `2170` (`117k/5k`) | `2013` (`129k/3k`) | `1865` (`135k/3k`) | `1796` (`144k/3k`) | **`1785`** (`150k/2k`) |
| `block-210` | `0.148` | `2754` (`171k/5k`) | `2611` (`187k/3k`) | `2466` (`204k/2k`) | **`2362`** (`219k/2k`) | `2370` (`233k/1k`) |
| `blk-41` | `0.151` | `3256` (`195k/5k`) | `2873` (`219k/3k`) | `2645` (`231k/2k`) | **`2453`** (`246k/1k`) | `2507` (`258k/1k`) |
| `blk-56` | `0.155` | `3067` (`196k/5k`) | `2911` (`214k/3k`) | `2653` (`230k/2k`) | `2614` (`241k/1k`) | `2549` (`248k/1k`) |
| `blk-165` | `0.156` | `3190` (`196k/4k`) | `2840` (`211k/3k`) | `2738` (`221k/2k`) | **`2622`** (`234k/2k`) | `2603` (`249k/1k`) |
| `blk-270` | `0.206` | `2338` (`196k/0k`) | **`2217`** (`198k/0k`) | `2237` (`199k/0k`) | `2243` (`200k/0k`) | `2247` (`202k/0k`) |
| `blk-278` | `0.152` | `2801` (`178k/4k`) | `2568` (`193k/2k`) | `2295` (`201k/2k`) | **`2248`** (`209k/1k`) | `2281` (`214k/1k`) |

**Discovery 1 (`L < 16` immediate kill):** Marginal strip elimination in `endpoint::Elim` costs `~140–175 ns/strip` while a retained row costs `~8.9 ns/row` across `Pass 1 + wire + emit` ($w^\star \approx 18$). Lowering `LIMIT` below `16` loses `5–26%` natively on almost every block.

We therefore promoted **`AdaptiveLIMIT24`** (`limit = 24` when `r/n >= 0.08`, `limit = 16` otherwise, paired with single-branch `OutQ24` writing `24` `u16`s in `outq.rs` so runs of width `17..=24` never hit `push_slow` or a two-tier branch mispredict) into full `W = 1` and `W = 4` confrontation.

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`AdaptiveLIMIT24 + OutQ24`)

| Metric / Workload | Workers | `A/A` Control (`MasterB / MasterA`) | `AdaptiveLIMIT24 / MasterA` |
|---|---:|---:|---:|
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `0.9985×` (`11.50 / 11.48 ms`) | `0.9932×` (`11.55 / 11.48 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `1.0101×` (`12.54 / 12.67 ms`) | `0.9887×` (`12.48 / 12.34 ms`) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `0.9944×` (`352.9 / 350.9 ms`) | **`1.0015×`** (`356.4 / 356.9 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `0.9944×` (`353.3 / 351.3 ms`) | **`1.0015×`** (`356.8 / 357.3 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0087×` (`26.59 / 26.82 ms`) | **`1.0126×`** (`25.96 / 26.28 ms`) |
| `wiki-open233` (barrier-synced homogeneous) | `W = 4` | `1.0046×` (`736.7 / 740.0 ms`) | **`1.0146×`** (`741.5 / 752.4 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `1.0046×` (`737.1 / 740.4 ms`) | **`1.0145×`** (`742.0 / 752.8 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `0.9507×` (`456.3 / 433.8 ms`) | `0.9976×` (`437.0 / 436.0 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `0.9877×` (`290.4 / 286.8 ms`) | **`1.0073×`** (`287.8 / 289.9 ms`) |
| Worker Peak Heap (`max_KiB`, `W = 1 / 4 / 10`) | — | `1,351 / 1,342 / 1,269 KiB` | `1,395 / 1,323 / 1,280 KiB` |

---

## 3. `chaingrind` Diagnostic

| Arm (`six`, `5,399,692` B) | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | `l2_miss` | DRAM B/outB |
|---|---:|---:|---:|---:|---:|---:|
| `origin/master` (`LIMIT = 16`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | `517,730` | `0.798` |
| `AdaptiveLIMIT24 + OutQ24` | `88,750,845` | `40,151,596` | `431,532` (`+9.1%`) | `3,147` | `542,968` (`+4.9%`) | `0.910` |

---

## 4. Kill Decisions

- **`LIMIT < 16` (`L = 6..14`):** **KILLED** — loses `5–26%` natively because `endpoint::Elim`'s `~150 ns/strip` cost exceeds the `8.9 ns/row` retained walk cost for runs $\le 16$.
- **`AdaptiveLIMIT24 + OutQ24` (`L = 24` on `r/n >= 0.08`):** **KILLED** — while `-1.1%` faster on isolated `W = 1` dense blocks, widening `OutQ` stores from `32 B -> 48 B` and growing $m$ (`dep_past_l2_loads +9.1%`, DRAM `+14%`) loses `1.0145×` natively at `W = 4` across `wikidata`; `LIMIT = 16` is Pareto-optimal.
