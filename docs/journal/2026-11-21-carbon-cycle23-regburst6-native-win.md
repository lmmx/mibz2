# Carbon Cycle 23: `RegBurst6` — Decoupled 6-Lane Scalar-Register Burst Walk (`0.850×` Dense / `0.978×` Corpus at `W = 4`)

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree)  
**Artifact:** [c23_regburst6.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle23/c23_regburst6.rs)

---

## 1. Beyond `DietRelay8Cells`: Eliminating Stack-State Stalls, Per-Lane `LONG` Branches, and Lane-`0`'s $\mu = 1$ Tail

Cycle 20's native diagnostic of `Relay8Ref` exposed three microarchitectural flaws in dynamic array-indexed 2-pass relays:
1. **Stack-array store-buffer stalls (`3.72M` cycles on `dense`):** Indexing `s_j[s]`, `s_acc[s]`, `lj[l]`, `lat[l]` by a runtime loop variable spills lane cursors to stack memory every step, saturating the store buffer and masking register-level taint in `chaingrind` (`CONTEXT.md` §5a).
2. **Lane-`0` $\mu = 1$ straggler tail:** Once scouts finish the cuts (~65% through Phase 4a), Lane `0` finishes the remaining cycle alone at $\mu = 1$.
3. **Cache-residency crossover in $m$:** Below $m \approx 140{,}000$ (`edges < 547 KiB`), `edges` has high cache hit rate so `2.0× m` loads loses ~50 µs to serial `1.0× m`; above $m \ge 140{,}000$, `emit` consumes `46–63%` of block time (`846–1,529 µs`) and misses cache heavily.

**`RegBurst6` (`emit_regburst6`):**
- **Decoupled 6-lane scalar-register bursts:** Both Stage 1 (length scouting) and Stage 3 (sloppy `16 B` emission) hold all 6 lanes' cursors (`j0..j5`, `at0..at5`, `rem0..rem5`) in dedicated scalar local variables (promoted to hardware GPRs, **zero stack-array writes inside the inner burst**), running **100%** of the block at $\mu = 6$ with a `<= 256`-step L1d macro-cycle prefix sum (`Stage 2`) and `<= 256 × 15 B` forward exact boundary fixup (`Stage 4`).
- **6-way bitwise `LONG` sentinel (`LONG = 0x80`):** Because all inline word lengths `1..=16` have bit 7 clear, `((l0 | l1 | l2 | l3 | l4 | l5) & 0x80) != 0` collapses 6 per-lane `LONG` branches into **1 predicted branch per 6 steps** (`0.33` branches/step vs `origin/master`'s `2.0` branches/step).
- **Physical spill gate (`BURST_MIN_M = 140_000`):** Blocks with $m < 140{,}000$ skip `word_len` population and run `emit_serial` with zero overhead.

Byte-exact parity (`warm` + `fresh` + `Plain`) passed on all `431` `wikidata.bz2` blocks; `W = 10` max worker heap is `1,275.2 KiB` vs `1,268.7 KiB` (`+6.5 KiB`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control (`MasterB / MasterA`) | Ungated `RegBurst6` (`m >= 16k`) | **Gated `RegBurst6` (`m >= 140k`)** |
|---|---:|---:|---:|---:|
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `0.9956×` (`11.52 / 11.47 ms`) | `0.9830×` | **`0.9767×`** (`11.57 / 11.30 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `1.0014×` (`12.54 / 12.56 ms`) | `0.9063×` | **`0.9094×`** (`12.54 / 11.40 ms`, `blk-270` **`0.8422×`**) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `1.0046×` (`351.2 / 352.8 ms`) | `1.0086×` | **`0.9927×`** (`353.0 / 350.5 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `1.0046×` (`351.5 / 353.1 ms`) | `1.0086×` | **`0.9928×`** (`353.4 / 350.8 ms`) |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `1.0298×` (`23.30 / 24.00 ms`) | `0.9652×` | **`0.9292×`** (`23.30 / 21.65 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0210×` (`25.84 / 26.38 ms`) | `0.8421×` | **`0.8498×`** (`25.77 / 21.90 ms`) |
| `wiki-open233` (barrier-synced homogeneous) | `W = 4` | `1.0071×` (`727.9 / 733.0 ms`) | `0.9940×` | **`0.9780×`** (`738.4 / 722.2 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `1.0069×` (`730.2 / 735.3 ms`) | `0.9913×` | **`0.9776×`** (`739.1 / 722.6 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `0.9969×` (`435.7 / 434.3 ms`) | `1.0097×` | **`0.9894×`** (`434.6 / 430.0 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `1.0113×` (`284.6 / 287.8 ms`) | `1.0175×` | **`0.9853×`** (`282.3 / 278.2 ms`) |

### Per-Block `W = 1` Breakdown (`Gated RegBurst6` vs `origin/master`)

- **Below gate (`m < 140k`, zero-overhead serial path):** `block-0` (`89.9k`): `1.0073×`, `block-2` (`112.1k`): `1.0012×`, `block-207` (`135.9k`): `0.9929×`.
- **Above gate (`m >= 140k`, `RegBurst6` path):** `block-85` (`156.4k`): **`0.9910×`**, `block-143` (`196.8k`): **`0.9575×`**, `block-210` (`204.3k`): **`0.9458×`**, `blk-41` (`231.6k`): **`0.9282×`**, `blk-56` (`230.0k`): **`0.9135×`**, `blk-165` (`221.1k`): **`0.9263×`**, `blk-270` (`199.2k`): **`0.8422×`**, `blk-278` (`201.4k`): **`0.9294×`**.

---

## 3. `chaingrind` Diagnostic

| Arm | Instructions | Proxy Cycles | `dep_past_l2_loads` | `max_chain_past_l2` | DRAM B/outB |
|---|---:|---:|---:|---:|---:|
| `origin/master` (`six`) | `88,300,666` | `38,485,418` | `395,446` | `2,881` | `0.798` |
| Ungated `RegBurst6` (`six`) | `114,626,915` | `34,496,082` | `258,106` (`-34.7%`) | **`491` (`-83.0%`)** | `0.861` |
| Gated `RegBurst6` (`six`) | `104,671,163` | `36,301,541` | `407,793` | **`774` (`-73.1%`)** | `0.846` |
| `origin/master` (`dense`) | `99,627,142` | `50,453,341` | `617,861` | `4,095` | `1.767` |
| `RegBurst6` (`dense`) | `130,170,338` | `42,037,709` | `428,538` (`-30.6%`) | **`145` (`-96.5%`)** | `1.806` |

---

## 4. Keep / Kill Decisions

- **Ungated `RegBurst6` on `m < 140_000`:** **KILLED** — loses `~4–5%` on `block-0`/`block-2` where `edges` fits comfortably in cache (`1.0086×` on `T1 wiki-all431`).
- **Gated `RegBurst6` (`m >= 140_000`):** **PROMOTED AS NEW WORKING BASELINE** — beats `origin/master` natively on every headline metric (`0.9094×` `T1 dense`, `0.8498×` `T4 dense`, `0.9767×` `T1 six`, `0.9292×` `T4 six`, `0.9928×` `T1 wiki`, `0.9776×` `T4 wiki`, `0.9853×` `E2E W=4`), and beats `Relay8Ref` by `7.0–8.5%` natively.
